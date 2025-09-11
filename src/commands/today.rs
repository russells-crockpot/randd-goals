use super::{ExecutableCommand, completion, tasks::CompleteTaskCommand};
use crate::{
    Error, Result, State,
    picker::pick_todays_tasks,
    task::{TaskInfo, TaskSet, TaskStatus},
};
use clap::Parser;
use clap_complete::{ArgValueCompleter, PathCompleter};
use notify_rust::Notification;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

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

fn get_and_print_task_list_items<S: AsRef<TaskSet>>(state: &State, tasks: S) -> Result<()> {
    let task_items: BTreeMap<String, TaskListItem> = tasks
        .as_ref()
        .resolve(state)?
        .into_iter()
        .map(|t| t.info(state))
        .map(|i| (i.slug.clone(), i.into()))
        .collect();
    let stdout = io::stdout();
    serde_yml::to_writer(stdout, &task_items)?;
    Ok(())
}

#[derive(Debug, Parser)]
#[command(rename_all = "kebab")]
pub enum TodayCommands {
    /// Get today's tasks.
    Get(GetTodaysTasksCommand),
    /// Replace some of today's tasks with new ones.
    Refresh(RefreshTodaysTasksCommand),
    /// Replace all of today's tasks.
    Reset(ResetTodaysTasksCommand),
    /// Mark task(s) as complete.
    Complete(CompleteTaskCommand),
}

impl ExecutableCommand for TodayCommands {
    fn execute(self, state: State) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(state),
            Self::Refresh(cmd) => cmd.execute(state),
            Self::Reset(cmd) => cmd.execute(state),
            Self::Complete(cmd) => cmd.execute(state),
        }
    }
}

#[derive(Debug, Parser)]
pub struct GetTodaysTasksCommand {
    #[arg(short, long)]
    /// Whether or not to send a desktop notification.
    pub notify: bool,
}

impl ExecutableCommand for GetTodaysTasksCommand {
    fn execute(self, mut state: State) -> Result<()> {
        if pick_todays_tasks(&mut state)? {
            state.save()?;
        }
        if self.notify {
            let mut task_strings: Vec<_> = state
                .todays_tasks()
                .resolve(&state)?
                .into_iter()
                .map(|t| format!(" - {}", t.task()))
                .collect();
            notify_rust::Notification::new()
                .summary("Today's random tasks")
                .body(&task_strings.join("\n"))
                .appname(env!("CARGO_PKG_NAME"))
                .show()?;
        }
        get_and_print_task_list_items(&state, state.todays_tasks())
    }
}

#[derive(Debug, Parser)]
pub struct RefreshTodaysTasksCommand {
    #[arg(short, long)]
    /// Whether or not all completed tasks should also be refreshed
    pub completed: bool,
    #[arg(add = ArgValueCompleter::new(completion::todays_tasks))]
    pub tasks: Vec<String>,
}

impl ExecutableCommand for RefreshTodaysTasksCommand {
    fn execute(self, mut state: State) -> Result<()> {
        let mut tasks: BTreeSet<_> = self.tasks.into_iter().collect();
        if self.completed {
            let task_objs = state.todays_tasks().resolve(&state)?;
            for task in task_objs {
                if task.completed() {
                    tasks.insert(String::from(task.slug()));
                }
            }
        }
        for task in tasks {
            if !state.todays_tasks_mut().remove(&task) {
                return Err(Error::task_not_found(task));
            }
        }
        let old_tasks = state.todays_tasks().clone();
        if pick_todays_tasks(&mut state)? {
            state.save()?;
        }
        let new_tasks = state.todays_tasks() - &old_tasks;
        get_and_print_task_list_items(&state, &new_tasks)
    }
}

#[derive(Debug, Parser)]
pub struct ResetTodaysTasksCommand {}

impl ExecutableCommand for ResetTodaysTasksCommand {
    fn execute(self, mut state: State) -> Result<()> {
        state.todays_tasks_mut().clear();
        if pick_todays_tasks(&mut state)? {
            state.save()?;
        }
        get_and_print_task_list_items(&state, state.todays_tasks())
    }
}
