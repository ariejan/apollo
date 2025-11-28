//! Apollo CLI - Music library manager

// Allow integer casts that are safe in practice for CLI display:
// - Track/album counts from database won't exceed reasonable limits
// - List lengths from Vec won't exceed u32::MAX in practice
#![allow(clippy::cast_possible_truncation)]

use anyhow::{Context, Result};
use apollo_audio::{ScanOptions, ScanProgress, scan_directory};
use apollo_db::SqliteLibrary;
use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Default library database filename.
const DEFAULT_DB_NAME: &str = "apollo.db";

/// Default library location (relative to home directory).
const DEFAULT_LIB_DIR: &str = ".apollo";

#[derive(Parser)]
#[command(name = "apollo")]
#[command(author, version, about = "A modern music library manager", long_about = None)]
struct Cli {
    /// Path to the library database
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
        /// Host to bind to
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
        /// Port to listen on
        #[arg(short, long, default_value = "8337")]
        port: u16,
    },
    /// Show library statistics
    Stats,
}

#[derive(Clone, Copy, ValueEnum)]
enum ListType {
    Tracks,
    Albums,
}

/// Get the default library path.
fn default_library_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(DEFAULT_LIB_DIR).join(DEFAULT_DB_NAME))
}

/// Get the library path from CLI args or default.
fn get_library_path(cli_path: Option<&Path>) -> Result<PathBuf> {
    cli_path.map_or_else(default_library_path, |p| Ok(p.to_path_buf()))
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

    match cli.command {
        Commands::Init { path } => cmd_init(path).await,
        Commands::Import {
            path,
            depth,
            follow_symlinks,
        } => {
            let lib_path = get_library_path(cli.library.as_deref())?;
            cmd_import(&lib_path, &path, depth, follow_symlinks).await
        }
        Commands::List {
            type_,
            limit,
            offset,
        } => {
            let lib_path = get_library_path(cli.library.as_deref())?;
            cmd_list(&lib_path, type_, limit, offset).await
        }
        Commands::Query { query, limit } => {
            let lib_path = get_library_path(cli.library.as_deref())?;
            cmd_query(&lib_path, &query, limit).await
        }
        Commands::Stats => {
            let lib_path = get_library_path(cli.library.as_deref())?;
            cmd_stats(&lib_path).await
        }
        Commands::Web { host, port } => {
            println!("Starting web server at {host}:{port}");
            println!("Web server not yet implemented");
            Ok(())
        }
    }
}

/// Initialize a new library.
async fn cmd_init(path: Option<PathBuf>) -> Result<()> {
    let lib_path = path.map_or_else(default_library_path, Ok)?;

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
