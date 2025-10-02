use crate::{state::State, util::today};
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct TaskState {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub disabled_on: Option<Date>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub last_chosen: Option<Date>,
    #[serde(default)]
    pub times_completed: u32,
    pub completed: bool,
}

impl TaskState {
    pub fn reset(&mut self) {
        self.completed = false;
    }

    pub fn complete(&mut self) {
        self.completed = true;
        self.times_completed += 1;
    }

    pub fn enable(&mut self) {
        self.disabled_on = None;
    }

    pub fn disable(&mut self) {
        //TODO account for cutoff
        self.disabled_on = Some(today());
    }

    pub fn choose(&mut self, state: &State) {
        self.reset();
        self.last_chosen = Some(state.todays_date());
    }
}
