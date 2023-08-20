mod isol;

use soulog::*;
use lazy_db::*;
use isol::*;
use diary_cli::entry::Section;

#[test]
fn isol_section() {
    let tmp = new_env();
    let logger = sbl::PanicLogger::new();
    let path = tmp.get_path().join("Section");

    let og_section = Section {
        title: String::from("Example Section"),
        notes: Box::new([String::from("note1"), String::from("note2"), String::from("note3")]),
        path: String::from("example-path.txt"),
    };

    og_section.store(LazyContainer::init(&path).unwrap(), logger.hollow());
    let new_section = Section::load(LazyContainer::load(path).unwrap(), logger);

    assert_eq!(og_section, new_section);
}