use std::path::{Path, PathBuf};

fn main() {
    // TODO: update to path of your local journal folder
    let dir = PathBuf::from("C:\\Users\\Ian\\Saved Games\\Frontier Developments\\Elite Dangerous");

    let options = watch_dir::Options::default().with_read_strategy_selector(|path: &Path| {
        if let Some(ext) = path.extension() {
            match ext.to_str().unwrap_or("") {
                "log" => watch_dir::ReadStrategy::TailLines,
                _ => watch_dir::ReadStrategy::Ignore
            }
        } else {
            watch_dir::ReadStrategy::Ignore
        }
    }).with_notify_debounce_duration(std::time::Duration::from_millis(50));

    let mut watcher = watch_dir::Watcher::new(&dir, options).unwrap();
    let rx = watcher.take_receiver().unwrap();

    for (path, content) in rx {
        println!("{:?}: {}", path.file_name().unwrap_or_default(), content);
    }
}