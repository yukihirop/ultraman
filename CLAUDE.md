# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ultraman is a Rust implementation of Foreman - a tool for managing Procfile-based applications. It allows you to run applications defined in a Procfile, export them to various process management formats (systemd, launchd, supervisord, etc.), and manage multi-process applications with proper signal handling.

## Key Architecture

- **Command Structure**: Four main commands implemented in `src/cmd/`:
  - `start`: Run processes defined in Procfile
  - `run`: Execute single command with app environment
  - `check`: Validate Procfile syntax
  - `export`: Generate config files for process managers

- **Core Components**:
  - `src/procfile.rs`: Parses Procfile format using regex patterns
  - `src/process.rs`: Manages child processes with proper signal handling
  - `src/signal.rs`: Handles SIGINT/SIGTERM for graceful shutdown
  - `src/log/`: Output formatting with timestamps and colors
  - `src/cmd/export/`: Contains exporters for different process management systems

- **Process Management**: Each process gets unique PORT assignments and PS names. Signal handling ensures proper cleanup of child processes on shutdown.

## Development Commands

```bash
# Build and run
cargo run start                    # Start with default Procfile
cargo run start -p ./Procfile     # Start with specific Procfile
cargo run -- --help               # Show help
cargo run start --help            # Show start command help

# Testing
cargo test --locked               # Run standard tests
cargo test --locked -- --ignored # Run signal handling tests (can interrupt other tests)
cargo test --locked -- --nocapture # Run with output

# Development shortcuts from README
cargo run export <format> <location>
cargo run run <app>
```

## Build and Release

```bash
# Development build
cargo build

# Testing with make
make test                         # Full test suite including ignored tests

# Man page generation  
make man                         # View man page
make create_man                  # Generate man page to ./tmp/ultraman.1
make install_man                 # Install man page system-wide

# Cross-platform releases
make release_linux               # Build for x86_64-unknown-linux-musl
make release_mac                 # Build for x86_64-apple-darwin  
make release_win                 # Build for x86_64-pc-windows-msvc
```

## Testing Notes

- Signal handling tests in `src/signal.rs` are marked with `#[ignore]` as they can interrupt other tests
- Use `cargo test -- --ignored` to run signal tests specifically
- Docker environment available for Linux testing on macOS: `docker-compose up -d`

## Release Process

Ultraman uses automated GitHub Actions to build and release pre-compiled binaries:

1. **Automated Releases**: When a new version tag (e.g., `v0.3.3`) is pushed, GitHub Actions automatically:
   - Builds binaries for Linux (x86_64), macOS (x86_64 and ARM64), and Windows (x86_64)
   - Uploads the binaries to GitHub Releases
   - Automatically updates the Homebrew formula with new version and SHA256 hashes

2. **Manual Release**: Create and push a new tag:
   ```bash
   git tag v0.3.3
   git push origin v0.3.3
   ```

3. **Pre-built Binaries**: Users can now install without compilation:
   - **Homebrew**: `brew install yukihirop/tap/ultraman` (uses pre-built binaries)
   - **Direct Download**: Download from GitHub Releases page
   - **Cargo**: `cargo install ultraman` (still compiles from source)

## Key Dependencies

- `structopt`: CLI argument parsing
- `nix`: Unix signal handling and process management
- `handlebars`: Template engine for export formats
- `signal-hook`: Signal handling utilities
- `crossbeam`: Thread coordination