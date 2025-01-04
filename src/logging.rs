use crate::cli::Cli;

pub fn init_logging(cli: &Cli) {
    if !cli.quiet {
        stderrlog::new()
            .verbosity(cli.verbosity_level)
            .quiet(cli.quiet)
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
