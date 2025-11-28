# CLAUDE.md - Instructions for Claude Code

This file contains specific instructions for Claude Code when working on this project.

## Quick Reference

```bash
# Start of session
git status && git log --oneline -5
cat DECISIONS_NEEDED.md
cat TASKS.md

# Quality checks (run before every commit)
cargo fmt --check
cargo clippy -- -D warnings
cargo test

# Commit and push
git add -A
git commit -m "<type>: <description>"
git push origin HEAD
```

## Commit Frequency

**IMPORTANT**: Commit and push at least every 30 minutes, or after completing any logical unit of work.

Rationale: This ensures progress is saved and visible, allows the human operator to monitor progress, and prevents loss of work if the session is interrupted.

## Commit Message Format

Use conventional commits:
- `feat:` - New feature
- `fix:` - Bug fix
- `refactor:` - Code refactoring
- `test:` - Adding tests
- `docs:` - Documentation
- `chore:` - Build, CI, dependencies

Examples:
```
feat: add Track struct with serde serialization
fix: handle missing album field gracefully
refactor: extract metadata parsing to separate module
test: add property tests for query parser
docs: add API documentation for web endpoints
chore: update sqlx to 0.8
```

## When to Create a Decision Request

Add to `DECISIONS_NEEDED.md` when:
- Choosing between significantly different architectural approaches
- Selecting external dependencies not in the approved list
- Making UX/design decisions that affect users
- Anything that could be contentious or have long-term implications

Do NOT create a decision request for:
- Implementation details within agreed architecture
- Bug fixes
- Refactoring
- Adding tests
- Minor dependency updates

## Git Worktree Usage

Use worktrees when:
1. Blocked on a decision and want to continue other work
2. Working on independent features in parallel
3. Need to test something without affecting current work

```bash
# Create worktree for feature
git worktree add ../apollo-<feature> -b feature/<name>

# List worktrees
git worktree list

# Remove when done
git worktree remove ../apollo-<feature>
```

## Code Style

### Rust
- Run `cargo fmt` before committing
- No `unwrap()` in library code (use `?` or proper error handling)
- Prefer `thiserror` for error types
- Document all public APIs

### File Organization
- One module per file for small modules
- Directory modules for larger components
- Keep `lib.rs` minimal (just re-exports)

### Testing
- Tests go in the same file (`#[cfg(test)] mod tests`)
- Integration tests in `tests/` directory
- Use `#[tokio::test]` for async tests

## Project-Specific Commands

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p apollo-core

# Run with logging
RUST_LOG=debug cargo run

# Check all crates compile
cargo check --workspace

# Generate docs
cargo doc --workspace --no-deps
```

## Handling Interruption

If the session is interrupted (timeout, error, etc.), ensure:

1. Current work is committed (even as WIP)
2. TASKS.md reflects current state
3. SESSION_LOG.md has latest progress

```bash
# Emergency save
git add -A
git commit -m "wip: interrupted session - <brief description>"
git push origin HEAD
```

## File Modification Rules

### Always Read First
Before modifying any file, read it completely to understand context.

### TASKS.md
- Mark tasks `[IN PROGRESS]` when starting
- Mark tasks `[DONE]` when complete
- Add date to completed tasks

### DECISIONS_NEEDED.md
- Add new decisions at the top (below template)
- Never remove or modify existing pending decisions
- Only mark as RESOLVED after human has responded

### SESSION_LOG.md
- Add session entry at the top when starting
- Update throughout the session
- Always include a summary at session end

## Environment Assumptions

- Rust stable toolchain installed
- `cargo`, `rustfmt`, `clippy` available
- Git configured and authenticated
- Network access for crates.io and MusicBrainz API

## Remember

1. **Quality over speed** - Don't commit broken code
2. **Incremental progress** - Small commits are better than big ones
3. **Document decisions** - Future Claude (and humans) will thank you
4. **Test everything** - Tests are documentation too
5. **Push often** - Progress not pushed is progress not saved
