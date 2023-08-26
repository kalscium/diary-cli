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
        #[arg(index=1, required=true, help="The path to the entry config toml file to commit.")]
        file_path: String,
    },
    #[command(about="Backs up the archive")]
    Backup {
        #[arg(short, long, required=false, help="Specifies path that you want the backup file to be generated.")]
        out_path: Option<String>,
    },
    #[command(about="Loads a backed up archive")]
    Load {
        #[arg(index=1, required=true, help="The path of the backup file you want to load.")]
        file_path: String,
    },
    #[command(about="Rolls back to the last backed up archive")]
    Rollback,
    #[command(about="Returns the days since 2020 from a specified date")]
    Since {
        #[arg(index=1, required=true)]
        year: u16,
        #[arg(index=2, required=true)]
        month: u16,
        #[arg(index=2, required=true)]
        day: u16,
    },
}

impl Commands {
    pub fn execute(&self) {
        use Commands::*;
        let logger = DiaryLogger::new();
        match self {
            Test => println!("Hello, world!"),
            Init => {Archive::init(logger);},
            Wipe => Archive::load(logger.hollow()).wipe(logger),
            Commit { file_path } => Archive::load(logger.hollow()).commit(file_path, logger),
            Load { file_path } => Archive::load_backup(file_path, logger),
            Rollback => Archive::rollback(logger),
            Backup { out_path } => {
                match out_path {
                    Some(path) => Archive::backup(path, logger),
                    None => Archive::backup(home_dir().join("backup.ldb"), logger),
                }
            },
            Since { year, month, day } => todo!(),
        }
    }
}

pub fn run() {
    let args = Cli::parse();
    args.command.execute();
}