use crate::folder_watcher::FolderWatcher;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Error, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub struct FileReader {
    rx: Option<mpsc::Receiver<(PathBuf, String)>>,
    _folder_watcher: FolderWatcher,
}

#[derive(Debug)]
pub enum ReadStrategy {
    Tail,      // emit whatever new bytes arrive
    TailLines, // buffer until newline, emit complete lines only
    Replace,   // read the whole file on each change
    Ignore,    // ignore the file
}

pub const REPLACE_STRATEGY: fn(&PathBuf) -> ReadStrategy = |_| ReadStrategy::Replace;
pub const TAIL_STRATEGY: fn(&PathBuf) -> ReadStrategy = |_| ReadStrategy::Tail;
pub const TAIL_LINES_STRATEGY: fn(&PathBuf) -> ReadStrategy = |_| ReadStrategy::TailLines;

impl FileReader {
    pub fn new(
        path: &PathBuf,
        read_strategy_selector: impl Fn(&PathBuf) -> ReadStrategy + Send + 'static,
    ) -> Result<Self, Error> {
        let offsets = std::fs::read_dir(path)?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .flat_map(|path| {
                let path = path.canonicalize()?;
                let f = File::open(&path)?;
                let len = f.metadata()?.len();
                Ok::<(PathBuf, u64), Error>((path, len))
            })
            .collect::<HashMap<PathBuf, u64>>();

        let extensions = offsets
            .keys()
            .filter_map(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_owned())
            })
            .collect::<HashSet<String>>();

        let mut folder_watcher = FolderWatcher::new(path, extensions);
        let watcher_rx = folder_watcher.take_receiver().unwrap();

        let (tx, rx) = mpsc::channel::<(PathBuf, String)>();

        std::thread::spawn(move || {
            Self::run(
                watcher_rx,
                tx,
                offsets,
                HashMap::new(),
                read_strategy_selector,
            )
        });

        Ok(Self {
            rx: Some(rx),
            _folder_watcher: folder_watcher,
        })
    }

    pub fn take_receiver(&mut self) -> Option<Receiver<(PathBuf, String)>> {
        self.rx.take()
    }

    fn run(
        watcher_rx: Receiver<PathBuf>,
        tx: Sender<(PathBuf, String)>,
        mut journal_offsets: HashMap<PathBuf, u64>,
        mut journal_file_buffer: HashMap<PathBuf, String>,
        read_strategy_selector: impl Fn(&PathBuf) -> ReadStrategy + Send + 'static,
    ) {
        for event in watcher_rx {
            let strategy = read_strategy_selector(&event);
            if let Err(e) = match strategy {
                ReadStrategy::Tail => Self::tail_strategy(&mut journal_offsets, event, &tx),
                ReadStrategy::TailLines => Self::tail_lines_strategy(
                    &mut journal_offsets,
                    &mut journal_file_buffer,
                    event,
                    &tx,
                ),
                ReadStrategy::Replace => Self::replace_strategy(event, &tx),
                ReadStrategy::Ignore => Ok(()),
            } {
                eprintln!("error processing file event: {e}");
            }
        }
    }

    fn read_tail(
        journal_offsets: &mut HashMap<PathBuf, u64>,
        event: &PathBuf,
    ) -> Result<String, Error> {
        let offset = journal_offsets.get(event).unwrap_or(&0);

        let mut f = File::open(event)?;
        let current_file_length = f.metadata()?.len();
        f.seek(SeekFrom::Start(*offset))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        journal_offsets.insert(event.clone(), current_file_length);
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    fn tail_strategy(
        journal_offsets: &mut HashMap<PathBuf, u64>,
        event: PathBuf,
        tx: &Sender<(PathBuf, String)>,
    ) -> Result<(), Error> {
        let tail = Self::read_tail(journal_offsets, &event)?;
        let _ = tx.send((event, tail));
        Ok(())
    }

    fn tail_lines_strategy(
        journal_offsets: &mut HashMap<PathBuf, u64>,
        journal_file_buffer: &mut HashMap<PathBuf, String>,
        event: PathBuf,
        tx: &Sender<(PathBuf, String)>,
    ) -> Result<(), Error> {
        let new_content = Self::read_tail(journal_offsets, &event)?;

        let buf = journal_file_buffer.entry(event.clone()).or_default();
        buf.push_str(&new_content);

        while let Some(pos) = buf.find('\n') {
            let line = buf.drain(..=pos).collect::<String>();
            let line = line.trim_end_matches('\n').to_string();
            if !line.is_empty() {
                let _ = tx.send((event.clone(), line));
            }
        }

        Ok(())
    }

    fn replace_strategy(event: PathBuf, tx: &Sender<(PathBuf, String)>) -> Result<(), Error> {
        let mut f = File::open(&event)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let string = String::from_utf8_lossy(&buf).to_string();
        let _ = tx.send((event, string));
        Ok(())
    }
}
