use notify::EventKind;
use notify::event::{DataChange, ModifyKind};
use notify::{Event, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;

pub struct FolderWatchHandle {
    _watcher: notify::RecommendedWatcher,
}

pub fn watch_folder_for_updates(
    path: &PathBuf,
    extensions_to_watch: HashSet<String>,
) -> (FolderWatchHandle, Receiver<PathBuf>) {
    let (notify_tx, notify_rx) = mpsc::channel::<notify::Result<Event>>();
    let (tx, rx) = mpsc::channel::<PathBuf>();

    let mut watcher = notify::recommended_watcher(notify_tx).unwrap();
    watcher.watch(path, RecursiveMode::NonRecursive).unwrap();

    std::thread::spawn(move || {
        for event in notify_rx {
            let evt = match event {
                Ok(evt) => evt,
                Err(e) => {
                    eprintln!("watch error: {:?}", e);
                    continue;
                }
            };

            if let EventKind::Modify(ModifyKind::Data(DataChange::Content)) = evt.kind {
                for path in evt.paths {
                    if path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext_str| extensions_to_watch.contains(ext_str))
                    {
                        let _ = tx.send(path);
                    }
                }
            }
        }
    });

    (FolderWatchHandle { _watcher: watcher }, rx)
}
