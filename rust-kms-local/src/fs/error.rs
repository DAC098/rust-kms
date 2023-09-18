use std::io::Error as IoError;

#[derive(Debug)]
pub enum Error {
    Io(IoError),

    #[cfg(feature = "binary")]
    Bincode(bincode::Error),

    #[cfg(feature = "json")]
    Json(serde_json::Error),

    #[cfg(feature = "crypto")]
    Crypto(crate::crypto::Error),
}
