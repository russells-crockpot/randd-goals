use super::ExecutableCommand;
use crate::{
    Error, Result, State, Task,
    task::{TaskInfo, TaskStatus},
};
use clap::Parser;
use serde::Serialize;

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
    Get,
    Refresh(RefreshTodaysTasksCommand),
    Reset(ResetTodaysTasksCommand),
}

impl ExecutableCommand for TodayCommands {
    fn execute(self, mut state: State) -> Result<()> {
        match self {
            Self::Get => todo!(),
            Self::Refresh(cmd) => cmd.execute(state),
            Self::Reset(cmd) => cmd.execute(state),
        }
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
