mod common;

use std::path::Path;
use std::sync::mpsc::TryRecvError;
use watch_dir;
use watch_dir::{REPLACE_STRATEGY, ReadStrategy};

#[test]
fn replace_strategy_simple_test() {
    let dir = common::TestDir::new("replace_strategy_simple_test");

    let mut file_reader =
        watch_dir::FileReader::new(Path::new(dir.path()), REPLACE_STRATEGY).unwrap();

    let rx = file_reader.take_receiver().unwrap();
    assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));

    let test_file = Path::new(dir.path()).join("test_file.txt");
    std::fs::write(test_file.clone(), "test").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));

    let msg = rx.try_recv().unwrap();
    assert_eq!(msg.1, "test");
    assert_eq!(test_file.canonicalize().unwrap(), msg.0.canonicalize().unwrap());

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty))
}

#[test]
fn replace_strategy_multiple_replace_test() {
    let dir = common::TestDir::new("replace_strategy_multiple_replace_test");
    let test_file = Path::new(dir.path()).join("test_file.txt");

    std::fs::write(test_file.clone(), "").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("file created");

    let strat = |path: &Path| -> ReadStrategy {
        if let Some(extension) = path.extension() {
            if extension == "txt" {
                ReadStrategy::Replace
            } else {
                ReadStrategy::Ignore
            }
        } else {
            ReadStrategy::Ignore
        }
    };

    let mut file_reader = watch_dir::FileReader::new(Path::new(dir.path()), strat).unwrap();
    let rx = file_reader.take_receiver().unwrap();

    std::fs::write(test_file.clone(), "test").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));

    let msg = rx.recv().unwrap();
    println!("{:?}", msg);
    assert_eq!(msg.1, "test");
    assert_eq!(test_file.canonicalize().unwrap(), msg.0.canonicalize().unwrap());

    std::fs::write(test_file.clone(), "test2").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let msg = rx.recv().unwrap();
    println!("{:?}", msg);
    assert_eq!(msg.1, "test2");
    assert_eq!(test_file.canonicalize().unwrap(), msg.0.canonicalize().unwrap());

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}
