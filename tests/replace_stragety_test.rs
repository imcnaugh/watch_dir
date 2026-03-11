mod common;

use std::path::Path;
use std::sync::mpsc::TryRecvError;
use watch_dir;
use watch_dir::REPLACE_STRATEGY;

#[test]
fn replace_strategy_test() {
    let _dir = common::TestDir::new();

    let mut file_reader =
        watch_dir::FileReader::new(Path::new(common::TEST_DIR_PATH), REPLACE_STRATEGY).unwrap();

    let rx = file_reader.take_receiver().unwrap();
    assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));

    let test_file = Path::new(common::TEST_DIR_PATH).join("test_file.txt");
    std::fs::write(test_file.clone(), "test").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));

    let msg = rx.try_recv().unwrap();
    assert_eq!(msg.1, "test");
    assert_eq!(test_file.canonicalize().unwrap(), msg.0);
}
