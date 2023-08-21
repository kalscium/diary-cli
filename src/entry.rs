pub mod section;
pub use section::*;
use toml::Table;
use soulog::*;
use lazy_db::*;

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
macro_rules! cache_field {
    ($name:ident($this:ident, $logger:ident) -> $type:ty $code:block) => {
        #[allow(unused_mut)]
        pub fn $name(&mut self, mut $logger: impl Logger) -> &$type {
            let $this = self;
            if $this.$name.is_none() {
                $this.$name = Some($code);
            }; $this.$name.as_ref().unwrap()
        }
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
    pub sections: Box<[Section]>,
    pub title: String,
    pub description: String,
    pub groups: Box<[String]>,
    pub notes: Box<[String]>,
    /// Date goes from `day` to `month` then to `year`
    pub date: [u16; 3],
}

impl Entry {
    pub fn new(table: Table, entry_path: &str, container: LazyContainer, mut logger: impl Logger) -> Self {
        let title = get!(title at entry_path from table as as_str with logger).to_string();
        let description = get!(description at entry_path from table as as_str with logger).to_string();
        let raw_notes = get!(notes at entry_path from table as as_array with logger);
        let raw_groups = get!(notes at entry_path from table as as_array with logger);
        let raw_sections = get!(section at entry_path from table as as_array with logger);

        // Get date
        let date: toml::value::Date = unwrap_opt!(
            (get!(date at entry_path from table as as_datetime with logger).date)
            with logger,
            format: Entry("Datetime 'date' must contain the date")
        ); let date = [ date.day as u16, date.month as u16, date.year ];

        // Parse simple arrays
        unpack_array!(notes from raw_notes with logger by x
            => unwrap_opt!((x.as_str()) with logger, format: Entry("All notes in entry '{entry_path}' must be strings")).to_string()
        );

        unpack_array!(groups from raw_groups with logger by x
            => unwrap_opt!((x.as_str()) with logger, format: Entry("All groups in entry '{entry_path}' must be strings")).to_string()
        );

        // Parse sections
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

        let this = Self {
            title,
            description,
            date,
            notes: notes.into_boxed_slice(),
            groups: groups.into_boxed_slice(),
            sections: sections.into_boxed_slice(),
        };
        this
    }
}