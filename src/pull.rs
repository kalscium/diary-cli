use soulog::*;
use std::path::PathBuf;
use crate::{archive::Archive, unwrap_opt};

pub fn pull(path: PathBuf, file_name: String, is_moc: bool, uid: String, logger: impl Logger) {
    let archive = Archive::load(logger.hollow());
    if is_moc {
        todo!()
    } else {
        pull_entry(archive, path, file_name, uid, logger)
    }
}

fn pull_entry(archive: Archive, path: PathBuf, file_name: String, uid: String, mut logger: impl Logger) {
    let error_msg = format!("Entry of uid '{uid}' not found in archive");
    let mut entry = unwrap_opt!((archive.get_entry(uid, logger.hollow())) with logger, format: Pull("{error_msg}"));
    std::mem::drop(error_msg);

    let map = entry.pull(&path, logger.hollow());
    let contents = if_err!((logger) [Pull, err => ("While encoding entry toml: {err:?}")] retry toml::to_string_pretty(&map));
    
    let path = path.join(file_name);
    if_err!((logger) [Pull, err => ("While writing toml to path '{}'", path.to_string_lossy())] retry std::fs::write(&path, &contents));
}