mod common;

use std::path::Path;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;
use watch_dir;
use watch_dir::REPLACE_STRATEGY;

#[test]
fn replace_strategy_simple_test() {
    let dir = common::TestDir::new("replace_strategy_simple_test");

    let options = watch_dir::Options::new()
        .with_read_strategy_selector(REPLACE_STRATEGY)
        .with_notify_debounce_duration(Duration::from_millis(50));
    let mut watcher = watch_dir::Watcher::new(Path::new(dir.path()), options).unwrap();

    let rx = watcher.take_receiver().unwrap();
    assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));

    let test_file = Path::new(dir.path()).join("test_file.txt");
    std::fs::write(test_file.clone(), "test").unwrap();
    std::thread::sleep(Duration::from_millis(100));

    let msg = rx.try_recv().unwrap();
    assert_eq!(msg.1, "test");
    assert_eq!(
        test_file.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty))
}

#[test]
fn replace_strategy_multiple_replace_test() {
    let dir = common::TestDir::new("replace_strategy_multiple_replace_test");
    let test_file = Path::new(dir.path()).join("test_file.txt");

    std::fs::write(test_file.clone(), "").unwrap();
    std::thread::sleep(Duration::from_millis(100));

    let options = watch_dir::Options::new()
        .with_read_strategy_selector(REPLACE_STRATEGY)
        .with_notify_debounce_duration(Duration::from_millis(50));

    let mut watcher = watch_dir::Watcher::new(Path::new(dir.path()), options).unwrap();
    let rx = watcher.take_receiver().unwrap();

    std::fs::write(test_file.clone(), "test").unwrap();
    std::thread::sleep(Duration::from_millis(100));

    let msg = rx.recv().unwrap();
    println!("{:?}", msg);
    assert_eq!(msg.1, "test");
    assert_eq!(
        test_file.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );

    std::fs::write(test_file.clone(), "test2").unwrap();
    std::thread::sleep(Duration::from_millis(100));
    let msg = rx.recv().unwrap();
    println!("{:?}", msg);
    assert_eq!(msg.1, "test2");
    assert_eq!(
        test_file.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}
