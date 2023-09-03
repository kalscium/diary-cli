use toml::{Table, Value};
use super::*;
use soulog::*;
use std::path::Path;
use crate::list;
use crate::unpack_array;
use std::fs;

// Some ease of life macros
macro_rules! get {
    ($key:ident at ($entry:ident, $idx:ident) from $table:ident as $func:ident with $logger:ident) => {{
        let key = stringify!($key);
        let obj = unwrap_opt!(($table.get(key)) with $logger, format: Section("Entry '{0}', section {1} must have '{key}' attribute", $entry, $idx));

        unwrap_opt!((obj.$func()) with $logger, format: Section("Entry '{0}', section {1}'s '{key}' attribute must be of the correct type", $entry, $idx))
    }};

    ($var:ident = $key:ident at ($entry:ident, $idx:ident) from $table:ident as $func:ident with $logger:ident or $default:expr) => {
        let key = stringify!($key);
        let default = $default;
        let $var = $table.get(key)
            .map(|x| unwrap_opt!((x.$func()) with $logger, format: Entry("Entry '{0}', section {1}'s '{key}' attribute must be of the correct type", $entry, $idx)))
            .unwrap_or(&default);
    };
}

pub struct Section {
    pub container: LazyContainer,
    pub title: Option<String>,
    pub notes: Option<Box<[String]>>,
    pub content: Option<String>,
}

impl Section {
    pub fn new(table: &Table, container: LazyContainer, entry: &str, idx: u8, mut logger: impl Logger) -> Self {
        log!((logger) Section("Parsing entry '{entry}'s section {idx}..."));

        // Get the basic needed data
        log!((logger) Section("Reading section's data..."));
        let title = get!(title at (entry, idx) from table as as_str with logger).to_string();
        get!(raw_notes = notes at (entry, idx) from table as as_array with logger or Vec::<toml::Value>::with_capacity(0));

        // Get contents
        let content = table.get("path")
            .map(|x| {
                let path = unwrap_opt!((x.as_str()) with logger, format: Entry("Entry '{entry}', section {idx}'s 'path' attribute must be of the correct type"));
                log!((logger) Section("Checking if path specified in the section is valid..."));
                // Check if path exists
                if !Path::new(path).exists() {
                    log!((logger.error) Section("Path '{path}' specified in entry '{entry}', section {idx} does not exist") as Fatal);
                    return logger.crash();
                };
                if_err!((logger) [Section, err => ("While reading entry '{entry}', section {idx}'s path contents: {err:?}")] retry fs::read_to_string(path))
            }).unwrap_or_else(|| {
                get!(contents at (entry, idx) from table as as_str with logger).to_string()
            });

        // Parse notes
        log!((logger) Section("Parsing section's notes"));
        unpack_array!(notes from raw_notes with logger by x
            => unwrap_opt!((x.as_str()) with logger, format: Section("All notes in entry '{entry}', section '{idx}' must be strings")).to_string()
        );

        log!((logger) Section("Writing entry '{entry}'s section {idx} into archive..."));
        log!((logger.error) Section("(failure to do so may corrupt archive!)") as Warning);
        let mut this = Self {
            container,
            title: Some(title),
            content: Some(content),
            notes: Some(notes.into_boxed_slice()),
        };

        this.store_lazy(logger.hollow());
        this.clear_cache();
        log!((logger) Section("Successfully parsed and written entry's section {idx} into archive"));
        log!((logger) Section("")); // spacer
        this
    }

    pub fn pull(&mut self, idx: u8, path: &Path, one_file: bool, mut logger: impl Logger) -> Table {
        let mut map = Table::new();

        // Insert title and notes
        map.insert("title".into(), Value::String(self.title(logger.hollow()).clone()));
        map.insert("notes".into(), self.notes(logger.hollow()).to_vec().into());

        if one_file {
            map.insert("contents".into(), Value::String(self.content(logger.hollow()).clone()));
        } else {
            let file_name = format!("section{idx}.txt");
            let path = path.join(&file_name);
            map.insert("path".into(), file_name.into());
            if_err!((logger) [Pull, err => ("While writing section as text file: {err:?}")] retry fs::write(&path, self.content(logger.hollow())));
        }

        self.clear_cache();

        map
    }

    pub fn store_lazy(&self, mut logger: impl Logger) {
        // Only store them if they are accessed (maybe modified)
        if let Some(x) = &self.title { write_db_container!(Section(self.container) title = new_string(x) with logger); }
        if let Some(x) = &self.content { write_db_container!(Section(self.container) content = new_string(x) with logger); }
        if let Some(x) = &self.notes {
            list::write(
                x.as_ref(),
                |file, data| LazyData::new_string(file, data),
                &if_err!((logger) [Section, err => ("While writing section's notes to archive: {:?}", err)] retry self.container.new_container("notes")),
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
            &if_err!((logger) [Section, err => ("While reading from section's notes: {err:?}")] retry this.container.child_container("notes")),
            logger
        )
    });

    cache_field!(title(this, logger) -> String {
        read_db_container!(title from Section(this.container) as collect_string with logger)
    });

    cache_field!(content(this, logger) -> String {
        read_db_container!(content from Section(this.container) as collect_string with logger)
    });
}