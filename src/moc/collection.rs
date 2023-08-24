use soulog::*;
use lazy_db::*;
use crate::entry::*;

pub struct Collection {
    pub container: LazyContainer,
    pub title: Option<String>,
    pub notes: Option<Box<[String]>>,
    pub include: Option<Box<[String]>>,
}

// impl the lazy stuff
impl Collection {
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
            &if_err!((logger) [Collection, err => ("While reading from collection's notes: {err:?}")] retry this.container.read_container("notes")),
            logger
        )
    });

    cache_field!(title(this, logger) -> String {
        read_db_container!(title from Collection(this.container) as collect_string with logger)
    });

    cache_field!(include(this, logger) -> Box<[String]> {
        list::read(
            |data| data.collect_string(),
            &if_err!((logger) [Collection, err => ("While reading from collection's included groups: {err:?}")] retry this.container.read_container("include")),
            logger
        )
    });
}