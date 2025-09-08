use crate::{
    Error, RcCell, Result, STATE_FILE_PATH,
    config::{Config, DisabledOptions, TaskConfig},
    util::{now, today},
};
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
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
        let file = OpenOptions::new().write(true).open(&*STATE_FILE_PATH)?;
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
            self.save()?;
            Ok(())
        } else {
            Err(Error::task_state_not_loaded(slug))
        }
    }

    pub fn disable_task<S: AsRef<str>>(&self, slug: S) -> Result<()> {
        if let Some(task) = self.tasks.get(slug.as_ref()) {
            task.disable();
            self.save()?;
            Ok(())
        } else {
            Err(Error::task_state_not_loaded(slug))
        }
    }

    fn add_task_no_save(&mut self, task_config: TaskConfig) -> Result<()> {
        let slug = String::from(task_config.slug());
        let task = Task::new(task_config, TaskState::default());
        self.config.add_task(RcCell::clone(&task.config))?;
        self.model
            .tasks
            .insert(slug.clone(), RcCell::clone(&task.state));
        self.tasks.insert(slug, task);
        Ok(())
    }

    fn update_task_no_save(&self, task_config: TaskConfig) -> Result<()> {
        if let Some(task) = self.tasks.get(task_config.slug()) {
            let mut borrowed = task.config.borrow_mut();
            (*borrowed) += task_config;
            Ok(())
        } else {
            Err(Error::task_not_found(task_config.slug()))
        }
    }

    fn upsert_task(&mut self, task_config: TaskConfig) {
        if self.tasks.contains_key(task_config.slug()) {
            self.update_task_no_save(task_config).unwrap()
        } else {
            self.add_task_no_save(task_config).unwrap()
        }
    }

    pub fn add_task(&mut self, task: TaskConfig) -> Result<()> {
        self.add_task_no_save(task)?;
        self.save()?;
        Ok(())
    }

    pub fn add_tasks<I>(&mut self, tasks: I) -> Result<()>
    where
        I: IntoIterator<Item = TaskConfig>,
    {
        tasks
            .into_iter()
            .try_for_each(|g| self.add_task_no_save(g))?;
        self.save()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Task {
    slug: String,
    config: RcCell<TaskConfig>,
    state: RcCell<TaskState>,
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

    /// Returns `true` if the task can be chosen today.
    pub fn choosable(&self, state: &State) -> bool {
        let task_config = self.config.borrow();
        let task_state = self.state.borrow();
        if state.model.todays_tasks.contains(&self.slug)
            || task_config.disabled == DisabledOptions::Disabled
        {
            false
        } else if let DisabledOptions::Until(ref until) = task_config.disabled {
            //let cutoff
            todo!()
        } else {
            true
        }
    }
}
