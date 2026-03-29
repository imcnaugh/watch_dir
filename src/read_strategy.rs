use std::path::Path;

/// Determines the [`ReadStrategy`] to apply to a file.
///
/// Implement this trait to provide custom per-file strategy selection, or use one of the
/// provided constants: [`TAIL_STRATEGY`], [`TAIL_LINES_STRATEGY`], [`REPLACE_STRATEGY`].
pub trait SelectStrategy: Send + 'static {
    /// Returns the [`ReadStrategy`] to use for the file at `path`.
    fn select(&self, path: &Path) -> ReadStrategy;
}

impl<F: Fn(&Path) -> ReadStrategy + Send + 'static> SelectStrategy for F {
    fn select(&self, path: &Path) -> ReadStrategy {
        self(path)
    }
}

/// Controls how a file's contents are read and emitted when it changes.
#[derive(Debug, Copy, Clone)]
pub enum ReadStrategy {
    /// Emit only bytes appended since the last read, tracking the file offset.
    Tail,
    /// Like [`ReadStrategy::Tail`], but buffers incomplete lines and only emits complete lines.
    TailLines,
    /// Emit the entire file contents on every change.
    Replace,
    /// Skip this file entirely.
    Ignore,
}

/// A [`SelectStrategy`] that applies [`ReadStrategy::Replace`] to all files.
pub const REPLACE_STRATEGY: fn(&Path) -> ReadStrategy = |_| ReadStrategy::Replace;

/// A [`SelectStrategy`] that applies [`ReadStrategy::Tail`] to all files.
pub const TAIL_STRATEGY: fn(&Path) -> ReadStrategy = |_| ReadStrategy::Tail;

/// A [`SelectStrategy`] that applies [`ReadStrategy::TailLines`] to all files.
pub const TAIL_LINES_STRATEGY: fn(&Path) -> ReadStrategy = |_| ReadStrategy::TailLines;
