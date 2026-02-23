use crate::folder_watcher::FolderWatchHandle;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc;

pub struct FileReader {
    rx: mpsc::Receiver<String>,
    offsets: HashMap<PathBuf, u64>,
    folder_watch_handle: FolderWatchHandle,
}

impl FileReader {
    pub fn get_rx(&self) -> &mpsc::Receiver<String> {
        &self.rx
    }
}

pub fn start(path: &PathBuf) -> Result<FileReader, std::io::Error> {
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

    let (folder_watch_handle, watcher_rx) =
        crate::folder_watcher::watch_folder_for_updates(path, extensions);

    let (tx, rx) = mpsc::channel::<String>();

    std::thread::spawn(move || {
        for event in watcher_rx {
            let as_string = event.to_str().unwrap().to_string();
            let _ = tx.send(as_string);
        }
    });

    Ok(FileReader {
        rx,
        offsets,
        folder_watch_handle,
    })
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
    use crate::file_reader::start;
    use std::path::PathBuf;

    #[test]
    fn it_works() {
        let path = PathBuf::from("./src");
        start(&path);
    }

    #[test]
    fn it_works_too() {
        let idk = start(&PathBuf::from("./src")).unwrap();
        for e in idk.get_rx() {
            println!("{}", e);
        }
    }
}
