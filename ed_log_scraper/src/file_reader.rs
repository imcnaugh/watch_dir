use std::collections::HashMap;
use std::sync::mpsc;

pub struct FileReader {
    input_channel: mpsc::Receiver<String>,
    offsets: HashMap<String, usize>,
}

