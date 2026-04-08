use std::fs::File;
use std::io::Write;
use std::time::Duration;
use std::{fs, thread};
use tempfile::tempdir;
use watch_dir::REPLACE_STRATEGY;

fn main() {
    let tmp_dir = tempdir().unwrap();
    let tmp_path = tmp_dir.path().to_path_buf();

    let options = watch_dir::Options::default().with_read_strategy_selector(REPLACE_STRATEGY);

    let mut watcher = watch_dir::Watcher::new(&tmp_dir.path(), options).unwrap();
    let tx = watcher.take_receiver().unwrap();

    thread::spawn(move || {
        let mut n = 1;
        let mut file_path = tmp_path.join(format!("file-{n:03}.txt"));

        loop {
            File::create(&file_path).unwrap();
            for i in 0..5 {
                let mut f = File::options()
                    .write(true)
                    .append(true)
                    .open(&file_path)
                    .unwrap();
                write!(f, "{i}").unwrap();
                drop(f);
                thread::sleep(Duration::from_secs(1));
            }

            n += 1;
            let target_path = tmp_path.join(format!("file-{n:03}.txt"));
            fs::rename(&file_path, &target_path).unwrap();
            file_path = target_path;
        }
    });

    for event in tx {
        let path = event.0;
        let content = event.1;
        println!("{}: {}", path.display(), content);
    }
}
