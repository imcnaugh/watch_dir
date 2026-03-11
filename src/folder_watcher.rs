use notify::EventKind;
use notify::event::ModifyKind;
use notify::{Event, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;

pub(crate) struct FolderWatcher {
    _watcher: notify::RecommendedWatcher,
    rx: Option<Receiver<PathBuf>>,
}

impl FolderWatcher {
    pub fn new(path: &Path) -> Result<Self, notify::Error> {
        let (notify_tx, notify_rx) = mpsc::channel::<notify::Result<Event>>();
        let (tx, rx) = mpsc::channel::<PathBuf>();

        let mut watcher = notify::recommended_watcher(notify_tx)?;
        watcher.watch(path, RecursiveMode::NonRecursive)?;

        std::thread::spawn(move || Self::run(notify_rx, tx));

        Ok(Self {
            _watcher: watcher,
            rx: Some(rx),
        })
    }

    pub fn take_receiver(&mut self) -> Option<Receiver<PathBuf>> {
        self.rx.take()
    }

    fn run(notify_rx: Receiver<notify::Result<Event>>, tx: mpsc::Sender<PathBuf>) {
        for event in notify_rx {
            let evt = match event {
                Ok(evt) => evt,
                Err(e) => {
                    eprintln!("watch error: {:?}", e);
                    continue;
                }
            };

            let is_modify = match evt.kind {
                #[cfg(target_os = "macos")]
                EventKind::Modify(ModifyKind::Data(_)) => true,
                #[cfg(target_os = "windows")]
                EventKind::Modify(ModifyKind::Any) => true,
                _ => false,
            };
            if is_modify {
                println!("{:?}", evt);
                for path in evt.paths {
                    let _ = tx.send(path);
                }
            }
        }
    }
}
