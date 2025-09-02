use super::CONFIG_FILE_PATH;
use crate::{Goal, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use time::{Time, macros::time};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct Config {
    #[serde(default)]
    pub goals: Vec<Goal>,
    pub cut_off: Time,
    pub daily_goals: usize,
}

impl Config {
    pub fn save(&self) -> Result<()> {
        let file = OpenOptions::new().write(true).open(&*CONFIG_FILE_PATH)?;
        serde_yml::to_writer(file, self)?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        if CONFIG_FILE_PATH.exists() {
            let data = fs::read(&*CONFIG_FILE_PATH)?;
            let config = serde_yml::from_slice(&data)?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn add_goal(&mut self, goal: Goal) {
        self.goals.push(goal);
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            goals: Vec::new(),
            cut_off: time!(04:00),
            daily_goals: 1,
        }
    }
}
