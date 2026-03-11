pub const TEST_DIR_PATH: &'static str = "tests/test_dir";

pub struct TestDir;

impl TestDir {
    pub fn new() -> Self {
        std::fs::create_dir_all(TEST_DIR_PATH).unwrap();
        TestDir
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(TEST_DIR_PATH);
    }
}
