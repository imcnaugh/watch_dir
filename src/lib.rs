mod file_reader;
mod folder_watcher;

// TODO rename file reader, its a horrible name for the library to expose
pub use file_reader::FileReader;
pub use file_reader::ReadStrategy;
pub use file_reader::{REPLACE_STRATEGY, TAIL_LINES_STRATEGY, TAIL_STRATEGY};
