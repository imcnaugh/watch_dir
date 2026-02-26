use crate::folder_watcher::FolderWatcher;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Error, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

pub struct FileReader {
    rx: Option<mpsc::Receiver<String>>,
    folder_watcher: FolderWatcher,
}

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
    ) -> Result<Self, std::io::Error> {
        let offsets = std::fs::read_dir(path)?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .map(|path| {
                let f = std::fs::File::open(&path)?;
                let len = f.metadata()?.len();
                Ok::<(PathBuf, u64), std::io::Error>((path, len))
            })
            .flatten()
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

        let (tx, rx) = mpsc::channel::<String>();

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
            folder_watcher,
        })
    }

    pub fn take_receiver(&mut self) -> Option<mpsc::Receiver<String>> {
        self.rx.take()
    }

    fn run(
        watcher_rx: mpsc::Receiver<PathBuf>,
        tx: Sender<String>,
        mut journal_offsets: HashMap<PathBuf, u64>,
        mut journal_file_buffer: HashMap<PathBuf, String>,
        read_strategy_selector: impl Fn(&PathBuf) -> ReadStrategy + Send + 'static,
    ) {
        for event in watcher_rx {
            let result: std::io::Result<()> = (|| {
                let content: Vec<String> = match read_strategy_selector(&event) {
                    ReadStrategy::Tail => Self::tail_strategy(&mut journal_offsets, event)?,
                    ReadStrategy::TailLines => Self::tail_lines_strategy(
                        &mut journal_offsets,
                        &mut journal_file_buffer,
                        event,
                    )?,
                    ReadStrategy::Replace => Self::replace_strategy(event)?,
                    ReadStrategy::Ignore => Vec::new(),
                };

                for content in content {
                    let _ = tx.send(content);
                }

                Ok(())
            })();
        }
    }

    fn tail_strategy(
        journal_offsets: &mut HashMap<PathBuf, u64>,
        event: PathBuf,
    ) -> Result<Vec<String>, Error> {
        let offset = journal_offsets.get(&event).unwrap_or(&0);

        let mut f = File::open(&event)?;
        f.seek(SeekFrom::Start(*offset))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        let new_offset = offset + buf.len() as u64;
        journal_offsets.insert(event, new_offset);
        Ok(vec![String::from_utf8_lossy(&buf).to_string()])
    }

    fn tail_lines_strategy(
        journal_offsets: &mut HashMap<PathBuf, u64>,
        journal_file_buffer: &mut HashMap<PathBuf, String>,
        event: PathBuf,
    ) -> Result<Vec<String>, Error> {
        todo!()
    }

    fn replace_strategy(event: PathBuf) -> Result<Vec<String>, Error> {
        let mut f = File::open(&event)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        Ok(vec![String::from_utf8_lossy(&buf).to_string()])
    }
}
