use core::fmt::Display;

#[derive(Debug)]
pub enum Error {
    InvalidMarker(u8),
    InvalidChar(u32),
    InvalidUtf8(core::str::Utf8Error),
    BufferTooSmall,
    CannotBorrow,
    ArrayLengthMismatch {
        expected: usize,
        actual: usize,
    },
    MapLengthMismatch {
        expected: usize,
        actual: usize,
    },
    UnknownKey(alloc::string::String),
    KeyNotFound(alloc::string::String),
    KeyDuplicated(alloc::string::String),
    InvalidTimestamp,
    DepthLimitExceeded {
        max: usize,
    },
    #[cfg(feature = "std")]
    IoError(std::io::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidMarker(byte) => write!(f, "Invalid marker: {}", byte),
            Error::InvalidChar(code) => write!(f, "Invalid char code: {}", code),
            Error::BufferTooSmall => write!(f, "Buffer too small"),
            Error::InvalidUtf8(err) => write!(f, "Invalid UTF-8: {}", err),
            Error::CannotBorrow => write!(f, "Cannot borrow data from original buffer"),
            Error::ArrayLengthMismatch { expected, actual } => {
                write!(
                    f,
                    "Array length mismatch: expected {}, actual {}",
                    expected, actual
                )
            }
            Error::MapLengthMismatch { expected, actual } => {
                write!(
                    f,
                    "Map length mismatch: expected {}, actual {}",
                    expected, actual
                )
            }
            Error::UnknownKey(key) => {
                write!(f, "Unknown key '{}'", key)
            }
            Error::KeyNotFound(key) => {
                write!(f, "Key '{}' not found", key)
            }
            Error::KeyDuplicated(key) => {
                write!(f, "Key '{}' is duplicated", key)
            }
            Error::InvalidTimestamp => write!(f, "Invalid timestamp value"),
            Error::DepthLimitExceeded { max } => {
                write!(f, "Maximum deserialization depth exceeded (max: {})", max)
            }
            #[cfg(feature = "std")]
            Error::IoError(err) => err.fmt(f),
        }
    }
}

impl core::error::Error for Error {}
