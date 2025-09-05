#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
extern crate lazy_static;

use camino::Utf8PathBuf;

pub mod state;
pub use state::{Goal, GoalState, State};
pub mod config;
pub use config::{Config, GoalConfig};
pub mod commands;
pub(crate) mod error;
pub use error::{Error, Result};
pub mod util;

lazy_static! {
    pub static ref CONFIG_FILE_PATH: Utf8PathBuf = {
        let mut path = Utf8PathBuf::try_from(dirs::config_dir().unwrap()).unwrap();
        path.set_file_name("randd-goals.yaml");
        path
    };
    pub static ref STATE_FILE_PATH: Utf8PathBuf = {
        let mut path = Utf8PathBuf::try_from(dirs::cache_dir().unwrap()).unwrap();
        path.push("randd-goals");
        path.set_file_name("state.yaml");
        path
    };
    pub static ref HISTORY_FILE_PATH: Utf8PathBuf = {
        let mut path = Utf8PathBuf::try_from(dirs::cache_dir().unwrap()).unwrap();
        path.push("randd-goals");
        path.set_file_name("history.jsonlines");
        path
    };
}
