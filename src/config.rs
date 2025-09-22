use crate::{CONFIG_FILE_PATH, Error, RcCell, Result, TaskConfig, util::now_with_cutoff};
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::{
    cell::OnceCell,
    collections::HashMap,
    fs::{self, OpenOptions},
};
use strum::EnumIs;
use time::{Date, Duration, OffsetDateTime, Time, UtcOffset, macros::time};

lazy_static! {
    pub static ref DEFAULT_CUT_OFF: Time = time!(04:00);
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
#[serde(rename_all = "kebab-case")]
#[getset(get = "pub")]
pub struct Config {
    #[serde(default)]
    tasks: Vec<RcCell<TaskConfig>>,
    cut_off: Time,
    limit_by: LimitTasksBy,
    #[serde(skip)]
    #[getset(skip)]
    // We want this to be a OnceCell just in case we pass the cut-off while running.
    effective_date: OnceCell<Date>,
    #[serde(skip)]
    #[getset(skip)]
    tasks_map: HashMap<String, RcCell<TaskConfig>>,
}

impl Config {
    pub fn save(&self) -> Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&*CONFIG_FILE_PATH)?;
        serde_norway::to_writer(file, self)?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let mut config = if CONFIG_FILE_PATH.exists() {
            let data = fs::read(&*CONFIG_FILE_PATH)?;
            serde_norway::from_slice(&data)?
        } else {
            let config = Self::default();
            config.save()?;
            config
        };
        config.tasks_map = config
            .tasks
            .iter()
            .map(|g| (String::from(g.borrow().slug()), RcCell::clone(g)))
            .collect();
        Ok(config)
    }

    pub(crate) fn add_task(&mut self, task: RcCell<TaskConfig>) -> Result<()> {
        let slug = String::from(task.borrow().slug());
        if self.contains_task(&slug) {
            Err(Error::task_already_exists(slug))
        } else {
            self.tasks.push(RcCell::clone(&task));
            self.tasks_map.insert(slug, task);
            Ok(())
        }
    }

    #[inline]
    pub fn contains_task<S: AsRef<str>>(&self, slug: S) -> bool {
        self.tasks_map.contains_key(slug.as_ref())
    }

    pub fn get_task<S: AsRef<str>>(&self, slug: S) -> Option<RcCell<TaskConfig>> {
        self.tasks_map.get(slug.as_ref()).cloned()
    }

    pub fn task_slugs(&self) -> Vec<String> {
        self.tasks
            .iter()
            .map(|t| String::from(t.borrow().slug()))
            .collect()
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

    pub fn remove_task<S: AsRef<str>>(&mut self, slug: S) {
        if let Some(pos) = self
            .tasks
            .iter()
            .position(|t| t.borrow().slug() == slug.as_ref())
        {
            self.tasks.remove(pos);
            self.tasks_map.remove(slug.as_ref());
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let config = Self {
            tasks: Vec::new(),
            tasks_map: HashMap::new(),
            cut_off: *DEFAULT_CUT_OFF,
            effective_date: OnceCell::new(),
            limit_by: LimitTasksBy::Tasks { tasks: 1 },
        };
        // Populate what today is ASAP
        let _ = config.today();
        config
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumIs, PartialEq)]
#[serde(untagged)]
pub enum LimitTasksBy {
    Tasks { tasks: usize },
    Spoons { spoons: u16 },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, EnumIs, PartialEq)]
pub enum DisabledOptions {
    For(u32),
    Until(Date),
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
