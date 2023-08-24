use clap::*;
use crate::archive::Archive;
use crate::home_dir;
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
        #[arg(short, long, required=false, help="Specifies the path the backup will be placed")]
        file_path: Option<String>,
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
    },
    #[command(about="Rolls back to the last backed up archive")]
    Rollback,
}

impl Commands {
    pub fn execute(&self) {
        use Commands::*;
        let logger = DiaryLogger::new();
        match self {
            Test => println!("Hello, world!"),
            Init => {Archive::init(logger);},
            Wipe => Archive::load(logger.hollow()).wipe(logger),
            Backup { out_path } => Archive::backup(out_path, logger),
            Load { file_path } => Archive::load_backup(file_path, logger),
            Rollback => Archive::rollback(logger),
            Commit { file_path } => {
                match file_path {
                    Some(path) => Archive::load(logger.hollow()).commit(path, logger),
                    None => Archive::load(logger.hollow()).commit(home_dir().join("backup.ldb"), logger),
                }
            }
        }
    }
}

pub fn run() {
    let args = Cli::parse();
    args.command.execute();
}