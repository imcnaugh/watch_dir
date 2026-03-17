mod file_reader;
mod folder_watcher;

pub use file_reader::ReadStrategy;
pub use file_reader::Watcher;
pub use file_reader::{REPLACE_STRATEGY, TAIL_LINES_STRATEGY, TAIL_STRATEGY};

pub struct Options {
    recursive: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { recursive: true }
    }
}

impl Options {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }
}
