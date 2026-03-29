//! Monitors a directory for file changes and streams new content over a channel.
//!
//! # Quick start
//!
//! ```no_run
//! use watch_dir::{Watcher, Options};
//! use std::path::Path;
//!
//! let mut watcher = Watcher::new(Path::new("/var/log"), Options::default())?;
//! let rx = watcher.take_receiver().unwrap();
//!
//! for (path, content) in rx {
//!     println!("{}: {}", path.display(), content);
//! }
//! # Ok::<(), watch_dir::WatchDirError>(())
//! ```

mod error;
mod options;
mod read_strategy;
mod watcher;
mod worker;

pub use error::*;
pub use options::*;
pub use read_strategy::*;
pub use watcher::*;
