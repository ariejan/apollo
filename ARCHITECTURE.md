# Apollo Architecture

## Overview

Apollo is designed as a modular, layered system that separates concerns clearly:

```
┌─────────────────────────────────────────────────────────────┐
│                    User Interfaces                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │    CLI      │  │   Web UI    │  │    Lua Scripts      │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      Web API Layer                          │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  REST API (axum)  │  WebSocket (live updates)           ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Core Domain Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Library    │  │  AutoTagger  │  │   Query Engine   │  │
│  │   Service    │  │   Service    │  │                  │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   Infrastructure Layer                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Database   │  │    Audio     │  │  External APIs   │  │
│  │   (SQLite)   │  │   (lofty)    │  │  (MusicBrainz)   │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Crate Dependency Graph

```
apollo-cli ─────┬─────────────────────────────────┐
                   │                                 │
apollo-web ─────┼─────────────────────────────────┤
                   │                                 │
                   ▼                                 ▼
            apollo-lua              apollo-sources
                   │                         │
                   ▼                         │
            apollo-core ◄─────────────────┤
                   │                         │
                   ▼                         │
            apollo-db                     │
                   │                         │
                   ▼                         │
            apollo-audio ◄────────────────┘
```

## Core Types

### Track

```rust
pub struct Track {
    pub id: TrackId,
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<i32>,
    pub genre: Vec<String>,
    pub duration: Duration,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub format: AudioFormat,
    pub musicbrainz_id: Option<String>,
    pub acoustid: Option<String>,
    pub added_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub file_hash: String,
}
```

### Album

```rust
pub struct Album {
    pub id: AlbumId,
    pub title: String,
    pub artist: String,
    pub year: Option<i32>,
    pub genre: Vec<String>,
    pub track_count: u32,
    pub disc_count: u32,
    pub musicbrainz_id: Option<String>,
    pub cover_art: Option<CoverArt>,
}
```

## Plugin System (Lua)

Plugins are Lua scripts that can hook into various events:

```lua
-- Example plugin: auto-genre.lua

plugin = {
    name = "auto-genre",
    version = "1.0.0",
    description = "Automatically assigns genre based on artist"
}

-- Called when a track is imported
function on_import(track)
    if track.genre == nil or #track.genre == 0 then
        local genre = lookup_genre(track.artist)
        if genre then
            track.genre = {genre}
            return track  -- Return modified track
        end
    end
    return nil  -- No changes
end

-- Called when track metadata is updated
function on_update(old_track, new_track)
    -- Custom logic
end
```

### Available Hooks

| Hook | Arguments | Return | Description |
|------|-----------|--------|-------------|
| `on_import` | `track` | `track` or `nil` | Called before track is added |
| `on_update` | `old`, `new` | `track` or `nil` | Called before track is updated |
| `on_delete` | `track` | `bool` | Called before track is deleted |
| `on_query` | `query` | `query` | Can modify search queries |
| `on_autotag` | `track`, `candidates` | `candidates` | Can filter/sort candidates |

### Lua API

```lua
-- Library access
library.get_track(id)
library.get_album(id)
library.search(query)
library.update_track(track)

-- HTTP requests
http.get(url, headers)
http.post(url, body, headers)

-- Logging
log.info(message)
log.warn(message)
log.error(message)

-- Configuration
config.get(key)
config.set(key, value)
```

## Database Schema

```sql
-- Core tables
CREATE TABLE tracks (
    id TEXT PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    album_artist TEXT,
    album_id TEXT REFERENCES albums(id),
    track_number INTEGER,
    disc_number INTEGER,
    year INTEGER,
    duration_ms INTEGER NOT NULL,
    bitrate INTEGER,
    sample_rate INTEGER,
    channels INTEGER,
    format TEXT NOT NULL,
    musicbrainz_id TEXT,
    acoustid TEXT,
    added_at TEXT NOT NULL,
    modified_at TEXT NOT NULL,
    file_hash TEXT NOT NULL
);

CREATE TABLE albums (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    year INTEGER,
    disc_count INTEGER,
    musicbrainz_id TEXT,
    added_at TEXT NOT NULL,
    modified_at TEXT NOT NULL
);

CREATE TABLE genres (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE track_genres (
    track_id TEXT REFERENCES tracks(id) ON DELETE CASCADE,
    genre_id INTEGER REFERENCES genres(id),
    PRIMARY KEY (track_id, genre_id)
);

CREATE TABLE album_genres (
    album_id TEXT REFERENCES albums(id) ON DELETE CASCADE,
    genre_id INTEGER REFERENCES genres(id),
    PRIMARY KEY (album_id, genre_id)
);

-- Full-text search
CREATE VIRTUAL TABLE tracks_fts USING fts5(
    title, artist, album_artist, content=tracks
);

-- Indexes
CREATE INDEX idx_tracks_artist ON tracks(artist);
CREATE INDEX idx_tracks_album_id ON tracks(album_id);
CREATE INDEX idx_tracks_year ON tracks(year);
CREATE INDEX idx_tracks_added ON tracks(added_at);
```

## Web API

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/tracks` | List tracks (paginated) |
| GET | `/api/tracks/:id` | Get single track |
| PUT | `/api/tracks/:id` | Update track |
| DELETE | `/api/tracks/:id` | Delete track |
| GET | `/api/albums` | List albums (paginated) |
| GET | `/api/albums/:id` | Get album with tracks |
| GET | `/api/search` | Full-text search |
| POST | `/api/import` | Trigger import |
| GET | `/api/stats` | Library statistics |

### Query Parameters

```
GET /api/tracks?
    artist=<string>&
    album=<string>&
    year=<int>&
    genre=<string>&
    sort=<field>&
    order=asc|desc&
    limit=<int>&
    offset=<int>
```

## Configuration

```toml
# ~/.config/apollo/config.toml

[library]
path = "~/Music"
database = "~/.local/share/apollo/library.db"

[import]
copy = true                    # Copy files to library
move = false                   # Move instead of copy
write_tags = true              # Write tags to files
autotag = true                 # Auto-lookup metadata

[musicbrainz]
enabled = true
search_limit = 5

[web]
host = "127.0.0.1"
port = 8337

[plugins]
enabled = ["fetchart", "lastgenre"]
path = "~/.config/apollo/plugins"
```

## Error Handling Strategy

All operations use `Result<T, Error>` with a custom error enum:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Track not found: {0}")]
    TrackNotFound(TrackId),
    
    #[error("Album not found: {0}")]
    AlbumNotFound(AlbumId),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Audio file error: {0}")]
    Audio(#[from] lofty::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("MusicBrainz API error: {0}")]
    MusicBrainz(String),
    
    #[error("Plugin error in {plugin}: {message}")]
    Plugin { plugin: String, message: String },
}
```

## Testing Strategy

### Unit Tests
- Test all pure functions
- Test serialization/deserialization
- Use `proptest` for property-based testing of parsers

### Integration Tests
- Test database operations with in-memory SQLite
- Test file operations with temporary directories
- Test API endpoints with test server

### End-to-End Tests
- Test CLI commands
- Test web UI flows (when implemented)

### Mocking
- Mock MusicBrainz API responses with `wiremock`
- Mock filesystem for import tests
