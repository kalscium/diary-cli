use toml::Table;
use lazy_db::*;
use soulog::*;
use std::path::Path;
use crate::list;
use super::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Section {
    pub title: String,
    pub notes: Box<[String]>,
    pub path: String,
}

impl Section {
    pub fn new(table: Table, entry: &str, idx: u8, mut logger: impl Logger) -> Self {
        // Get the basic needed data
        let title = get!(title at (entry, idx) from table as as_str with logger).to_string();
        let path = get!(path at (entry, idx) from table as as_str with logger);
        let raw_notes = get!(notes at (entry, idx) from table as as_array with logger);

        // Check if path exists
        if !Path::new(path).exists() {
            logger.error(Log::new(LogType::Fatal, "EntrySection", &format!("Path '{path}' specified in entry '{entry}', section {idx} does not exist"), &[]));
            return logger.crash();
        };

        // Parse notes
        let mut notes = Vec::with_capacity(raw_notes.len());
        for i in raw_notes {
            notes.push(
                match i.as_str() {
                    Some(x) => x.to_string(),
                    None => {
                        logger.error(Log::new(LogType::Fatal, "EntrySection", &format!("All notes in entry '{entry}', section {idx} must be strings"), &[]));
                        logger.crash()
                    }
                }
            )
        };

        // Construct Self
        Self {
            title,
            path: path.to_string(),
            notes: notes.into_boxed_slice(),
        }
    }

    pub fn load(container: LazyContainer, mut logger: impl Logger) -> Self {
        let title = read_container!(title from container as collect_string with logger);
        let path = read_container!(path from container as collect_string with logger);
        let notes = list::read(
            |data| data.collect_string(),
            if_err!((logger) [EntrySection, err => ("While reading from database: {:?}", err)] retry container.read_container("notes")),
            logger,
        );

        Self {
            title,
            path,
            notes,
        }
    }

    pub fn store(&self, container: LazyContainer, mut logger: impl Logger) {
        write_container!((&self.title) into container at title as new_string with logger);
        write_container!((&self.path) into container at path as new_string with logger);
        list::write(
            self.notes.as_ref(),
            |file, data| LazyData::new_string(file, data),
            if_err!((logger) [EntrySection, err => ("While writing to database: {:?}", err)] retry container.new_container("notes")),
            logger
        );
    }
}