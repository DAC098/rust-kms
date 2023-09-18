use std::path::{PathBuf, Path};
use std::fs::OpenOptions;
use std::io::{Read, Write, BufReader, BufWriter};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::fs::error::Error;
use crate::fs::traits::Wrapper;
use crate::local::Local;
use crate::crypto;

pub struct Options {
    path: PathBuf,
    key: crypto::Key,
}

pub struct Encrypted<KeyType> {
    manager: Local<KeyType>,
    path: Box<Path>,
    key: crypto::Key,
}

impl<KeyType> Encrypted<KeyType> {
    pub fn new<P>(manager: Local<KeyType>, path: P, key: crypto::Key) -> Self
    where
        P: Into<PathBuf>
    {
        let buf = path.into();

        Encrypted {
            manager,
            path: buf.into(),
            key,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn key(&self) -> &crypto::Key {
        &self.key
    }
}

impl<KeyType> std::ops::Deref for Encrypted<KeyType> {
    type Target = Local<KeyType>;

    fn deref(&self) -> &Self::Target {
        &self.manager
    }
}

impl<KeyType> Wrapper for Encrypted<KeyType>
where
    KeyType: Serialize + DeserializeOwned
{
    type Error = Error;
    type Args = Options;

    fn load(options: Self::Args) -> Result<Self, Self::Error> {
        let path = options.path.into();
        let key = options.key;

        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|e| Error::Io(e))?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();

        reader.read_to_end(&mut buffer)
            .map_err(|e| Error::Io(e))?;

        let decrypted = crypto::decrypt_data(&key, buffer)
            .map_err(|e| Error::Crypto(e))?;

        let manager = bincode::deserialize(decrypted.as_slice())
            .map_err(|e| match *e {
                bincode::ErrorKind::Io(io) => Error::Io(io),
                _ => Error::Bincode(e),
            })?;

        Ok(Encrypted {
            manager,
            path,
            key
        })
    }

    fn save(&self) -> Result<(), Self::Error> {
        let serialize = bincode::serialize(&self.manager)
            .map_err(|e| match *e {
                bincode::ErrorKind::Io(io) => Error::Io(io),
                _ => Error::Bincode(e)
            })?;

        let encrypted = crypto::encrypt_data(&self.key, serialize)
            .map_err(|e| Error::Crypto(e))?;

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|e| Error::Io(e))?;
        let mut writer = BufWriter::new(file);

        writer.write_all(encrypted.as_slice())
            .map_err(|e| Error::Io(e))?;

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
        let file_name = "test.encrypted";
        let manager = local::test::create_store();

        fs::test::create_test_file(file_name);

        let wrapper = Encrypted::new(manager, file_name, crypto::empty_key());

        wrapper.save().expect("failed to save to encrypted file");

        let and_back: Encrypted<u64> = Encrypted::load(Options {
            path: PathBuf::from(file_name),
            key: crypto::empty_key(),
        }).expect("failed to load encrypted file");

        local::test::assert_local_eq(&wrapper.manager, &and_back.manager);
    }
}
