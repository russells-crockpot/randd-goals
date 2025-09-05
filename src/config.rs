use crate::{
    CONFIG_FILE_PATH, Error, RcCell, Result, State,
    util::{LOCAL_OFFSET, now, now_with_cutoff, today},
};
use derive_builder::Builder;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::{
    cell::OnceCell,
    collections::HashMap,
    fs::{self, OpenOptions},
    ops::AddAssign,
};
use strum::EnumIs;
use time::{Date, Duration, OffsetDateTime, Time, UtcOffset, macros::time};

pub const DEFAULT_WEIGHT: f64 = 1.0;

lazy_static! {
    pub static ref DEFAULT_CUT_OFF: Time = time!(04:00);
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
#[serde(rename_all = "kebab-case")]
#[getset(get = "pub")]
pub struct Config {
    #[serde(default)]
    goals: Vec<RcCell<GoalConfig>>,
    cut_off: Time,
    daily_goals: usize,
    #[serde(skip)]
    #[getset(skip)]
    // We want this to be a OnceCell just in case we pass the cut-off while running.
    effective_date: OnceCell<Date>,
    #[serde(skip)]
    #[getset(skip)]
    goals_map: HashMap<String, RcCell<GoalConfig>>,
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
            .map(|g| (g.borrow().slug.clone(), RcCell::clone(g)))
            .collect();
        Ok(config)
    }

    pub(crate) fn add_goal(&mut self, goal: RcCell<GoalConfig>) -> Result<()> {
        let slug = goal.borrow().slug.clone();
        if self.contains_goal(&slug) {
            Err(Error::goal_already_exists(slug))
        } else {
            self.goals.push(RcCell::clone(&goal));
            self.goals_map.insert(slug, goal);
            Ok(())
        }
    }

    #[inline]
    pub fn contains_goal<S: AsRef<str>>(&self, slug: S) -> bool {
        self.goals_map.contains_key(slug.as_ref())
    }

    //pub fn merge_goal(&mut self, goal:)

    pub fn get_goal<S: AsRef<str>>(&self, slug: S) -> Option<RcCell<GoalConfig>> {
        self.goals_map.get(slug.as_ref()).cloned()
    }

    /// What today's date should be considered, taken the config's cut-off time.
    pub fn today(&self) -> Date {
        *self
            .effective_date
            .get_or_init(|| now_with_cutoff(self.cut_off))
    }

    pub fn date_with_cutoff(&self, date: Date) -> OffsetDateTime {
        let offset = UtcOffset::current_local_offset().unwrap();
        OffsetDateTime::new_in_offset(date, self.cut_off, offset)
    }
}

impl Default for Config {
    fn default() -> Self {
        let config = Self {
            goals: Vec::new(),
            goals_map: HashMap::new(),
            cut_off: *DEFAULT_CUT_OFF,
            effective_date: OnceCell::new(),
            daily_goals: 1,
        };
        // Populate what today is ASAP
        let _ = config.today();
        config
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, EnumIs, PartialEq)]
pub enum DisabledOptions {
    For(Duration),
    Until(OffsetDateTime),
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
    slug: String,
    pub goal: String,
    #[builder(default = "DEFAULT_WEIGHT")]
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

    pub fn tag<S: AsRef<str>>(&mut self, tag: S) -> &mut Self {
        let tag = String::from(tag.as_ref());
        self.tags = Some(if let Some(mut tags) = self.tags.take() {
            tags.push(tag);
            tags
        } else {
            vec![tag]
        });
        self
    }
}

impl GoalConfig {
    pub fn enable(&mut self) {
        self.disabled = DisabledOptions::Enabled;
    }

    pub fn disable(&mut self) {
        self.disabled = DisabledOptions::Disabled;
    }

    /// Takes the values from the `other` argument, and overrides the values in this struct as long
    /// as the value in the other struct is not the default value. **Note**: the `slug` property is
    /// never overwritten.
    pub(crate) fn merge(&mut self, other: Self) {
        if !other.goal.is_empty() {
            self.goal = other.goal;
        }
        if other.weight != DEFAULT_WEIGHT {
            self.weight = other.weight;
        }
        if other.disabled != DisabledOptions::Enabled {
            self.disabled = other.disabled;
        }
        for tag in other.tags.into_iter() {
            if !self.tags.contains(&tag) {
                self.tags.push(tag);
            }
        }
    }

    pub fn update(&mut self, other: GoalBuilder) {
        if let Some(goal) = other.goal {
            self.goal = goal;
        }
        if let Some(weight) = other.weight {
            self.weight = weight;
        }
        if let Some(disabled) = other.disabled {
            self.disabled = disabled;
        }
        if let Some(tags) = other.tags {
            for tag in tags.into_iter() {
                if !self.tags.contains(&tag) {
                    self.tags.push(tag);
                }
            }
        }
    }
}

impl AddAssign for GoalConfig {
    fn add_assign(&mut self, other: Self) {
        self.merge(other);
    }
}

impl AddAssign<GoalBuilder> for GoalConfig {
    fn add_assign(&mut self, other: GoalBuilder) {
        self.update(other);
    }
}
