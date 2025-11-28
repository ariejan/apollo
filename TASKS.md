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
- [IN PROGRESS] Write property-based tests for types

### Priority 3: Database Layer
- [TODO] Design SQLite schema
- [TODO] Set up sqlx migrations
- [TODO] Implement CRUD for tracks
- [TODO] Implement CRUD for albums
- [TODO] Implement search queries
- [TODO] Write integration tests

### Priority 4: Audio File Handling
- [TODO] Implement MP3 tag reading
- [TODO] Implement FLAC tag reading
- [TODO] Implement OGG tag reading
- [TODO] Implement tag writing
- [TODO] Directory scanning
- [TODO] File hashing for deduplication

### Priority 5: MusicBrainz Integration
- [TODO] MusicBrainz API client
- [TODO] Search by metadata
- [TODO] Search by fingerprint
- [TODO] Rate limiting
- [TODO] Caching responses

### Priority 6: CLI Foundation
- [DONE] Basic CLI structure with clap (2025-11-28)
- [TODO] `init` command (create library)
- [TODO] `import` command (scan directory)
- [TODO] `list` command (show library)
- [TODO] `query` command (search)

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
