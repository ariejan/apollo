# Apollo Task Board

## Current Sprint: Foundation

### Priority 1: Project Setup
- [TODO] Initialize Cargo workspace with all crates
- [TODO] Set up CI/CD with GitHub Actions
- [TODO] Configure code coverage reporting
- [TODO] Set up pre-commit hooks

### Priority 2: Core Types
- [TODO] Define Track metadata struct
- [TODO] Define Album metadata struct  
- [TODO] Define Artist metadata struct
- [TODO] Implement Display traits
- [TODO] Implement Serialize/Deserialize
- [TODO] Write property-based tests for types

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
- [TODO] Basic CLI structure with clap
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

(Move completed tasks here with date)

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
