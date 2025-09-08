use crate::{
    Error, RcCell, Result, STATE_DIR, STATE_FILE_PATH,
    config::{Config, DisabledOptions, TaskConfig},
    state::{State, TaskState},
    util::{now, today},
};
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, DirBuilder, OpenOptions},
};
use time::{Date, Duration, OffsetDateTime};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Disabled,
    Complete,
    InProgress,
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
    pub(crate) config: RcCell<TaskConfig>,
    pub(crate) state: RcCell<TaskState>,
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
    pub(crate) fn new_raw(config: RcCell<TaskConfig>, state: RcCell<TaskState>) -> Self {
        let slug = String::from(config.borrow().slug());
        Self {
            slug,
            config,
            state,
        }
    }
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
        } else if state.todays_tasks().contains(&self.slug) {
            if self.state.borrow().completed {
                TaskStatus::Complete
            } else {
                TaskStatus::InProgress
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
        !(self.disabled(state) || state.todays_tasks().contains(&self.slug))
    }
}
