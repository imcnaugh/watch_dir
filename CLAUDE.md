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

`watch_dir` is a Rust library that monitors a directory for file changes and streams modified file contents over an `mpsc` channel.

**Current target architecture** (in-progress refactor on branch `1-simplify-logic-into-a-single-threads`):

- **`Watcher`** (`src/watcher.rs`) — the single consolidated public API. Uses `notify_debouncer_full` directly (no separate folder watcher thread), runs one background thread that receives debounced notify events, applies the `ReadStrategy`, reads files, and sends `(PathBuf, String)` tuples to the consumer. File reading logic (tail offsets, tail_lines buffering, replace) is being ported here from `file_reader.rs`.

- **`ReadStrategy`** (`src/read_strategy/mod.rs`) — already extracted as its own module:
  - `Tail` — read only new bytes appended since last read (tracks byte offset)
  - `TailLines` — like Tail but buffers incomplete lines and only emits full lines
  - `Replace` — always read the full file contents
  - `Ignore` — skip this file
  - Also exports convenience constants: `TAIL_STRATEGY`, `TAIL_LINES_STRATEGY`, `REPLACE_STRATEGY`

**Legacy files** (being replaced, not yet deleted):
- `src/folder_watcher.rs` — old two-thread design wrapping raw `notify::recommended_watcher`
- `src/file_reader.rs` — old `Watcher` that sat on top of `FolderWatcher`; contains the full file reading logic to be ported to `watcher.rs`

The caller receives a `Receiver<(PathBuf, String)>` and pulls updates at their own pace.

Integration tests (`tests/`) use `TestDir` (in `tests/common/mod.rs`) to create isolated temp directories and write files, then assert on channel messages.

## Notes

- Edition 2024 in `Cargo.toml` (not the usual 2021).
- There is a known Windows issue with the `notify` crate's event handling — see commit `641ce6e` and the linked upstream issue.
