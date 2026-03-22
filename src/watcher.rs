use crate::ReadStrategy;
use crate::error::WatchDirError;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
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

pub trait SelectStrategy: Send + 'static {
    fn select(&self, path: &Path) -> ReadStrategy;
}

impl<F: Fn(&Path) -> ReadStrategy + Send + 'static> SelectStrategy for F {
    fn select(&self, path: &Path) -> ReadStrategy {
        self(path)
    }
}

pub struct Options {
    recursive: bool,
    read_strategy_selector: Box<dyn SelectStrategy>,
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
            options.recursive(),
            &*options.read_strategy_selector,
            &mut offsets,
        )?;

        let worker = Worker {
            notify_rx,
            tx,
            control_rx,
            read_strategy_selector: options.read_strategy_selector,
            offsets,
            line_buffers: Default::default(),
        };

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

struct Worker {
    notify_rx: Receiver<DebounceEventResult>,
    tx: Sender<(PathBuf, String)>,
    control_rx: Receiver<Actions>,
    read_strategy_selector: Box<dyn SelectStrategy>,
    offsets: HashMap<PathBuf, u64>,
    line_buffers: HashMap<PathBuf, String>,
}

impl Worker {
    fn run(mut self) {
        let mut paused = false;
        loop {
            while let Ok(action) = self.control_rx.try_recv() {
                match action {
                    Actions::Pause => paused = true,
                    Actions::Run => paused = false,
                    Actions::Stop => return,
                }
            }

            if paused {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }

            match self.notify_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(event) => {
                    if let Ok(event) = event {
                        event
                            .iter()
                            .filter(|e| e.kind.is_modify())
                            .flat_map(|e| &e.paths)
                            .for_each(|path| {
                                let _ = match self.read_strategy_selector.select(path) {
                                    ReadStrategy::Tail => self.tail_strategy(path),
                                    ReadStrategy::TailLines => self.tail_lines_strategy(path),
                                    ReadStrategy::Replace => self.replace_strategy(path),
                                    ReadStrategy::Ignore => Ok(()),
                                };
                            })
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    fn read_tail(&mut self, path: &Path) -> Result<String, std::io::Error> {
        let offset = self.offsets.get(path).copied().unwrap_or(0);

        let mut f = File::open(path)?;
        let current_file_length = f.metadata()?.len();

        let offset = if current_file_length < offset {
            0
        } else {
            offset
        };

        f.seek(SeekFrom::Start(offset))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        self.offsets.insert(path.to_path_buf(), current_file_length);
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    fn tail_strategy(&mut self, path: &Path) -> Result<(), std::io::Error> {
        let tail = self.read_tail(path)?;
        let _ = self.tx.send((path.to_path_buf(), tail));
        Ok(())
    }

    fn tail_lines_strategy(&mut self, path: &Path) -> Result<(), std::io::Error> {
        let new_content = self.read_tail(path)?;

        let buf = self.line_buffers.entry(path.to_path_buf()).or_default();
        buf.push_str(&new_content);

        while let Some(pos) = buf.find('\n') {
            let line: String = buf.drain(..pos).collect();
            buf.drain(..1);
            let line = line.trim_end_matches('\r');
            if !line.is_empty() {
                let _ = self.tx.send((path.to_path_buf(), line.to_string()));
            }
        }

        Ok(())
    }

    fn replace_strategy(&mut self, path: &Path) -> Result<(), std::io::Error> {
        let mut f = File::open(path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let string = String::from_utf8_lossy(&buf).to_string();
        let _ = self.tx.send((path.to_path_buf(), string));
        Ok(())
    }
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

impl Options {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    pub fn recursive(&self) -> bool {
        self.recursive
    }

    fn recursive_mode(&self) -> RecursiveMode {
        match self.recursive {
            true => RecursiveMode::Recursive,
            false => RecursiveMode::NonRecursive,
        }
    }

    pub fn with_read_strategy_selector(
        mut self,
        read_strategy_selector: impl SelectStrategy,
    ) -> Self {
        self.read_strategy_selector = Box::new(read_strategy_selector);
        self
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            recursive: false,
            read_strategy_selector: Box::new(crate::TAIL_STRATEGY),
        }
    }
}

enum Actions {
    Run,
    Pause,
    Stop,
}
