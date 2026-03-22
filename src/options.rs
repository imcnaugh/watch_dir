use crate::SelectStrategy;
use notify::RecursiveMode;

pub struct Options {
    pub(crate) recursive: bool,
    pub(crate) read_strategy_selector: Box<dyn SelectStrategy>,
}

impl Options {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    pub(crate) fn recursive_mode(&self) -> RecursiveMode {
        match self.recursive {
            true => RecursiveMode::Recursive,
            false => RecursiveMode::NonRecursive,
        }
    }

    pub fn with_read_strategy_selector(
        mut self,
        read_strategy_selector: impl SelectStrategy,
    ) -> Self {
        self.read_strategy_selector = Box::new(read_strategy_selector);
        self
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            recursive: false,
            read_strategy_selector: Box::new(crate::TAIL_STRATEGY),
        }
    }
}
