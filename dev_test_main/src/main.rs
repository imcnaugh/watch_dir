use watch_dir::file_reader::ReadStrategy;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

const ED_LOG_PATH: &str = "C:\\Users\\Ian\\Saved Games\\Frontier Developments\\Elite Dangerous";
const TEST_PATH: &str = "/Users/ian/Documents/code/ed_log_parser";

fn main() {
    let path = PathBuf::from(TEST_PATH);
    let mut idk = watch_dir::file_reader::FileReader::new(&path, test_read_strategy).unwrap();
    let rx: Receiver<(PathBuf, String)> = idk.take_receiver().unwrap();
    for (p, c) in rx {
        if p.extension().unwrap().to_str().unwrap() == "log" {
            println!("File: {:?}\nContent: {}", p, c);
        }
    }
}

fn ed_read_strategy(path: &PathBuf) -> ReadStrategy {
    println!("{:?}", path);
    match path.extension().unwrap().to_str() {
        Some("log") => ReadStrategy::TailLines,
        Some("json") => ReadStrategy::Replace,
        _ => ReadStrategy::Ignore,
    }
}

fn test_read_strategy(path: &PathBuf) -> ReadStrategy {
    match path.extension().unwrap().to_str() {
        Some("txt") => ReadStrategy::Replace,
        Some("log") => ReadStrategy::TailLines,
        Some("json") => ReadStrategy::Replace,
        _ => ReadStrategy::Ignore,
    }
}
