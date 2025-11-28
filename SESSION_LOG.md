# Session Log

This file tracks what was accomplished in each Claude Code session.

---

## Log Format

```markdown
## Session: YYYY-MM-DD HH:MM

### Completed
- Task or subtask completed
- Another accomplishment

### In Progress
- Task that was started but not finished

### Blockers Encountered
- Any issues that came up

### Decisions Made
- Any decisions that were made autonomously

### Decisions Requested
- Any decisions added to DECISIONS_NEEDED.md

### Notes
- Any relevant observations or context for next session
```

---

## Sessions

## Session: 2025-11-28 (Web API)

### Completed
- Implemented complete REST API in apollo-web crate:
  - `GET /api/tracks` - List all tracks with pagination
  - `GET /api/tracks/:id` - Get single track by ID
  - `GET /api/albums` - List all albums with pagination
  - `GET /api/albums/:id` - Get single album by ID
  - `GET /api/albums/:id/tracks` - Get all tracks in an album
  - `GET /api/search?q=` - Full-text search for tracks
  - `GET /api/stats` - Library statistics (track/album counts)
  - `GET /health` - Health check endpoint
- Proper error handling with JSON error responses
- CORS support for cross-origin requests
- HTTP tracing middleware for request logging
- 12 integration tests using axum-test
- Total 53 tests passing across workspace

### In Progress
- None

### Blockers Encountered
- None

### Decisions Made
- Use `:id` path syntax for Axum 0.7 (not `{id}` which is for 0.8+)
- Default pagination limit of 50, max 500
- Simple FTS5 prefix matching for search (word* pattern)

### Decisions Requested
- None

### Notes
- OpenAPI documentation still TODO
- Lua integration is next major feature
- Web server can be started by creating `AppState` and calling `create_router()`

---

## Session: 2025-11-28 (MusicBrainz integration)

### Completed
- Implemented MusicBrainz API client in apollo-sources crate:
  - `MusicBrainzClient` with proper User-Agent configuration
  - Built-in rate limiting (1 request/second)
  - `search_recordings()`: Search for songs by title/artist
  - `search_releases()`: Search for albums by title/artist
  - `lookup_recording()`: Get recording by MBID
  - `lookup_release()`: Get release by MBID
  - `find_best_recording()`: Smart matching with score, album, duration
- Complete API response types: Recording, Release, Artist, Track, etc.
- Lucene query escaping for safe search queries
- Added urlencoding dependency for URL encoding
- Total 41 tests passing across workspace

### In Progress
- None

### Blockers Encountered
- None

### Decisions Made
- Use 1.1 second delay between requests (slightly over 1s for safety)
- Return Option from find_best_recording when no good match found
- Use checked_sub for Instant math to satisfy clippy

### Decisions Requested
- None

### Notes
- Client is functional but doesn't implement fingerprint search yet
- Response caching would be beneficial to reduce API calls
- Next priority: Web API or add `tag` command to CLI for MusicBrainz tagging

---

## Session: 2025-11-28 (CLI commands)

### Completed
- Implemented complete CLI with all core commands:
  - `init`: Create a new library database (default: ~/.apollo/apollo.db)
    - Creates parent directories if needed
    - Checks for existing library
  - `import`: Scan directory and import audio files
    - Progress bars for scanning and importing
    - Handles duplicates gracefully (skips)
    - Shows summary of imported/skipped/failed
  - `list`: List tracks or albums with pagination
    - Supports --type tracks/albums
    - Supports --limit and --offset
    - Shows formatted output with duration, track numbers
  - `query`: Search the library using FTS
    - Supports simple text search (auto-wildcards)
    - Supports FTS5 syntax for advanced queries
  - `stats`: Show library statistics
  - Global `--library` flag for custom db path
- Added `dirs` dependency for home directory detection
- Total 38 tests passing across workspace

### In Progress
- None

### Blockers Encountered
- None

### Decisions Made
- Store library database at ~/.apollo/apollo.db by default
- Use FTS5 prefix matching (word*) for simple queries
- Allow raw FTS5 syntax when query contains special chars
- Add #[allow(clippy::too_many_lines)] for import function
- Add #[allow(clippy::cast_possible_truncation)] for CLI display casts

### Decisions Requested
- None

### Notes
- CLI is now fully functional for basic music library management
- Next priority: MusicBrainz integration or Web API
- The import command doesn't create Album entries yet (tracks only)

---

## Session: 2025-11-28 (audio handling)

### Completed
- Implemented complete apollo-audio crate with:
  - `read_metadata()`: Read tags from audio files (MP3, FLAC, OGG, etc.)
    - Extracts title, artist, album, track/disc numbers, year, genres
    - Extracts MusicBrainz IDs and AcoustID fingerprint IDs
    - Falls back to filename if title is missing
    - Auto-detects audio format from file type
  - `write_metadata()`: Write Track data back to audio files
    - Supports ID3v2 for MP3, VorbisComments for FLAC/OGG, MP4 tags
    - Creates tags if none exist
  - `scan_directory()`: Recursively scan directories for audio files
    - Configurable recursion depth, symlink following
    - Optional progress callback and cancellation support
    - Returns ScanResult with tracks and errors
  - `compute_file_hash()`: SHA-256 hash for deduplication
- Added dependencies: sha2, hex, walkdir
- All 10 tests passing in apollo-audio
- Total 38 tests passing across workspace

### In Progress
- None

### Blockers Encountered
- None

### Decisions Made
- Use lofty crate for tag reading/writing (supports all common formats)
- Store AcoustID under custom key "ACOUSTID_ID" (not standard in lofty)
- Default to ID3v2 tag type when creating new tags for unknown formats
- Use 64KB buffer for file hashing (balance of speed and memory)

### Decisions Requested
- None

### Notes
- Audio handling is complete for MVP requirements
- Next priorities: CLI commands (init, import, list, query)
- MusicBrainz integration can come after basic CLI works

---

## Session: 2025-11-28 (continued)

### Completed
- Added property-based tests using proptest:
  - Track, Album, Artist serialization roundtrips
  - AudioFormat display validity
  - TrackId/AlbumId uniqueness and UUID format
  - Query parser field:value, year ranges, text queries
  - Duration serialization with millisecond precision
  - Total 22 tests in apollo-core (up from 6)

- Implemented SQLite database layer (apollo-db):
  - Designed schema with tracks, albums tables
  - Added full-text search using FTS5 virtual tables
  - Created triggers to keep FTS index in sync
  - Implemented SqliteLibrary with full CRUD operations:
    - get_track, add_track, update_track, remove_track
    - get_album, add_album, update_album, remove_album
    - get_album_tracks, search_tracks
    - list_tracks, list_albums with pagination
    - count_tracks, count_albums
  - 5 integration tests for database operations
  - Total 27 tests passing across workspace

### In Progress
- None

### Blockers Encountered
- None

### Decisions Made
- Store genres as JSON arrays in SQLite (flexible, searchable)
- Store timestamps as ISO8601 strings (portable, human-readable)
- Use FTS5 for full-text search (built into SQLite, no external deps)
- Allow clippy integer casts in schema.rs (documented as safe for music data)

### Decisions Requested
- None

### Notes
- Database layer is fully functional with in-memory and file-based storage
- FTS search is working with automatic index sync
- Next priorities: Audio file handling, then CLI implementation

---

## Session: 2025-11-28 (initial)

### Completed
- Fixed dependency configuration to work without pkg-config:
  - Changed reqwest to use rustls-tls instead of native-tls (avoids OpenSSL dependency)
  - Added vendored feature to mlua (bundles Lua 5.4 source)
- Fixed all clippy warnings across the workspace:
  - Added `# Errors` documentation to Library trait methods
  - Fixed doc_markdown warnings for MusicBrainz/AcoustID/Discogs links
  - Made Artist::new const fn
  - Fixed collapsible_if in query parser
  - Fixed uninlined_format_args in CLI
  - Removed unnecessary Result wrapper from main()
- Ran cargo fmt to fix formatting issues
- All 6 tests passing in apollo-core

### In Progress
- Priority 2: Core Types - property-based tests still needed

### Blockers Encountered
- System doesn't have pkg-config installed (required by openssl-sys and mlua-sys)
- Resolved by using rustls and vendored Lua instead

### Decisions Made
- Use rustls-tls for HTTP client instead of native-tls (avoids OpenSSL system dependency)
- Use vendored Lua 5.4 (bundles source code, no system library needed)

### Decisions Requested
- None

### Notes
- Workspace structure already exists with all crates scaffolded
- Core types (Track, Album, Artist) are implemented with serde serialization
- Basic query parser is implemented (field:value syntax)
- Library trait is defined but no implementations yet
- CLI is scaffolded with subcommands but unimplemented
- Next priority: Add property-based tests, then move to database layer
