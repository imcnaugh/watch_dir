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
use watch_dir::{Watcher, Options, TAIL_LINES_STRATEGY};
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("/path/to/watch");
    let options = Options::new()
        .with_read_strategy_selector(TAIL_LINES_STRATEGY);
    let mut watcher = Watcher::new(&path, options).unwrap();
    let rx = watcher.take_receiver().unwrap();

    watcher.run();
    for (path, content) in rx {
        println!("{:?}: {}", path, content);
    }
}
```

### Custom strategy per file type

```rust
use watch_dir::{Watcher, Options, ReadStrategy};
use std::path::Path;
use std::path::PathBuf;

fn my_strategy(path: &Path) -> ReadStrategy {
    match path.extension().and_then(|e| e.to_str()) {
        Some("log") => ReadStrategy::TailLines, // emit complete lines as they appear
        Some("json") => ReadStrategy::Replace,  // re-read the whole file on each change
        _ => ReadStrategy::Ignore,
    }
}

fn main() {
    let path = PathBuf::from("/path/to/watch");
    let options = Options::new()
        .with_read_strategy_selector(my_strategy)
        .with_recursive(true);
    let mut watcher = Watcher::new(&path, options).unwrap();
    let rx = watcher.take_receiver().unwrap();

    watcher.run();
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

### `Options`

Builder for watcher configuration:

| Method | Description | Default |
|--------|-------------|---------|
| `with_read_strategy_selector(fn)` | Strategy selector called per-file to choose how it is read | `TAIL_STRATEGY` |
| `with_recursive(bool)` | Watch subdirectories recursively | `false` |
| `with_notify_debounce_duration(Duration)` | Debounce window for file system events | 250ms |

### `Watcher::new(path, options) -> Result<Watcher, Error>`

Creates a watcher for the given directory using the provided `Options`. The strategy selector is called with the path of each changed file and returns the `ReadStrategy` to use. `ReadStrategy::Ignore` is how you opt individual files out.

### `Watcher::take_receiver() -> Option<Receiver<(PathBuf, String)>>`

Returns the channel receiver. Each message is a `(path, content)` tuple where `content` depends on the read strategy applied to that file.

### `Watcher::run()` / `Watcher::pause()` / `Watcher::stop()`

Control the watcher's background worker, the worker is automatically started when the watcher is created.

## Dependencies

- [`notify`](https://crates.io/crates/notify) — cross-platform file system event detection
