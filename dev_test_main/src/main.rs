use ed_log_scraper;
use std::collections::HashSet;
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from(".");
    let mut set = HashSet::new();
    set.insert("txt".to_string());
    let idk = ed_log_scraper::file_reader::start(&path).unwrap();
    let rx = idk.get_rx();
    for e in rx {
        println!("{}", e);
    }
}
