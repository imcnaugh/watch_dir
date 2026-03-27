mod common;

use crate::common::{DEFAULT_CHANNEL_RECV_TIMEOUT, DEFAULT_WATCHER_DEBOUNCE_DURATION};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::TryRecvError;
use watch_dir::TAIL_STRATEGY;

#[test]
fn tail_strategy_simple_test() {
    let dir = common::TestDir::new("tail_strategy_simple_test");
    let test_file_path = Path::new(dir.path()).join("test_file.txt");

    let options = watch_dir::Options::new()
        .with_read_strategy_selector(TAIL_STRATEGY)
        .with_notify_debounce_duration(DEFAULT_WATCHER_DEBOUNCE_DURATION);
    let mut watcher = watch_dir::Watcher::new(dir.path(), options).unwrap();
    let rx = watcher.take_receiver().unwrap();

    let create_test_file_handle = File::create(&test_file_path).unwrap();
    drop(create_test_file_handle);

    assert!(matches!(
        rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT),
        Err(RecvTimeoutError::Timeout)
    ));

    let mut test_file_handle = File::options().append(true).open(&test_file_path).unwrap();
    write!(test_file_handle, "test").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT).unwrap();
    assert_eq!(msg.1, "test");
    assert_eq!(
        test_file_path.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );

    assert!(matches!(
        rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT),
        Err(RecvTimeoutError::Timeout)
    ));

    let mut test_file_handle = File::options().append(true).open(&test_file_path).unwrap();
    write!(test_file_handle, " more text!").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT).unwrap();
    assert_eq!(msg.1, " more text!");
    assert_eq!(
        test_file_path.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));

    watcher.stop();
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
}

#[test]
fn no_message_sent_for_empty_file() {
    let dir = common::TestDir::new("no_message_sent_for_empty_file");
    let test_file_path = Path::new(dir.path()).join("test_file.txt");

    let create_test_file_handle = File::create(&test_file_path).unwrap();
    drop(create_test_file_handle);

    let options = watch_dir::Options::new()
        .with_read_strategy_selector(TAIL_STRATEGY)
        .with_notify_debounce_duration(DEFAULT_WATCHER_DEBOUNCE_DURATION);
    let mut watcher = watch_dir::Watcher::new(dir.path(), options).unwrap();
    let rx = watcher.take_receiver().unwrap();

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));

    let mut test_file_handle = File::options().append(true).open(&test_file_path).unwrap();
    write!(test_file_handle, "").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT);
    assert!(matches!(msg, Err(RecvTimeoutError::Timeout)));

    watcher.stop();
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
}

#[test]
fn offset_is_reset_when_file_is_smaller_then_current_offset() {
    let dir = common::TestDir::new("offset_is_reset_when_file_is_smaller_then_current_offset");
    let test_file_path = Path::new(dir.path()).join("test_file.txt");

    let create_test_file_handle = File::create(&test_file_path).unwrap();
    drop(create_test_file_handle);

    let mut test_file_handle = File::options().append(true).open(&test_file_path).unwrap();
    write!(test_file_handle, "quite a lot of text\nit just keeps going").unwrap();
    drop(test_file_handle);

    let options = watch_dir::Options::new()
        .with_read_strategy_selector(TAIL_STRATEGY)
        .with_notify_debounce_duration(DEFAULT_WATCHER_DEBOUNCE_DURATION);
    let mut watcher = watch_dir::Watcher::new(dir.path(), options).unwrap();
    let rx = watcher.take_receiver().unwrap();

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));

    let mut test_file_handle = File::options()
        .write(true)
        .truncate(true)
        .open(&test_file_path)
        .unwrap();
    write!(test_file_handle, "replacement text").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT).unwrap();
    assert!(msg.1.contains("replacement text"));
    assert_eq!(msg.0, test_file_path.to_path_buf().canonicalize().unwrap());

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));

    watcher.stop();
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
}
