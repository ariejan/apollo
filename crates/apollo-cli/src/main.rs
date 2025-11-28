//! Apollo CLI - Music library manager

// Allow integer casts that are safe in practice for CLI display:
// - Track/album counts from database won't exceed reasonable limits
// - List lengths from Vec won't exceed u32::MAX in practice
#![allow(clippy::cast_possible_truncation)]

use anyhow::{Context, Result};
use apollo_audio::{OrganizeOptions, ScanOptions, ScanProgress, organize_file, scan_directory};
use apollo_core::playlist::{Playlist, PlaylistId, PlaylistSort};
use apollo_core::query::Query;
use apollo_core::{Config, PathTemplate, TrackId};
use apollo_db::SqliteLibrary;
use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[derive(Parser)]
#[command(name = "apollo")]
#[command(author, version, about = "A modern music library manager", long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Path to the library database (overrides config)
    #[arg(short, long, global = true)]
    library: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new library
    Init {
        /// Path to the library database (default: ~/.apollo/apollo.db)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Import music files
    Import {
        /// Directory to import from
        path: PathBuf,

        /// Recursion depth (default: unlimited)
        #[arg(short, long)]
        depth: Option<usize>,

        /// Follow symbolic links
        #[arg(short = 's', long)]
        follow_symlinks: bool,
    },
    /// List items in the library
    List {
        /// Filter by type (tracks, albums)
        #[arg(short, long, value_enum, default_value = "tracks")]
        type_: ListType,

        /// Maximum number of items to show
        #[arg(short, long, default_value = "50")]
        limit: u32,

        /// Offset for pagination
        #[arg(short, long, default_value = "0")]
        offset: u32,
    },
    /// Search the library
    Query {
        /// Search query (searches title, artist, album)
        query: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "50")]
        limit: u32,
    },
    /// Start the web server
    Web {
        /// Host to bind to (overrides config)
        #[arg(short = 'H', long)]
        host: Option<String>,
        /// Port to listen on (overrides config)
        #[arg(short, long)]
        port: Option<u16>,
        /// Path to directory containing static web UI files
        #[arg(short, long)]
        static_dir: Option<PathBuf>,
    },
    /// Show library statistics
    Stats,
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Find duplicate tracks
    Duplicates {
        /// Detection type
        #[arg(short = 't', long, value_enum, default_value = "exact")]
        type_: DuplicateType,

        /// Duration tolerance for similar detection (in seconds)
        #[arg(short = 'd', long, default_value = "3")]
        duration_tolerance: u32,

        /// Show file paths
        #[arg(short, long)]
        paths: bool,
    },
    /// Organize files using path templates
    Organize {
        /// Destination directory for organized files
        destination: PathBuf,

        /// Path template (default from config, or "$artist/$album/$track - $title")
        #[arg(short, long)]
        template: Option<String>,

        /// Move files instead of copying
        #[arg(short, long)]
        move_files: bool,

        /// Overwrite existing files
        #[arg(short = 'f', long)]
        force: bool,

        /// Preview changes without making them
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Only organize specific tracks (by ID)
        #[arg(short = 'i', long)]
        track_ids: Vec<String>,

        /// Maximum number of tracks to organize
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Manage playlists
    Playlist {
        #[command(subcommand)]
        action: PlaylistAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Initialize a new configuration file
    Init {
        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,
    },
    /// Show configuration file path
    Path,
    /// Edit a configuration value
    Set {
        /// Configuration key (e.g., `web.port`, `acoustid.api_key`)
        key: String,
        /// Value to set
        value: String,
    },
    /// Get a configuration value
    Get {
        /// Configuration key (e.g., `web.port`, `acoustid.api_key`)
        key: String,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum ListType {
    Tracks,
    Albums,
}

#[derive(Clone, Copy, ValueEnum)]
enum DuplicateType {
    /// Exact byte-for-byte duplicates (same file hash)
    Exact,
    /// Similar tracks based on metadata (title, artist, duration)
    Similar,
    /// Both exact and similar duplicates
    All,
}

#[derive(Subcommand)]
enum PlaylistAction {
    /// Create a new playlist
    Create {
        /// Playlist name
        name: String,

        /// Description
        #[arg(short, long)]
        description: Option<String>,

        /// Create a smart playlist with a query
        #[arg(short, long)]
        query: Option<String>,

        /// Sort order for smart playlists
        #[arg(short, long, value_enum, default_value = "artist")]
        sort: PlaylistSortArg,

        /// Maximum number of tracks (for smart playlists)
        #[arg(short, long)]
        max_tracks: Option<u32>,
    },
    /// List all playlists
    List,
    /// Show a playlist's details and tracks
    Show {
        /// Playlist ID or name
        playlist: String,
    },
    /// Add a track to a static playlist
    AddTrack {
        /// Playlist ID or name
        playlist: String,

        /// Track ID(s) to add
        #[arg(required = true)]
        track_ids: Vec<String>,
    },
    /// Remove a track from a static playlist
    RemoveTrack {
        /// Playlist ID or name
        playlist: String,

        /// Track ID(s) to remove
        #[arg(required = true)]
        track_ids: Vec<String>,
    },
    /// Delete a playlist
    Delete {
        /// Playlist ID or name
        playlist: String,

        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(Clone, Copy, ValueEnum, Default)]
enum PlaylistSortArg {
    /// Sort by artist name, then album, then track number
    #[default]
    Artist,
    /// Sort by album name, then track number
    Album,
    /// Sort by track title
    Title,
    /// Sort by date added (newest first)
    AddedDesc,
    /// Sort by date added (oldest first)
    AddedAsc,
    /// Sort by year (newest first)
    YearDesc,
    /// Sort by year (oldest first)
    YearAsc,
    /// Random order
    Random,
}

impl From<PlaylistSortArg> for PlaylistSort {
    fn from(arg: PlaylistSortArg) -> Self {
        match arg {
            PlaylistSortArg::Artist => Self::Artist,
            PlaylistSortArg::Album => Self::Album,
            PlaylistSortArg::Title => Self::Title,
            PlaylistSortArg::AddedDesc => Self::AddedDesc,
            PlaylistSortArg::AddedAsc => Self::AddedAsc,
            PlaylistSortArg::YearDesc => Self::YearDesc,
            PlaylistSortArg::YearAsc => Self::YearAsc,
            PlaylistSortArg::Random => Self::Random,
        }
    }
}

/// Load configuration from file or use defaults.
fn load_config(config_path: Option<&Path>) -> Result<Config> {
    config_path.map_or_else(
        || Config::load().context("Failed to load configuration"),
        |path| Config::load_from(path).context("Failed to load configuration file"),
    )
}

/// Get the library path from CLI args, config, or default.
fn get_library_path(cli_path: Option<&Path>, config: &Config) -> PathBuf {
    cli_path.map_or_else(|| config.library_path(), Path::to_path_buf)
}

/// Ensure the parent directory for a path exists.
fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    Ok(())
}

/// Format a duration as MM:SS or HH:MM:SS.
fn format_duration(duration: std::time::Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Load configuration
    let config = load_config(cli.config.as_deref())?;

    match cli.command {
        Commands::Init { path } => cmd_init(path, &config).await,
        Commands::Import {
            path,
            depth,
            follow_symlinks,
        } => {
            let lib_path = get_library_path(cli.library.as_deref(), &config);
            cmd_import(&lib_path, &path, depth, follow_symlinks).await
        }
        Commands::List {
            type_,
            limit,
            offset,
        } => {
            let lib_path = get_library_path(cli.library.as_deref(), &config);
            cmd_list(&lib_path, type_, limit, offset).await
        }
        Commands::Query { query, limit } => {
            let lib_path = get_library_path(cli.library.as_deref(), &config);
            cmd_query(&lib_path, &query, limit).await
        }
        Commands::Stats => {
            let lib_path = get_library_path(cli.library.as_deref(), &config);
            cmd_stats(&lib_path).await
        }
        Commands::Web {
            host,
            port,
            static_dir,
        } => {
            let host = host.unwrap_or_else(|| config.web.host.clone());
            let port = port.unwrap_or(config.web.port);
            let lib_path = get_library_path(cli.library.as_deref(), &config);
            cmd_web(&lib_path, &host, port, static_dir.as_deref()).await
        }
        Commands::Config { action } => cmd_config(action, cli.config.as_deref()),
        Commands::Duplicates {
            type_,
            duration_tolerance,
            paths,
        } => {
            let lib_path = get_library_path(cli.library.as_deref(), &config);
            cmd_duplicates(&lib_path, type_, duration_tolerance, paths).await
        }
        Commands::Organize {
            destination,
            template,
            move_files,
            force,
            dry_run,
            track_ids,
            limit,
        } => {
            let lib_path = get_library_path(cli.library.as_deref(), &config);
            let template_str = template.unwrap_or_else(|| config.paths.path_template.clone());
            cmd_organize(
                &lib_path,
                &destination,
                &template_str,
                move_files,
                force,
                dry_run,
                &track_ids,
                limit,
            )
            .await
        }
        Commands::Playlist { action } => {
            let lib_path = get_library_path(cli.library.as_deref(), &config);
            cmd_playlist(&lib_path, action).await
        }
    }
}

/// Initialize a new library.
async fn cmd_init(path: Option<PathBuf>, config: &Config) -> Result<()> {
    let lib_path = path.unwrap_or_else(|| config.library_path());

    // Check if library already exists
    if lib_path.exists() {
        println!("Library already exists at: {}", lib_path.display());
        println!("Use --path to specify a different location");
        return Ok(());
    }

    // Ensure parent directory exists
    ensure_parent_dir(&lib_path)?;

    // Create the database
    let db_url = format!("sqlite:{}?mode=rwc", lib_path.display());
    let _db = SqliteLibrary::new(&db_url)
        .await
        .context("Failed to create library database")?;

    println!("Initialized new library at: {}", lib_path.display());
    println!();
    println!("Next steps:");
    println!("  apollo import /path/to/music   Import music files");
    println!("  apollo list                    List tracks in library");
    println!("  apollo query \"search term\"     Search the library");

    Ok(())
}

/// Import music files from a directory.
#[allow(clippy::too_many_lines)]
async fn cmd_import(
    lib_path: &Path,
    source_path: &Path,
    depth: Option<usize>,
    follow_symlinks: bool,
) -> Result<()> {
    // Check if library exists
    if !lib_path.exists() {
        eprintln!("Library not found at: {}", lib_path.display());
        eprintln!("Run 'apollo init' first to create a library");
        std::process::exit(1);
    }

    // Check if source directory exists
    if !source_path.exists() {
        eprintln!("Source directory not found: {}", source_path.display());
        std::process::exit(1);
    }

    if !source_path.is_dir() {
        eprintln!("Source path is not a directory: {}", source_path.display());
        std::process::exit(1);
    }

    // Connect to database
    let db_url = format!("sqlite:{}", lib_path.display());
    let db = SqliteLibrary::new(&db_url)
        .await
        .context("Failed to open library database")?;

    println!("Scanning: {}", source_path.display());

    // Set up progress tracking
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(
        ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );

    // Configure scan options
    let options = ScanOptions {
        recursive: true,
        max_depth: depth,
        follow_symlinks,
        compute_hashes: true,
    };

    // Cancellation token (not used in CLI for now, but API requires it)
    let cancel = Arc::new(AtomicBool::new(false));

    // Progress callback
    let progress_callback = |progress: &ScanProgress| {
        if let Some(ref current) = progress.current_file {
            let filename = current
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("...");
            // Note: we can't access progress_bar inside the closure easily
            // so we just log via tracing
            tracing::debug!(
                "Processing {}/{}: {}",
                progress.files_processed,
                progress.files_found,
                filename
            );
        }
    };

    // Run the scan
    let result = scan_directory(
        source_path,
        &options,
        Some(&cancel),
        Some(progress_callback),
    )
    .context("Failed to scan directory")?;

    progress_bar.finish_and_clear();

    let total_found = result.tracks.len();
    let errors = result.errors.len();

    if total_found == 0 {
        println!("No audio files found in {}", source_path.display());
        return Ok(());
    }

    println!("Found {total_found} audio files");
    if errors > 0 {
        println!("Skipped {errors} files with errors");
    }

    // Import tracks into database
    let import_bar = ProgressBar::new(total_found as u64);
    import_bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
        )
        .unwrap()
        .progress_chars("█▓▒░"),
    );

    let mut imported = 0u64;
    let mut skipped = 0u64;
    let mut failed = 0u64;

    for track in &result.tracks {
        import_bar.inc(1);

        // Try to add track; handle duplicate errors gracefully
        match db.add_track(track).await {
            Ok(_) => imported += 1,
            Err(apollo_db::DbError::Sqlx(ref e)) if e.to_string().contains("UNIQUE constraint") => {
                skipped += 1;
            }
            Err(e) => {
                tracing::warn!("Failed to import {}: {}", track.path.display(), e);
                failed += 1;
            }
        }
    }

    import_bar.finish_and_clear();

    println!();
    println!("Import complete:");
    println!("  Imported: {imported}");
    if skipped > 0 {
        println!("  Skipped (duplicates): {skipped}");
    }
    if failed > 0 {
        println!("  Failed: {failed}");
    }

    // Show summary
    let total_tracks = db.count_tracks().await?;
    println!();
    println!("Library now contains {total_tracks} tracks");

    Ok(())
}

/// List items in the library.
async fn cmd_list(lib_path: &Path, list_type: ListType, limit: u32, offset: u32) -> Result<()> {
    // Check if library exists
    if !lib_path.exists() {
        eprintln!("Library not found at: {}", lib_path.display());
        eprintln!("Run 'apollo init' first to create a library");
        std::process::exit(1);
    }

    // Connect to database
    let db_url = format!("sqlite:{}", lib_path.display());
    let db = SqliteLibrary::new(&db_url)
        .await
        .context("Failed to open library database")?;

    match list_type {
        ListType::Tracks => {
            let tracks = db.list_tracks(limit, offset).await?;
            let total = db.count_tracks().await?;

            if tracks.is_empty() {
                println!("No tracks in library");
                return Ok(());
            }

            let count = tracks.len() as u32;
            println!(
                "Showing tracks {}-{} of {total}",
                offset + 1,
                offset + count
            );
            println!();

            for track in &tracks {
                let duration = format_duration(track.duration);
                let album = track.album_title.as_deref().unwrap_or("-");
                let track_num = track
                    .track_number
                    .map_or_else(|| "--".to_string(), |n| format!("{n:02}"));

                println!(
                    "{track_num}. {} - {} [{album}] ({duration})",
                    track.artist, track.title
                );
            }

            if offset + count < total as u32 {
                println!();
                println!("Use --offset {} to see more", offset + count);
            }
        }
        ListType::Albums => {
            let albums = db.list_albums(limit, offset).await?;
            let total = db.count_albums().await?;

            if albums.is_empty() {
                println!("No albums in library");
                return Ok(());
            }

            let count = albums.len() as u32;
            println!(
                "Showing albums {}-{} of {total}",
                offset + 1,
                offset + count
            );
            println!();

            for album in &albums {
                let year = album.year.map_or_else(String::new, |y| format!(" ({y})"));
                let tracks = album.track_count;

                println!("{} - {}{year} [{tracks} tracks]", album.artist, album.title);
            }

            if offset + count < total as u32 {
                println!();
                println!("Use --offset {} to see more", offset + count);
            }
        }
    }

    Ok(())
}

/// Search the library.
async fn cmd_query(lib_path: &Path, query: &str, limit: u32) -> Result<()> {
    // Check if library exists
    if !lib_path.exists() {
        eprintln!("Library not found at: {}", lib_path.display());
        eprintln!("Run 'apollo init' first to create a library");
        std::process::exit(1);
    }

    // Connect to database
    let db_url = format!("sqlite:{}", lib_path.display());
    let db = SqliteLibrary::new(&db_url)
        .await
        .context("Failed to open library database")?;

    // FTS5 requires special query syntax; wrap in quotes for phrase search
    // or use * for prefix matching
    let fts_query = if query.contains(':') || query.contains('"') || query.contains('*') {
        // User provided FTS syntax, use as-is
        query.to_string()
    } else {
        // Simple search: add wildcards for prefix matching
        query
            .split_whitespace()
            .map(|word| format!("{word}*"))
            .collect::<Vec<_>>()
            .join(" ")
    };

    let tracks = db.search_tracks(&fts_query).await?;

    if tracks.is_empty() {
        println!("No tracks found matching: {query}");
        return Ok(());
    }

    let shown = tracks.len().min(limit as usize);
    println!("Found {} tracks matching: {query}", tracks.len());
    println!();

    for track in tracks.iter().take(shown) {
        let duration = format_duration(track.duration);
        let album = track.album_title.as_deref().unwrap_or("-");

        println!("{} - {} [{album}] ({duration})", track.artist, track.title);
    }

    if tracks.len() > shown {
        println!();
        println!("...and {} more", tracks.len() - shown);
    }

    Ok(())
}

/// Show library statistics.
async fn cmd_stats(lib_path: &Path) -> Result<()> {
    // Check if library exists
    if !lib_path.exists() {
        eprintln!("Library not found at: {}", lib_path.display());
        eprintln!("Run 'apollo init' first to create a library");
        std::process::exit(1);
    }

    // Connect to database
    let db_url = format!("sqlite:{}", lib_path.display());
    let db = SqliteLibrary::new(&db_url)
        .await
        .context("Failed to open library database")?;

    let track_count = db.count_tracks().await?;
    let album_count = db.count_albums().await?;

    println!("Library: {}", lib_path.display());
    println!();
    println!("Tracks: {track_count}");
    println!("Albums: {album_count}");

    Ok(())
}

/// Find duplicate tracks in the library.
async fn cmd_duplicates(
    lib_path: &Path,
    dup_type: DuplicateType,
    duration_tolerance_secs: u32,
    show_paths: bool,
) -> Result<()> {
    // Check if library exists
    if !lib_path.exists() {
        eprintln!("Library not found at: {}", lib_path.display());
        eprintln!("Run 'apollo init' first to create a library");
        std::process::exit(1);
    }

    // Connect to database
    let db_url = format!("sqlite:{}", lib_path.display());
    let db = SqliteLibrary::new(&db_url)
        .await
        .context("Failed to open library database")?;

    let duration_tolerance_ms = i64::from(duration_tolerance_secs) * 1000;
    let mut total_groups = 0;
    let mut total_duplicates = 0;

    // Find exact duplicates
    if matches!(dup_type, DuplicateType::Exact | DuplicateType::All) {
        let exact_groups = db.find_exact_duplicates().await?;

        if !exact_groups.is_empty() {
            println!("=== Exact Duplicates (Same File Hash) ===");
            println!();

            for (i, group) in exact_groups.iter().enumerate() {
                total_groups += 1;
                total_duplicates += group.len() - 1; // All but the first are duplicates

                println!("Group {} ({} files with same content):", i + 1, group.len());

                for track in group {
                    let duration = format_duration(track.duration);
                    println!("  {} - {} ({duration})", track.artist, track.title);
                    if show_paths {
                        println!("    {}", track.path.display());
                    }
                }
                println!();
            }
        }
    }

    // Find similar duplicates
    if matches!(dup_type, DuplicateType::Similar | DuplicateType::All) {
        let similar_groups = db.find_similar_duplicates(duration_tolerance_ms).await?;

        if !similar_groups.is_empty() {
            println!("=== Similar Duplicates (Matching Metadata) ===");
            println!("(tolerance: {duration_tolerance_secs} seconds)");
            println!();

            for (i, group) in similar_groups.iter().enumerate() {
                total_groups += 1;
                total_duplicates += group.len() - 1;

                println!("Group {} ({} similar tracks):", i + 1, group.len());

                for track in group {
                    let duration = format_duration(track.duration);
                    let album = track.album_title.as_deref().unwrap_or("-");
                    let format = &track.format;

                    println!(
                        "  {} - {} [{album}] ({duration}, {format})",
                        track.artist, track.title
                    );
                    if show_paths {
                        println!("    {}", track.path.display());
                    }
                }
                println!();
            }
        }
    }

    // Summary
    if total_groups == 0 {
        println!("No duplicates found.");
    } else {
        println!("Summary: {total_groups} groups, {total_duplicates} potential duplicates");
        println!();
        println!("Tip: Use --paths to see file locations");
    }

    Ok(())
}

/// Organize files using path templates.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
async fn cmd_organize(
    lib_path: &Path,
    destination: &Path,
    template_str: &str,
    move_files: bool,
    force: bool,
    dry_run: bool,
    track_ids: &[String],
    limit: Option<u32>,
) -> Result<()> {
    // Check if library exists
    if !lib_path.exists() {
        eprintln!("Library not found at: {}", lib_path.display());
        eprintln!("Run 'apollo init' first to create a library");
        std::process::exit(1);
    }

    // Parse the template
    let template = PathTemplate::parse(template_str)
        .with_context(|| format!("Invalid path template: {template_str}"))?;

    println!("Using template: {template_str}");
    println!("Destination: {}", destination.display());
    if move_files {
        println!("Mode: MOVE (files will be moved, not copied)");
    } else {
        println!("Mode: COPY");
    }
    if dry_run {
        println!("DRY RUN - no files will be modified");
    }
    println!();

    // Connect to database
    let db_url = format!("sqlite:{}", lib_path.display());
    let db = SqliteLibrary::new(&db_url)
        .await
        .context("Failed to open library database")?;

    // Get tracks to organize
    let tracks = if track_ids.is_empty() {
        // Get all tracks (with optional limit)
        let limit = limit.unwrap_or(u32::MAX);
        db.list_tracks(limit, 0).await?
    } else {
        // Get specific tracks by ID
        let mut result = Vec::new();
        for id_str in track_ids {
            let id = uuid::Uuid::parse_str(id_str)
                .with_context(|| format!("Invalid track ID: {id_str}"))?;
            let track_id = apollo_core::TrackId(id);
            if let Some(track) = db.get_track(&track_id).await? {
                result.push(track);
            } else {
                eprintln!("Warning: Track not found: {id_str}");
            }
        }
        result
    };

    if tracks.is_empty() {
        println!("No tracks to organize.");
        return Ok(());
    }

    let total = tracks.len();
    println!("Found {total} tracks to organize");
    println!();

    // Set up progress bar
    let progress_bar = ProgressBar::new(total as u64);
    progress_bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
        )
        .unwrap()
        .progress_chars("█▓▒░"),
    );

    let mut organized = 0u64;
    let mut skipped = 0u64;
    let mut failed = 0u64;

    let options = OrganizeOptions {
        move_files,
        overwrite: force,
        create_dirs: true,
    };

    for track in &tracks {
        progress_bar.inc(1);

        // Check if source file exists
        if !track.path.exists() {
            tracing::warn!("Source file missing: {}", track.path.display());
            skipped += 1;
            continue;
        }

        if dry_run {
            // Just preview the destination
            let ctx = apollo_core::TemplateContext::from_track(track);
            match template.render_with_extension(&ctx) {
                Ok(relative) => {
                    let dest = destination.join(&relative);
                    println!("{} -> {}", track.path.display(), dest.display());
                    organized += 1;
                }
                Err(e) => {
                    eprintln!("Template error for {}: {e}", track.path.display());
                    failed += 1;
                }
            }
        } else {
            // Actually organize the file
            match organize_file(&track.path, destination, &template, track, &options) {
                Ok(result) => {
                    tracing::debug!(
                        "{} {} -> {}",
                        if result.moved { "Moved" } else { "Copied" },
                        result.source.display(),
                        result.destination.display()
                    );
                    organized += 1;
                }
                Err(e) => {
                    // Check if it's just a "file exists" error and we should skip
                    let err_str = e.to_string();
                    if err_str.contains("already exists") {
                        skipped += 1;
                    } else {
                        tracing::warn!("Failed to organize {}: {e}", track.path.display());
                        failed += 1;
                    }
                }
            }
        }
    }

    progress_bar.finish_and_clear();

    println!();
    if dry_run {
        println!("Dry run complete:");
        println!("  Would organize: {organized}");
    } else {
        println!("Organization complete:");
        println!("  Organized: {organized}");
    }
    if skipped > 0 {
        println!("  Skipped: {skipped}");
    }
    if failed > 0 {
        println!("  Failed: {failed}");
    }

    Ok(())
}

/// Start the web server.
async fn cmd_web(lib_path: &Path, host: &str, port: u16, static_dir: Option<&Path>) -> Result<()> {
    // Check if library exists
    if !lib_path.exists() {
        eprintln!("Library not found at: {}", lib_path.display());
        eprintln!("Run 'apollo init' first to create a library");
        std::process::exit(1);
    }

    // Connect to database
    let db_url = format!("sqlite:{}", lib_path.display());
    let db = SqliteLibrary::new(&db_url)
        .await
        .context("Failed to open library database")?;

    let state = std::sync::Arc::new(apollo_web::AppState::new(db));
    let app = apollo_web::create_router_with_static_files(state, static_dir);

    let addr = format!("{host}:{port}");
    println!("Starting Apollo web server at http://{addr}");
    if static_dir.is_some() {
        println!("Web UI available at http://{addr}/");
    }
    println!("Swagger UI available at http://{addr}/swagger-ui");
    println!();
    println!("Press Ctrl+C to stop");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("Failed to bind to address")?;

    axum::serve(listener, app)
        .await
        .context("Web server error")?;

    Ok(())
}

/// Handle configuration commands.
fn cmd_config(action: ConfigAction, config_path: Option<&Path>) -> Result<()> {
    match action {
        ConfigAction::Show => {
            let config = load_config(config_path)?;
            let toml = config.to_toml().context("Failed to serialize config")?;
            println!("{toml}");
            Ok(())
        }
        ConfigAction::Init { force } => {
            let path = config_path
                .map(PathBuf::from)
                .or_else(Config::default_path)
                .context("Could not determine config path")?;

            if path.exists() && !force {
                eprintln!("Configuration file already exists at: {}", path.display());
                eprintln!("Use --force to overwrite");
                std::process::exit(1);
            }

            let config = Config::default();
            config.save_to(&path).context("Failed to save config")?;

            println!("Created configuration file at: {}", path.display());
            println!();
            println!("Edit this file to customize Apollo settings.");
            println!("Run 'apollo config show' to view current settings.");

            Ok(())
        }
        ConfigAction::Path => {
            let path = config_path
                .map(PathBuf::from)
                .or_else(Config::default_path)
                .context("Could not determine config path")?;

            println!("{}", path.display());

            if path.exists() {
                println!("(exists)");
            } else {
                println!("(not created yet - run 'apollo config init')");
            }

            Ok(())
        }
        ConfigAction::Get { key } => {
            let config = load_config(config_path)?;
            let value = get_config_value(&config, &key)?;
            println!("{value}");
            Ok(())
        }
        ConfigAction::Set { key, value } => {
            let mut config = load_config(config_path)?;
            set_config_value(&mut config, &key, &value)?;

            let path = config_path
                .map(PathBuf::from)
                .or_else(Config::default_path)
                .context("Could not determine config path")?;

            config.save_to(&path).context("Failed to save config")?;
            println!("Set {key} = {value}");

            Ok(())
        }
    }
}

/// Get a configuration value by key path.
fn get_config_value(config: &Config, key: &str) -> Result<String> {
    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["library", "path"] => Ok(config.library.path.display().to_string()),
        ["import", "move_files"] => Ok(config.import.move_files.to_string()),
        ["import", "write_tags"] => Ok(config.import.write_tags.to_string()),
        ["import", "copy_album_art"] => Ok(config.import.copy_album_art.to_string()),
        ["import", "auto_create_albums"] => Ok(config.import.auto_create_albums.to_string()),
        ["import", "compute_hashes"] => Ok(config.import.compute_hashes.to_string()),
        ["paths", "music_directory"] => Ok(config
            .paths
            .music_directory
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default()),
        ["paths", "path_template"] => Ok(config.paths.path_template.clone()),
        ["musicbrainz", "enabled"] => Ok(config.musicbrainz.enabled.to_string()),
        ["musicbrainz", "auto_tag"] => Ok(config.musicbrainz.auto_tag.to_string()),
        ["musicbrainz", "app_name"] => Ok(config.musicbrainz.app_name.clone()),
        ["musicbrainz", "app_version"] => Ok(config.musicbrainz.app_version.clone()),
        ["musicbrainz", "contact_email"] => Ok(config.musicbrainz.contact_email.clone()),
        ["acoustid", "enabled"] => Ok(config.acoustid.enabled.to_string()),
        ["acoustid", "api_key"] => Ok(config.acoustid.api_key.clone()),
        ["acoustid", "auto_lookup"] => Ok(config.acoustid.auto_lookup.to_string()),
        ["web", "host"] => Ok(config.web.host.clone()),
        ["web", "port"] => Ok(config.web.port.to_string()),
        ["web", "swagger_ui"] => Ok(config.web.swagger_ui.to_string()),
        ["plugins", "directory"] => Ok(config.plugins.directory.display().to_string()),
        ["plugins", "enabled"] => Ok(config.plugins.enabled.join(", ")),
        _ => anyhow::bail!("Unknown configuration key: {key}"),
    }
}

/// Set a configuration value by key path.
fn set_config_value(config: &mut Config, key: &str, value: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["library", "path"] => config.library.path = PathBuf::from(value),
        ["import", "move_files"] => config.import.move_files = parse_bool(value)?,
        ["import", "write_tags"] => config.import.write_tags = parse_bool(value)?,
        ["import", "copy_album_art"] => config.import.copy_album_art = parse_bool(value)?,
        ["import", "auto_create_albums"] => config.import.auto_create_albums = parse_bool(value)?,
        ["import", "compute_hashes"] => config.import.compute_hashes = parse_bool(value)?,
        ["paths", "music_directory"] => {
            config.paths.music_directory = if value.is_empty() {
                None
            } else {
                Some(PathBuf::from(value))
            };
        }
        ["paths", "path_template"] => config.paths.path_template = value.to_string(),
        ["musicbrainz", "enabled"] => config.musicbrainz.enabled = parse_bool(value)?,
        ["musicbrainz", "auto_tag"] => config.musicbrainz.auto_tag = parse_bool(value)?,
        ["musicbrainz", "app_name"] => config.musicbrainz.app_name = value.to_string(),
        ["musicbrainz", "app_version"] => config.musicbrainz.app_version = value.to_string(),
        ["musicbrainz", "contact_email"] => config.musicbrainz.contact_email = value.to_string(),
        ["acoustid", "enabled"] => config.acoustid.enabled = parse_bool(value)?,
        ["acoustid", "api_key"] => config.acoustid.api_key = value.to_string(),
        ["acoustid", "auto_lookup"] => config.acoustid.auto_lookup = parse_bool(value)?,
        ["web", "host"] => config.web.host = value.to_string(),
        ["web", "port"] => config.web.port = value.parse().context("Invalid port number")?,
        ["web", "swagger_ui"] => config.web.swagger_ui = parse_bool(value)?,
        ["plugins", "directory"] => config.plugins.directory = PathBuf::from(value),
        ["plugins", "enabled"] => {
            config.plugins.enabled = value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        _ => anyhow::bail!("Unknown configuration key: {key}"),
    }

    Ok(())
}

/// Parse a boolean value from string.
fn parse_bool(value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => anyhow::bail!("Invalid boolean value: {value} (use true/false)"),
    }
}

/// Handle playlist commands.
#[allow(clippy::too_many_lines)]
async fn cmd_playlist(lib_path: &Path, action: PlaylistAction) -> Result<()> {
    // Check if library exists
    if !lib_path.exists() {
        eprintln!("Library not found at: {}", lib_path.display());
        eprintln!("Run 'apollo init' first to create a library");
        std::process::exit(1);
    }

    // Connect to database
    let db_url = format!("sqlite:{}", lib_path.display());
    let db = SqliteLibrary::new(&db_url)
        .await
        .context("Failed to open library database")?;

    match action {
        PlaylistAction::Create {
            name,
            description,
            query,
            sort,
            max_tracks,
        } => {
            let playlist = if let Some(query_str) = query {
                // Parse the query
                let parsed_query = Query::parse(&query_str)
                    .with_context(|| format!("Invalid query: {query_str}"))?;

                let mut pl = Playlist::new_smart(&name, parsed_query).with_sort(sort.into());

                if let Some(max) = max_tracks {
                    pl = pl.with_max_tracks(max);
                }

                if let Some(desc) = description {
                    pl = pl.with_description(desc);
                }

                pl
            } else {
                let mut pl = Playlist::new_static(&name);

                if let Some(desc) = description {
                    pl = pl.with_description(desc);
                }

                pl
            };

            let kind = if playlist.is_smart() {
                "smart"
            } else {
                "static"
            };
            db.add_playlist(&playlist).await?;

            println!("Created {kind} playlist: {name}");
            println!("ID: {}", playlist.id);

            if playlist.is_smart() {
                if let Some(ref q) = playlist.query {
                    println!("Query: {q}");
                }
                println!("Sort: {}", playlist.sort);
                if let Some(ref limit) = playlist.limit
                    && let Some(max) = limit.max_tracks
                {
                    println!("Max tracks: {max}");
                }
            }

            Ok(())
        }
        PlaylistAction::List => {
            let playlists = db.list_playlists().await?;

            if playlists.is_empty() {
                println!("No playlists in library");
                return Ok(());
            }

            println!("Playlists ({} total):", playlists.len());
            println!();

            for playlist in playlists {
                let kind = if playlist.is_smart() {
                    "smart"
                } else {
                    "static"
                };
                let track_count = if playlist.is_static() {
                    playlist.track_ids.len()
                } else {
                    // For smart playlists, we'd need to evaluate the query
                    // Just show "?" for now to avoid expensive queries
                    0
                };

                let desc = playlist
                    .description
                    .as_ref()
                    .map(|d| format!(" - {d}"))
                    .unwrap_or_default();

                if playlist.is_smart() {
                    let query_str = playlist
                        .query
                        .as_ref()
                        .map(|q| format!(" [{q}]"))
                        .unwrap_or_default();
                    println!("  {} ({kind}){query_str}{desc}", playlist.name);
                } else {
                    println!("  {} ({kind}, {track_count} tracks){desc}", playlist.name);
                }
                println!("    ID: {}", playlist.id);
            }

            Ok(())
        }
        PlaylistAction::Show {
            playlist: name_or_id,
        } => {
            let playlist = find_playlist(&db, &name_or_id).await?;

            println!("Playlist: {}", playlist.name);
            println!("ID: {}", playlist.id);
            println!(
                "Type: {}",
                if playlist.is_smart() {
                    "smart"
                } else {
                    "static"
                }
            );

            if let Some(ref desc) = playlist.description {
                println!("Description: {desc}");
            }

            if playlist.is_smart() {
                if let Some(ref q) = playlist.query {
                    println!("Query: {q}");
                }
                println!("Sort: {}", playlist.sort);
                if let Some(ref limit) = playlist.limit
                    && let Some(max) = limit.max_tracks
                {
                    println!("Max tracks: {max}");
                }
            }

            println!();
            println!("Tracks:");

            let tracks = db.get_playlist_tracks(&playlist.id).await?;

            if tracks.is_empty() {
                println!("  (no tracks)");
            } else {
                for (i, track) in tracks.iter().enumerate() {
                    let duration = format_duration(track.duration);
                    let album = track.album_title.as_deref().unwrap_or("-");
                    println!(
                        "  {:3}. {} - {} [{album}] ({duration})",
                        i + 1,
                        track.artist,
                        track.title
                    );
                }
                println!();
                println!("Total: {} tracks", tracks.len());
            }

            Ok(())
        }
        PlaylistAction::AddTrack {
            playlist: name_or_id,
            track_ids,
        } => {
            let playlist = find_playlist(&db, &name_or_id).await?;

            if playlist.is_smart() {
                anyhow::bail!("Cannot add tracks to a smart playlist");
            }

            let mut added = 0;
            for id_str in &track_ids {
                let uuid = uuid::Uuid::parse_str(id_str)
                    .with_context(|| format!("Invalid track ID: {id_str}"))?;
                let track_id = TrackId(uuid);

                // Verify track exists
                if db.get_track(&track_id).await?.is_none() {
                    eprintln!("Warning: Track not found: {id_str}");
                    continue;
                }

                db.add_track_to_playlist(&playlist.id, &track_id).await?;
                added += 1;
            }

            println!("Added {added} track(s) to playlist '{}'", playlist.name);

            Ok(())
        }
        PlaylistAction::RemoveTrack {
            playlist: name_or_id,
            track_ids,
        } => {
            let playlist = find_playlist(&db, &name_or_id).await?;

            if playlist.is_smart() {
                anyhow::bail!("Cannot remove tracks from a smart playlist");
            }

            let mut removed = 0;
            for id_str in &track_ids {
                let uuid = uuid::Uuid::parse_str(id_str)
                    .with_context(|| format!("Invalid track ID: {id_str}"))?;
                let track_id = TrackId(uuid);

                db.remove_track_from_playlist(&playlist.id, &track_id)
                    .await?;
                removed += 1;
            }

            println!(
                "Removed {removed} track(s) from playlist '{}'",
                playlist.name
            );

            Ok(())
        }
        PlaylistAction::Delete {
            playlist: name_or_id,
            yes,
        } => {
            let playlist = find_playlist(&db, &name_or_id).await?;

            if !yes {
                println!(
                    "Delete playlist '{}' ({})? [y/N] ",
                    playlist.name, playlist.id
                );
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled");
                    return Ok(());
                }
            }

            db.remove_playlist(&playlist.id).await?;
            println!("Deleted playlist: {}", playlist.name);

            Ok(())
        }
    }
}

/// Find a playlist by ID or name.
async fn find_playlist(db: &SqliteLibrary, name_or_id: &str) -> Result<Playlist> {
    // Try parsing as UUID first
    if let Ok(uuid) = uuid::Uuid::parse_str(name_or_id) {
        let id = PlaylistId(uuid);
        if let Some(playlist) = db.get_playlist(&id).await? {
            return Ok(playlist);
        }
    }

    // Search by name
    let playlists = db.list_playlists().await?;
    for playlist in playlists {
        if playlist.name.eq_ignore_ascii_case(name_or_id) {
            return Ok(playlist);
        }
    }

    anyhow::bail!("Playlist not found: {name_or_id}")
}
