use notify::EventKind;
use notify::{Event, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;

pub struct FolderWatcher {
    _watcher: notify::RecommendedWatcher,
    rx: Option<Receiver<PathBuf>>,
}

impl FolderWatcher {
    pub fn new(path: &PathBuf, extensions_to_watch: HashSet<String>) -> Self {
        let (notify_tx, notify_rx) = mpsc::channel::<notify::Result<Event>>();
        let (tx, rx) = mpsc::channel::<PathBuf>();

        let mut watcher = notify::recommended_watcher(notify_tx).unwrap();
        watcher.watch(path, RecursiveMode::NonRecursive).unwrap();

        std::thread::spawn(move || Self::run(notify_rx, tx, extensions_to_watch));

        Self {
            _watcher: watcher,
            rx: Some(rx),
        }
    }

    pub fn take_receiver(&mut self) -> Option<Receiver<PathBuf>> {
        self.rx.take()
    }

    fn run(
        notify_rx: Receiver<notify::Result<Event>>,
        tx: mpsc::Sender<PathBuf>,
        extensions_to_watch: HashSet<String>,
    ) {
        for event in notify_rx {
            let evt = match event {
                Ok(evt) => evt,
                Err(e) => {
                    eprintln!("watch error: {:?}", e);
                    continue;
                }
            };

            if let EventKind::Modify(_) = evt.kind {
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
    }
}
