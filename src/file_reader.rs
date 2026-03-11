use crate::folder_watcher::FolderWatcher;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub struct FileReader {
    rx: Option<Receiver<(PathBuf, String)>>,
    _folder_watcher: FolderWatcher,
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

impl FileReader {
    pub fn new(
        path: &Path,
        read_strategy_selector: impl Fn(&Path) -> ReadStrategy + Send + 'static,
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

        let mut folder_watcher =
            FolderWatcher::new(path).map_err(|e| Error::new(ErrorKind::Other, e))?;
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
        read_strategy_selector: impl Fn(&Path) -> ReadStrategy + Send + 'static,
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
}
