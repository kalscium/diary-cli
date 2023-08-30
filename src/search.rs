use soulog::*;
use crate::{archive::Archive, entry::Entry, moc::MOC};

pub trait Searchable {
    fn get_uid(&self) -> String;
    #[allow(clippy::ptr_arg)]
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

pub fn list_command(strict: bool, show_mocs: bool, show_entries: bool, filter: Option<Vec<String>>, mut logger: impl Logger) {
    let archive = Archive::load(logger.hollow());

    let mut entries = archive.list_entries(logger.hollow());
    let mut mocs = archive.list_mocs(logger.hollow());

    let filter = match filter {
        Some(x) => x,
        None => {
            log!((logger) List("Listing selected items..."));

            let tags = get_unique_tags(&mut entries, &mut mocs, logger.hollow());
            log!((logger) List("{}", colour_format![green("Tags"), blue(": "), none(&format!("{tags:#?}"))]));

            std::mem::drop(tags);

            let entry_uids: Vec<String> = entries.into_iter().map(|e| e.uid).collect();
            let moc_uids: Vec<String> = mocs.into_iter().map(|m| m.uid).collect();

            if show_entries {
                log!((logger) List("{}", colour_format![green("Entries"), blue(": "), none(&format!("{entry_uids:#?}"))]));
            } if show_mocs {
                log!((logger) List("{}", colour_format![green("MOCs"), blue(": "), none(&format!("{moc_uids:#?}"))]));
            } return;
        }
    };

    let entry_uids: Vec<String>;
    let moc_uids: Vec<String>;

    if strict {
        log!((logger) List("Searching strictly with tags {filter:?} in mocs and entries..."));
        entry_uids = search_strict(&filter, entries, logger.hollow());
        moc_uids = search_strict(&filter, mocs, logger.hollow());
    } else {
        log!((logger) List("Searching with tags {filter:?} in mocs and entries..."));
        entry_uids = search(&filter, entries, logger.hollow());
        moc_uids = search(&filter, mocs, logger.hollow());
    }

    log!((logger) List("Listing found entries and mocs..."));

    log!((logger) List("{}", colour_format![green("Tags"), blue(": "), none(&format!("{filter:?}"))]));

    if show_entries {
        log!((logger) List("{}", colour_format![green("Entries"), blue(": "), none(&format!("{entry_uids:?}"))]));
    } if show_mocs {
        log!((logger) List("{}", colour_format![green("MOCs"), blue(": "), none(&format!("{moc_uids:?}"))]));
    }
}

use std::collections::HashSet;
fn get_unique_tags<'a>(entries: &'a mut [Entry], mocs: &'a mut [MOC], logger: impl Logger) -> HashSet<&'a String> {
    let mut tags = HashSet::new();

    tags.extend(entries.iter_mut().flat_map(|x| x.tags(logger.hollow()).iter()));
    tags.extend(mocs.iter_mut().flat_map(|x| x.tags(logger.hollow()).iter()));

    tags
}