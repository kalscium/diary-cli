use soulog::*;

pub struct DiaryLogger {
    retry_count: u8,
}

impl Logger for DiaryLogger {
    fn new() -> Self { Self { retry_count: 2 } }
    fn hollow(&self) -> Self { Self::new() }

    fn crash<T>(&self) -> T {
        let mut logger = Self::new();
        log!((logger.error) Diary("if the fatal error occurred during any writing to the archive, the archive may be corrupted! If so, then use `diary-cli rollback` to roll-back to the latest backup (that was made before any modification of the archive") as Warning);
        std::process::exit(1)
    }

    fn log(&mut self, log: Log) {
        self.retry_count = 2;
        println!("{}", colour_format!(blue("["), cyan(log.origin), blue("] "), none(log.message)));
    }

    fn error(&mut self, log: Log) -> ErrorResponse {
        let message = match log.log_type {
            LogType::Log => panic!("meta error: invalid error log type 'Log'"),
            LogType::Inconvenience => colour_format![blue("["), yellow(log.origin), blue("] "), yellow("Inconvenience"), blue(": "), none(log.message)],
            LogType::Warning => colour_format![blue("["), yellow(log.origin), blue("]"), yellow("Warning"), blue(": "), none(log.message)],
            LogType::Failure => colour_format![blue("["), red(log.origin), blue("] "), red("Failure"), blue(": "), none(log.message)],
            LogType::Fatal => colour_format![blue("["), red(log.origin), blue("] "), red("Fatal"), blue(": "), none(log.message)],
        }; println!("{message}");

        if ErrorResponse::AskUser.allowed_in(&log) { return ErrorResponse::AskUser };
        if ErrorResponse::Retry.allowed_in(&log) && self.retry_count > 0 {
            self.retry_count -= 1;
            // wait for a bit
            std::thread::sleep(std::time::Duration::from_millis(500));
            return ErrorResponse::Retry;
        };

        ErrorResponse::Crash
    }
}