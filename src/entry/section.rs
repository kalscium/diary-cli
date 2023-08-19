use toml::*;
use soulog::*;
use std::path::Path;

// Some ease of life utils for section
macro_rules! get {
    ($key:ident at ($entry:ident, $idx:ident) from $table:ident as $func:ident with $logger:ident) => {{
        let key = stringify!($key);
        let obj = match $table.get(key) {
            Some(x) => x,
            None => {
                $logger.error(Log::new(LogType::Fatal, "EntrySection", &format!("Entry '{0}', section {1} must have {key} attribute", $entry, $idx), &[]));
                $logger.crash()
            }
        };

        match obj.$func() {
            Some(x) => x,
            None => {
                $logger.error(Log::new(LogType::Fatal, "EntrySection", &format!("Entry '{0}', section {1} must have {key} attribute", $entry, $idx), &[]));
                $logger.crash()
            }
        }
    }}
}

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
            logger.error(Log::new(LogType::Fatal, "EntrySection", &format!("Path '{path}' specified in entry '{entry}', section {idx}"), &[]));
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
}

