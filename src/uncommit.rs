use std::fs;
use lazy_db::*;
use soulog::*;
use crate::{archive::Archive, home_dir};

pub fn uncommmit(uid: String, is_moc: bool, mut logger: impl Logger) {
    todo!();
    // Confirm with the user about the action
    let expected = "mhm, yep, I do wanna remove this entry/moc permanently";
    log!((logger.vital) Remove("To confirm with removing an entry/moc of uid '{uid}' PERMANENTLY enter the phrase below (without quotes):") as Log);
    if_err!((logger) [Remove, err => ("Entered phrase incorrect, please retry")] retry {
        log!((logger.vital) Remove("\"{expected}\"") as Log);
        let input = logger.ask("Remove", "Enter the phrase");
        if &input[0..input.len() - 1] != expected { Err(()) }
        else { Ok(()) }
    });

    let archive = Archive::load(logger.hollow());
    
    let path = if is_moc {
        archive.database().path().join("mocs").join(&uid)
    } else {
        archive.database().path().join("entries").join(&uid)
    };
    
    // Check if path exists
    if !path.exists() {
        if is_moc {
            log!((logger.error) Remove("moc of uid '{uid}' doesn't exist") as Fatal);
        } else {
            log!((logger.error) Remove("entry of uid '{uid}' doesn't exist") as Fatal);
        } return logger.crash();
    }
    
    // Backup archive before modification
    log!((logger) Remove("Backing up archive before removal, if you want to revert back, run `diary-cli rollback -f`"));
    let _ = std::fs::remove_file(home_dir().join("backup.ldb")); // Clean up
    Archive::backup(home_dir().join("backup.ldb"), logger.hollow());

    log!((logger) Remove("Removing entry/moc of uid '{uid}'..."));

    // Remove the entry/moc
    if_err!((logger) [Remove, err => ("While removing entry/moc from archive: {err:?}")] retry fs::remove_dir_all(&path));

    // Update order lists
    if is_moc { return; }

    let unsorted_container = if_err!((logger) [Remove, err => ("While loading unsorted list: {err:?}")] retry search_database!((archive.database()) /order/unsorted));
}