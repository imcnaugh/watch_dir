use crate::common::DEFAULT_WATCHER_DEBOUNCE_DURATION;
use std::fs::File;
use std::io::Write;
use std::sync::mpsc::{RecvTimeoutError, TryRecvError};
use watch_dir::TAIL_LINES_STRATEGY;

mod common;

#[test]
fn tail_lines_strategy_simple_test() {
    let dir = common::TestDir::new("tail_lines_strategy_simple_test");
    let test_file_path = dir.path().join("test_file.txt");

    let options = watch_dir::Options::new()
        .with_read_strategy_selector(TAIL_LINES_STRATEGY)
        .with_notify_debounce_duration(DEFAULT_WATCHER_DEBOUNCE_DURATION);
    let mut watcher = watch_dir::Watcher::new(dir.path(), options).unwrap();
    let rx = watcher.take_receiver().unwrap();

    let create_test_file_handle = File::create(&test_file_path).unwrap();
    drop(create_test_file_handle);

    assert!(matches!(
        rx.recv_timeout(DEFAULT_WATCHER_DEBOUNCE_DURATION),
        Err(RecvTimeoutError::Timeout)
    ));

    let mut test_file_handle = File::options().write(true).open(&test_file_path).unwrap();
    test_file_handle.write_all(b"test\nmore text\n").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(DEFAULT_WATCHER_DEBOUNCE_DURATION).unwrap();
    assert_eq!(msg.1, "test");
    assert_eq!(
        test_file_path.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );
    let msg = rx.recv_timeout(DEFAULT_WATCHER_DEBOUNCE_DURATION).unwrap();
    assert_eq!(msg.1, "more text");
    assert_eq!(
        test_file_path.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));

    let mut test_file_handle = File::options().append(true).open(&test_file_path).unwrap();
    test_file_handle.write_all(b"not a complete line").unwrap();
    drop(test_file_handle);

    assert!(matches!(
        rx.recv_timeout(DEFAULT_WATCHER_DEBOUNCE_DURATION),
        Err(RecvTimeoutError::Timeout)
    ));

    let mut test_file_handle = File::options().append(true).open(&test_file_path).unwrap();
    test_file_handle.write_all(b" just yet\n").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(DEFAULT_WATCHER_DEBOUNCE_DURATION).unwrap();
    assert_eq!(msg.1, "not a complete line just yet");
    assert_eq!(
        test_file_path.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));

    watcher.stop();
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
}
