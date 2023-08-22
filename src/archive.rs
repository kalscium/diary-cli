use lazy_db::*;
use crate::home_dir;
use soulog::*;
use std::path::PathBuf;

pub struct Archive {
    database: LazyDB,
    uid: u64,
    itver: u16,
}

impl Archive {
    /// Initialises a new archive, will throw error if one already exists
    pub fn init(mut logger: impl Logger) -> Self {
        let path = home_dir().join("archive");
        let path_string = path.to_string_lossy();
        // Check if archive already exists
        if !path.exists() {
            log!((logger.error) Archive("Archive '{path_string}' already exists, try wiping it before initialising again") as Fatal);
            return logger.crash()
        }

        log!((logger) Archive("Initialising a new archive at '{path_string}'..."));
        let database = if_err!((logger) [Archive, err => ("While initialising database: {err:?}")] retry LazyDB::init(&path));
        
        let uid = {
            use std::collections::hash_map::RandomState;
            use std::hash::{BuildHasher, Hasher};
            RandomState::new().build_hasher().finish()
        };
        let itver = 0u16;

        log!((logger) Archive("Writing uid and itver to archive..."));
        if_err!((logger) [Archive, err => ("While writing uid: {err:?}")] retry write_database!((&database) uid = new_u64(uid)));
        if_err!((logger) [Archive, err => ("While writing itver: {err:?}")] retry write_database!((&database) itver = new_u16(itver)));

        log!((logger) Archive("Successfully initialised archive '{path_string}'"));
        Self {
            database,
            uid,
            itver,
        }
    }

    pub fn load(mut logger: impl Logger) -> Self {
        let path = home_dir().join("archive");
        let path_string = path.to_string_lossy();
        log!((logger) Archive("Loading archive '{path_string}'..."));

        // Checks if path exists or not
        if !path.exists() {
            log!((logger.error) Archive("Archive '{path_string}' not found; initialising a new one...") as Inconvenience);
            return Self::init(logger)
        };

        let database = if_err!((logger) [Archive, err => ("While loading archive '{path_string}': {err:?}")] retry LazyDB::load_dir(&path));
        log!((logger) Archive("Loading uid and itver of archive..."));
        let uid = if_err!((logger) [Archive, err => ("While loading archive uid: {err:?}")] retry (|| search_database!((&database) uid)?.collect_u64())());
        let itver = if_err!((logger) [Archive, err => ("While loading archive itver: {err:?}")] retry (|| search_database!((&database) itver)?.collect_u16())());

        Self {
            database,
            uid,
            itver,
        }
    }

    pub fn backup(out_path: PathBuf, mut logger: impl Logger) {
        let path = home_dir().join("archive");
        let path_string = path.to_string_lossy();
        let out_string = out_path.to_string_lossy();
        
        log!((logger) Archive("Backing up archive '{path_string}' as '{out_string}'..."));
        let database = if_err!((logger) [Backup, err => ("While backing up archive: {err:?}")] retry LazyDB::load_dir(&path));
        if_err!((logger) [Backup, err => ("While backing up archive: {err:?}")] retry database.compile(&out_path));
        log!((logger) Archive("Successfully backed up archive '{path_string}' as '{out_string}'"));
        log!((logger) Archive(""));
    }
}