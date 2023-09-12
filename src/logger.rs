use soulog::*;
use crate::cli::VERBOSE;

pub struct DynamicLogger {
    verbose: Option<Verbose>,
    quiet: Option<Quiet>,
}

macro_rules! one_or_other {
    ($name:ident($one:expr, $two:expr) $code:block) => {'code: {
        if let Some($name) = &mut $one {
            break 'code ($code);
        }
        if let Some($name) = &mut $two {
            break 'code ($code);
        }

        panic!("DynamicLogger of no logger type (should not happen)");
    }}
}

impl Logger for DynamicLogger {
    fn new() -> Self {
        unsafe { if VERBOSE {
            Self {
                verbose: Some(Verbose::new()),
                quiet: None,
            }
        } else {
            Self {
                verbose: None,
                quiet: Some(Quiet::new()),
            }
        }}
    }

    fn hollow(&self) -> Self { Self::new() }

    fn crash<T>(&mut self) -> T {one_or_other!(x(self.verbose, self.quiet) {
        x.crash()
    })}

    fn verbose(&mut self, log: Log) {one_or_other!(x(self.verbose, self.quiet) {
        x.verbose(log)
    })}

    fn vital(&mut self, log: Log) {one_or_other!(x(self.verbose, self.quiet) {
        x.vital(log)
    })}

    fn error(&mut self, log: Log) -> ErrorResponse {one_or_other!(x(self.verbose, self.quiet) {
        x.error(log)
    })}
}

pub struct Verbose {
    retry_count: u8,
}

impl Logger for Verbose {
    fn new() -> Self { Self { retry_count: 2 } }
    fn hollow(&self) -> Self { Self::new() }

    fn crash<T>(&mut self) -> T {
        let mut logger = Self::new();
        log!((logger.vital) Diary("if the fatal error occurred during any writing to the archive, the archive may be corrupted! If so, then use `diary-cli rollback` to roll-back to the latest backup (that was made before any modification of the archive") as Warning);
        std::process::exit(1)
    }

    fn verbose(&mut self, log: Log) {
        self.retry_count = 2;
        println!("{}", colour_format!(blue("["), cyan(log.origin), blue("] "), none(log.message)));
    }

    fn error(&mut self, log: Log) -> ErrorResponse {
        let message = match log.log_type {
            LogType::Failure => colour_format![blue("["), red(log.origin), blue("] "), red("Failure"), blue(": "), none(log.message)],
            LogType::Fatal => colour_format![blue("["), red(log.origin), blue("] "), red("Fatal"), blue(": "), none(log.message)],
            _ => panic!("meta error: invalid error log type '{:?}'", log.log_type),
        }; println!("{message}");

        if ErrorResponse::AskUser.allowed_in(&log) { return ErrorResponse::AskUser };
        if ErrorResponse::Retry.allowed_in(&log) && self.retry_count > 0 {
            self.retry_count -= 1;
            // wait for a bit
            std::thread::sleep(std::time::Duration::from_millis(800));
            return ErrorResponse::Retry;
        };

        ErrorResponse::Crash
    }

    fn vital(&mut self, log: Log) {
        let message = match log.log_type {
            LogType::Inconvenience => colour_format![blue("["), yellow(log.origin), blue("] "), yellow("Inconvenience"), blue(": "), none(log.message)],
            LogType::Warning => colour_format![blue("["), yellow(log.origin), blue("] "), yellow("Warning"), blue(": "), none(log.message)],
            LogType::Result => colour_format![blue("["), green("Result"), blue("] "), green(log.origin), blue(": "), none(log.message)],
            LogType::Log => colour_format!(blue("["), cyan(log.origin), blue("] "), none(log.message)),
            _ => panic!("meta error: invalid error log type '{:?}'", log.log_type),
        }; println!("{message}");
    }
}

pub struct Quiet {
    retry_count: u8,
}

impl Logger for Quiet {
    fn new() -> Self { Self { retry_count: 2 } }
    fn hollow(&self) -> Self { Self::new() }

    fn crash<T>(&mut self) -> T {
        let mut logger = Self::new();
        log!((logger.vital) Diary("The archive may now be corrupted! Use `diary-cli rollback` to roll-back to the latest backup (that was made before any modification of the archive") as Warning);
        std::process::exit(1)
    }

    fn verbose(&mut self, _: Log) {
        self.retry_count = 2;
    }

    fn error(&mut self, log: Log) -> ErrorResponse {
        let message = match log.log_type {
            LogType::Failure => colour_format![blue("["), red(log.origin), blue("] "), red("Failure"), blue(": "), none(log.message)],
            LogType::Fatal => colour_format![blue("["), red(log.origin), blue("] "), red("Fatal"), blue(": "), none(log.message)],
            _ => panic!("meta error: invalid error log type '{:?}'", log.log_type),
        }; println!("{message}");

        if ErrorResponse::AskUser.allowed_in(&log) { return ErrorResponse::AskUser };
        if ErrorResponse::Retry.allowed_in(&log) && self.retry_count > 0 {
            self.retry_count -= 1;
            // wait for a bit
            std::thread::sleep(std::time::Duration::from_millis(800));
            return ErrorResponse::Retry;
        };

        ErrorResponse::Crash
    }

    fn vital(&mut self, log: Log) {
        let message = match log.log_type {
            LogType::Inconvenience => colour_format![blue("["), yellow(log.origin), blue("] "), yellow("Inconvenience"), blue(": "), none(log.message)],
            LogType::Warning => colour_format![blue("["), yellow(log.origin), blue("] "), yellow("Warning"), blue(": "), none(log.message)],
            LogType::Result => colour_format![blue("["), green("Result"), blue("] "), green(log.origin), blue(": "), none(log.message)],
            LogType::Log => colour_format!(blue("["), cyan(log.origin), blue("] "), none(log.message)),
            _ => panic!("meta error: invalid error log type '{:?}'", log.log_type),
        }; println!("{message}");
    }
}