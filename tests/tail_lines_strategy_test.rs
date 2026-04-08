use crate::common::{DEFAULT_CHANNEL_RECV_TIMEOUT, DEFAULT_WATCHER_DEBOUNCE_DURATION};
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
        rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT),
        Err(RecvTimeoutError::Timeout)
    ));

    let mut test_file_handle = File::options().write(true).open(&test_file_path).unwrap();
    test_file_handle.write_all(b"test\nmore text\n").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT).unwrap();
    assert_eq!(msg.1, "test");
    assert_eq!(
        test_file_path.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );
    let msg = rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT).unwrap();
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
        rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT),
        Err(RecvTimeoutError::Timeout)
    ));

    let mut test_file_handle = File::options().append(true).open(&test_file_path).unwrap();
    test_file_handle.write_all(b" just yet\n").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT).unwrap();
    assert_eq!(msg.1, "not a complete line just yet");
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
    let test_file_path = dir.path().join("test_file.txt");

    let options = watch_dir::Options::new()
        .with_read_strategy_selector(TAIL_LINES_STRATEGY)
        .with_notify_debounce_duration(DEFAULT_WATCHER_DEBOUNCE_DURATION);
    let mut watcher = watch_dir::Watcher::new(dir.path(), options).unwrap();
    let rx = watcher.take_receiver().unwrap();

    let create_test_file_handle = File::create(&test_file_path).unwrap();
    drop(create_test_file_handle);

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));

    let mut test_file_handle = File::options().append(true).open(&test_file_path).unwrap();
    test_file_handle.write(b"\n").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT);
    assert!(matches!(msg, Err(RecvTimeoutError::Timeout)));

    watcher.stop();
}

#[test]
fn offset_is_reset_when_file_is_smaller_then_current_offset() {
    let dir = common::TestDir::new("offset_is_reset_when_file_is_smaller_then_current_offset");
    let test_file_path = dir.path().join("test_file.txt");

    let options = watch_dir::Options::new()
        .with_read_strategy_selector(TAIL_LINES_STRATEGY)
        .with_notify_debounce_duration(DEFAULT_WATCHER_DEBOUNCE_DURATION);
    let mut watcher = watch_dir::Watcher::new(dir.path(), options).unwrap();
    let rx = watcher.take_receiver().unwrap();

    let create_test_file_handle = File::create(&test_file_path).unwrap();
    drop(create_test_file_handle);

    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));

    let mut test_file_handle = File::options().write(true).open(&test_file_path).unwrap();
    test_file_handle
        .write_all(b"quite a lot of text it just keeps going longer and longer\n\n\n")
        .unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT).unwrap();
    assert_eq!(
        msg.1,
        "quite a lot of text it just keeps going longer and longer"
    );
    assert_eq!(
        test_file_path.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));

    let mut test_file_handle = File::options()
        .truncate(true)
        .write(true)
        .open(&test_file_path)
        .unwrap();
    test_file_handle.write_all(b"truncated\n").unwrap();
    drop(test_file_handle);

    let msg = rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT).unwrap();
    assert_eq!(msg.1, "truncated");
    assert_eq!(
        test_file_path.canonicalize().unwrap(),
        msg.0.canonicalize().unwrap()
    );
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));

    let msg = rx.recv_timeout(DEFAULT_CHANNEL_RECV_TIMEOUT);
    assert!(matches!(msg, Err(RecvTimeoutError::Timeout)));

    watcher.stop();
}
