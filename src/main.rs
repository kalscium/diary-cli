use diary_cli::cli::run;
use diary_cli::archive::Archive;
use soulog::sbl::PanicLogger;
use soulog::*;

fn main() {
    let logger = PanicLogger::new();
    let archive = Archive::init(logger.hollow());
    archive.wipe(logger.hollow());
}
