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
