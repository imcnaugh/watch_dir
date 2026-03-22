use crate::error::WatchDirError;
use crate::options::Options;
use crate::worker::Worker;
use crate::{ReadStrategy, SelectStrategy};
use notify::RecommendedWatcher;
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};
use std::collections::HashMap;
use std::fs::File;
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

impl Watcher {
    pub fn new(path: &Path, options: Options) -> Result<Self, WatchDirError> {
        let (notify_tx, notify_rx) = mpsc::channel::<DebounceEventResult>();
        let (tx, rx) = mpsc::channel::<(PathBuf, String)>();
        let (control_tx, control_rx) = mpsc::channel::<Actions>();

        let mut debouncer = new_debouncer(Duration::from_secs(1), None, notify_tx)?;
        debouncer.watch(path, options.recursive_mode())?;

        let mut offsets = HashMap::new();
        populate_offsets(
            path,
            options.recursive,
            &*options.read_strategy_selector,
            &mut offsets,
        )?;

        let worker = Worker::new(
            notify_rx,
            tx,
            control_rx,
            options.read_strategy_selector,
            offsets,
        );

        let handle = std::thread::Builder::new()
            .name("watch_dir-rs Watcher".to_string())
            .spawn(move || worker.run())?;

        Ok(Self {
            notify_watcher: debouncer,
            rx: Some(rx),
            control_tx,
            handle,
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

    pub fn stop(self) {
        let _ = self.control_tx.send(Actions::Stop);
        self.notify_watcher.stop();
        let _ = self.handle.join();
    }
}

pub(crate) enum Actions {
    Run,
    Pause,
    Stop,
}

fn populate_offsets(
    path: &Path,
    recursive: bool,
    read_strategy_selector: &dyn SelectStrategy,
    offsets: &mut HashMap<PathBuf, u64>,
) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(path)?.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() && recursive {
            populate_offsets(&entry_path, recursive, read_strategy_selector, offsets)?;
        } else if entry_path.is_file() {
            let read_strategy = read_strategy_selector.select(&entry_path);
            if matches!(read_strategy, ReadStrategy::Tail | ReadStrategy::TailLines) {
                let canonical = entry_path.canonicalize()?;
                let len = File::open(&canonical)?.metadata()?.len();
                offsets.insert(canonical, len);
            }
        }
    }
    Ok(())
}
