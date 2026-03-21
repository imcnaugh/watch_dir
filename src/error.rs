use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ErrorKind {
    Notify(notify::Error),
}

#[derive(Debug)]
pub struct WatchDirError {
    kind: ErrorKind,
}

impl Display for WatchDirError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::Notify(e) => write!(f, "notify error: {}", e),
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
