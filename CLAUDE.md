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

- **`Watcher`** (`src/watcher.rs`) — the public API. Takes a `Path` and an `Options` struct. Sets up `notify_debouncer_full`, populates initial file offsets, spawns a `Worker` thread, and exposes `run()`, `pause()`, `stop()`, and `take_receiver()`.

- **`Options`** (`src/options.rs`) — builder for watcher config: `with_read_strategy_selector`, `with_recursive`, `with_notify_debounce_duration`. Defaults: `TAIL_STRATEGY`, non-recursive, 250ms debounce.

- **`Worker`** (`src/worker.rs`) — the background thread struct. Loops on debounced notify events and a control channel (`Actions::Run/Pause/Stop`). Applies the `ReadStrategy` per file, tracks tail offsets and per-file line buffers for `TailLines`. All file reading logic lives here.

- **`ReadStrategy`** (`src/read_strategy.rs`) — enum with four variants:
  - `Tail` — read only new bytes appended since last read (tracks byte offset)
  - `TailLines` — like Tail but buffers incomplete lines and only emits full lines
  - `Replace` — always read the full file contents
  - `Ignore` — skip this file
  - `SelectStrategy` trait for the strategy selector function; implemented for `Fn(&Path) -> ReadStrategy`
  - Convenience constants: `TAIL_STRATEGY`, `TAIL_LINES_STRATEGY`, `REPLACE_STRATEGY`

- **`Error`** (`src/error.rs`) — `WatchDirError` type.

The caller receives a `Receiver<(PathBuf, String)>` and pulls updates at their own pace.

Integration tests (`tests/`) use `TestDir` (in `tests/common/mod.rs`) to create isolated temp directories and write files, then assert on channel messages.

## Notes

- Edition 2024 in `Cargo.toml` (not the usual 2021).
