# Apollo - Autonomous Development Guide

You are an autonomous AI developer working on **Apollo**, a modern Rust alternative to the Python-based [beets](https://beets.io/) music library manager.

## Project Vision

Apollo is a cross-platform music collection organizer with:
- **Core written in Rust** for performance and cross-platform deployment
- **Lua scripting** for user customization and plugins
- **Web interface** as a first-class citizen (not an afterthought)
- **Offline-first** with optional sync capabilities

## Current Session Instructions

When you start, follow this workflow:

### 1. Check Current State
```bash
# Check what branch you're on and current status
git status
git log --oneline -5

# Check if there are any DECISIONS_NEEDED.md entries
cat DECISIONS_NEEDED.md 2>/dev/null || echo "No decisions pending"

# Check the task board
cat TASKS.md
```

### 2. Pick a Task
- Look at `TASKS.md` for the current priority
- If a task is marked `[IN PROGRESS]`, continue it
- If no task is in progress, pick the highest priority `[TODO]` task
- Update `TASKS.md` to mark your chosen task as `[IN PROGRESS]`

### 3. Work Loop
For each task:

1. **Create a feature branch** (if not already on one):
   ```bash
   git checkout -b feature/<task-name>
   ```

2. **Implement incrementally** with frequent commits:
   - Write tests first when applicable
   - Commit every logical unit of work
   - Use conventional commits: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`

3. **Run quality checks before committing**:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```

4. **Commit and push regularly** (at minimum every 30 minutes of work):
   ```bash
   git add -A
   git commit -m "<type>: <description>"
   git push origin HEAD
   ```

### 4. When You Need Human Input

If you encounter a decision that requires human input:

1. **DO NOT BLOCK** - continue working on other tasks
2. Add the decision to `DECISIONS_NEEDED.md`:
   ```markdown
   ## [DATE] Decision: <Title>
   
   **Context:** <Why this decision is needed>
   
   **Options:**
   1. Option A - <pros/cons>
   2. Option B - <pros/cons>
   
   **Recommendation:** <Your suggested approach>
   
   **Blocked Tasks:** <What's waiting on this>
   
   **Status:** PENDING
   ```
3. Switch to a different task that doesn't depend on this decision
4. Create a git worktree if needed for parallel work

### 5. Using Git Worktrees for Parallel Work

When blocked on one task, use worktrees to work on independent tasks:

```bash
# Create a worktree for a different feature
git worktree add ../apollo-<feature> -b feature/<other-feature>
cd ../apollo-<feature>

# Work on the other feature...

# When done, clean up
cd ../apollo
git worktree remove ../apollo-<feature>
```

### 6. End of Session

Before your session ends (or when interrupted):

1. Commit all current work (even if incomplete):
   ```bash
   git add -A
   git commit -m "wip: <description of current state>"
   git push origin HEAD
   ```

2. Update `TASKS.md` with current progress
3. Update `SESSION_LOG.md` with what was accomplished

---

## Architecture Overview

```
apollo/
├── crates/
│   ├── apollo-core/       # Core library (no IO)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── metadata.rs   # Track/Album metadata types
│   │   │   ├── library.rs    # Library abstraction
│   │   │   ├── query.rs      # Query language parser
│   │   │   └── autotag.rs    # Autotagging logic
│   │   └── Cargo.toml
│   │
│   ├── apollo-db/         # Database layer (SQLite)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── schema.rs
│   │   │   └── migrations/
│   │   └── Cargo.toml
│   │
│   ├── apollo-audio/      # Audio file handling
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── formats/      # MP3, FLAC, OGG, etc.
│   │   │   └── fingerprint.rs
│   │   └── Cargo.toml
│   │
│   ├── apollo-sources/    # Metadata sources
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── musicbrainz.rs
│   │   │   ├── discogs.rs
│   │   │   └── traits.rs
│   │   └── Cargo.toml
│   │
│   ├── apollo-lua/        # Lua scripting support
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── bindings.rs
│   │   └── Cargo.toml
│   │
│   ├── apollo-web/        # Web interface
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── api/          # REST/GraphQL API
│   │   │   └── handlers/
│   │   ├── frontend/         # Svelte/React frontend
│   │   └── Cargo.toml
│   │
│   └── apollo-cli/        # CLI application
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
│
├── plugins/                   # Built-in Lua plugins
│   ├── fetchart.lua
│   ├── lastgenre.lua
│   └── duplicates.lua
│
├── PROMPT.md                  # This file
├── TASKS.md                   # Current task board
├── DECISIONS_NEEDED.md        # Human decisions queue
├── SESSION_LOG.md             # Work log
├── ARCHITECTURE.md            # Detailed architecture docs
└── Cargo.toml                 # Workspace config
```

## Core Dependencies (Approved)

These dependencies are pre-approved for use:

```toml
# Core
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
anyhow = "1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }

# Audio
symphonia = "0.5"           # Audio decoding
lofty = "0.21"              # Audio tag reading/writing
chromaprint = "0.1"         # Audio fingerprinting (if available)

# Web
axum = "0.7"
tower = "0.4"
tower-http = "0.5"

# Lua
mlua = { version = "0.9", features = ["lua54", "async", "serialize"] }

# HTTP client
reqwest = { version = "0.12", features = ["json"] }

# CLI
clap = { version = "4", features = ["derive"] }
indicatif = "0.17"          # Progress bars
dialoguer = "0.11"          # Interactive prompts

# Testing
proptest = "1"
wiremock = "0.6"
```

## Quality Standards

### Code Quality
- All code must pass `cargo fmt --check`
- All code must pass `cargo clippy -- -D warnings`
- All public APIs must have documentation
- Test coverage goal: >80% for core crates

### Commit Standards
- Use conventional commits
- Each commit should be atomic and buildable
- Write meaningful commit messages

### Testing Requirements
- Unit tests for all business logic
- Integration tests for database operations
- Property-based tests for parsers
- Mock external APIs in tests

## Priority Features (MVP)

1. **Import music files** - Scan directories, read metadata
2. **MusicBrainz integration** - Fetch and match metadata
3. **Database storage** - Store library in SQLite
4. **Basic CLI** - import, list, query commands
5. **Web API** - REST API for library access
6. **Basic web UI** - Browse and search library

## Out of Scope (for now)

- Transcoding
- ReplayGain calculation
- Playlist management
- Streaming/playback
- Mobile apps

---

## Communication Protocol

### Asking for Human Input
Write to `DECISIONS_NEEDED.md` and continue with other work.

### Reporting Progress
Update `SESSION_LOG.md` after completing tasks.

### Reporting Blockers
If truly blocked (all tasks depend on decisions), write to `BLOCKERS.md` and exit gracefully.

---

## Remember

1. **Commit early, commit often** - Push at least every 30 minutes
2. **Don't block on decisions** - Log them and move on
3. **Use worktrees** - Work on independent features in parallel
4. **Run tests** - Never commit failing code
5. **Document as you go** - Code without docs is incomplete
