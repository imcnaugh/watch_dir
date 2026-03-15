# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build          # build
cargo test           # run all tests
cargo test <name>    # run a single test by name
cargo clippy         # lint
cargo fmt            # format
cargo check          # fast type-check without building
```

## Architecture

`watch_dir` is a Rust library that monitors a directory for file changes and streams modified file contents over an `mpsc` channel. There are two layers:

1. **`FolderWatcher`** (`src/folder_watcher.rs`) — wraps `notify::recommended_watcher`, filters for modify events (platform-specific: macOS vs Windows differ), and forwards changed `PathBuf`s over a channel.

2. **`Watcher`** (`src/file_reader.rs`) — the main public API. Receives paths from `FolderWatcher`, reads files using a caller-supplied strategy selector function `(PathBuf) -> ReadStrategy`, and sends `(PathBuf, String)` tuples to the consumer.

**`ReadStrategy`** controls how each file is read:
- `Tail` — read only new bytes appended since last read (tracks byte offset)
- `TailLines` — like Tail but buffers incomplete lines and only emits full lines
- `Replace` — always read the full file contents
- `Ignore` — skip this file

The library uses `std::sync::mpsc` channels and spawns background threads. The caller receives a `Receiver<(PathBuf, String)>` and pulls updates at their own pace.

Integration tests (`tests/`) use `TestDir` (in `tests/common/mod.rs`) to create isolated temp directories and write files, then assert on channel messages.

## Notes

- Edition 2024 in `Cargo.toml` (not the usual 2021).
- There is a known Windows issue with the `notify` crate's event handling — see commit `641ce6e` and the linked upstream issue.
