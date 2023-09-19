use std::io::Error as IoError;
use std::fmt;

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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(_) => f.write_str("Io"),

            #[cfg(feature = "binary")]
            Error::Bincode(_) => f.write_str("Bincode"),

            #[cfg(feature = "json")]
            Error::Json(_) => f.write_str("Json"),

            #[cfg(feature = "crypto")]
            Error::Crypto(_) => f.write_str("Crypto"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),

            #[cfg(feature = "binary")]
            Error::Bincode(e) => Some(e),

            #[cfg(feature = "json")]
            Error::Json(e) => Some(e),

            #[cfg(feature = "crypto")]
            Error::Crypto(e) => Some(e),
        }
    }
}
