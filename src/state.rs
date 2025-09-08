use crate::{
    Error, RcCell, Result, STATE_DIR, STATE_FILE_PATH,
    config::{Config, DisabledOptions, TaskConfig},
    util::{now, today},
};
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, DirBuilder, OpenOptions},
};
use time::{Date, Duration, OffsetDateTime};

/// Model of the way data is serialized in the state file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StateModel {
    pub last_generated: OffsetDateTime,
    pub tasks: HashMap<String, RcCell<TaskState>>,
    pub todays_tasks: Vec<String>,
}

impl Default for StateModel {
    fn default() -> Self {
        StateModel {
            last_generated: now() - Duration::DAY,
            tasks: HashMap::new(),
            todays_tasks: Vec::new(),
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct TaskState {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub disabled_at: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub last_chosen: Option<Date>,
    pub completed: bool,
}

impl TaskState {
    pub fn reset(&mut self) {
        self.completed = false;
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }

    pub fn enable(&mut self) {
        self.disabled_at = None;
    }

    pub fn disable(&mut self) {
        self.disabled_at = Some(now());
    }

    pub fn choose(&mut self) {
        self.reset();
        self.last_chosen = Some(today());
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
                let task = Task {
                    slug: slug.clone(),
                    config: task_cfg,
                    state: RcCell::clone(task_state),
                };
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

    pub fn upsert_tasks<I>(&mut self, tasks: I) -> Result<()>
    where
        I: IntoIterator<Item = TaskConfig>,
    {
        tasks.into_iter().for_each(|t| self.upsert_task(t));
        Ok(())
    }

    pub fn task_names(&self) -> Vec<String> {
        self.tasks.keys().map(Clone::clone).collect()
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Disabled,
    Complete,
    Pending,
    Inactive,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct TaskInfo {
    pub task: String,
    pub status: TaskStatus,
    #[serde(skip)]
    pub slug: String,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub description: Option<String>,
    pub weight: f64,
    #[serde(skip_serializing_if = "DisabledOptions::is_enabled")]
    pub disabled: DisabledOptions,
    #[serde(skip_serializing_if = "std::vec::Vec::is_empty")]
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub struct Task {
    slug: String,
    config: RcCell<TaskConfig>,
    state: RcCell<TaskState>,
}

macro_rules! impl_task_config_getters {
    ($($name:ident: $type:ty,)+) => {
        impl Task {
        $(
            #[inline(always)]
            pub fn $name(&self) -> $type {
                self.config.borrow().$name().clone()
            }
        )*
        }
    }
}

impl_task_config_getters! {
   task: String,
   description: Option<String>,
   weight: f64,
   tags: Vec<String>,
}

macro_rules! impl_task_state_getters {
    ($($name:ident: $type:ty,)+) => {
        impl Task {
        $(
            #[inline(always)]
            pub fn $name(&self) -> $type {
                self.state.borrow().$name.clone()
            }
        )*
        }
    }
}
impl_task_state_getters! {
    disabled_at: Option<OffsetDateTime>,
    last_chosen: Option<Date>,
    completed: bool,
}

impl Task {
    pub(crate) fn new(config: TaskConfig, state: TaskState) -> Self {
        Self {
            slug: String::from(config.slug()),
            config: RcCell::new(config),
            state: RcCell::new(state),
        }
    }

    pub fn reset(&self) {
        self.state.borrow_mut().reset();
    }

    pub fn complete(&self) {
        self.state.borrow_mut().complete();
    }

    pub fn enable(&self) {
        self.state.borrow_mut().enable();
        self.config.borrow_mut().enable();
    }

    pub fn disable(&self) {
        self.state.borrow_mut().disable();
        self.config.borrow_mut().disable();
    }

    pub fn choose(&self) {
        self.state.borrow_mut().choose();
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn disabled_opts(&self) -> DisabledOptions {
        self.config.borrow().disabled().clone()
    }

    pub fn info(&self, state: &State) -> TaskInfo {
        let config = self.config.borrow();
        TaskInfo {
            slug: self.slug.clone(),
            status: self.status(state),
            task: config.task.clone(),
            description: config.description.clone(),
            disabled: config.disabled.clone(),
            tags: config.tags.clone(),
            weight: config.weight,
        }
    }

    pub fn status(&self, state: &State) -> TaskStatus {
        if self.disabled(state) {
            TaskStatus::Disabled
        } else if state.model.todays_tasks.contains(&self.slug) {
            if self.state.borrow().completed {
                TaskStatus::Complete
            } else {
                TaskStatus::Pending
            }
        } else {
            TaskStatus::Inactive
        }
    }

    pub fn disabled(&self, state: &State) -> bool {
        let task_config = self.config.borrow();
        let task_state = self.state.borrow();
        match task_config.disabled {
            DisabledOptions::Enabled => false,
            DisabledOptions::Disabled => true,
            DisabledOptions::Until(ref until) => todo!(),
            DisabledOptions::For(ref for_) => todo!(),
        }
    }

    /// Returns `true` if the task can be chosen today.
    pub fn choosable(&self, state: &State) -> bool {
        !(self.disabled(state) || state.model.todays_tasks.contains(&self.slug))
    }
}
