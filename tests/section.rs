mod isol;

use soulog::*;
use lazy_db::*;
use isol::*;
use diary_cli::entry::Section;
use std::fs;
use toml::Table;

#[test]
fn isol_section() {
    // Setup
    let tmp = new_env();
    let logger = sbl::PanicLogger::new();
    let path = tmp.get_path().join("Section");
    let container = LazyContainer::init(path).unwrap();
    fs::write("example-path.txt", "nothing").unwrap();
    let toml = "
        title = 'Example Title'
        path = 'example-path.txt'
        notes = [ 'note1', 'note2', 'note3' ]
    ";

    // Store
    let mut section = Section::new(
        toml.parse::<Table>().unwrap(),
        container,
        "example-entry.toml",
        0,
        logger.hollow(),
    );

    // Loading the stuff
    section.clear_cache();
    let title = section.title(logger.hollow()).clone();
    let path = section.path(logger.hollow()).clone();
    let notes = section.notes(logger.hollow());

    assert_eq!(title, "Example Title");
    assert_eq!(path, "example-path.txt");
    assert_eq!(notes, &vec![String::from("note1"), String::from("note2"), String::from("note3")].into_boxed_slice());
}