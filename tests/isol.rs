use std::fs;
use std::path::{Path, PathBuf};
use std::fmt::Display;
use std::fmt;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

pub fn new_env() -> TmpPath {
    // creates 'test_tmp' folder if it doesn't exist
    let path = Path::new("./test_tmp/");
    if !path.exists() {
        fs::create_dir_all(path).expect("Error: couldn't create directory!");
    }; TmpPath::new(format!("./test_tmp/{}", gen_random()))
}

pub fn gen_random() -> u64 { RandomState::new().build_hasher().finish() }

pub struct TmpPath(PathBuf);

impl TmpPath {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        if !path.exists() {
            match fs::create_dir_all(path) {
                Ok(_) => (),
                Err(e) => panic!("Error: creating directory: {:?}", e),
            }
        } Self(path.to_path_buf())
    }

    pub fn get_path(&self) -> &Path {
        self.0.as_path()
    }
}

impl Display for TmpPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}",  self.0.to_string_lossy())
    }
}

impl Drop for TmpPath {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}