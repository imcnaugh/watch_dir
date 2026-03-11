use std::path::{Path, PathBuf};

pub const TEST_DIR_PATH: &'static str = "tests";

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
