use camino::Utf8PathBuf;

pub mod config;
pub mod history;
pub mod state;

pub use config::Config;
pub use state::State;

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
