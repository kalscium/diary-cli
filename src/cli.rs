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
    #[command(about="A mere test command")]
    Test,
    #[command(about="Initialises a new archive")]
    Init,
    #[command(about="Wipes the archive")]
    Wipe,
    #[command(about="Commit an entry into the archive")]
    Commit {
        #[arg(index=1, required=true, help="The path to the entry config toml file to commit.")]
        file_path: String,
    },
    #[command(about="Backs up the arhive")]
    Backup {
        #[arg(index=1, required=true, help="The path that you want the backup file generated.")]
        out_path: String,
    },
    #[command(about="Loads a backed up archive")]
    Load {
        #[arg(index=1, required=true, help="The path of the backup file you want to load.")]
        file_path: String,
    }
}

impl Commands {
    pub fn execute(&self) {
        use Commands::*;
        let logger = DiaryLogger::new();
        match self {
            Test => println!("Hello, world!"),
            Init => {Archive::init(logger.hollow());},
            Wipe => Archive::load(logger.hollow()).wipe(logger.hollow()),
            Commit { file_path } => Archive::load(logger.hollow()).commit(file_path, logger.hollow()),
            Backup { out_path } => Archive::backup(out_path, logger.hollow()),
            Load { file_path } => {Archive::load_backup(file_path, logger.hollow());}
        }
    }
}

pub fn run() {
    let args = Cli::parse();
    args.command.execute();
}