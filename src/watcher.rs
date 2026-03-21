use crate::ReadStrategy;
use crate::error::WatchDirError;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

pub struct Watcher {
    notify_watcher: Debouncer<RecommendedWatcher, RecommendedCache>,
    rx: Option<Receiver<(PathBuf, String)>>,
    control_tx: Sender<Actions>,
    handle: std::thread::JoinHandle<()>,
}

pub struct Options {
    recursive: bool,
}

impl Watcher {
    pub fn new(
        path: &Path,
        read_strategy_selector: impl Fn(&Path) -> ReadStrategy + Send + 'static,
        options: Options,
    ) -> Result<Self, WatchDirError> {
        let (notify_tx, notify_rx) = mpsc::channel::<DebounceEventResult>();
        let (tx, rx) = mpsc::channel::<(PathBuf, String)>();
        let (control_tx, control_rx) = mpsc::channel::<Actions>();

        let mut debouncer = new_debouncer(Duration::from_secs(1), None, notify_tx)?;
        debouncer.watch(path, options.recursive_mode())?;

        let handle = std::thread::Builder::new()
            .name("watch_dir-rs Watcher".to_string())
            .spawn(move || run(notify_rx, tx, control_rx, read_strategy_selector));

        Ok(Self {
            notify_watcher: debouncer,
            rx: Some(rx),
            control_tx,
            handle: handle.unwrap(),
        })
    }

    pub fn take_receiver(&mut self) -> Option<Receiver<(PathBuf, String)>> {
        self.rx.take()
    }

    pub fn run(&self) {
        let _ = self.control_tx.send(Actions::Run);
    }

    pub fn pause(&self) {
        let _ = self.control_tx.send(Actions::Pause);
    }
}

fn run(
    notify_rx: Receiver<DebounceEventResult>,
    tx: Sender<(PathBuf, String)>,
    control_rx: Receiver<Actions>,
    read_strategy_selector: impl Fn(&Path) -> ReadStrategy + Send + 'static,
) {
    let mut paused = false;
    loop {
        while let Ok(action) = control_rx.try_recv() {
            match action {
                Actions::Pause => paused = true,
                Actions::Run => paused = false,
            }
        }

        if paused {
            std::thread::sleep(Duration::from_millis(50));
            continue;
        }

        match notify_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(event) => {
                if let Ok(event) = event {
                    event
                        .iter()
                        .filter(|e| e.kind.is_modify())
                        .flat_map(|e| &e.paths)
                        .for_each(|path| match read_strategy_selector(path) {
                            ReadStrategy::Tail => {}
                            ReadStrategy::TailLines => {}
                            ReadStrategy::Replace => {}
                            ReadStrategy::Ignore => {}
                        })
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
    }
}

impl Options {
    pub fn new() -> Self {
        Self { recursive: false }
    }

    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    fn recursive_mode(&self) -> RecursiveMode {
        match self.recursive {
            true => RecursiveMode::Recursive,
            false => RecursiveMode::NonRecursive,
        }
    }
}

enum Actions {
    Run,
    Pause,
}
