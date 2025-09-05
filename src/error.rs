use camino::FromPathBufError;
use paste::paste;
use serde_yml::Error as YamlError;
use snafu::{Backtrace, Snafu};
use std::{io::Error as IoError, result::Result as BaseResult, string::FromUtf8Error};
use time::error::IndeterminateOffset as IndeterminateOffsetError;

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
    IndeterminateOffset {
        source: IndeterminateOffsetError,
        backtrace: Backtrace,
    },
    #[snafu(display("No goal named '{slug}' was found."))]
    GoalNotFound { slug: String, backtrace: Backtrace },
    #[snafu(display("The current state for the {slug} goal has not be loaded."))]
    GoalStateNotLoaded { slug: String, backtrace: Backtrace },
    #[snafu(display("{message}"))]
    Other {
        message: String,
        backtrace: Backtrace,
    },
}

impl Error {
    pub fn backtrace(&self) -> &Backtrace {
        match self {
            Self::Io { backtrace, .. } => backtrace,
            Self::FromUtf8 { backtrace, .. } => backtrace,
            Self::Yaml { backtrace, .. } => backtrace,
            Self::NonUtf8Path { backtrace, .. } => backtrace,
            Self::GoalStateNotLoaded { backtrace, .. } => backtrace,
            Self::GoalNotFound { backtrace, .. } => backtrace,
            Self::IndeterminateOffset { backtrace, .. } => backtrace,
            Self::Other { backtrace, .. } => backtrace,
        }
    }

    pub fn simple<S: AsRef<str>>(message: S) -> Self {
        Self::Other {
            message: String::from(message.as_ref()),
            backtrace: Backtrace::capture(),
        }
    }

    pub fn goal_not_found<S: AsRef<str>>(slug: S) -> Self {
        Self::GoalNotFound {
            slug: String::from(slug.as_ref()),
            backtrace: Backtrace::capture(),
        }
    }

    pub fn goal_state_not_loaded<S: AsRef<str>>(slug: S) -> Self {
        Self::GoalStateNotLoaded {
            slug: String::from(slug.as_ref()),
            backtrace: Backtrace::capture(),
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
impl_from! {IndeterminateOffset}

pub type Result<V> = BaseResult<V, Error>;
