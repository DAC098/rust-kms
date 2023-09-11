pub trait KeyBuilder {
    type Version;
    type Output;

    fn build(self, version: Self::Version) -> Self::Output;
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

    fn update(&mut self, key: Self::Key) -> Result<Self::Key, Self::Error>;
    fn drop(&mut self, version: Self::Version) -> Result<Self::Key, Self::Error>;
}

