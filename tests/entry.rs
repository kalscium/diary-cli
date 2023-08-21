mod isol;

use soulog::*;
use lazy_db::*;
use isol::*;
use diary_cli::entry::Entry;
use std::fs;
use toml::Table;

#[test]
fn isol_entry() {
    let tmp = new_env();
    let logger = sbl::PanicLogger::new();
    let content = tmp.get_path().join("Entry");
    let container = LazyContainer::init(content).unwrap();
    let example_path = tmp.get_path().to_string_lossy().to_string() + "/example-path.txt";
    fs::write(&example_path, "example content of a file").unwrap();
    let toml = format!("
        [entry]
        title = 'Example Entry Title'
        description = 'Example Entry Description'
        groups = [ '2023', 'entry', 'term1' ]
        notes = [ 'entry-note1', 'entry-note2', 'entry-note3', 'entry-note4' ]
        date = 2023-08-21

        [[section]]
        title = 'Example Section Title'
        notes = [ 'note1', 'note2', 'note3' ]
        path = '{example_path}'
    ");

    // Store
    let mut entry = Entry::new(
        toml.parse::<Table>().unwrap(),
        "example-entry.toml",
        container,
        logger.hollow(),
    );

    // Load
    entry.clear_cache();
    entry.fill_cache(logger.hollow());
    entry.sections(logger.hollow())[0].fill_cache(logger);
}