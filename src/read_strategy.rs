use std::path::Path;

pub trait SelectStrategy: Send + 'static {
    fn select(&self, path: &Path) -> ReadStrategy;
}

impl<F: Fn(&Path) -> ReadStrategy + Send + 'static> SelectStrategy for F {
    fn select(&self, path: &Path) -> ReadStrategy {
        self(path)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ReadStrategy {
    Tail,      // emit whatever new bytes arrive
    TailLines, // buffer until newline, emit complete lines only
    Replace,   // read the whole file on each change
    Ignore,    // ignore the file
}

pub const REPLACE_STRATEGY: fn(&Path) -> ReadStrategy = |_| ReadStrategy::Replace;
pub const TAIL_STRATEGY: fn(&Path) -> ReadStrategy = |_| ReadStrategy::Tail;
pub const TAIL_LINES_STRATEGY: fn(&Path) -> ReadStrategy = |_| ReadStrategy::TailLines;
