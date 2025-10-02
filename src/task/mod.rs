use crate::{RcCell, config::DisabledOptions, state::State};
use crate::{Result, util::days_elapsed};
use serde::Serialize;
use time::{Date, OffsetDateTime};

mod config;
mod set;
mod state;
pub use config::*;
pub use set::*;
pub use state::*;

pub const DEFAULT_WEIGHT: f64 = 1.0;
pub const DEFAULT_SPOONS: u16 = 3;

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
    pub spoons: u16,
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
   spoons: u16,
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
    disabled_on: Option<Date>,
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

    pub fn choose(&self, state: &State) {
        self.state.borrow_mut().choose(state);
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
            spoons: config.spoons,
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
            DisabledOptions::Until(until) => until >= state.todays_date(),
            DisabledOptions::For(for_) => {
                state.days_since_today(task_state.disabled_on.unwrap()) >= for_ as i64
            }
        }
    }

    /// Returns `true` if the task can be chosen today.
    pub fn choosable(&self, the_state: &State) -> bool {
        let config = self.config.borrow();
        let state = self.state.borrow();
        if !(self.disabled(the_state) || the_state.todays_tasks().contains(&self.slug)) {
            false
        } else if let Some(max_occurrences) = config.max_occurrences
            && state.times_completed >= max_occurrences
        {
            false
        } else if let Some(min_frequency) = config.min_frequency {
            if let Some(last_chosen) = state.last_chosen {
                the_state.days_since_today(last_chosen) >= min_frequency as i64
            } else {
                true
            }
        } else {
            true
        }
    }
}
