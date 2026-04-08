mod common;

use crate::common::{DEFAULT_CHANNEL_RECV_TIMEOUT, DEFAULT_WATCHER_DEBOUNCE_DURATION};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::{RecvTimeoutError, TryRecvError};
use watch_dir;
use watch_dir::REPLACE_STRATEGY;

#[test]
fn replace_strategy_simple_test() {
    let dir = common::TestDir::new("replace_strategy_simple_test");
    let test_file_path = Path::new(dir.path()).join("test_file.txt");

    let options = watch_dir::Options::new()
        .with_read_strategy_selector(REPLACE_STRATEGY)
        .with_notify_debounce_duration(DEFAULT_WATCHER_DEBOUNCE_DURATION);
    let mut watcher = watch_dir::Watcher::new(Path::new(dir.path()), options).unwrap();
    let create_test_file_handle = File::create(&test_file_path).unwrap();
    drop(create_test_file_handle);

    let rx = watcher.take_receiver().unwrap();
    assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));

    let mut test_file_handle = File::options().write(true).open(&test_file_path).unwrap();
    write!(test_file_handle, "test").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT).unwrap();
    assert_eq!(msg.1, "test");
    assert_eq!(
        test_file_path.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));

    watcher.stop();
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
}

#[test]
fn replace_strategy_multiple_replace_test() {
    let dir = common::TestDir::new("replace_strategy_multiple_replace_test");
    let test_file_path = Path::new(dir.path()).join("test_file.txt");

    let options = watch_dir::Options::new()
        .with_read_strategy_selector(REPLACE_STRATEGY)
        .with_notify_debounce_duration(DEFAULT_WATCHER_DEBOUNCE_DURATION);

    let mut watcher = watch_dir::Watcher::new(Path::new(dir.path()), options).unwrap();
    let rx = watcher.take_receiver().unwrap();
    let create_test_file_handle = File::create(&test_file_path).unwrap();
    drop(create_test_file_handle);

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));

    let mut test_file_handle = File::options().write(true).open(&test_file_path).unwrap();
    write!(test_file_handle, "test").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT).unwrap();
    assert_eq!(msg.1, "test");
    assert_eq!(
        test_file_path.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );

    let mut test_file_handle = File::options().write(true).open(&test_file_path).unwrap();
    write!(test_file_handle, "test2").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT).unwrap();
    println!("{:?}", msg);
    assert_eq!(msg.1, "test2");
    assert_eq!(
        test_file_path.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );

    assert_eq!(
        rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT),
        Err(RecvTimeoutError::Timeout)
    );

    watcher.stop();
    assert_eq!(
        rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT),
        Err(RecvTimeoutError::Disconnected)
    );
}
