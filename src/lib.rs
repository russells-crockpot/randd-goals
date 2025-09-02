#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
extern crate lazy_static;

pub mod files;
pub use files::{Config, State};
mod goal;
pub use goal::{Goal, GoalBuilder};
pub mod commands;

pub(crate) mod error;
pub use error::{Error, Result};
