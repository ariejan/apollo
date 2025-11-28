# Apollo Task Board

## Current Sprint: Foundation

### Priority 1: Project Setup
- [DONE] Initialize Cargo workspace with all crates (2025-11-28)
- [TODO] Set up CI/CD with GitHub Actions
- [TODO] Configure code coverage reporting
- [TODO] Set up pre-commit hooks

### Priority 2: Core Types
- [DONE] Define Track metadata struct (2025-11-28)
- [DONE] Define Album metadata struct (2025-11-28)
- [DONE] Define Artist metadata struct (2025-11-28)
- [DONE] Implement Display traits (2025-11-28)
- [DONE] Implement Serialize/Deserialize (2025-11-28)
- [DONE] Write property-based tests for types (2025-11-28)

### Priority 3: Database Layer
- [DONE] Design SQLite schema (2025-11-28)
- [DONE] Set up sqlx migrations (2025-11-28)
- [DONE] Implement CRUD for tracks (2025-11-28)
- [DONE] Implement CRUD for albums (2025-11-28)
- [DONE] Implement search queries (2025-11-28)
- [DONE] Write integration tests (2025-11-28)

### Priority 4: Audio File Handling
- [DONE] Implement MP3 tag reading (2025-11-28)
- [DONE] Implement FLAC tag reading (2025-11-28)
- [DONE] Implement OGG tag reading (2025-11-28)
- [DONE] Implement tag writing (2025-11-28)
- [DONE] Directory scanning (2025-11-28)
- [DONE] File hashing for deduplication (2025-11-28)

### Priority 5: MusicBrainz Integration
- [TODO] MusicBrainz API client
- [TODO] Search by metadata
- [TODO] Search by fingerprint
- [TODO] Rate limiting
- [TODO] Caching responses

### Priority 6: CLI Foundation
- [DONE] Basic CLI structure with clap (2025-11-28)
- [DONE] `init` command (create library) (2025-11-28)
- [DONE] `import` command (scan directory) (2025-11-28)
- [DONE] `list` command (show library) (2025-11-28)
- [DONE] `query` command (search) (2025-11-28)

### Priority 7: Web API
- [TODO] Axum server setup
- [TODO] GET /api/tracks endpoint
- [TODO] GET /api/albums endpoint
- [TODO] Search endpoint
- [TODO] OpenAPI documentation

### Priority 8: Lua Integration
- [TODO] mlua setup
- [TODO] Expose Track/Album to Lua
- [TODO] Plugin loading system
- [TODO] Event hooks (on_import, on_update)
- [TODO] Example plugin

---

## Backlog

### Future Features
- [ ] Discogs integration
- [ ] Album art fetching
- [ ] Duplicate detection
- [ ] Smart playlists
- [ ] Web UI frontend
- [ ] Configuration file support
- [ ] Path templates for file organization

### Technical Debt
- (none yet)

---

## Completed

- [2025-11-28] Initialize Cargo workspace with all crates
- [2025-11-28] Define Track/Album/Artist metadata structs
- [2025-11-28] Implement Display traits for core types
- [2025-11-28] Implement Serialize/Deserialize for core types
- [2025-11-28] Basic CLI structure with clap
- [2025-11-28] Write property-based tests for core types and query parser
- [2025-11-28] Implement SQLite database layer with full CRUD and search
- [2025-11-28] Implement audio file handling (MP3/FLAC/OGG reading, writing, scanning, hashing)
- [2025-11-28] Implement CLI commands (init, import, list, query, stats)

---

## Task Format

```
- [STATUS] Task description
  - Subtask 1
  - Subtask 2
```

Statuses:
- `[TODO]` - Not started
- `[IN PROGRESS]` - Currently being worked on
- `[BLOCKED]` - Waiting on decision/dependency
- `[DONE]` - Completed
