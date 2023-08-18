use clap::*;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Test,
}

impl Commands {
    pub fn execute(&self) {
        use Commands::*;
        match self {
            Test => println!("Hello, world!")
        }
    }
}

pub fn run() {
    let args = Cli::parse();
    args.command.execute();
}