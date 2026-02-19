use ed_log_scraper;
use std::collections::HashSet;
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from(".");
    let extensions = HashSet::from(["log".to_string()]);
    let handler = ed_log_scraper::folder_watcher::watch_folder_for_updates(&path, extensions);
    let rx = handler.get_receiver();

    for e in rx {
        println!("{:?}", e);
    }
}
