use crate::{Actions, ReadStrategy, SelectStrategy};
use notify::EventKind;
use notify::event::{CreateKind, DataChange, ModifyKind};
use notify_debouncer_full::DebounceEventResult;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

pub struct Worker {
    notify_rx: Receiver<DebounceEventResult>,
    tx: Sender<(PathBuf, String)>,
    control_rx: Receiver<Actions>,
    read_strategy_selector: Box<dyn SelectStrategy>,
    offsets: HashMap<PathBuf, u64>,
    line_buffers: HashMap<PathBuf, String>,
}

impl Worker {
    pub(crate) fn new(
        notify_rx: Receiver<DebounceEventResult>,
        tx: Sender<(PathBuf, String)>,
        control_rx: Receiver<Actions>,
        read_strategy_selector: Box<dyn SelectStrategy>,
        offsets: HashMap<PathBuf, u64>,
    ) -> Self {
        Self {
            notify_rx,
            tx,
            control_rx,
            read_strategy_selector,
            offsets,
            line_buffers: Default::default(),
        }
    }

    pub(crate) fn run(mut self) {
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
                            .filter(|&e| {
                                matches!(
                                    e.event.kind,
                                    EventKind::Create(CreateKind::File) | EventKind::Modify(_)
                                )
                            })
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
        let path = path.canonicalize()?;
        let offset = self.offsets.get(&path.to_path_buf()).copied().unwrap_or(0);

        let mut f = File::open(&path)?;
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
        if !tail.is_empty() {
            let _ = self.tx.send((path.to_path_buf(), tail));
        }
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

    fn replace_strategy(&self, path: &Path) -> Result<(), std::io::Error> {
        let mut f = File::open(path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let string = String::from_utf8_lossy(&buf).to_string();
        let _ = self.tx.send((path.to_path_buf(), string));
        Ok(())
    }
}
