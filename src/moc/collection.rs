use soulog::*;
use lazy_db::*;
use crate::entry::*;
use toml::{Table, Value};

// Some ease of life macros
macro_rules! get {
    ($key:ident at ($entry:ident, $idx:ident) from $table:ident as $func:ident with $logger:ident) => {{
        let key = stringify!($key);
        let obj = unwrap_opt!(($table.get(key)) with $logger, format: Collection("moc '{0}', collection {1} must have '{key}' attribute", $entry, $idx));

        unwrap_opt!((obj.$func()) with $logger, format: Collection("moc '{0}', collection {1}'s '{key}' attribute must be of the correct type", $entry, $idx))
    }}
}

pub struct Collection {
    pub container: LazyContainer,
    pub title: Option<String>,
    pub notes: Option<Box<[String]>>,
    pub include: Option<Box<[String]>>,
}

impl Collection {
    pub fn new(table: &Table, container: LazyContainer, moc: &str, idx: u8, mut logger: impl Logger) -> Self {
        log!((logger) Collection("Parsing moc '{moc}'s collection {idx}..."));

        // Get the basic needed data
        log!((logger) Collection("Reading collection's data..."));
        let title = get!(title at (moc, idx) from table as as_str with logger).to_string();
        let raw_notes = get!(notes at (moc, idx) from table as as_array with logger);
        let raw_include = get!(include at (moc, idx) from table as as_array with logger);

        // Parse arrays
        unpack_array!(notes from raw_notes with logger by x
            => unwrap_opt!((x.as_str()) with logger, format: Collection("All notes in moc '{moc}', collection '{idx}' must be strings")).to_string()
        );

        unpack_array!(include from raw_include with logger by x
            => unwrap_opt!((x.as_str()) with logger, format: Collection("All included groups in moc '{moc}', collection '{idx}' must be strings")).to_string()
        );

        log!((logger) Collection("Writing moc '{moc}'s collection {idx} into archive..."));
        log!((logger.error) Collection("(failure to do so may corrupt archive!)") as Warning);
        let mut this = Self {
            container,
            title: Some(title),
            notes: Some(notes.into_boxed_slice()),
            include: Some(include.into_boxed_slice()),
        };

        this.store_lazy(logger.hollow());
        this.clear_cache();
        log!((logger) Collection("Successfully parsed and written moc's collection {idx} into archive"));
        log!((logger) Collection("")); // spacer
        this
    }

    pub fn pull(&mut self, logger: impl Logger) -> Table {
        let mut map = Table::new();

        // Insert title and notes
        map.insert("title".into(), Value::String(self.title(logger.hollow()).clone()));
        map.insert("notes".into(), self.notes(logger.hollow()).to_vec().into());
        map.insert("include".into(), self.include(logger.hollow()).to_vec().into());

        self.clear_cache();

        map
    }

    pub fn store_lazy(&self, mut logger: impl Logger) {
        // Only store them if they are accessed (maybe modified)
        if let Some(x) = &self.title { write_db_container!(Collection(self.container) title = new_string(x) with logger); }
        if let Some(x) = &self.notes {
            list::write(
                x.as_ref(),
                |file, data| LazyData::new_string(file, data),
                &if_err!((logger) [Collection, err => ("While writing collection's notes to archive: {:?}", err)] retry self.container.new_container("notes")),
                logger.hollow()
            );
        }
        if let Some(x) = &self.include {
            list::write(
                x.as_ref(),
                |file, data| LazyData::new_string(file, data),
                &if_err!((logger) [Collection, err => ("While writing collection's included groups to archive: {:?}", err)] retry self.container.new_container("include")),
                logger
            );
        }
    }

    pub fn load_lazy(container: LazyContainer) -> Self {
        Self {
            container,
            title: None,
            notes: None,
            include: None,
        }
    }

    pub fn clear_cache(&mut self) {
        self.title = None;
        self.notes = None;
        self.include = None;
    }

    pub fn fill_cache(&mut self, logger: impl Logger) {
        self.title(logger.hollow());
        self.include(logger.hollow());
        self.notes(logger.hollow());
    }

    cache_field!(notes(this, logger) -> Box<[String]> {
        list::read(
            |data| data.collect_string(),
            &if_err!((logger) [Collection, err => ("While reading from collection's notes: {err:?}")] retry this.container.child_container("notes")),
            logger
        )
    });

    cache_field!(title(this, logger) -> String {
        read_db_container!(title from Collection(this.container) as collect_string with logger)
    });

    cache_field!(include(this, logger) -> Box<[String]> {
        list::read(
            |data| data.collect_string(),
            &if_err!((logger) [Collection, err => ("While reading from collection's included groups: {err:?}")] retry this.container.child_container("include")),
            logger
        )
    });
}