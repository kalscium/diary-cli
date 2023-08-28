use clap::*;
use crate::archive::Archive;
use crate::logger::DiaryLogger;
use crate::*;
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
        #[arg(short, long, required=false, help="Specifies the path that you want the backup file to be generated.")]
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
        #[arg(short, long, number_of_values=3, value_names=&["year", "month", "day"])]
        date: Option<Vec<u16>>,
        #[arg(short, long)]
        today: bool,
    },
    #[command(about="Pulls a entry or moc from the archive as toml in case you need to change something")]
    Pull {
        #[arg(short='m', long, help="Specifies if it is a moc or not (otherwise it is an entry).")]
        is_moc: bool,
        #[arg(short, long, required=true, help="The uid of the entry or moc.")]
        uid: String,
        #[arg(short, long, required=true, help="Specfies path of the containing folder of the config file.")]
        path: String,
        #[arg(short, long, default_value="config.toml", help="Specifies the name of the output config file.")]
        file_name: String,
    },
    #[command(about="Searches the archive with specified tags")]
    Search {
        #[arg(short, long, help="Sets if the search is strict or not (if the item must implement all tags)")]
        strict: bool,
        #[arg(short, long, required=true, value_delimiter=' ', num_args=1..)]
        tags: Vec<String>,
    },
    #[command(about="Exports the archive as an `Obsidian.md` vault.")]
    Export {
        #[arg(short, long, value_delimiter=' ', num_args=1.., help="Filters out entries and mocs that don't have all these tags")]
        tags: Option<Vec<String>>,
        #[arg(short, long, required=true, help="The path the `Obsidian.md` vault is going to be placed")]
        path: String,
    }
}

impl Commands {
    pub fn execute(self) {
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
            Since { date, today: _ } => since::since_2023(date, logger),
            Pull { is_moc, uid, path, file_name } => pull::pull(std::path::PathBuf::from(path), file_name, is_moc, uid, logger),
            Search { strict, tags } => search::search_command(strict, tags, logger),
            Export { .. } => todo!(),
        }
    }
}

pub fn run() {
    let args = Cli::parse();
    args.command.execute();
}