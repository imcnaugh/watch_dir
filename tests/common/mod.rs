use std::path::{Path, PathBuf};

pub const TEST_DIR_PATH: &'static str = "tests";
pub const DEFAULT_WATCHER_DEBOUNCE_DURATION: std::time::Duration =
    std::time::Duration::from_millis(500);
pub const DEFAULT_CHANNEL_RECV_TIMEOUT: std::time::Duration =
    std::time::Duration::from_millis(1000);

pub struct TestDir {
    folder: PathBuf,
}

impl TestDir {
    pub fn new(folder: &str) -> Self {
        let test_dir_path = Path::new(TEST_DIR_PATH).join(folder);
        std::fs::create_dir_all(test_dir_path.clone()).unwrap();
        TestDir {
            folder: test_dir_path,
        }
    }

    pub fn path(&self) -> &Path {
        &self.folder
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.folder.clone());
    }
}
