pub struct LoggingConfiguration {
    pub verbosity_level: usize,
    pub quiet: bool,
}

impl Default for LoggingConfiguration {
    fn default() -> Self {
        Self {
            verbosity_level: 2,
            quiet: false,
        }
    }
}

pub fn init_logging(configuration: &LoggingConfiguration) {
    if configuration.quiet {
        stderrlog::new()
            .verbosity(configuration.verbosity_level)
            .quiet(configuration.quiet)
            .init()
            .unwrap();
    }
}

#[macro_export]
macro_rules! snob_debug {
    ($($arg:tt)*) => {
        // Log and prepend "snob" to the log message
        log::debug!("snob: {}", format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! snob_info {
    ($($arg:tt)*) => {
        // Log and prepend "snob" to the log message
        log::info!("snob: {}", format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! snob_warn {
    ($($arg:tt)*) => {
        // Log and prepend "snob" to the log message
        log::warn!("snob: {}", format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! snob_error {
    ($($arg:tt)*) => {
        // Log and prepend "snob" to the log message
        log::error!("snob: {}", format_args!($($arg)*))
    };
}
