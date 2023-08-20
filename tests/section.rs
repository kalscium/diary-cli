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
    let content = tmp.get_path().join("Section");
    let container = LazyContainer::init(&content).unwrap();
    let example_path = tmp.get_path().to_string_lossy().to_string() + "/example-path.txt";
    fs::write(&example_path, "example content of a file").unwrap();
    let toml = format!("
        title = 'Example Title'
        path = '{example_path}'
        notes = [ 'note1', 'note2', 'note3' ]
    ");

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
    let content = section.content(logger.hollow()).clone();
    let notes = section.notes(logger.hollow());

    assert_eq!(title, "Example Title");
    assert_eq!(content, "example content of a file");
    assert_eq!(notes, &vec![String::from("note1"), String::from("note2"), String::from("note3")].into_boxed_slice());
}