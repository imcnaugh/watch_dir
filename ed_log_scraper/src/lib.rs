use notify::event::DataChange;
use notify::event::ModifyKind::Data;
use notify::{Event, EventKind, RecursiveMode, Result, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub fn watch_folder(folder: String) {
    let (tx, rx) = mpsc::channel::<Result<Event>>();

    let mut watcher = notify::recommended_watcher(tx).unwrap();
    watcher
        .watch(Path::new(&folder), RecursiveMode::NonRecursive)
        .unwrap();

    let mut file_and_offset: HashMap<&PathBuf, usize> = HashMap::new();

    for event in rx {
        match event {
            Ok(evt) => {
                if let EventKind::Modify(Data(dat)) = evt.kind
                    && dat == DataChange::Content
                {
                    evt.paths.iter().for_each(|path| {
                        let offset = file_and_offset.get(&path).unwrap_or(&0);
                        get_new_content_from_file(&path, *offset);
                    })
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn get_new_content_from_file(path: &PathBuf, offset: usize) -> String {
    std::fs::read_to_string(path).unwrap()
}
