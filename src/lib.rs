#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
extern crate lazy_static;

use camino::Utf8PathBuf;

pub mod state;
pub use state::State;
pub mod task;
pub use task::{Task, TaskBuilder, TaskConfig, TaskInfo, TaskState};
pub mod config;
pub use config::Config;
pub mod commands;
pub use commands::Cli;
pub(crate) mod error;
pub use error::{Error, Result};
pub mod util;
pub use util::RcCell;
mod picker;
pub(crate) use picker::*;
pub mod serializers;

lazy_static! {
    pub static ref CONFIG_FILE_PATH: Utf8PathBuf = {
        let mut path = Utf8PathBuf::try_from(dirs::config_dir().unwrap()).unwrap();
        path.push("randd-tasks.yaml");
        path
    };
    pub static ref STATE_DIR: Utf8PathBuf = {
        let mut path = Utf8PathBuf::try_from(dirs::cache_dir().unwrap()).unwrap();
        path.push("randd-tasks");
        path
    };
    pub static ref STATE_FILE_PATH: Utf8PathBuf = {
        let mut path = STATE_DIR.clone();
        path.push("state.yaml");
        path
    };
    pub static ref HISTORY_FILE_PATH: Utf8PathBuf = {
        let mut path = STATE_DIR.clone();
        path.push("history.jsonlines");
        path
    };
}
