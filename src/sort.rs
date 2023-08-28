use soulog::*;
use lazy_db::*;
use crate::{list, archive::Archive};

pub fn younger(this: &[u16; 3], other: &[u16; 3]) -> bool {
    let this_date = this[2] as u32 * 10000 + this[1] as u32 * 100 + this[0] as u32;
    let other_date = other[2] as u32 * 10000 + other[1] as u32 * 100 + other[0] as u32;
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
        let usort_date = *archive.get_entry(usort_uid.clone(), logger.hollow()).unwrap().date(logger.hollow());
        
        sorted.retain(|item| item != &usort_uid); // remove duplicates

        let mut is_oldest = true;
        for i in (0..sorted.len()).rev() {
            let sort_date = *archive.get_entry(sorted[i].clone(), logger.hollow()).unwrap().date(logger.hollow());

            if younger(&usort_date, &sort_date) {
                sorted.insert(i + 1, usort_uid.clone()); // replace the one before
                is_oldest = false;
                break;
            }
        } if is_oldest {
            sorted.insert(0, usort_uid);
        }
    }

    // Store updates
    log!((logger) Sort("Sorted list length: {}", sorted.len()));
    list::write( // store newly sorted list
        sorted.as_ref(),
        |file, x| LazyData::new_string(file, x),
        &if_err!((logger) [Sort, err => ("While initing sorted list: {err:?}")] retry search_database!((archive.database()) /order/sorted)),
        logger.hollow(),
    );

    list::write(
        &[],
        |file, x: &String| LazyData::new_string(file, x),
        &if_err!((logger) [Sort, err => ("While initing sorted list: {err:?}")] retry search_database!((archive.database()) /order/unsorted)),
        logger.hollow()
    );

    log!((logger) Sort("Successfully sorted entries"));
}