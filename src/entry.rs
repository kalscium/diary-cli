pub mod section;
pub use section::*;
use toml::Table;
use soulog::*;
use lazy_db::*;
pub use crate::{
    list,
    unwrap_opt,
    read_container,
    write_container,
};

// Some ease of life utils for section
#[macro_export]
macro_rules! unwrap_opt {
    (($opt:expr) with $logger:ident, format: $origin:ident$error:tt) => {
        match $opt {
            Some(x) => x,
            None => {
                $logger.error(Log::new(LogType::Fatal, stringify!($origin), &format!$error, &[]));
                $logger.crash()
            }
        }
    }
}

#[macro_export]
macro_rules! read_container {
    ($key:ident from $name:ident($container:expr) as $func:ident with $logger:ident) => {{
        let data = if_err!(($logger) [$name, err => ("While reading from database: {:?}", err)] retry $container.read_data(stringify!($key)));
        if_err!(($logger) [$name, err => ("While reading from database: {:?}", err)] {data.$func()} manual {
            Crash => {
                $logger.error(Log::new(LogType::Fatal, stringify!($name), &format!("{:#?}", err), &[]));
                $logger.crash()
            }
        })
    }}
}

#[macro_export]
macro_rules! write_container {
    (($value:expr) into $name:ident($container:expr) at $key:ident as $func:ident with $logger:ident) => {
        let data_writer = if_err!(($logger) [$name, err => ("While writing to database: {:?}", err)] retry $container.data_writer(stringify!($key)));
        if_err!(($logger) [$name, err => ("While writing to database: {:?}", err)] {LazyData::$func(data_writer, $value)} manual {
            Crash => {
                $logger.error(Log::new(LogType::Fatal, stringify!($name), &format!("{:#?}", err), &[]));
                $logger.crash()
            }
        });
    }
}

macro_rules! unpack_array {
    ($result:ident from $raw:ident with $logger:ident by $x:ident => $code:expr) => {
        let mut $result = Vec::with_capacity($raw.len());
        for $x in $raw {
            $result.push($code)
        };
    };

    ($result:ident from $raw:ident with $logger:ident by ($i:ident, $x:ident) => $code:expr) => {
        let mut $result = Vec::with_capacity($raw.len());
        for ($i, $x) in $raw.iter().enumerate() {
            $result.push($code)
        };
    };
}

macro_rules! get {
    ($key:ident at $entry:ident from $table:ident as $func:ident with $logger:ident) => {{
        let key = stringify!($key);
        let obj = unwrap_opt!(($table.get(key)) with $logger, format: Entry("Entry '{0}' must have '{key}' attribute", $entry));

        unwrap_opt!((obj.$func()) with $logger, format: Entry("Entry '{0}'s '{key}' attribute must be of correct type", $entry))
    }}
}

pub struct Entry {
    pub container: LazyContainer,
    pub sections: Option<Box<[Section]>>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub groups: Option<Box<[String]>>,
    pub notes: Option<Box<[String]>>,
    /// Date goes from `day` to `month` then to `year`
    pub date: Option<[u16; 3]>,
}

impl Entry {
    pub fn new(table: Table, entry_path: &str, container: LazyContainer, mut logger: impl Logger) -> Self {
        log!((logger) Entry("Reading entry '{entry_path}'s raw unchecked data..."));

        let entry_table = get!(entry at entry_path from table as as_table with logger); // For nice entry nesting

        let title = get!(title at entry_path from entry_table as as_str with logger).to_string();
        let description = get!(description at entry_path from entry_table as as_str with logger).to_string();
        let raw_notes = get!(notes at entry_path from entry_table as as_array with logger);
        let raw_groups = get!(groups at entry_path from entry_table as as_array with logger);
        let raw_sections = get!(section at entry_path from table as as_array with logger);

        // Get date
        log!((logger) Entry("Parsing date..."));
        let date: toml::value::Date = unwrap_opt!(
            (get!(date at entry_path from entry_table as as_datetime with logger).date)
            with logger,
            format: Entry("Datetime 'date' must contain the date")
        ); let date = [ date.day as u16, date.month as u16, date.year ];

        // Parse simple arrays
        log!((logger) Entry("Parsing notes & groups..."));
        unpack_array!(notes from raw_notes with logger by x
            => unwrap_opt!((x.as_str()) with logger, format: Entry("All notes in entry '{entry_path}' must be strings")).to_string()
        );

        unpack_array!(groups from raw_groups with logger by x
            => unwrap_opt!((x.as_str()) with logger, format: Entry("All groups in entry '{entry_path}' must be strings")).to_string()
        );

        // Parse sections
        log!((logger) Entry("Parsing entry's sections..."));
        let list = if_err!((logger) [Entry, err => ("While initialising sections: {err:?}")] retry container.new_container("sections"));
        unpack_array!(sections from raw_sections with logger by (i, x) => {
            let container = if_err!((logger) [Entry, err => ("While initialising section {i}: {err:?}")] retry list.new_container(i.to_string()));
            let table = unwrap_opt!((x.as_table()) with logger, format: Entry("Entry '{entry_path}', section {i} must be a toml table"));
            Section::new(table, container, entry_path, i as u8, logger.hollow()) // Write into that container
        });
        { // For cleaner context
            let writer = if_err!((logger) [Entry, err => ("While writing list length: {err:?}")] retry list.data_writer("length"));
            if_err!((logger) [Entry, err => ("While writing list length: {err:?}")] {LazyData::new_u8(writer, raw_sections.len() as u8)} manual {
                Crash => {
                    logger.error(Log::new(LogType::Fatal, "Entry", &format!("{err:#?}"), &[]));
                    logger.crash()
                }
            })
        }

        log!((logger) Entry("Storing entry's parsed and checked data into database..."));
        log!((logger) Entry("(if this fails, this may leave your database (diary) in a corrupted state!)"));

        let mut this = Self {
            container,
            title: Some(title),
            description: Some(description),
            date: Some(date),
            notes: Some(notes.into_boxed_slice()),
            groups: Some(groups.into_boxed_slice()),
            sections: Some(sections.into_boxed_slice()),
        };
        this.store_lazy(logger.hollow());
        log!((logger) Entry("Successfully written entry into database"));
        this.clear_cache();
        this
    }

    pub fn store_lazy(&self, mut logger: impl Logger) {
        // Only store them if modified
        if let Some(x) = &self.title { write_container!((x) into Entry(self.container) at title as new_string with logger); }
        if let Some(x) = &self.description { write_container!((x) into Entry(self.container) at description as new_string with logger); }
        
        // The bloody lists & arrays
        if let Some(x) = &self.notes {
            list::write(
                x.as_ref(),
                |file, data| LazyData::new_string(file, data),
                &if_err!((logger) [EntrySection, err => ("While writing section to database: {:?}", err)] retry self.container.new_container("notes")),
                logger.hollow()
            );
        }

        if let Some(x) = &self.groups {
            list::write(
                x.as_ref(),
                |file, data| LazyData::new_string(file, data),
                &if_err!((logger) [EntrySection, err => ("While writing section to database: {:?}", err)] retry self.container.new_container("groups")),
                logger.hollow()
            );
        }

        if let Some(x) = &self.date {
            list::write(
                x.as_ref(),
                |file, data| LazyData::new_u16(file, *data),
                &if_err!((logger) [EntrySection, err => ("While writing section to database: {:?}", err)] retry self.container.new_container("date")),
                logger
            );
        }
    }

    pub fn load_lazy(container: LazyContainer) -> Self {
        Self {
            title: None,
            sections: None,
            description: None,
            groups: None,
            notes: None,
            date: None,
            container,
        }
    }

    pub fn clear_cache(&mut self) {
        self.title = None;
        self.sections = None;
        self.description = None;
        self.groups = None;
        self.notes = None;
        self.date = None;
    }

    pub fn fill_cache(&mut self, logger: impl Logger) {
        self.title(logger.hollow());
        self.sections(logger.hollow());
        self.groups(logger.hollow());
        self.notes(logger.hollow());
        self.date(logger.hollow());
    }

    cache_field!(title(this, logger) -> String {
        read_container!(title from EntrySection(this.container) as collect_string with logger)
    });

    cache_field!(description(this, logger) -> String {
        read_container!(title from Entry(this.container) as collect_string with logger)
    });

    cache_field!(notes(this, logger) -> Box<[String]> {
        list::read(
            |data| data.collect_string(),
            &if_err!((logger) [Entry, err => ("While reading from entry's notes: {err:?}")] retry this.container.read_container("notes")),
            logger
        )
    });

    cache_field!(groups(this, logger) -> Box<[String]> {
        list::read(
            |data| data.collect_string(),
            &if_err!((logger) [Entry, err => ("While reading from entry's groups: {err:?}")] retry this.container.read_container("groups")),
            logger
        )
    });

    cache_field!(date(this, logger) -> [u16; 3] {
        let array = list::read(
            |data| data.collect_u16(),
            &if_err!((logger) [Entry, err => ("While reading from entry's date: {err:?}")] retry this.container.read_container("date")),
            logger
        ); [array[0], array[1], array[2]]
    });

    cache_field!(sections(this, logger) -> Box<[Section]> {
        let container = if_err!((logger) [Entry, err => ("While reading from entry's sections: {err:?}")] retry this.container.read_container("sections"));
        let length = if_err!((logger) [Entry, err => ("While reading from entry's sections' length: {err:?}")] retry container.read_data("length"));
        let length = if_err!((logger) [Entry, err => ("While reading from entry's sections' length: {err:?}")] {length.collect_u8()} manual {
            Crash => {
                logger.error(Log::new(LogType::Fatal, "Entry", &format!("{err:#?}"), &[]));
                logger.crash()
            }
        });
        let mut sections = Vec::with_capacity(length as usize);

        for i in 0..length {
            sections.push(Section::load_lazy(
                if_err!((logger) [Entry, err => ("While reading entry section {i}: {err:?}")] retry container.read_container(i.to_string()))
            ));
        }

        sections.into_boxed_slice()
    });
}