use toml::Table;
use super::*;
use soulog::*;
use std::path::Path;
use crate::list;
use std::fs;

// Some ease of life macros
macro_rules! get {
    ($key:ident at ($entry:ident, $idx:ident) from $table:ident as $func:ident with $logger:ident) => {{
        let key = stringify!($key);
        let obj = unwrap_opt!(($table.get(key)) with $logger, format: EntrySection("Entry '{0}', seciton {1} must have '{key}' attribute", $entry, $idx));

        unwrap_opt!((obj.$func()) with $logger, format: EntrySection("Entry '{0}', section {1}'s '{key}' attribute must be of the correct type", $entry, $idx))
    }}
}

pub struct Section {
    pub container: LazyContainer,
    pub title: Option<String>,
    pub notes: Option<Box<[String]>>,
    pub content: Option<String>,
}

impl Section {
    pub fn new(table: &Table, container: LazyContainer, entry: &str, idx: u8, mut logger: impl Logger) -> Self {
        // Get the basic needed data
        let title = get!(title at (entry, idx) from table as as_str with logger).to_string();
        let path = get!(path at (entry, idx) from table as as_str with logger).to_string();
        let raw_notes = get!(notes at (entry, idx) from table as as_array with logger);

        // Check if path exists
        if !Path::new(&path).exists() {
            logger.error(Log::new(LogType::Fatal, "EntrySection", &format!("Path '{path}' specified in entry '{entry}', section {idx} does not exist"), &[]));
            return logger.crash();
        };

        let content = if_err!((logger) [EntrySection, err => ("While reading entry '{entry}', section {idx}'s path contents: {err:?}")] retry fs::read_to_string(&path));

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
        
        let mut this = Self {
            container,
            title: Some(title),
            content: Some(content),
            notes: Some(notes.into_boxed_slice()),
        };

        this.store_lazy(logger);
        this.clear_cache();
        this
    }

    pub fn store_lazy(&self, mut logger: impl Logger) {
        // Only store them if they are accessed (maybe modified)
        if let Some(x) = &self.title { write_container!((x) into EntrySection(self.container) at title as new_string with logger); }
        if let Some(x) = &self.content { write_container!((x) into EntrySection(self.container) at path as new_string with logger); }
        if let Some(x) = &self.notes {
            list::write(
                x.as_ref(),
                |file, data| LazyData::new_string(file, data),
                &if_err!((logger) [EntrySection, err => ("While writing section to database: {:?}", err)] retry self.container.new_container("notes")),
                logger
            );
        }
    }

    pub fn load_lazy(container: LazyContainer) -> Self {
        Self {
            container,
            title: None,
            notes: None,
            content: None,
        }
    }

    pub fn clear_cache(&mut self) {
        self.title = None;
        self.content = None;
        self.notes = None;
    }

    pub fn fill_cache(&mut self, logger: impl Logger) {
        self.title(logger.hollow());
        self.content(logger.hollow());
        self.notes(logger.hollow());
    }

    cache_field!(notes(this, logger) -> Box<[String]> {
        list::read(
            |data| data.collect_string(),
            &if_err!((logger) [EntrySection, err => ("While reading from section notes: {err:?}")] retry this.container.read_container("notes")),
            logger
        )
    });

    cache_field!(title(this, logger) -> String {
        read_container!(title from EntrySection(this.container) as collect_string with logger)
    });

    cache_field!(content(this, logger) -> String {
        read_container!(path from EntrySection(this.container) as collect_string with logger)
    });
}

impl Drop for Section {
    fn drop(&mut self) {
        self.store_lazy(crate::DiaryLogger::new());
    }
}