use crate::SelectStrategy;
use notify::RecursiveMode;
use std::time::Duration;

/// Configuration for a [`Watcher`](crate::Watcher).
///
/// Constructed via [`Options::new()`] or [`Options::default()`], then customized with the
/// builder methods. Defaults: [`TAIL_STRATEGY`](crate::TAIL_STRATEGY), non-recursive, 250ms debounce.
pub struct Options {
    pub(crate) recursive: bool,
    pub(crate) read_strategy_selector: Box<dyn SelectStrategy>,
    pub(crate) notify_debounce_duration: Duration,
}

impl Options {
    /// Creates a new `Options` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Watch subdirectories recursively. Defaults to `false`.
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

    /// Sets the [`SelectStrategy`] used to choose a [`ReadStrategy`](crate::ReadStrategy) per file.
    ///
    /// Defaults to [`TAIL_STRATEGY`](crate::TAIL_STRATEGY).
    pub fn with_read_strategy_selector(
        mut self,
        read_strategy_selector: impl SelectStrategy,
    ) -> Self {
        self.read_strategy_selector = Box::new(read_strategy_selector);
        self
    }

    /// Sets the debounce window for filesystem events. Defaults to 250ms.
    pub fn with_notify_debounce_duration(mut self, duration: Duration) -> Self {
        self.notify_debounce_duration = duration;
        self
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            recursive: false,
            read_strategy_selector: Box::new(crate::TAIL_STRATEGY),
            notify_debounce_duration: Duration::from_millis(250),
        }
    }
}
