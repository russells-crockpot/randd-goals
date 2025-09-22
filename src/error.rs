use crate::task::{StepBuilderError, TaskBuilderError};
use camino::FromPathBufError as NonUtf8PathError;
use csv::Error as CsvError;
use notify_rust::error::Error as NotificationError;
use pastey::paste;
use rand::distr::weighted::Error as RandWeightError;
use serde_norway::Error as YamlError;
use snafu::{AsBacktrace, Backtrace, Snafu};
use std::{io::Error as IoError, result::Result as BaseResult, string::FromUtf8Error};
use time::{
    error::IndeterminateOffset as IndeterminateOffsetError, error::Parse as DateParsingError,
};

macro_rules! impl_error {
    ($($name:ident,)+) => {
        paste! {

            #[derive(Snafu, Debug)]
            #[snafu(visibility(pub))]
            pub enum Error {
                $(
                    $name {
                        source: [<$name Error>],
                        backtrace: Backtrace,
                    },
                )*
            }

            impl Error {
                pub fn backtrace(&self) -> &Backtrace {
                    match self {$(
                        Self::$name { backtrace, .. } => backtrace,
                    )*}
                }
            }
            $(
                impl From<[<$name Error>]> for Error {
                    fn from(error: [<$name Error>]) -> Self {
                        Self::$name {
                            source: error,
                            backtrace: Backtrace::new(),
                        }
                    }
                }
            )*
        }
    }
}

impl_error! {
    RanddGoals,
    NonUtf8Path,
    Io,
    Notification,
    RandWeight,
    FromUtf8,
    Yaml,
    IndeterminateOffset,
    TaskBuilder,
    StepBuilder,
    Csv,
    DateParsing,
}

impl Error {
    #[inline(always)]
    pub(crate) fn simple<S: AsRef<str>>(message: S) -> Self {
        let source = RanddGoalsError::Other {
            message: String::from(message.as_ref()),
        };
        Self::RanddGoals {
            source,
            backtrace: Backtrace::new(),
        }
    }

    #[inline(always)]
    pub(crate) fn task_not_found<S: AsRef<str>>(slug: S) -> Self {
        let source = RanddGoalsError::TaskNotFound {
            slug: String::from(slug.as_ref()),
        };
        Self::RanddGoals {
            source,
            backtrace: Backtrace::new(),
        }
    }

    #[inline(always)]
    pub(crate) fn unsupported_file_type<S: AsRef<str>>(extension: S) -> Self {
        let source = RanddGoalsError::UnsupportedFileType {
            extension: String::from(extension.as_ref()),
        };
        Self::RanddGoals {
            source,
            backtrace: Backtrace::new(),
        }
    }

    #[inline(always)]
    pub(crate) fn task_already_exists<S: AsRef<str>>(slug: S) -> Self {
        let source = RanddGoalsError::TaskAlreadyExists {
            slug: String::from(slug.as_ref()),
        };
        Self::RanddGoals {
            source,
            backtrace: Backtrace::new(),
        }
    }

    #[inline(always)]
    pub(crate) fn task_state_not_loaded<S: AsRef<str>>(slug: S) -> Self {
        let source = RanddGoalsError::TaskStateNotLoaded {
            slug: String::from(slug.as_ref()),
        };
        Self::RanddGoals {
            source,
            backtrace: Backtrace::new(),
        }
    }
}

impl AsBacktrace for Error {
    #[inline(always)]
    fn as_backtrace(&self) -> Option<&Backtrace> {
        Some(self.backtrace())
    }
}

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum RanddGoalsError {
    #[snafu(display("A task named '{slug}' already exists."))]
    TaskAlreadyExists { slug: String },
    #[snafu(display("No task named '{slug}' was found."))]
    TaskNotFound { slug: String },
    #[snafu(display("The current state for the {slug} task has not be loaded."))]
    TaskStateNotLoaded { slug: String },
    #[snafu(display("Files with the extension '{extension}' are not supported"))]
    UnsupportedFileType { extension: String },
    #[snafu(display("{message}"))]
    Other { message: String },
}

pub type Result<V, E = Error> = BaseResult<V, E>;
