use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ErrorKind {
    Notify(notify::Error),
    Io(std::io::Error),
}

#[derive(Debug)]
pub struct WatchDirError {
    kind: ErrorKind,
}

impl Display for WatchDirError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::Notify(e) => write!(f, "notify error: {}", e),
            ErrorKind::Io(e) => write!(f, "io error: {}", e),
        }
    }
}

impl From<notify::Error> for WatchDirError {
    fn from(e: notify::Error) -> Self {
        Self {
            kind: ErrorKind::Notify(e),
        }
    }
}

impl From<std::io::Error> for WatchDirError {
    fn from(e: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(e),
        }
    }
}
