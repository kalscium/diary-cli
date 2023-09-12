use std::marker::PhantomData;

use lazy_db::*;
use soulog::*;

pub fn write<T>(list: &[T], f: impl Fn(FileWrapper, &T) -> Result<(), LDBError>, container: &LazyContainer, mut logger: impl Logger) {
    if_err!((logger) [ListIO, err => ("While clearing container: {err:?}")] retry container.wipe());

    for (i, x) in list.iter().enumerate() {
        let data_writer = 
            if_err!((logger) [ListIO, err => ("While writing element of list: {:?}", err)] retry container.data_writer(i.to_string()));
        if_err!((logger) [ListIO, err => ("While writing element of list: {:?}", err)] {f(data_writer, x)} crash {
            log!((logger.error) ListIO("{err:#?}") as Fatal);
            logger.crash()
        })
    }

    if_err!((logger) [ListIO, err => ("{:?}", err)] retry {
        let data_writer = if_err!((logger) [ListIO, err => ("While writing list length: {:?}", err)] retry container.data_writer("length"));
        LazyData::new_u16(data_writer, list.len() as u16)
    })
}

pub fn push(f: impl Fn(FileWrapper) -> Result<(), LDBError>, container: &LazyContainer, mut logger: impl Logger) {
    let length = load_length(container, logger.hollow());

    let data_writer = if_err!((logger) [ListIO, err => ("While pushing to list: {:?}", err)] retry container.data_writer(length.to_string()));
    if_err!((logger) [ListIO, err => ("While pushing to list: {:?}", err)] {f(data_writer)} crash {
        log!((logger.error) ListIO("{err:#?}") as Fatal);
        logger.crash()
    });

    if_err!((logger) [ListIO, err => ("{:?}", err)] retry {
        let data_writer = if_err!((logger) [ListIO, err => ("While writing list length: {:?}", err)] retry container.data_writer("length"));
        LazyData::new_u16(data_writer, length + 1)
    })
}

pub fn pop<T>(f: impl Fn(LazyData) -> Result<T, LDBError>, container: &LazyContainer, mut logger: impl Logger) -> Option<T> {
    let length = load_length(container, logger.hollow());

    if length == 0 {
        return None;
    }

    let idx = (length - 1).to_string();
    let item = if_err!((logger) [ListIO, err => ("While reading list element: {:?}", err)] retry container.read_data(&idx));
    let item = if_err!((logger) [ListIO, err => ("While reading list element: {:?}", err)] {f(item)} crash {
        log!((logger.error) ListIO("{err:#?}") as Fatal);
        logger.crash()
    });

    if_err!((logger) {container.remove(idx)} else(err) {
        log!((logger.vital) ListIO("While removing item: {err:?}; ignoring error...") as Inconvenience)
    });

    if_err!((logger) [ListIO, err => ("{:?}", err)] retry {
        let data_writer = if_err!((logger) [ListIO, err => ("While writing list length: {:?}", err)] retry container.data_writer("length"));
        LazyData::new_u16(data_writer, length - 1)
    });

    Some(item)
}

pub fn read<T>(f: impl Fn(LazyData) -> Result<T, LDBError>, container: &LazyContainer, mut logger: impl Logger) -> Box<[T]> {
    let length = load_length(container, logger.hollow());

    let mut list = Vec::<T>::with_capacity(length as usize);

    for i in 0..length {
        let item = if_err!((logger) [ListIO, err => ("While reading list element: {:?}", err)] retry container.read_data(i.to_string()));
        let item = if_err!((logger) [ListIO, err => ("While reading list element: {:?}", err)] {f(item)} crash {
            log!((logger.error) ListIO("{err:#?}") as Fatal);
            logger.crash()
        });
        list.push(item)
    }

    list.into_boxed_slice()
}

pub fn load_length(container: &LazyContainer, mut logger: impl Logger) -> u16 {
    let length = if_err!((logger) [ListIO, err => ("While reading list length: {:?}", err)] retry container.read_data("length"));
    if_err!((logger) [ListIO, err => ("While reading list length: {:?}", err)] {length.collect_u16()} crash {
        log!((logger.error) ListIO("{err:#?}") as Fatal);
        logger.crash()
    })
}

pub struct List<T, L: Logger, F: Fn(LazyData) -> Result<T, LDBError>> {
    _phantom_marker: PhantomData<T>,
    read_f: F,
    container: LazyContainer,
    logger: L,
    length: u16,
    idx: u16,
}

impl<T, L: Logger, F: Fn(LazyData) -> Result<T, LDBError>> List<T, L, F> {
    pub fn init(container: LazyContainer, read_f: F, mut logger: L) -> Self {
        if_err!((logger) [List, err => ("While inititalising list: {err:?}")] retry write_container!((container) length = new_u16(0)));
        Self {
            _phantom_marker: PhantomData,
            container,
            read_f,
            logger: logger.hollow(),
            length: 0,
            idx: 0,
        }
    }

    pub fn load(container: LazyContainer, read_f: F, mut logger: L) -> Self {
        let length = if_err!((logger) [List, err => ("While loading list: {err:?}")] retry 
            if_err!((logger) [List, err => ("While loading list: {err:?}")] retry
                search_container!((container) length)
            ).collect_u16()
        );

        Self {
            _phantom_marker: PhantomData,
            container,
            read_f,
            length,
            logger: logger.hollow(),
            idx: 0,
        }
    }
}

impl<T, L:Logger, F: Fn(LazyData) -> Result<T, LDBError>> Iterator for List<T, L, F> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.idx >= self.length { return None };
        let mut logger = self.logger.hollow();
        let f: &F = &self.read_f;

        let item = if_err!((logger) [ListIO, err => ("While reading list element: {:?}", err)] retry self.container.read_data(self.idx.to_string()));
        Some(if_err!((logger) [ListIO, err => ("While reading list element: {:?}", err)] {f(item)} crash {
            log!((logger.error) ListIO("{err:#?}") as Fatal);
            logger.crash()
        }))
    }
}