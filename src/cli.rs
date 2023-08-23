use clap::*;
use crate::archive::Archive;
use crate::logger::DiaryLogger;
use soulog::*;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Test,
    Init,
    Wipe,
}

impl Commands {
    pub fn execute(&self) {
        use Commands::*;
        let logger = DiaryLogger::new();
        match self {
            Test => println!("Hello, world!"),
            Init => {Archive::init(logger.hollow());},
            Wipe => Archive::load(logger.hollow()).wipe(logger.hollow()),
        }
    }
}

pub fn run() {
    let args = Cli::parse();
    args.command.execute();
}