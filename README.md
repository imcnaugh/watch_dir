# watch_dir

A Rust library that watches a directory for file changes and streams new content over a channel.

Most file-watching crates stop at detecting that a file changed. `watch_dir` goes further: it reads the file, applies a configurable strategy (tail new bytes, emit complete lines, re-read the whole file, or ignore), and delivers `(path, content)` pairs to your code via a `std::sync::mpsc` channel. Debouncing, offset tracking, and background threading are handled for you.

## Usage

```toml
[dependencies]
watch_dir = "1.0.0"
```

### Tail files in a directory

Emit each new complete line as it is appended to any file modified in the directory:

```rust
use watch_dir::{Watcher, Options, TAIL_LINES_STRATEGY};
use std::path::PathBuf;

fn main() {
    let watcher = Watcher::new(
        &PathBuf::from("/var/log/myapp"),
        Options::default(),
    ).unwrap();

    let rx = watcher.take_receiver().unwrap();

    for (path, line) in rx {
        println!("{}: {}", path.display(), line);
    }
}
```

### Different strategy per file type

Use a selector function to choose how each file is read:

```rust
use watch_dir::{Watcher, Options, ReadStrategy};
use std::path::{Path, PathBuf};

fn strategy(path: &Path) -> ReadStrategy {
    match path.extension().and_then(|e| e.to_str()) {
        Some("log")  => ReadStrategy::TailLines, // emit complete lines as they appear
        Some("json") => ReadStrategy::Replace,   // re-read the whole file on each change
        Some("txt")  => ReadStrategy::Tail,      // emit appended text as it appears
        _            => ReadStrategy::Ignore,    // ignore anything that's not matched.
    }
}

fn main() {
    let watcher = Watcher::new(
        &PathBuf::from("/path/to/watch"),
        Options::new()
            .with_read_strategy_selector(strategy) // set your custom strategy,
            .with_recursive(true)                  // recursively watch subdirectories
    ).unwrap();

    let rx = watcher.take_receiver().unwrap();

    for (path, content) in rx {
        println!("{}: {}", path.display(), content);
    }
}
```

## Read strategies

| Strategy    | Behaviour                                                   |
|-------------|-------------------------------------------------------------|
| `Tail`      | Emit new text since the last read                           |
| `TailLines` | Buffer new text; emit only complete newline-delimited lines |
| `Replace`   | Re-read the entire file on every change                     |
| `Ignore`    | Skip this file                                              |

Convenience constants `TAIL_STRATEGY`, `TAIL_LINES_STRATEGY`, and `REPLACE_STRATEGY` apply one strategy to all files. Pass a function `fn(&Path) -> ReadStrategy` to vary it per file.

## Options

| Method                                    | Default         |
|-------------------------------------------|-----------------|
| `with_read_strategy_selector(fn)`         | `TAIL_STRATEGY` |
| `with_recursive(bool)`                    | `false`         |
| `with_notify_debounce_duration(Duration)` | 250ms           |

## Dependencies

- [`notify`](https://crates.io/crates/notify) — cross-platform file system event detection
- [`notify-debouncer-mini`](https://crates.io/crates/notify-debouncer-mini) — debouncing layer that collapses rapid events per path
