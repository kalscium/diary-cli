use soulog::*;
use crate::archive::Archive;

pub trait Searchable {
    fn get_uid(&self) -> String;
    fn contains_tag(&mut self, tag: &String, logger: impl Logger) -> bool;
}

pub fn search_strict(tags: &[String], items: Vec<impl Searchable>, logger: impl Logger) -> Vec<String> {
    let mut result = Vec::new();
    for mut item in items.into_iter() {
        let mut all_tags_present = true;
        for tag in tags.iter() {
            if !item.contains_tag(tag, logger.hollow()) {
                all_tags_present = false;
                break;
            }
        } if all_tags_present {
            result.push(item.get_uid());
        }
    }

    result
}

pub fn search<T: Searchable>(tags: &[String], items: Vec<T>, logger: impl Logger) -> Vec<String> {
    let mut result = Vec::new();
    for mut item in items.into_iter() {
        for tag in tags {
            if item.contains_tag(tag, logger.hollow()) {
                result.push(item.get_uid());
                break;
            }
        }
    }
    result
}

pub fn search_command(strict: bool, tags: Vec<String>, mut logger: impl Logger) {
    let archive = Archive::load(logger.hollow());

    let entries = archive.list_entries(logger.hollow());
    let mocs = archive.list_mocs(logger.hollow());
    let entry_uids: Vec<String>;
    let moc_uids: Vec<String>;

    if strict {
        log!((logger) Search("Searching strictly with tags {tags:?} in mocs and entries..."));
        entry_uids = search_strict(&tags, entries, logger.hollow());
        moc_uids = search_strict(&tags, mocs, logger.hollow());
    } else {
        log!((logger) Search("Searching strictly with tags {tags:?} in mocs and entries..."));
        entry_uids = search(&tags, entries, logger.hollow());
        moc_uids = search(&tags, mocs, logger.hollow());
    }

    log!((logger) Search("Listing found entries and mocs..."));
    log!((logger) Search("{}", colour_format![green("MOCs"), blue(": "), none(&format!("{entry_uids:?}"))]));
    log!((logger) Search("{}", colour_format![green("Entries"), blue(": "), none(&format!("{moc_uids:?}"))]));
}