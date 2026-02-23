use crate::folder_watcher::FolderWatcher;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc;

pub struct FileReader {
    rx: Option<mpsc::Receiver<String>>,
    folder_watcher: FolderWatcher,
}

impl FileReader {
    pub fn new(path: &PathBuf) -> Result<Self, std::io::Error> {
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

        std::thread::spawn(move || Self::run(watcher_rx, tx, offsets));

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
        tx: mpsc::Sender<String>,
        offsets: HashMap<PathBuf, u64>,
    ) {
        for event in watcher_rx {
            let as_string = event.to_str().unwrap().to_string();
            let _ = tx.send(as_string);
        }
    }
}

/*

fn read_last_10_percent(path: &std::path::Path) -> io::Result<Vec<u8>> {
    let mut f = File::open(path)?;
    let len = f.metadata()?.len(); // bytes

    let start = (len * 90) / 100; // last 10%
    f.seek(SeekFrom::Start(start))?;

    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    Ok(buf)
}
 */

#[cfg(test)]
mod tests {
    use crate::file_reader::FileReader;
    use std::path::PathBuf;

    #[test]
    fn it_works() {
        let path = PathBuf::from("./src");
        FileReader::new(&path).unwrap();
    }

    #[test]
    fn it_works_too() {
        let mut reader = FileReader::new(&PathBuf::from("./src")).unwrap();
        for e in reader.take_receiver().unwrap() {
            println!("{}", e);
        }
    }
}
