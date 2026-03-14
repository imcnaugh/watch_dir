# watch_dir

A Rust library for monitoring a directory and reading file changes with configurable read strategies.

## Overview

`watch_dir` watches a directory for file modifications and delivers new content over a channel. You control how each file is read by providing a strategy selector function — useful for log tailing, config file reloading, or any scenario where you need to react to file changes.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
watch_dir = { path = "../watch_dir" }
```

### Basic example

```rust
use watch_dir::{Watcher, TAIL_LINES_STRATEGY};
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("/path/to/watch");
    let mut watcher = Watcher::new(&path, TAIL_LINES_STRATEGY).unwrap();
    let rx = watcher.take_receiver().unwrap();

    for (path, content) in rx {
        println!("{:?}: {}", path, content);
    }
}
```

### Custom strategy per file type

```rust
use watch_dir::{Watcher, ReadStrategy};
use std::path::Path;

fn my_strategy(path: &Path) -> ReadStrategy {
    match path.extension().and_then(|e| e.to_str()) {
        Some("log") => ReadStrategy::TailLines, // emit complete lines as they appear
        Some("json") => ReadStrategy::Replace,  // re-read the whole file on each change
        _ => ReadStrategy::Ignore,
    }
}

fn main() {
    let path = PathBuf::from("/path/to/watch");
    let mut watcher = Watcher::new(&path, my_strategy).unwrap();
    let rx = watcher.take_receiver().unwrap();

    for (path, content) in rx {
        println!("{:?}: {}", path, content);
    }
}
```

## Read strategies

| Strategy | Behaviour |
|----------|-----------|
| `ReadStrategy::Tail` | Emits new bytes since the last read |
| `ReadStrategy::TailLines` | Buffers new bytes and emits complete lines (newline-delimited) |
| `ReadStrategy::Replace` | Re-reads the entire file on every change |
| `ReadStrategy::Ignore` | Skips the file entirely |

Convenience constants `TAIL_STRATEGY`, `TAIL_LINES_STRATEGY`, and `REPLACE_STRATEGY` are provided for cases where you want the same strategy for all files.

## API

### `Watcher::new(path, strategy_fn) -> Result<Watcher, Error>`

Creates a watcher for the given directory. `strategy_fn` is called with the path of each changed file and returns the `ReadStrategy` to use. All modified files are observed; `strategy_fn` returning `ReadStrategy::Ignore` is how you opt individual files out.

### `Watcher::take_receiver() -> Option<Receiver<(PathBuf, String)>>`

Returns the channel receiver. Each message is a `(path, content)` tuple where `content` depends on the read strategy applied to that file.

## How it works

```
Directory
    │  (fs events via notify)
    ▼
FolderWatcher
    │  (PathBuf of modified file)
    ▼
Watcher
    │  (applies ReadStrategy, reads file)
    ▼
Receiver<(PathBuf, String)>
    │
    ▼
Your code
```

`Watcher` spawns a background thread that:
1. Receives file modification events from `FolderWatcher`
2. Applies the configured read strategy
3. Sends `(path, content)` pairs to a channel you consume

## Dependencies

- [`notify`](https://crates.io/crates/notify) — cross-platform file system event detection
