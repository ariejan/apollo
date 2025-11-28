# 🎵 Apollo

A modern, cross-platform music library manager written in Rust. Inspired by [beets](https://beets.io/), reimagined for the future.

> ⚠️ **Work in Progress** - This project is under active development.

## Features (Planned)

- **Fast & Cross-Platform** - Native Rust performance on Windows, macOS, and Linux
- **Web-First** - Built-in REST API and web interface
- **Extensible** - Lua scripting for custom plugins and workflows
- **Smart Tagging** - Automatic metadata from MusicBrainz, Discogs, and more
- **Modern CLI** - Intuitive command-line interface with progress indicators

## Quick Start

```bash
# Build from source
cargo build --release

# Initialize a library
apollo init --path ~/Music

# Import your music
apollo import /path/to/music

# Search your library
apollo query "artist:Beatles"

# Start the web interface
apollo web --port 8337
```

## Architecture

Apollo is built as a collection of focused crates:

| Crate | Description |
|-------|-------------|
| `apollo-core` | Core types and business logic |
| `apollo-db` | SQLite database layer |
| `apollo-audio` | Audio file handling and metadata |
| `apollo-sources` | External metadata sources (MusicBrainz, etc.) |
| `apollo-lua` | Lua scripting support |
| `apollo-web` | REST API and web interface |
| `apollo-cli` | Command-line interface |

## Development

### Prerequisites

- Rust 1.75+ (stable)
- SQLite 3.x
- Lua 5.4 (for plugin development)

### Building

```bash
# Check all crates compile
cargo check --workspace

# Run tests
cargo test --workspace

# Build release binary
cargo build --release
```

### Code Quality

This project enforces strict code quality:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests with coverage
cargo llvm-cov --workspace
```

## Contributing

This project is developed with assistance from Claude Code. See:

- `PROMPT.md` - AI development guide
- `TASKS.md` - Current task board
- `DECISIONS_NEEDED.md` - Pending decisions
- `ARCHITECTURE.md` - Technical documentation

## License

MIT OR Apache-2.0

## Acknowledgments

- [beets](https://beets.io/) - The original inspiration
- [MusicBrainz](https://musicbrainz.org/) - Metadata database
- [lofty](https://crates.io/crates/lofty) - Audio tag library
