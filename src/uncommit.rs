use std::fs;
use lazy_db::*;
use soulog::*;
use crate::{archive::Archive, home_dir, list, sort};

pub fn uncommmit(uid: String, is_moc: bool, mut logger: impl Logger) {
    let archive = Archive::load(logger.hollow());
    
    let path = if is_moc {
        archive.database().path().join("mocs").join(&uid)
    } else {
        archive.database().path().join("entries").join(&uid)
    };
    
    // Check if path exists
    if !path.is_dir() {
        println!("{path:?}");
        if is_moc {
            log!((logger.error) Remove("moc of uid '{uid}' doesn't exist") as Fatal);
        } else {
            log!((logger.error) Remove("entry of uid '{uid}' doesn't exist") as Fatal);
        } return logger.crash();
    }

    // Confirm with the user about the action
    let expected = "mhm, yep, I do wanna remove this entry/moc permanently";
    log!((logger.vital) Remove("To confirm with removing an entry/moc of uid '{uid}' PERMANENTLY enter the phrase below (without quotes):") as Log);
    if_err!((logger) [Remove, err => ("Entered phrase incorrect, please retry")] retry {
        log!((logger.vital) Remove("\"{expected}\"") as Log);
        let input = logger.ask("Remove", "Enter the phrase");
        if &input[0..input.len() - 1] != expected { Err(()) }
        else { Ok(()) }
    });
    
    // Backup archive before modification
    log!((logger) Remove("Backing up archive before removal, if you want to revert back, run `diary-cli rollback -f`"));
    let _ = std::fs::remove_file(home_dir().join("backup.ldb")); // Clean up
    Archive::backup(home_dir().join("backup.ldb"), logger.hollow());

    log!((logger) Remove("Removing entry/moc of uid '{uid}'..."));

    // Remove the entry/moc
    sort::sort(logger.hollow());
    if_err!((logger) [Remove, err => ("While removing entry/moc from archive: {err:?}")] retry fs::remove_dir_all(&path));

    // Update order lists
    if is_moc { return; }

    let sorted_container = if_err!((logger) [Remove, err => ("While loading sorted list: {err:?}")] retry search_database!((archive.database()) /order/sorted));
    let sorted: Box<[String]> = sort::read_sorted(&archive, logger.hollow()).into_vec().into_iter().filter(|x| *x != uid).collect();

    list::write(&sorted, |f, x| LazyData::new_string(f, x), &sorted_container, logger.hollow());

    // Update itver
    log!((logger) Commit("Updating archive itver..."));
    if_err!((logger) [Commit, err => ("While update archive itver: {err:?}")] retry write_database!((archive.database()) itver = new_u16(archive.itver + 1)));

    log!((logger.vital) Remove("Successfully removed entry/moc of uid '{uid}'") as Log)
}