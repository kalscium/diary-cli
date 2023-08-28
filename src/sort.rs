use soulog::*;
use lazy_db::*;
use crate::{list, archive::Archive};

pub fn older(this: &[u16; 3], other: &[u16; 3]) -> bool {
    let this_date = this[0] * 10000 + this[1] * 100 + this[2];
    let other_date = other[0] * 10000 + other[1] * 100 + other[2];
    this_date > other_date
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

    let mut sorted = list::read(
        |x| x.collect_string(),
        &if_err!((logger) [Sort, err => ("While reading unsorted stack length: {err:?}")] retry search_database!((archive.database()) /order/unsorted)),
        logger.hollow(),
    ).into_vec();

    log!((logger) Sort("Sorting unsorted entries..."));
    // main sorting code
    for usort_uid in unsorted.into_vec() {
        let mut usort = archive.get_entry(usort_uid.clone(), logger.hollow()).unwrap();
        let usort = usort.date(logger.hollow());

        for i in (0..sorted.len()).rev() {
            if usort_uid == sorted[i] { sorted.remove(i); } // remove duplicates

            let mut sort = archive.get_entry(sorted[i].clone(), logger.hollow()).unwrap();
            let sort = sort.date(logger.hollow());

            if older(usort, sort) { continue }; // if it's older skip the next part
            sorted.insert(i + 1, usort_uid); // replace the one before
            break;
        }
    }

    // Store updates
    list::write( // store newly sorted list
        sorted.as_ref(),
        |file, x| LazyData::new_string(file, x),
        &if_err!((logger) [Sort, err => ("While initing sorted list: {err:?}")] retry write_database!((archive.database()) /order/sorted)),
        logger.hollow(),
    );

    if_err!((logger) [Sort, err => ("While wiping unsorted stack: {err:?}")] retry
        if_err!((logger) [Sort, err => ("While wiping unsorted stack: {err:?}")] retry
        search_database!((archive.database()) /order/unsorted)
        ).wipe()
    );

    log!((logger) Sort("Successfully sorted entries"));
}