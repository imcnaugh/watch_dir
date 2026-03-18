use notify::EventKind;
use notify::event::ModifyKind;
use notify::{Event, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;
use crate::folder_watcher::Actions::{Pause, Run, Stop};

pub(crate) struct FolderWatcher {
    _watcher: notify::RecommendedWatcher,
    rx: Option<Receiver<PathBuf>>,
    control_tx: mpsc::Sender<Actions>,
    handle: Option<std::thread::JoinHandle<()>>,
}

enum Actions {
    Run,
    Stop,
    Pause,
}

impl FolderWatcher {
    pub fn new(path: &Path, options: crate::Options) -> Result<Self, notify::Error> {
        let (notify_tx, notify_rx) = mpsc::channel::<notify::Result<Event>>();
        let (tx, rx) = mpsc::channel::<PathBuf>();
        let (control_tx, control_rx) = mpsc::channel::<Actions>();

        let mut watcher = notify::recommended_watcher(notify_tx)?;
        let recursive_mode = match options.recursive {
            true => RecursiveMode::Recursive,
            false => RecursiveMode::NonRecursive,
        };
        watcher.watch(path, recursive_mode)?;

        let handle = std::thread::spawn(move || run(notify_rx, tx, control_rx));

        Ok(Self {
            _watcher: watcher,
            rx: Some(rx),
            control_tx,
            handle: Some(handle),
        })
    }

    pub fn take_receiver(&mut self) -> Option<Receiver<PathBuf>> {
        self.rx.take()
    }

    pub fn run(&self) {
        let _ = self.control_tx.send(Run);
    }

    pub fn pause(&self) {
        let _ = self.control_tx.send(Pause);
    }

    pub fn stop(&mut self) {
        let _ = self.control_tx.send(Stop);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn run(
    notify_rx: Receiver<notify::Result<Event>>,
    tx: mpsc::Sender<PathBuf>,
    control_rx: Receiver<Actions>,
) {
    let mut paused = false;

    loop {
        while let Ok(action) = control_rx.try_recv() {
            match action {
                Stop => return,
                Pause => paused = true,
                Run => paused = false,
            }
        }

        if paused {
            std::thread::sleep(Duration::from_millis(50));
            continue;
        }

        match notify_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(event) => {
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
                    for path in evt.paths {
                        let _ = tx.send(path);
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}
