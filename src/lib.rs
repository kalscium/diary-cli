pub mod cli;
pub mod logger;
pub mod list;
pub mod entry;
pub mod archive;
pub mod moc;
pub mod since;
pub mod pull;
pub mod export;
pub mod search;
pub mod sort;
pub mod scribe;

pub use logger::*;

pub fn home_dir() -> std::path::PathBuf {
    // Linux only; change this if you want to go cross platform
    match std::env::var("HOME") {
        Ok(path) => std::path::Path::new(&path).join(".diary-cli"),
        Err(_) => std::path::PathBuf::from("/etc/diary-cli/"),
    }
}