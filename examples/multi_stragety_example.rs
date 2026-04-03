use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use std::{fs, thread};
use tempfile::tempdir;
use watch_dir::ReadStrategy;

fn main() {
    let tmp_dir = tempdir().unwrap();
    let tmp_path = tmp_dir.path().to_path_buf();

    fn strategy(path: &Path) -> ReadStrategy {
        match path.extension().and_then(|e| e.to_str()) {
            Some("log") => ReadStrategy::TailLines,
            Some("json") => ReadStrategy::Replace,
            Some("txt") => ReadStrategy::Tail,
            _ => ReadStrategy::Ignore,
        }
    }

    let options = watch_dir::Options::default().with_read_strategy_selector(strategy);

    let mut watcher = watch_dir::Watcher::new(&tmp_dir.path(), options).unwrap();
    let tx = watcher.take_receiver().unwrap();

    thread::spawn(move || {
        let mut n = 1;
        let mut txt_file_path = tmp_path.join(format!("file-{n:03}.txt"));
        let mut json_file_path = tmp_path.join(format!("file-{n:03}.json"));
        let mut log_file_path = tmp_path.join(format!("file-{n:03}.log"));

        loop {
            File::create(&txt_file_path).unwrap();
            File::create(&json_file_path).unwrap();
            File::create(&log_file_path).unwrap();

            for i in 0..5 {
                let mut txt_file_handle = File::options()
                    .write(true)
                    .append(true)
                    .open(&txt_file_path)
                    .unwrap();
                write!(txt_file_handle, "{i}").unwrap();
                drop(txt_file_handle);

                let mut json_file_handle = File::options()
                    .write(true)
                    .open(&json_file_path)
                    .unwrap();
                write!(json_file_handle, "{{ \"id\": {i} }}").unwrap();
                drop(json_file_handle);

                let mut log_file_handle = File::options()
                    .write(true)
                    .append(true)
                    .open(&log_file_path)
                    .unwrap();
                write!(log_file_handle, "{i}").unwrap();
                if i == 4 {
                    write!(log_file_handle, "\n").unwrap();
                }
                drop(log_file_handle);

                thread::sleep(Duration::from_secs(1));
            }

            n += 1;
            let txt_target_path = tmp_path.join(format!("file-{n:03}.txt"));
            fs::rename(&txt_file_path, &txt_target_path).unwrap();
            txt_file_path = txt_target_path;

            let json_target_path = tmp_path.join(format!("file-{n:03}.json"));
            fs::rename(&json_file_path, &json_target_path).unwrap();
            json_file_path = json_target_path;

            let log_target_path = tmp_path.join(format!("file-{n:03}.log"));
            fs::rename(&log_file_path, &log_target_path).unwrap();
            log_file_path = log_target_path;
        }
    });

    for event in tx {
        let path = event.0;
        let content = event.1;
        println!("{}: {}", path.display(), content);
    }
}
