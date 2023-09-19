use std::path::{PathBuf, Path};
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::fs::error::Error;
use crate::fs::traits::Wrapper;
use crate::local::Local;

pub struct Options {
    path: PathBuf,
}

pub struct Binary<KeyType> {
    manager: Local<KeyType>,
    path: Box<Path>,
}

impl<KeyType> Binary<KeyType> {
    pub fn new<P>(manager: Local<KeyType>, path: P) -> Self
    where
        P: Into<PathBuf>
    {
        let buf = path.into();

        Binary {
            manager,
            path: buf.into(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl<KeyType> std::ops::Deref for Binary<KeyType> {
    type Target = Local<KeyType>;

    fn deref(&self) -> &Self::Target {
        &self.manager
    }
}

impl<KeyType> std::fmt::Debug for Binary<KeyType>
where
    KeyType: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Binary")
            .field("manager", &self.manager)
            .field("path", &self.path)
            .finish()
    }
}

impl<KeyType> Wrapper for Binary<KeyType>
where
    KeyType: Serialize + DeserializeOwned
{
    type Error = Error;
    type Args = Options;

    fn load(options: Self::Args) -> Result<Self, Self::Error> {
        let path = options.path.into();
        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|e| Error::Io(e))?;
        let reader = BufReader::new(file);

        let manager = bincode::deserialize_from(reader)
            .map_err(|e| match *e {
                bincode::ErrorKind::Io(io) => Error::Io(io),
                _ => Error::Bincode(e)
            })?;

        Ok(Binary {
            manager,
            path
        })
    }

    fn save(&self) -> Result<(), Self::Error> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|e| Error::Io(e))?;
        let writer = BufWriter::new(file);

        bincode::serialize_into(writer, &self.manager)
            .map_err(|e| match *e {
                bincode::ErrorKind::Io(io) => Error::Io(io),
                _ => Error::Bincode(e)
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::local;
    use crate::fs;

    #[test]
    fn base() {
        let file_name = "test.binary";
        let manager = local::test::create_store();

        fs::test::create_test_file(file_name);

        let wrapper = Binary::new(manager, file_name);

        wrapper.save().expect("failed to save to binary file");

        let and_back: Binary<u64> = Binary::load(Options{
            path: PathBuf::from(file_name),
        }).expect("failed to load binary file");

        local::test::assert_local_eq(&wrapper.manager, &and_back.manager);
    }
}
