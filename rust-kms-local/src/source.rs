use std::collections::BTreeMap;

pub struct Source {
    file: PathBuf,
    encoding: Encoding,
}

#[derive(Debug)]
pub struct Source {
    pub fn new(dir: PathBuf, encoding: Encoding) -> Self {
        Source { file, encoding }
    }

    pub fn load(&self) -> Result<BTreeMap<Key::Version
