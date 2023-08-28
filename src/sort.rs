use soulog::*;
use lazy_db::*;
use crate::{list, archive::Archive};

pub fn younger(this: &[u16; 3], other: &[u16; 3]) -> bool {
    let this_date = this[0] * 10000 + this[1] * 100 + this[2];
    let other_date = other[0] * 10000 + other[1] * 100 + other[2];
    this_date < other_date
}

pub fn sort(mut logger: impl Logger) {
    // load archive
    let archive = Archive::load(logger.hollow());

    let unsorted = list::read(
        |x| x.collect_string(),
        &if_err!((logger) [Sort, err => ("While reading unsorted stack length: {err:?}")] retry search_database!((archive.database()) /order/unsorted)),
        logger.hollow(),
    );

    if unsorted.len() == 0 {
        log!((logger.error) Sort("No unsorted items on unsorted stack; doing nothing") as Inconvenience);
        return;
    }
}