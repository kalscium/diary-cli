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

    /// Loads an archive at the cli's home
    #[inline]
    pub fn load(logger: impl Logger) -> Self {
        let path = home_dir().join("archive");
        Self::load_dir(path, logger)
    }

    /// Loads an archive at a specified path
    pub fn load_dir(path: PathBuf, mut logger: impl Logger) -> Self {
        let path_string = path.to_string_lossy();
        log!((logger) Archive("Loading archive '{path_string}'..."));

        // Checks if path exists or not
        if !path.is_dir() {
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

    /// Backs up home archive to specified path
    pub fn backup(out_path: PathBuf, mut logger: impl Logger) {
        let path = home_dir().join("archive");
        let path_string = path.to_string_lossy();
        let out_string = out_path.to_string_lossy();
        
        log!((logger) Archive("Backing up archive '{path_string}' as '{out_string}'..."));
        let database = if_err!((logger) [Archive, err => ("While backing up archive: {err:?}")] retry LazyDB::load_dir(&path));
        if_err!((logger) [Archive, err => ("While backing up archive: {err:?}")] retry database.compile(&out_path));
        log!((logger) Archive("Successfully backed up archive '{path_string}' as '{out_string}'"));
        log!((logger) Archive(""));
    }

    /// Loads a backup if that backup is the same as the active archive and or newer than the active archive, otherwise errors will be thrown
    pub fn load_backup(path: PathBuf, mut logger: impl Logger) -> Self {
        let archive = home_dir().join("archive");
        let archive_string = archive.to_string_lossy();
        let path_string = path.to_string_lossy();

        log!((logger) Archive("Loading archive backup '{path_string}'..."));

        // Check if backup exists
        if !path.is_file() {
            log!((logger.error) Archive("Backup file '{path_string}' does not exist") as Fatal);
            return logger.crash();
        }

        // Check if archive already exists
        if archive.is_dir() {
            log!((logger) Archive("Detected that there is already a loaded archive at '{archive_string}'"));
            let old = Archive::load(logger.hollow()); // Loads old archive

            // Load new archive
            let new = home_dir().join("new");
            if_err!((logger) [Archive, err => ("While decompiling backup '{path_string}': {err:?}")] retry LazyDB::decompile(&path, &new));
            let new = Archive::load_dir(new, logger.hollow());

            // Check if uid is the same and that the itver is higher
            if new.uid != old.uid {
                log!((logger.error) Archive("Cannot load backup as it is a backup of a different archive (uids don't match)") as Fatal);
                return logger.crash();
            }

            if !old.itver < new.itver {
                log!((logger.error) Archive("Cannot load backup as it is older than the currently loaded archive (itver is less)") as Fatal);
                return logger.crash();
            }
            
            let _ = std::fs::remove_dir_all(new.database.path()); // cleanup
            let _ = std::fs::remove_dir_all(&archive); // cleanup
        }

        if_err!((logger) [Archive, err => ("While decompiling backup '{path_string}': {err:?}")] retry LazyDB::decompile(&path, &archive));
        Self::load(logger)
    }


}