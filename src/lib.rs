mod file_reader;
mod folder_watcher;

pub use file_reader::ReadStrategy;
pub use file_reader::Watcher;
pub use file_reader::{REPLACE_STRATEGY, TAIL_LINES_STRATEGY, TAIL_STRATEGY};
