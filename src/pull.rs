use soulog::*;
use std::{path::PathBuf, fs};
use crate::{archive::Archive, unwrap_opt};

pub fn pull(path: PathBuf, file_name: String, is_moc: bool, uid: String, one_file: bool, mut logger: impl Logger) {
    let archive = Archive::load(logger.hollow());

    if_err!((logger) {fs::create_dir_all(&path)} else(err) {
        log!((logger.vital) Pull("While initialising path '{}': {err:?}; ignoring error...", path.to_string_lossy()) as Inconvenience) 
    });
    
    if is_moc {
        log!((logger) Pull("Pulling moc with uid '{uid}' from archive..."));
        pull_moc(archive, path, file_name, uid, logger.hollow());
    } else {
        log!((logger) Pull("Pulling entry with uid '{uid}' from archive..."));
        pull_entry(archive, path, file_name, uid, one_file, logger.hollow());
    }

    log!((logger.vital) Pull("Successfully pulled config file from archive") as Log);
}

fn pull_entry(archive: Archive, path: PathBuf, file_name: String, uid: String, one_file: bool, mut logger: impl Logger) {
    let error_msg = format!("Entry of uid '{uid}' not found in archive");
    let mut entry = unwrap_opt!((archive.get_entry(uid, logger.hollow())) with logger, format: Pull("{error_msg}"));
    std::mem::drop(error_msg);

    let map = entry.pull(&path, one_file, logger.hollow());
    let contents = if_err!((logger) [Pull, err => ("While encoding entry toml: {err:?}")] retry toml::to_string_pretty(&map));
    
    let path = path.join(file_name);
    if_err!((logger) [Pull, err => ("While writing toml to path '{}'", path.to_string_lossy())] retry std::fs::write(&path, &contents));
}

fn pull_moc(archive: Archive, path: PathBuf, file_name: String, uid: String, mut logger: impl Logger) {
    let error_msg = format!("moc of uid '{uid}' not found in archive");
    let mut moc = unwrap_opt!((archive.get_moc(uid, logger.hollow())) with logger, format: Pull("{error_msg}"));
    std::mem::drop(error_msg);

    let map = moc.pull(logger.hollow());
    let contents = if_err!((logger) [Pull, err => ("While encoding moc toml: {err:?}")] retry toml::to_string_pretty(&map));
    
    let path = path.join(file_name);
    if_err!((logger) [Pull, err => ("While writing toml to path '{}'", path.to_string_lossy())] retry std::fs::write(&path, &contents));
}