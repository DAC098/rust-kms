pub trait Key {
    type Version;

    fn version(&self) -> Version;
}

pub trait Manager {
    type Key;
    type Version;
    type Error;

    fn get(&self, version: Self::Version) -> Result<Self::Key, Self::Error>;
    fn latest(&self) -> Result<Self::Key, Self::Error>;
}

pub trait MutManager {
    type Key;
    type Version;
    type Error;

    fn create(&mut self, key: Self::Key) -> Result<Self::Key, Self::Error>;
    fn delete(&mut self, version: Self::Version) -> Result<Self::Key, Self::Error>;
}
