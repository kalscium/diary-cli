pub mod section;
pub use section::*;
pub use crate::{
    get,
    read_container,
    write_container,
};

// Some ease of life utils for section
#[macro_export]
macro_rules! get {
    ($key:ident at ($entry:ident, $idx:ident) from $table:ident as $func:ident with $logger:ident) => {{
        let key = stringify!($key);
        let obj = match $table.get(key) {
            Some(x) => x,
            None => {
                $logger.error(Log::new(LogType::Fatal, "EntrySection", &format!("Entry '{0}', section {1} must have '{key}' attribute", $entry, $idx), &[]));
                $logger.crash()
            }
        };

        match obj.$func() {
            Some(x) => x,
            None => {
                $logger.error(Log::new(LogType::Fatal, "EntrySection", &format!("Entry '{0}', section {1} must have '{key}' attribute", $entry, $idx), &[]));
                $logger.crash()
            }
        }
    }}
}

#[macro_export]
macro_rules! read_container {
    ($key:ident from $container:ident as $func:ident with $logger:ident) => {{
        let data = if_err!(($logger) [EntrySection, err => ("While reading from database: {:?}", err)] retry $container.read_data(stringify!($key)));
        if_err!(($logger) [EntrySection, err => ("While reading from database: {:?}", err)] {data.$func()} manual {
            Crash => {
                $logger.error(Log::new(LogType::Fatal, "EntrySection", &format!("{:#?}", err), &[]));
                $logger.crash()
            }
        })
    }}
}

#[macro_export]
macro_rules! write_container {
    (($value:expr) into $container:ident at $key:ident as $func:ident with $logger:ident) => {
        let data_writer = if_err!(($logger) [EntrySection, err => ("While writing to database: {:?}", err)] retry $container.data_writer(stringify!($key)));
        if_err!(($logger) [EntrySection, err => ("While writing to database: {:?}", err)] {LazyData::$func(data_writer, $value)} manual {
            Crash => {
                $logger.error(Log::new(LogType::Fatal, "EntrySection", &format!("{:#?}", err), &[]));
                $logger.crash()
            }
        });
    }
}

pub struct Entry {
    pub sections: Box<[Section]>,
    pub title: String,
    pub description: String,
    pub groups: Box<[String]>,
    pub notes: Box<[String]>,
}