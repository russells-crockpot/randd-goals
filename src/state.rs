use crate::{
    Error, Result, STATE_FILE_PATH,
    config::{Config, DisabledOptions, GoalConfig},
    util::{now, today},
};
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, OpenOptions},
};
use time::{Date, Duration, OffsetDateTime};

/// Model of the way data is serialized in the state file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StateModel {
    pub last_generated: OffsetDateTime,
    pub goals: HashMap<String, RefCell<GoalState>>,
    pub todays_goals: Vec<String>,
}

impl Default for StateModel {
    fn default() -> Self {
        StateModel {
            last_generated: now() - Duration::DAY,
            goals: HashMap::new(),
            todays_goals: Vec::new(),
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
pub struct GoalState {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub disabled_at: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub last_chosen: Option<Date>,
    pub completed: bool,
}

impl GoalState {
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
    goals: HashMap<String, Goal>,
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
        let mut goals = HashMap::new();
        for (slug, goal_state) in model.goals.iter() {
            if let Some(goal_cfg) = config.get_goal(slug) {
                let goal = Goal {
                    config: goal_cfg,
                    state: RefCell::clone(goal_state),
                };
                goals.insert(slug.clone(), goal);
            } else {
                orphans.push(String::from(slug));
            }
        }
        //TODO report on orphans
        Ok(Self {
            config,
            model,
            goals,
        })
    }

    pub fn save(&self) -> Result<()> {
        self.model.save()?;
        self.config.save()?;
        Ok(())
    }

    pub fn enable_goal<S: AsRef<str>>(&self, slug: S) -> Result<()> {
        if let Some(goal) = self.goals.get(slug.as_ref()) {
            goal.enable();
            Ok(())
        } else {
            Err(Error::goal_state_not_loaded(slug))
        }
    }
}

#[derive(Debug)]
pub struct Goal {
    config: RefCell<GoalConfig>,
    state: RefCell<GoalState>,
}

impl Goal {
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
}
