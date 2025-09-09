use super::ExecutableCommand;
use crate::{
    Error, Result, State,
    picker::pick_todays_tasks,
    task::{Task, TaskInfo, TaskStatus},
};
use clap::Parser;
use serde::Serialize;
use std::{collections::BTreeMap, io};

#[derive(Debug, Serialize)]
struct TaskListItem {
    #[serde(skip)]
    slug: String,
    task: String,
    status: TaskStatus,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    description: Option<String>,
}

impl From<TaskInfo> for TaskListItem {
    fn from(info: TaskInfo) -> Self {
        Self {
            slug: info.slug,
            task: info.task,
            status: info.status,
            description: info.description,
        }
    }
}

#[derive(Debug, Parser)]
#[command(rename_all = "kebab")]
pub enum TodayCommands {
    Get(GetTodaysTasksCommand),
    Refresh(RefreshTodaysTasksCommand),
    Reset(ResetTodaysTasksCommand),
}

impl ExecutableCommand for TodayCommands {
    fn execute(self, mut state: State) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(state),
            Self::Refresh(cmd) => cmd.execute(state),
            Self::Reset(cmd) => cmd.execute(state),
        }
    }
}

#[derive(Debug, Parser)]
pub struct GetTodaysTasksCommand {
    #[arg()]
    pub notify: bool,
}

impl ExecutableCommand for GetTodaysTasksCommand {
    fn execute(self, mut state: State) -> Result<()> {
        pick_todays_tasks(&mut state)?;
        state.save()?;
        let task_items: BTreeMap<String, TaskListItem> = state
            .todays_tasks()
            .resolve(&state)?
            .into_iter()
            .map(|t| t.info(&state))
            .map(|i| (i.slug.clone(), i.into()))
            .collect();
        let mut stdout = io::stdout();
        serde_yml::to_writer(stdout, &task_items)?;
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct RefreshTodaysTasksCommand {
    #[arg()]
    pub tasks: Vec<String>,
}

impl ExecutableCommand for RefreshTodaysTasksCommand {
    fn execute(self, mut state: State) -> Result<()> {
        todo!();
        //state.save()
    }
}

#[derive(Debug, Parser)]
pub struct ResetTodaysTasksCommand {
    #[arg()]
    pub tasks: Vec<String>,
}

impl ExecutableCommand for ResetTodaysTasksCommand {
    fn execute(self, mut state: State) -> Result<()> {
        todo!();
        //state.save()
    }
}
