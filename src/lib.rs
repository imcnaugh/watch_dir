mod file_reader;
mod folder_watcher;

pub use file_reader::ReadStrategy;
pub use file_reader::FileReader;
pub use file_reader::{TAIL_LINES_STRATEGY, TAIL_STRATEGY, REPLACE_STRATEGY};