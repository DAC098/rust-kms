use std::path::{PathBuf, Path};
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::fs::error::Error;
use crate::fs::traits::Wrapper;
use crate::local::Local;

pub struct Options {
    pub path: PathBuf,
}

pub struct Json<KeyType> {
    manager: Local<KeyType>,
    path: Box<Path>,
}

impl<KeyType> Json<KeyType> {
    pub fn new<P>(manager: Local<KeyType>, path: P) -> Self
    where
        P: Into<PathBuf>
    {
        let buf = path.into();

        Json {
            manager,
            path: buf.into(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl<KeyType> std::ops::Deref for Json<KeyType> {
    type Target = Local<KeyType>;

    fn deref(&self) -> &Self::Target {
        &self.manager
    }
}

impl<KeyType> std::fmt::Debug for Json<KeyType>
where
    KeyType: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Json")
            .field("manager", &self.manager)
            .field("path", &self.path)
            .finish()
    }
}

impl<KeyType> Wrapper for Json<KeyType>
where
    KeyType: Serialize + DeserializeOwned
{
    type Error = Error;
    type Args = Options;

    fn load(options: Self::Args) -> Result<Self, Self::Error> {
        use serde_json::error::Category;

        let path = options.path.into();

        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|e| Error::Io(e))?;
        let reader = BufReader::new(file);

        let manager = serde_json::from_reader(reader)
            .map_err(|e| match e.classify() {
                Category::Io => Error::Io(e.into()),
                _ => Error::Json(e)
            })?;

        Ok(Json {
            manager,
            path
        })
    }

    fn save(&self) -> Result<(), Self::Error> {
        use serde_json::error::Category;

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|e| Error::Io(e))?;
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, &self.manager)
            .map_err(|e| match e.classify() {
                Category::Io => Error::Io(e.into()),
                _ => Error::Json(e)
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
        let file_name = "test.json";
        let manager = local::test::create_store();

        fs::test::create_test_file(file_name);

        let wrapper = Json::new(manager, file_name);

        wrapper.save().expect("failed to save to json file");

        let and_back: Json<u64> = Json::load(Options {
            path: PathBuf::from(file_name),
        }).expect("failed to load json file");

        local::test::assert_local_eq(&wrapper.manager, &and_back.manager);
    }
}
