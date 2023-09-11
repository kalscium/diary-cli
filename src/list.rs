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

pub struct List<T> {
    _phantom_marker: PhantomData<T>,
    container: LazyContainer,
    length: u16,
    idx: u16,
}

impl<T> List<T> {
    pub fn init(container: LazyContainer) {
        
    }

    pub fn load(container: LazyContainer) {

    }
}

impl<T> Iterator for List<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        todo!()
    }
}