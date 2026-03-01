use ed_log_scraper::file_reader::TAIL_LINES_STRATEGY;
use std::collections::HashSet;
use std::path::PathBuf;

const ED_LOG_PATH: &str = "C:\\Users\\Ian\\Saved Games\\Frontier Developments\\Elite Dangerous\\";

fn main() {
    let path = PathBuf::from(ED_LOG_PATH);
    let mut set = HashSet::new();
    set.insert("txt".to_string());
    let mut idk = ed_log_scraper::file_reader::FileReader::new(&path, TAIL_LINES_STRATEGY).unwrap();
    let rx = idk.take_receiver().unwrap();
    for e in rx {
        println!("File: {:?}\nContent: {}\n\n", e.0, e.1);
    }
}
