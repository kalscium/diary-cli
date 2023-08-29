use std::{io::{BufWriter, Write}, fs::File, path::Path};
use soulog::*;

pub struct Scribe<T: Logger>(BufWriter<File>, T);

impl<T: Logger> Scribe<T> {
    pub fn new(path: impl AsRef<Path>, mut logger: T) -> Self {
        let file = if_err!((logger) [Scribe, err => ("While creating text file: {err:?}")] retry File::create(&path));
        let buffer = BufWriter::new(file);
        Self(buffer, logger)
    }

    #[inline]
    pub fn write_line(&mut self, line: &str) {
        self.write(line);
        self.new_line();
    }

    pub fn write(&mut self, text: &str) {
        let mut logger = self.1.hollow();
        if_err!((logger) [Scribe, err => ("While writing to text file: {err:?}")] retry self.0.write_all(text.as_bytes()));
    }

    #[inline]
    pub fn new_line(&mut self) {
        let mut logger = self.1.hollow();
        if_err!((logger) [Scribe, err => ("While writing to text file: {err:?}")] retry self.0.write_all("\n".as_bytes()));
    }

    pub fn flush(&mut self) {
        let logger = &mut self.1;
        if_err!((logger) [Sribe, err => ("While writing to text file: {err:?}")] retry self.0.flush());
    }

    #[inline]
    pub fn finish(mut self) { self.flush() }
}

impl<T: Logger> Drop for Scribe<T> {
    #[inline]
    fn drop(&mut self) { self.flush() }
}

#[macro_export]
macro_rules! scribe_write {
    (($scribe:ident) $($text:expr),* $(,)?) => {{
        $($scribe.write($text);)*
    }}
}