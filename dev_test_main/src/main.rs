use ed_log_scraper;
use std::collections::HashSet;
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from(".");
    let extensions = HashSet::from(["json".to_string()]);
    let folder_watch_handler = ed_log_scraper::folder_watcher::watch_folder_for_updates(&path, extensions);

    for e in folder_watch_handler.get_receiver() {
        println!("{:?}", e);
    }
}
