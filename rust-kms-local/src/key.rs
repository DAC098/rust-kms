use std::time::SystemTime;
use std::fmt;

#[cfg(feature = "rand")]
use rand::RngCore;


#[derive(Debug)]
pub enum Error {
    Timestamp,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Timestamp => f.write_str("Timestamp")
        }
    }
}

impl std::error::Error for Error {}

pub struct KeyBuilder<Data> {
    data: Data,
    created: Option<u64>,
}

impl<Data> KeyBuilder<Data> {
    pub fn set_created(&mut self, created: u64) -> () {
        self.created = Some(created);
    }

    pub fn build(self) -> Result<Key<Data>, Error> {
        let created = match self.created {
            Some(v) => v,
            None => {
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(|_| Error::Timestamp)?
                    .as_secs()
            }
        };

        Ok(Key {
            data: self.data,
            created
        })
    }
}

#[derive(Debug)]
pub struct Key<Data = Vec<u8>> {
    data: Data,
    created: u64
}

impl<Data> Key<Data> {
    pub fn builder(data: Data) -> KeyBuilder<Data> {
        KeyBuilder {
            data,
            created: None,
        }
    }

    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn created(&self) -> &u64 {
        &self.created
    }
}

#[cfg(feature = "rand")]
impl Key<Vec<u8>> {
    pub fn builder_thread_rng(&mut self, size: usize) -> Result<KeyBuilder<Vec<u8>>, rand::Error> {
        let mut bytes = vec![0; size];

        rand::thread_rng().try_fill_bytes(bytes.as_mut_slice())?;

        Ok(KeyBuilder {
            data: bytes,
            created: None
        })
    }

    pub fn builder_os_rng(&mut self, size: usize) -> Result<KeyBuilder<Vec<u8>>, rand::Error> {
        let mut bytes = vec![0; size];

        rand::rngs::OsRng.try_fill_bytes(bytes.as_mut_slice())?;

        Ok(KeyBuilder {
            data: bytes,
            created: None
        })
    }
}

#[cfg(feature = "rand")]
impl<const N: usize> Key<[u8; N]> {
    pub fn builder_thread_rng(&mut self) -> Result<KeyBuilder<[u8; N]>, rand::Error> {
        let mut bytes = [0; N];

        rand::thread_rng().try_fill_bytes(&mut bytes)?;

        Ok(KeyBuilder {
            data: bytes,
            created: None
        })
    }

    pub fn builder_os_rng(&mut self) -> Result<KeyBuilder<[u8; N]>, rand::Error> {
        let mut bytes = [0; N];

        rand::rngs::OsRng.try_fill_bytes(&mut bytes)?;

        Ok(KeyBuilder {
            data: bytes,
            created: None
        })
    }
}

impl<T> Key<Vec<T>> {
    pub fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }
}

impl<T, const N: usize> Key<[T; N]> {
    pub fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }
}

impl<Data> Clone for Key<Data>
where
    Data: Clone
{
    fn clone(&self) -> Self {
        Key {
            data: self.data.clone(),
            created: self.created
        }
    }
}

impl<Data> Copy for Key<Data>
where
    Data: Copy
{}

use serde::ser::{Serialize, Serializer, SerializeStruct};
use serde::de::{self, Deserialize, Deserializer, Visitor, MapAccess, SeqAccess};

impl<Data> Serialize for Key<Data>
where
    Data: Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Key", 2)?;
        state.serialize_field("data", &self.data)?;
        state.serialize_field("created", &self.created)?;
        state.end()
    }
}

impl<'de, Data> Deserialize<'de> for Key<Data>
where
    Data: Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        const STRUCT_FIELDS: &'static [&'static str] = &["data", "created"];

        enum KeyField {
            Data,
            Created
        }

        impl<'de> Deserialize<'de> for KeyField {
            fn deserialize<D>(deserializer: D) -> Result<KeyField, D::Error>
            where
                D: Deserializer<'de>
            {
                struct KeyFieldVisitor;

                impl<'de> Visitor<'de> for KeyFieldVisitor {
                    type Value = KeyField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("'data' or 'created'")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error
                    {
                        match value {
                            "data" => Ok(KeyField::Data),
                            "created" => Ok(KeyField::Created),
                            _ => Err(de::Error::unknown_field(value, STRUCT_FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(KeyFieldVisitor)
            }
        }

        struct KeyVisitor<Data> {
            _data: std::marker::PhantomData<Data>
        }

        impl<'de, Data> Visitor<'de> for KeyVisitor<Data>
        where
            Data: Deserialize<'de>
        {
            type Value = Key<Data>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Key")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>
            {
                let data = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let created = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                Ok(Key { data, created })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>
            {
                let mut data = None;
                let mut created = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        KeyField::Data => {
                            if data.is_some() {
                                return Err(de::Error::duplicate_field("data"));
                            }

                            data = Some(map.next_value()?);
                        }
                        KeyField::Created => {
                            if created.is_some() {
                                return Err(de::Error::duplicate_field("created"));
                            }

                            created = Some(map.next_value()?);
                        }
                    }
                }

                let data = data.ok_or_else(|| de::Error::missing_field("data"))?;
                let created = created.ok_or_else(|| de::Error::missing_field("created"))?;

                Ok(Key { data, created })
            }
        }

        deserializer.deserialize_struct(
            "Key",
            STRUCT_FIELDS,
            KeyVisitor {
                _data: std::marker::PhantomData
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serde() {
        let mut builder = Key::builder(1);
        builder.set_created(2);

        let key: Key<u64> = builder.build()
            .unwrap();

        let to_json = serde_json::to_string(&key)
            .expect("failed to serialize Key to json string");

        let and_back: Key<u64> = serde_json::from_str(&to_json)
            .expect("failed to deserialize Key from json string");

        assert_eq!(key.data, and_back.data, "data values are not equal");
        assert_eq!(key.created, and_back.created, "created values are not equal");
    }
}
