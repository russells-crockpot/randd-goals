use camino::FromPathBufError;
use paste::paste;
use serde_yml::Error as YamlError;
use snafu::{Backtrace, Snafu};
use std::{io::Error as IoError, string::FromUtf8Error};

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum Error {
    Io {
        source: IoError,
        backtrace: Backtrace,
    },
    FromUtf8 {
        source: FromUtf8Error,
        backtrace: Backtrace,
    },
    Yaml {
        source: YamlError,
        backtrace: Backtrace,
    },
    NonUtf8Path {
        source: FromPathBufError,
        backtrace: Backtrace,
    },
    #[snafu(display("{message}"))]
    Other {
        message: String,
        //backtrace: Backtrace,
    },
}

impl Error {
    pub fn backtrace(&self) -> Option<&Backtrace> {
        match self {
            Self::Io { backtrace, .. } => Some(backtrace),
            Self::FromUtf8 { backtrace, .. } => Some(backtrace),
            Self::Yaml { backtrace, .. } => Some(backtrace),
            Self::NonUtf8Path { backtrace, .. } => Some(backtrace),
            _ => None,
        }
    }

    pub fn simple<S: AsRef<str>>(message: S) -> Self {
        Self::Other {
            message: String::from(message.as_ref()),
        }
    }
}

macro_rules! impl_from {
    ($type:path, $error:ident, $base_error:ident) => {
        impl From<$type> for $base_error {
            fn from(error: $type) -> Self {
                Self::$error {
                    source: error,
                    backtrace: Backtrace::capture(),
                }
            }
        }
    };
    ($type:path, $error:ident) => {
        impl_from! { $type, $error, Error }
    };
    ($name:ident) => {
        paste! {
            impl_from! { [<$name Error>], $name }
        }
    };
}

impl_from! {FromPathBufError, NonUtf8Path}
impl_from! {Io}
impl_from! {FromUtf8}
impl_from! {Yaml}

pub type Result<V> = core::result::Result<V, Error>;
