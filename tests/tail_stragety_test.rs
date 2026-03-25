mod common;

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;
use watch_dir::TAIL_STRATEGY;

#[test]
fn tail_strategy_simple_test() {
    let dir = common::TestDir::new("tail_strategy_simple_test");
    let test_file_path = Path::new(dir.path()).join("test_file.txt");

    let options = watch_dir::Options::new()
        .with_read_strategy_selector(TAIL_STRATEGY)
        .with_notify_debounce_duration(Duration::from_millis(50));
    let mut watcher = watch_dir::Watcher::new(dir.path(), options).unwrap();
    let create_test_file_handle = File::create(&test_file_path).unwrap();
    drop(create_test_file_handle);

    let rx = watcher.take_receiver().unwrap();
    assert!(matches!(
        rx.recv_timeout(Duration::from_millis(100)),
        Err(RecvTimeoutError::Timeout)
    ));

    let mut test_file_handle = File::options().append(true).open(&test_file_path).unwrap();
    write!(test_file_handle, "test").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(Duration::from_millis(100)).unwrap();
    assert_eq!(msg.1, "test");
    assert_eq!(
        test_file_path.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );

    assert!(matches!(
        rx.recv_timeout(Duration::from_millis(100)),
        Err(RecvTimeoutError::Timeout)
    ));

    let mut test_file_handle = File::options().append(true).open(&test_file_path).unwrap();
    write!(test_file_handle, " more text!").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(Duration::from_millis(100)).unwrap();
    assert_eq!(msg.1, " more text!");
    assert_eq!(
        test_file_path.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}
