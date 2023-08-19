mod isol;

use diary_cli::list::*;
use soulog::*;
use lazy_db::*;
use isol::*;

#[test]
fn list() {
    let tmp = new_env();
    let logger = sbl::PanicLogger::new();
    let path = tmp.get_path().join("Container");
    let og_list = [12u8, 32, 86, 75, 98, 128, 255];

    write(
        &og_list,
        |file, data| LazyData::new_u8(file, *data),
        LazyContainer::init(&path).unwrap(),
        logger.hollow(),
    );

    let new_list = read(
        |data| data.collect_u8(),
        LazyContainer::load(path).unwrap(),
        logger,
    );

    assert!(og_list.iter().enumerate().all(|(i, x)| *x == new_list[i]))
}