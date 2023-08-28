pub mod section;
pub use section::*;
use toml::Table;
use soulog::*;
use lazy_db::*;
use std::path::Path;
use crate::search::Searchable;
pub use crate::{
    list,
    unpack_array,
    unwrap_opt,
    read_db_container,
    write_db_container,
};

// Some ease of life utils for section
#[macro_export]
macro_rules! unwrap_opt {
    (($opt:expr) with $logger:ident, format: $origin:ident$error:tt) => {
        match $opt {
            Some(x) => x,
            None => {
                log!(($logger.error) $origin$error as Fatal);
                $logger.crash()
            }
        }
    }
}

#[macro_export]
macro_rules! read_db_container {
    ($key:ident from $name:ident($container:expr) as $func:ident with $logger:ident) => {{
        let data = if_err!(($logger) [$name, err => ("While reading from archive: {:?}", err)] retry $container.read_data(stringify!($key)));
        if_err!(($logger) [$name, err => ("While reading from archive: {:?}", err)] {data.$func()} crash {
            log!(($logger.error) $name("{err:#?}") as Fatal);
            $logger.crash()
        })
    }}
}

#[macro_export]
macro_rules! write_db_container {
    ($name:ident($container:expr) $key:ident = $func:ident($value:expr) with $logger:ident) => {
        if_err!(($logger) [$name, err => ("While writing to archive: {err:?}")] retry write_container!(($container) $key = $func($value)));
    }
}

#[macro_export]
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
    pub uid: String,
    pub sections: Option<Box<[Section]>>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Box<[String]>>,
    pub notes: Option<Box<[String]>>,
    /// Date goes from `day` to `month` then to `year`
    pub date: Option<[u16; 3]>,
}

impl Entry {
    pub fn new(table: Table, entry_path: &str, database: LazyContainer, mut logger: impl Logger) -> Self {
        log!((logger) Entry("Reading entry '{entry_path}'s raw unchecked data..."));

        let entry_table = get!(entry at entry_path from table as as_table with logger); // For nice entry nesting
        let uid = get!(uid at entry_path from entry_table as as_str with logger).to_string();

        let title = get!(title at entry_path from entry_table as as_str with logger).to_string();
        let description = get!(description at entry_path from entry_table as as_str with logger).to_string();
        let raw_notes = get!(notes at entry_path from entry_table as as_array with logger);
        let raw_tags = get!(tags at entry_path from entry_table as as_array with logger);
        let raw_sections = get!(section at entry_path from table as as_array with logger);

        // set the container
        let container =
            if_err!((logger) [Entry, err => ("While initialising entry: '{err:?}'")] retry database.new_container(&uid));

        // Get date
        log!((logger) Entry("Parsing date..."));
        let date: toml::value::Date = unwrap_opt!(
            (get!(date at entry_path from entry_table as as_datetime with logger).date)
            with logger,
            format: Entry("Datetime 'date' must contain the date")
        ); let date = [ date.day as u16, date.month as u16, date.year ];

        // Parse simple arrays
        log!((logger) Entry("Parsing notes & tags..."));
        unpack_array!(notes from raw_notes with logger by x
            => unwrap_opt!((x.as_str()) with logger, format: Entry("All notes in entry '{entry_path}' must be strings")).to_string()
        );

        unpack_array!(tags from raw_tags with logger by x
            => unwrap_opt!((x.as_str()) with logger, format: Entry("All tags in entry '{entry_path}' must be strings")).to_string()
        );

        // Parse sections
        log!((logger) Entry("Parsing entry's sections..."));
        let list = if_err!((logger) [Entry, err => ("While initialising sections: {err:?}")] retry container.new_container("sections"));
        unpack_array!(sections from raw_sections with logger by (i, x) => {
            let container = if_err!((logger) [Entry, err => ("While initialising section {i}: {err:?}")] retry list.new_container(i.to_string()));
            let table = unwrap_opt!((x.as_table()) with logger, format: Entry("Entry '{entry_path}', section {i} must be a toml table"));
            Section::new(table, container, entry_path, i as u8, logger.hollow()) // Write into that container
        });
        if_err!((logger) [Entry, err => ("While writing section list length: {err:?}")] retry write_container!((list) length = new_u16(raw_sections.len() as u16)));

        log!((logger) Entry("Storing entry's parsed and checked data into archive..."));
        log!((logger.error) Entry("if this fails, this may leave your archive (diary) in a corrupted state!") as Warning);

        let mut this = Self {
            container,
            uid,
            title: Some(title),
            description: Some(description),
            date: Some(date),
            notes: Some(notes.into_boxed_slice()),
            tags: Some(tags.into_boxed_slice()),
            sections: Some(sections.into_boxed_slice()),
        };
        this.store_lazy(logger.hollow());
        log!((logger) Entry("Successfully written entry into archive"));
        log!((logger) Entry("")); // spacer
        this.clear_cache();
        this
    }

    pub fn pull(&mut self, path: &Path, logger: impl Logger) -> Table {
        let mut map = Table::new();
        let mut entry = Table::new();

        // Insert uid, title, description, notes, tags, and date
        entry.insert("uid".into(), self.uid.clone().into());
        entry.insert("title".into(), self.title(logger.hollow()).clone().into());
        entry.insert("description".into(), self.description(logger.hollow()).clone().into());
        entry.insert("notes".into(), self.notes(logger.hollow()).to_vec().into());
        entry.insert("tags".into(), self.tags(logger.hollow()).to_vec().into());
        entry.insert("date".into(), Self::array_to_date(self.date(logger.hollow()), logger.hollow()));
        map.insert("entry".into(), entry.into());

        self.clear_cache();

        map.insert("section".into(), self.sections(logger.hollow())
            .iter_mut()
            .enumerate()
            .map(|(i, x)| x.pull(i as u8, path, logger.hollow()))
            .collect::<Vec<Table>>()
            .into()
        );

        self.clear_cache();

        map
    }

    fn array_to_date(arr: &[u16; 3], mut logger: impl Logger) -> toml::Value {
        // Format the array of u16s to a string in the RFC 3339 date format
        let date_string = format!("{:04}-{:02}-{:02}",
            arr[2], // Year
            arr[1], // Month
            arr[0], // Day
        );
    
        // Parse the string to a toml::Value::Datetime
        toml::Value::Datetime(if_err!((logger) [Pull, _err => ("Invalid entry date")]
            {date_string.parse()}
            crash logger.crash()
        ))
    }

    pub fn store_lazy(&self, mut logger: impl Logger) {
        log!((logger) Entry("Storing entry into archive..."));
        // Only store them if modified
        if let Some(x) = &self.title { write_db_container!(Entry(self.container) title = new_string(x) with logger); }
        if let Some(x) = &self.description { write_db_container!(Entry(self.container) description = new_string(x) with logger); }
        if let Some(x) = &self.date { write_db_container!(Entry(self.container) date = new_u16_array(x) with logger); }

        // The bloody lists & arrays
        if let Some(x) = &self.notes {
            list::write(
                x.as_ref(),
                |file, data| LazyData::new_string(file, data),
                &if_err!((logger) [Entry, err => ("While writing notes to archive: {:?}", err)] retry self.container.new_container("notes")),
                logger.hollow()
            );
        }

        if let Some(x) = &self.tags {
            list::write(
                x.as_ref(),
                |file, data| LazyData::new_string(file, data),
                &if_err!((logger) [Entry, err => ("While writing tags to archive: {:?}", err)] retry self.container.new_container("tags")),
                logger.hollow()
            );
        }
    }

    pub fn load_lazy(uid: String, container: LazyContainer) -> Self {
        Self {
            container,
            uid,
            title: None,
            sections: None,
            description: None,
            tags: None,
            notes: None,
            date: None,
        }
    }

    pub fn clear_cache(&mut self) {
        self.title = None;
        self.sections = None;
        self.description = None;
        self.tags = None;
        self.notes = None;
        self.date = None;
    }

    pub fn fill_cache(&mut self, logger: impl Logger) {
        self.title(logger.hollow());
        self.sections(logger.hollow());
        self.tags(logger.hollow());
        self.notes(logger.hollow());
        self.date(logger.hollow());
    }

    cache_field!(title(this, logger) -> String {
        read_db_container!(title from EntrySection(this.container) as collect_string with logger)
    });

    cache_field!(description(this, logger) -> String {
        read_db_container!(description from Entry(this.container) as collect_string with logger)
    });

    cache_field!(notes(this, logger) -> Box<[String]> {
        list::read(
            |data| data.collect_string(),
            &if_err!((logger) [Entry, err => ("While reading from entry's notes: {err:?}")] retry this.container.read_container("notes")),
            logger
        )
    });

    cache_field!(tags(this, logger) -> Box<[String]> {
        list::read(
            |data| data.collect_string(),
            &if_err!((logger) [Entry, err => ("While reading from entry's tags: {err:?}")] retry this.container.read_container("tags")),
            logger
        )
    });

    cache_field!(date(this, logger) -> [u16; 3] {
        let array = read_db_container!(date from Entry(this.container) as collect_u16_array with logger);
        [array[0], array[1], array[2]]
    });

    cache_field!(sections(this, logger) -> Box<[Section]> {
        let container = if_err!((logger) [Entry, err => ("While reading from entry's sections: {err:?}")] retry this.container.read_container("sections"));
        let length = if_err!((logger) [Entry, err => ("While reading from entry's sections' length: {err:?}")] retry container.read_data("length"));
        let length = if_err!((logger) [Entry, err => ("While reading from entry's sections' length: {err:?}")] {length.collect_u16()} crash {
            log!((logger) Entry("{err:#?}") as Fatal);
            logger.crash()
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

impl Drop for Entry {
    fn drop(&mut self) {
        self.store_lazy(crate::DiaryLogger::new());
    }
}

impl Searchable for Entry {
    fn get_uid(&self) -> String {
        self.uid.clone()
    }

    fn contains_tag(&mut self, tag: &String, logger: impl Logger) -> bool {
        let result = self.tags(logger).contains(tag);
        self.tags = None;
        result
    }
}