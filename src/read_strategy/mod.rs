use std::path::Path;

#[derive(Debug)]
pub enum ReadStrategy {
    Tail,      // emit whatever new bytes arrive
    TailLines, // buffer until newline, emit complete lines only
    Replace,   // read the whole file on each change
    Ignore,    // ignore the file
}

pub const REPLACE_STRATEGY: fn(&Path) -> ReadStrategy = |_| ReadStrategy::Replace;
pub const TAIL_STRATEGY: fn(&Path) -> ReadStrategy = |_| ReadStrategy::Tail;
pub const TAIL_LINES_STRATEGY: fn(&Path) -> ReadStrategy = |_| ReadStrategy::TailLines;
