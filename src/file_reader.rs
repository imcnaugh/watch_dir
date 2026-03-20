use crate::folder_watcher::FolderWatcher;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::thread;

pub struct Watcher {
    rx: Option<Receiver<(PathBuf, String)>>,
    folder_watcher: FolderWatcher,
    control_tx: mpsc::Sender<Actions>,
    handle: Option<std::thread::JoinHandle<()>>,
}

#[derive(Debug)]
pub enum ReadStrategy {
    Tail,      // emit whatever new bytes arrive
    TailLines, // buffer until newline, emit complete lines only
    Replace,   // read the whole file on each change
    Ignore,    // ignore the file
}

pub const REPLACE_STRATEGY: fn(&Path) -> ReadStrategy = |_| ReadStrategy::Replace;
pub const TAIL_STRATEGY: fn(&Path) -> ReadStrategy = |_| ReadStrategy::Tail;
pub const TAIL_LINES_STRATEGY: fn(&Path) -> ReadStrategy = |_| ReadStrategy::TailLines;

enum Actions {
    Run,
    Stop,
    Pause,
}

impl Watcher {
    pub fn new(
        path: &Path,
        read_strategy_selector: impl Fn(&Path) -> ReadStrategy + Send + 'static,
        options: crate::Options,
    ) -> Result<Self, Error> {
        let offsets = std::fs::read_dir(path)?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .filter(|path| {
                matches!(
                    read_strategy_selector(path),
                    ReadStrategy::Tail | ReadStrategy::TailLines
                )
            })
            .flat_map(|path| {
                let path = path.canonicalize()?;
                let f = File::open(&path)?;
                let len = f.metadata()?.len();
                Ok::<(PathBuf, u64), Error>((path, len))
            })
            .collect::<HashMap<PathBuf, u64>>();

        let (control_tx, control_rx) = mpsc::channel::<Actions>();

        let mut folder_watcher = FolderWatcher::new(path, options).map_err(Error::other)?;
        let watcher_rx = folder_watcher.take_receiver().unwrap();

        let (tx, rx) = mpsc::channel::<(PathBuf, String)>();

        let handle = thread::Builder::new()
            .name("watch_dir-rs File Reader".to_string())
            .spawn(move || {
                run(
                    watcher_rx,
                    tx,
                    control_rx,
                    offsets,
                    HashMap::new(),
                    read_strategy_selector,
                )
            })?;

        Ok(Self {
            rx: Some(rx),
            folder_watcher,
            control_tx,
            handle: Some(handle),
        })
    }

    pub fn take_receiver(&mut self) -> Option<Receiver<(PathBuf, String)>> {
        self.rx.take()
    }

    pub fn run(&self) {
        self.folder_watcher.run();
        let _ = self.control_tx.send(Actions::Run);
    }

    pub fn pause(&self) {
        self.folder_watcher.pause();
        let _ = self.control_tx.send(Actions::Pause);
    }

    pub fn stop(&mut self) {
        self.folder_watcher.stop();
        let _ = self.control_tx.send(Actions::Stop);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn run(
    watcher_rx: Receiver<PathBuf>,
    tx: Sender<(PathBuf, String)>,
    control_rx: Receiver<Actions>,
    mut journal_offsets: HashMap<PathBuf, u64>,
    mut journal_file_buffer: HashMap<PathBuf, String>,
    read_strategy_selector: impl Fn(&Path) -> ReadStrategy + Send + 'static,
) {
    let mut paused = false;

    loop {
        while let Ok(action) = control_rx.try_recv() {
            match action {
                Actions::Stop => return,
                Actions::Pause => paused = true,
                Actions::Run => paused = false,
            }
        }

        if paused {
            thread::sleep(std::time::Duration::from_millis(50));
            continue;
        }

        match watcher_rx.recv_timeout(std::time::Duration::from_millis(50)) {
            Ok(event) => {
                if let Err(e) = match read_strategy_selector(&event) {
                    ReadStrategy::Tail => tail_strategy(&mut journal_offsets, event, &tx),
                    ReadStrategy::TailLines => tail_lines_strategy(
                        &mut journal_offsets,
                        &mut journal_file_buffer,
                        event,
                        &tx,
                    ),
                    ReadStrategy::Replace => replace_strategy(event, &tx),
                    ReadStrategy::Ignore => Ok(()),
                } {
                    eprintln!("error processing file event: {e}");
                }
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}

fn read_tail(
    journal_offsets: &mut HashMap<PathBuf, u64>,
    event: &PathBuf,
) -> Result<String, Error> {
    let offset = journal_offsets.get(event).copied().unwrap_or(0);

    let mut f = File::open(event)?;
    let current_file_length = f.metadata()?.len();

    // Reset offset if the file was truncated (e.g. log rotation)
    let offset = if current_file_length < offset {
        0
    } else {
        offset
    };

    f.seek(SeekFrom::Start(offset))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;

    journal_offsets.insert(event.clone(), current_file_length);
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

fn tail_strategy(
    journal_offsets: &mut HashMap<PathBuf, u64>,
    event: PathBuf,
    tx: &Sender<(PathBuf, String)>,
) -> Result<(), Error> {
    let tail = read_tail(journal_offsets, &event)?;
    let _ = tx.send((event, tail));
    Ok(())
}

fn tail_lines_strategy(
    journal_offsets: &mut HashMap<PathBuf, u64>,
    journal_file_buffer: &mut HashMap<PathBuf, String>,
    event: PathBuf,
    tx: &Sender<(PathBuf, String)>,
) -> Result<(), Error> {
    let new_content = read_tail(journal_offsets, &event)?;

    let buf = journal_file_buffer.entry(event.clone()).or_default();
    buf.push_str(&new_content);

    while let Some(pos) = buf.find('\n') {
        let line: String = buf.drain(..pos).collect();
        buf.drain(..1); // consume the newline
        let line = line.trim_end_matches('\r'); // handle \r\n
        if !line.is_empty() {
            let _ = tx.send((event.clone(), line.to_string()));
        }
    }

    Ok(())
}

fn replace_strategy(event: PathBuf, tx: &Sender<(PathBuf, String)>) -> Result<(), Error> {
    let mut f = File::open(&event)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let string = String::from_utf8_lossy(&buf).into_owned();
    let _ = tx.send((event, string));
    Ok(())
}
