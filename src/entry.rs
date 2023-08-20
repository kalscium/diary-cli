pub mod section;
pub use section::*;
use toml::Table;

// Some ease of life utils for section


pub struct Entry {
    pub sections: Box<[Section]>,
    pub title: String,
    pub description: String,
    pub groups: Box<[String]>,
    pub notes: Box<[String]>,
}

impl Entry {
    pub fn new(table: Table) -> Self {
        todo!()
    }
}