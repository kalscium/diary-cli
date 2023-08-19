use lazy_db::*;
use soulog::*;

pub fn write<T>(list: &[T], f: impl Fn(FileWrapper, &T) -> Result<(), LDBError>, container: LazyContainer, mut logger: impl Logger) {
    for (i, x) in list.iter().enumerate() {
        let data_writer = 
            if_err!((logger) [ListIO, err => ("While writing element of list: {:?}", err)] retry container.data_writer(i.to_string()));
        if_err!((logger) [ListIO, err => ("While writing element of list: {:?}", err)] {f(data_writer, x)} manual {
            Crash => {
                logger.error(Log::new(LogType::Fatal, "ListIO", &format!("{:#?}", err), &[]));
                logger.crash()
            }
        })
    }

    if_err!((logger) [ListIO, err => ("{:?}", err)] retry {
        let data_writer = if_err!((logger) [ListIO, err => ("While writing list length: {:?}", err)] retry container.data_writer("length"));
        LazyData::new_u8(data_writer, list.len() as u8)
    })
}

pub fn read<T>(f: impl Fn(LazyData) -> Result<T, LDBError>, container: LazyContainer, mut logger: impl Logger) -> Box<[T]> {
    let length = if_err!((logger) [ListIO, err => ("While reading list length: {:?}", err)] retry container.read_data("length"));
    let length = if_err!((logger) [ListIO, err => ("While reading list length: {:?}", err)] {length.collect_u8()} manual {
        Crash => {
            logger.error(Log::new(LogType::Fatal, "ListIO", &format!("{:#?}", err), &[]));
            logger.crash()
        }
    }) as usize;

    let mut list = Vec::<T>::with_capacity(length);

    for i in 0..length {
        let item = if_err!((logger) [ListIO, err => ("While reading list element: {:?}", err)] retry container.read_data(i.to_string()));
        let item = if_err!((logger) [ListIO, err => ("While reading list element: {:?}", err)] {f(item)} manual {
            Crash => {
                logger.error(Log::new(LogType::Fatal, "ListIO", &format!("{:#?}", err), &[]));
                logger.crash()
            }
        });
        list.push(item)
    }

    list.into_boxed_slice()
}