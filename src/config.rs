use crate::{CONFIG_FILE_PATH, Error, Result, State};
use derive_builder::Builder;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, OpenOptions},
};
use strum::EnumIs;
use time::{Date, Duration, PrimitiveDateTime};
use time::{Time, macros::time};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct Config {
    #[serde(default)]
    pub goals: Vec<RefCell<GoalConfig>>,
    #[serde(skip)]
    goals_map: HashMap<String, RefCell<GoalConfig>>,
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
        let mut config = if CONFIG_FILE_PATH.exists() {
            let data = fs::read(&*CONFIG_FILE_PATH)?;
            serde_yml::from_slice(&data)?
        } else {
            let config = Self::default();
            config.save()?;
            config
        };
        config.goals_map = config
            .goals
            .iter()
            .map(|g| (g.borrow().slug.clone(), RefCell::clone(g)))
            .collect();
        Ok(config)
    }

    pub(crate) fn add_goal(&mut self, goal: GoalConfig) -> RefCell<GoalConfig> {
        let slug = goal.slug.clone();
        let goal = RefCell::new(goal);
        self.goals.push(RefCell::clone(&goal));
        self.goals_map.insert(slug, RefCell::clone(&goal));
        goal
    }

    pub fn get_goal<S: AsRef<str>>(&self, slug: S) -> Option<RefCell<GoalConfig>> {
        self.goals_map.get(slug.as_ref()).cloned()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            goals: Vec::new(),
            goals_map: HashMap::new(),
            cut_off: time!(04:00),
            daily_goals: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, EnumIs)]
pub enum DisabledOptions {
    For(Duration),
    Until(PrimitiveDateTime),
    //TODO (ser/de)serialize from bool
    Disabled,
    #[default]
    Enabled,
}

impl From<bool> for DisabledOptions {
    fn from(value: bool) -> Self {
        if value { Self::Disabled } else { Self::Enabled }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder, Getters)]
#[serde(rename_all = "kebab-case")]
#[builder(name = "GoalBuilder")]
#[getset(get = "pub")]
pub struct GoalConfig {
    #[builder(default = "self.default_slug()")]
    pub slug: String,
    pub goal: String,
    #[builder(default = "1.0")]
    pub weight: f64,
    #[serde(skip_serializing_if = "DisabledOptions::is_enabled")]
    pub disabled: DisabledOptions,
    #[serde(skip_serializing_if = "std::vec::Vec::is_empty")]
    pub tags: Vec<String>,
}

impl GoalBuilder {
    fn default_slug(&self) -> String {
        slug::slugify(self.goal.as_ref().unwrap())
    }
}

impl GoalConfig {
    pub fn enable(&mut self) {
        self.disabled = DisabledOptions::Enabled;
    }

    pub fn disable(&mut self) {
        self.disabled = DisabledOptions::Disabled;
    }
}
