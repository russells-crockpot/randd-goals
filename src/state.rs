use crate::{
    Error, RcCell, Result, STATE_DIR, STATE_FILE_PATH,
    config::Config,
    task::{Task, TaskConfig, TaskSet, TaskState},
    util::{dt_with_cutoff, now},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, DirBuilder, OpenOptions},
};
use time::{Date, Duration, OffsetDateTime, Time};

/// Model of the way data is serialized in the state file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StateModel {
    pub last_generated: OffsetDateTime,
    pub tasks: HashMap<String, RcCell<TaskState>>,
    pub todays_tasks: TaskSet,
}

impl Default for StateModel {
    fn default() -> Self {
        StateModel {
            last_generated: now() - Duration::DAY,
            tasks: HashMap::new(),
            todays_tasks: TaskSet::new(),
        }
    }
}

impl StateModel {
    pub fn load() -> Result<Self> {
        if STATE_FILE_PATH.exists() {
            let data = fs::read(&*STATE_FILE_PATH)?;
            serde_yml::from_slice(&data).map_err(|e| e.into())
        } else {
            let state = Self::default();
            state.save()?;
            Ok(state)
        }
    }

    pub fn save(&self) -> Result<()> {
        DirBuilder::new().recursive(true).create(&*STATE_DIR)?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&*STATE_FILE_PATH)?;
        serde_yml::to_writer(file, self).map_err(|e| e.into())
    }
}

#[derive(Debug)]
pub struct State {
    config: Config,
    model: StateModel,
    tasks: HashMap<String, Task>,
}

impl State {
    pub fn load() -> Result<Self> {
        let config = Config::load()?;
        let model = if STATE_FILE_PATH.exists() {
            let data = fs::read(&*STATE_FILE_PATH)?;
            serde_yml::from_slice(&data)?
        } else {
            StateModel::default()
        };
        let mut orphans = Vec::new();
        let mut tasks = HashMap::new();
        for (slug, task_state) in model.tasks.iter() {
            if let Some(task_cfg) = config.get_task(slug) {
                let task = Task::new_raw(task_cfg, RcCell::clone(task_state));
                tasks.insert(slug.clone(), task);
            } else {
                orphans.push(String::from(slug));
            }
        }
        //TODO report on orphans
        Ok(Self {
            config,
            model,
            tasks,
        })
    }

    pub fn save(&self) -> Result<()> {
        self.model.save()?;
        self.config.save()?;
        Ok(())
    }

    pub fn enable_task<S: AsRef<str>>(&self, slug: S) -> Result<()> {
        if let Some(task) = self.tasks.get(slug.as_ref()) {
            task.enable();
            Ok(())
        } else {
            Err(Error::task_not_found(slug))
        }
    }

    pub fn enable_tasks<I, S>(&self, slugs: I) -> Result<()>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        slugs.into_iter().try_for_each(|t| self.enable_task(t))?;
        Ok(())
    }

    pub fn disable_task<S: AsRef<str>>(&self, slug: S) -> Result<()> {
        if let Some(task) = self.tasks.get(slug.as_ref()) {
            task.disable();
            Ok(())
        } else {
            Err(Error::task_not_found(slug))
        }
    }

    pub fn disable_tasks<I, S>(&self, slugs: I) -> Result<()>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        slugs.into_iter().try_for_each(|t| self.disable_task(t))?;
        Ok(())
    }

    pub fn get_task<S: AsRef<str>>(&self, slug: S) -> Option<&Task> {
        self.tasks.get(slug.as_ref())
    }

    pub fn remove_task<S: AsRef<str>>(&mut self, slug: S) -> Result<()> {
        if self.tasks.remove(slug.as_ref()).is_some() {
            Ok(())
        } else {
            Err(Error::task_not_found(slug))
        }
    }

    pub fn remove_tasks<I, S>(&mut self, tasks: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        tasks.into_iter().try_for_each(|s| self.remove_task(s))?;
        Ok(())
    }

    pub fn add_task(&mut self, task_config: TaskConfig) -> Result<()> {
        let slug = String::from(task_config.slug());
        let task = Task::new(task_config, TaskState::default());
        self.config.add_task(RcCell::clone(&task.config))?;
        self.model
            .tasks
            .insert(slug.clone(), RcCell::clone(&task.state));
        self.tasks.insert(slug, task);
        Ok(())
    }

    pub fn add_tasks<I>(&mut self, tasks: I) -> Result<()>
    where
        I: IntoIterator<Item = TaskConfig>,
    {
        tasks.into_iter().try_for_each(|t| self.add_task(t))?;
        Ok(())
    }

    pub fn update_task(&self, task_config: TaskConfig) -> Result<()> {
        if let Some(task) = self.tasks.get(task_config.slug()) {
            let mut borrowed = task.config.borrow_mut();
            (*borrowed) += task_config;
            Ok(())
        } else {
            Err(Error::task_not_found(task_config.slug()))
        }
    }

    pub fn update_tasks<I>(&mut self, tasks: I) -> Result<()>
    where
        I: IntoIterator<Item = TaskConfig>,
    {
        tasks.into_iter().try_for_each(|t| self.update_task(t))?;
        Ok(())
    }

    pub fn upsert_task(&mut self, task_config: TaskConfig) {
        if self.tasks.contains_key(task_config.slug()) {
            self.update_task(task_config).unwrap()
        } else {
            self.add_task(task_config).unwrap()
        }
    }

    pub fn upsert_tasks<I>(&mut self, tasks: I)
    where
        I: IntoIterator<Item = TaskConfig>,
    {
        tasks.into_iter().for_each(|t| self.upsert_task(t));
    }

    #[inline]
    pub fn task_names(&self) -> Vec<String> {
        self.tasks.keys().map(Clone::clone).collect()
    }

    #[inline]
    pub fn tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    #[inline]
    pub fn todays_tasks(&self) -> &TaskSet {
        &self.model.todays_tasks
    }

    #[inline]
    pub fn todays_tasks_mut(&mut self) -> &mut TaskSet {
        &mut self.model.todays_tasks
    }

    #[inline]
    pub fn last_generated(&self) -> &OffsetDateTime {
        &self.model.last_generated
    }

    /// Returns the generated date, with the configured cut-off taken into account.
    #[inline]
    pub fn last_generated_date(&self) -> Date {
        dt_with_cutoff(&self.model.last_generated, self.cut_off())
    }

    #[inline]
    pub fn mark_generated(&mut self) {
        self.model.last_generated = now();
    }
}

// Implements config delegates
impl State {
    #[inline]
    pub fn config(&self) -> &Config {
        &self.config
    }

    #[inline]
    pub fn cut_off(&self) -> Time {
        *self.config.cut_off()
    }

    #[inline]
    pub fn daily_tasks(&self) -> usize {
        *self.config.daily_tasks()
    }

    #[inline]
    pub fn todays_date(&self) -> Date {
        self.config.today()
    }
}
