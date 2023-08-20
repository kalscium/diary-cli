pub mod section;
pub use section::*;
use toml::Table;
use soulog::*;

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

macro_rules! get {
    ($key:ident at $entry:ident from $table:ident as $func:ident with $logger:ident) => {{
        let key = stringify!($key);
        let obj = unwrap_opt!(($table.get(key)) with $logger, format: Entry("Entry '{0}' must have '{key}' attribute", $entry));

        unwrap_opt!((obj.$func()) with $logger, format: Entry("Entry '{0}'s '{key}' attribute must be of correect type", $entry))
    }}
}

pub struct Entry {
    pub sections: Box<[Section]>,
    pub title: String,
    pub description: String,
    pub groups: Box<[String]>,
    pub notes: Box<[String]>,
    /// Date goes from `day` to `month` then to `year`
    pub date: [u8; 3],
}

impl Entry {
    // pub fn new(table: Table, entry_path: &str, mut logger: impl Logger) -> Self {
    //     let title = get!(title at entry_path from table as as_str with logger).to_string();
    //     let description = get!(description at entry_path from table as as_str with logger).to_string();
    //     // let date = get!(date at entry_path from table as as_datetime with logger).date.unwrap().
    //     todo!()
    // }
}