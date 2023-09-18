mod traits;
pub use traits::Wrapper;

mod error;
pub use error::Error;

#[cfg(feature = "binary")]
pub mod binary;
#[cfg(feature = "binary")]
pub use binary::Binary;

#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "json")]
pub use json::Json;

#[cfg(feature = "crypto")]
pub mod encrypted;
#[cfg(feature = "crypto")]
pub use encrypted::Encrypted;

#[cfg(test)]
pub(crate) mod test {
    pub fn create_test_file<P>(path: P) -> std::fs::File
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
}
