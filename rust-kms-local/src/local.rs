use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::{Mutex, RwLock, PoisonError};
use std::sync::RwLockReadGuard;
use std::path::Path;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Poisoned,

    #[cfg(any(feature = "binary", feature = "json"))]
    Io(std::io::Error),

    #[cfg(feature = "binary")]
    Bincode(bincode::Error),

    #[cfg(feature = "json")]
    Json(serde_json::Error)
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_e: PoisonError<T>) -> Self {
        Error::Poisoned
    }
}

#[derive(Debug)]
pub struct Local<KeyType> {
    store: RwLock<BTreeMap<u64, KeyType>>,
    count: Mutex<u64>,
}

impl<KeyType> Local<KeyType> {
    pub fn new() -> Self {
        Local {
            store: RwLock::new(BTreeMap::new()),
            count: Mutex::new(0),
        }
    }

    pub fn store_reader<'a>(&'a self) -> Result<RwLockReadGuard<'a, BTreeMap<u64, KeyType>>, Error> {
        Ok(self.store.read()?)
    }

    pub fn count(&self) -> Result<u64, Error> {
        let count_lock = self.count.lock()?;

        Ok(*count_lock)
    }

    pub fn update(&self, key: KeyType) -> Result<(), Error> {
        let mut version_lock = self.count.lock()?;
        let new_version = *version_lock + 1;

        {
            let mut store_writer = self.store.write()?;

            store_writer.insert(new_version, key);
        }

        *version_lock = new_version;

        Ok(())
    }

    pub fn drop(&self, version: &u64) -> Result<Option<KeyType>, Error> {
        let mut store_writer = self.store.write()?;

        Ok(store_writer.remove(version))
    }
}

impl<KeyType> Local<KeyType>
where
    KeyType: Clone
{
    pub fn get(&self, version: &u64) -> Result<Option<KeyType>, Error> {
        let store_reader = self.store.read()?;

        let Some(key) = store_reader.get(version) else {
            return Ok(None);
        };

        Ok(Some(key.clone()))
    }

    pub fn get_version_key(&self, version: &u64) -> Result<Option<(u64, KeyType)>, Error> {
        let store_reader = self.store.read()?;

        let Some((ver, key)) = store_reader.get_key_value(version) else {
            return Ok(None);
        };

        Ok(Some((*ver, key.clone())))
    }

    pub fn latest(&self) -> Result<Option<KeyType>, Error> {
        let store_reader = self.store.read()?;

        let Some((_, key)) = store_reader.last_key_value() else {
            return Ok(None);
        };

        Ok(Some(key.clone()))
    }

    pub fn latest_version_key(&self) -> Result<Option<(u64, KeyType)>, Error> {
        let store_reader = self.store.read()?;

        let Some((version, key)) = store_reader.last_key_value() else {
            return Ok(None);
        };

        Ok(Some((*version, key.clone())))
    }
}

use serde::ser::{Serialize, Serializer, SerializeStruct};
use serde::de::{self, Deserialize, Deserializer, Visitor, MapAccess, SeqAccess};

#[cfg(any(feature = "binary", feature = "json"))]
impl<KeyType> Local<KeyType>
where
    KeyType: de::DeserializeOwned
{
    #[cfg(feature = "binary")]
    fn from_binary_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>
    {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|e| Error::Io(e))?;
        let reader = std::io::BufReader::new(file);

        let deserialized = bincode::deserialize_from(reader)
            .map_err(|e| match *e {
                bincode::ErrorKind::Io(io) => Error::Io(io),
                _ => Error::Bincode(e)
            })?;

        Ok(deserialized)
    }

    #[cfg(feature = "json")]
    fn from_json_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>
    {
        use serde_json::error::Category;

        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|e| Error::Io(e))?;
        let reader = std::io::BufReader::new(file);

        let deserialized = serde_json::from_reader(reader)
            .map_err(|e| match e.classify() {
                Category::Io => Error::Io(e.into()),
                _ => Error::Json(e)
            })?;

        Ok(deserialized)
    }
}

#[cfg(any(feature = "binary", feature = "json"))]
impl<KeyType> Local<KeyType>
where
    KeyType: Serialize
{
    #[cfg(feature = "binary")]
    fn to_binary_file<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>
    {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|e| Error::Io(e))?;
        let writer = std::io::BufWriter::new(file);

        bincode::serialize_into(writer, self)
            .map_err(|e| match *e {
                bincode::ErrorKind::Io(io) => Error::Io(io),
                _ => Error::Bincode(e)
            })?;

        Ok(())
    }

    #[cfg(feature = "json")]
    fn to_json_file<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>
    {
        use serde_json::error::Category;

        let file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|e| Error::Io(e))?;
        let writer = std::io::BufWriter::new(file);

        serde_json::to_writer(writer, self)
            .map_err(|e| match e.classify() {
                Category::Io => Error::Io(e.into()),
                _ => Error::Json(e)
            })?;

        Ok(())
    }
}

impl<KeyType> Serialize for Local<KeyType>
where
    KeyType: Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Local", 2)?;
        state.serialize_field("count", &self.count)?;
        state.serialize_field("store", &self.store)?;
        state.end()
    }
}

impl<'de, KeyType> Deserialize<'de> for Local<KeyType>
where
    KeyType: Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        const STRUCT_FIELDS: &'static [&'static str] = &["count", "store"];

        enum LocalField {
            Count,
            Store,
        }

        impl<'de> Deserialize<'de> for LocalField {
            fn deserialize<D>(deserializer: D) -> Result<LocalField, D::Error>
            where
                D: Deserializer<'de>
            {
                struct LocalFieldVisitor;

                impl<'de> Visitor<'de> for LocalFieldVisitor {
                    type Value = LocalField;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("'count' for 'store'")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error
                    {
                        match value {
                            "count" => Ok(LocalField::Count),
                            "store" => Ok(LocalField::Store),
                            _ => Err(de::Error::unknown_field(value, STRUCT_FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(LocalFieldVisitor)
            }
        }

        struct LocalVisitor<KeyType> {
            _key: PhantomData<KeyType>
        }

        impl<'de, KeyType> Visitor<'de> for LocalVisitor<KeyType>
        where
            KeyType: Deserialize<'de>
        {
            type Value = Local<KeyType>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Local")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>
            {
                let count = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let store = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                Ok(Local { count, store })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>
            {
                let mut count = None;
                let mut store = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        LocalField::Count => {
                            if count.is_some() {
                                return Err(de::Error::duplicate_field("count"));
                            }

                            count = Some(map.next_value()?);
                        }
                        LocalField::Store => {
                            if store.is_some() {
                                return Err(de::Error::duplicate_field("store"));
                            }

                            store = Some(map.next_value()?);
                        }
                    }
                }

                let count = count.ok_or_else(|| de::Error::missing_field("count"))?;
                let store = store.ok_or_else(|| de::Error::missing_field("store"))?;

                Ok(Local { count, store })
            }
        }

        deserializer.deserialize_struct(
            "Local",
            STRUCT_FIELDS,
            LocalVisitor {
                _key: PhantomData
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    type TestLocal = Local<u64>;

    fn assert_local_eq<K>(a: &Local<K>, b: &Local<K>)
    where
        K: std::cmp::PartialEq + std::fmt::Debug
    {
        {
            let count_a = a.count.lock().unwrap();
            let count_b = b.count.lock().unwrap();

            assert_eq!(*count_a, *count_b, "counts are not equal");
        }

        {
            let store_a = a.store.read().unwrap();
            let store_b = b.store.read().unwrap();

            assert_eq!(*store_a, *store_b, "stores are not equal");
        }
    }

    fn create_store() -> TestLocal {
        let local = Local::new();
        let values = [0, 1, 2, 4, 5, 9, 11, 12];

        for v in &values {
            local.update(*v).expect("failed to add value");
        }

        local
    }

    fn create_test_file<P>(path: P) -> std::fs::File
    where
        P: AsRef<std::path::Path>
    {
        std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .expect("failed to create test file")
    }

    #[test]
    fn serde() {
        let local = create_store();

        let to_json = serde_json::to_string(&local)
            .expect("failed to serialize Local to json string");

        let and_back: TestLocal = serde_json::from_str(&to_json)
            .expect("failed to deserialize Local from json string");

        assert_local_eq(&local, &and_back)
    }

    #[test]
    #[cfg(feature = "binary")]
    fn binary() {
        let file_name = "test.binary";
        let local = create_store();

        create_test_file(file_name);

        local.to_binary_file(file_name)
            .expect("failed to save to binary file");

        let and_back: TestLocal = Local::from_binary_file(file_name)
            .expect("failed to load binary file");

        assert_local_eq(&local, &and_back);
    }

    #[test]
    #[cfg(feature = "json")]
    fn json() {
        let file_name = "test.json";
        let local = create_store();

        create_test_file(file_name);

        local.to_json_file(file_name)
            .expect("failed to save to json file");

        let and_back: TestLocal = Local::from_json_file(file_name)
            .expect("failed to load json file");

        assert_local_eq(&local, &and_back);
    }
}
