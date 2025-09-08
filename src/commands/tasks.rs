use super::ExecutableCommand;
use crate::{
    Error, Result, State,
    config::TaskBuilder,
    task::{Task, TaskInfo},
};
use clap::Parser;
use cli_table::{Cell, Table};
use std::{collections::BTreeMap, io};

#[derive(Debug, Parser)]
#[command(rename_all = "kebab")]
pub enum TaskCommands {
    Add(AddTaskCommand),
    Upsert(UpsertTaskCommand),
    Update(UpdateTaskCommand),
    #[command(name = "rm")]
    Remove(RemoveTaskCommand),
    List,
    Details(TaskDetailsCommand),
    Enable(EnableTaskCommand),
    Disable(DisableTaskCommand),
    Complete(CompleteTaskCommand),
}

fn list_tasks(state: State) -> Result<()> {
    let table = state
        .task_names()
        .into_iter()
        .flat_map(|s| state.get_task(&s).ok_or_else(|| Error::task_not_found(&s)))
        .map(|t| vec![t.slug().cell(), t.task().cell()])
        .collect::<Vec<_>>()
        .table();
    cli_table::print_stdout(table)?;
    Ok(())
}

impl ExecutableCommand for TaskCommands {
    fn execute(self, state: State) -> Result<()> {
        match self {
            Self::List => list_tasks(state),
            Self::Add(cmd) => cmd.execute(state),
            Self::Upsert(cmd) => cmd.execute(state),
            Self::Update(cmd) => cmd.execute(state),
            Self::Details(cmd) => cmd.execute(state),
            Self::Enable(cmd) => cmd.execute(state),
            Self::Disable(cmd) => cmd.execute(state),
            Self::Remove(cmd) => cmd.execute(state),
            Self::Complete(cmd) => cmd.execute(state),
        }
    }
}

macro_rules! impl_into_task_builder {
    (
        $type:ident {
            required: ($($required:ident),*),
            optional: ($($optional:ident),*),
            copy: ($($copy:ident),*),
        }
    ) => {
        impl From<$type> for TaskBuilder {
            fn from(value: $type) -> Self {
                let mut builder = TaskBuilder::default();
                $(
                    builder.$required(value.$required);
                )*
                $(
                    if let Some(attr) = value.$copy {
                        builder.$copy(attr);
                    }
                )*
                $(
                    if let Some(attr) = value.$optional {
                        builder.$optional(attr);
                    }
                )*
                builder
            }
        }
        impl From<&$type> for TaskBuilder {
            fn from(value: &$type) -> Self {
                let mut builder = TaskBuilder::default();
                $(
                    builder.$required(value.$required.clone());
                )*
                $(
                    if let Some(attr) = value.$copy {
                        builder.$copy(attr);
                    }
                )*
                $(
                    if let Some(ref attr) = value.$optional {
                        builder.$optional(attr.clone());
                    }
                )*
                builder
            }
        }
    }
}

#[derive(Debug, Parser)]
pub struct AddTaskCommand {
    #[arg(short, long)]
    pub slug: Option<String>,
    #[arg(short, long)]
    pub weight: Option<f64>,
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(short, long)]
    pub description: Option<String>,
    #[arg()]
    pub task: String,
}

impl_into_task_builder! {
    AddTaskCommand {
        required: (task, tags, description),
        optional: (slug),
        copy: (weight),
    }
}

impl ExecutableCommand for AddTaskCommand {
    fn execute(self, mut state: State) -> Result<()> {
        let task = TaskBuilder::from(self).build()?;
        state.add_task(task)?;
        state.save()
    }
}

#[derive(Debug, Parser)]
pub struct UpsertTaskCommand {
    #[arg(short, long)]
    pub weight: Option<f64>,
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(short, long)]
    pub description: Option<String>,
    #[arg(short, long)]
    pub task: Option<String>,
    #[arg()]
    pub slug: String,
}

impl_into_task_builder! {
    UpsertTaskCommand {
        required: (slug, tags, description),
        optional: (task),
        copy: (weight),
    }
}

impl ExecutableCommand for UpsertTaskCommand {
    fn execute(self, mut state: State) -> Result<()> {
        let task = TaskBuilder::from(self).build()?;
        state.upsert_task(task);
        state.save()
    }
}

#[derive(Debug, Parser)]
pub struct UpdateTaskCommand {
    #[arg(short, long)]
    pub weight: Option<f64>,
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(short, long)]
    pub description: Option<String>,
    #[arg(short, long)]
    pub task: Option<String>,
    #[arg()]
    pub slug: String,
}

impl_into_task_builder! {
    UpdateTaskCommand {
        required: (slug, tags, description),
        optional: (task),
        copy: (weight),
    }
}

impl ExecutableCommand for UpdateTaskCommand {
    fn execute(self, mut state: State) -> Result<()> {
        let task = TaskBuilder::from(self).build()?;
        state.update_task(task)?;
        state.save()
    }
}

#[derive(Debug, Parser)]
pub struct EnableTaskCommand {
    #[arg()]
    pub tasks: Vec<String>,
}

impl ExecutableCommand for EnableTaskCommand {
    fn execute(self, mut state: State) -> Result<()> {
        state.enable_tasks(self.tasks)?;
        state.save()
    }
}

#[derive(Debug, Parser)]
//TODO add options
pub struct DisableTaskCommand {
    #[arg()]
    pub tasks: Vec<String>,
}

impl ExecutableCommand for DisableTaskCommand {
    fn execute(self, mut state: State) -> Result<()> {
        state.disable_tasks(self.tasks)?;
        state.save()
    }
}

#[derive(Debug, Parser)]
pub struct TaskDetailsCommand {
    #[arg()]
    pub tasks: Vec<String>,
}

impl ExecutableCommand for TaskDetailsCommand {
    fn execute(self, mut state: State) -> Result<()> {
        let tasks = if self.tasks.is_empty() {
            state.task_names()
        } else {
            self.tasks
        };
        let infos: BTreeMap<_, _> = tasks
            .into_iter()
            .map(|s| state.get_task(&s).ok_or_else(|| Error::task_not_found(&s)))
            //TODO handle missing
            .flat_map(|r| r.map(|t| t.info(&state)))
            .map(|i| (i.slug.clone(), i))
            .collect();
        let mut stdout = io::stdout();
        serde_yml::to_writer(stdout, &infos)?;
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct RemoveTaskCommand {
    #[arg()]
    pub tasks: Vec<String>,
}

impl ExecutableCommand for RemoveTaskCommand {
    fn execute(self, mut state: State) -> Result<()> {
        state.remove_tasks(self.tasks)?;
        state.save()
    }
}

#[derive(Debug, Parser)]
pub struct CompleteTaskCommand {
    #[arg()]
    pub tasks: Vec<String>,
}

impl ExecutableCommand for CompleteTaskCommand {
    fn execute(self, mut state: State) -> Result<()> {
        self.tasks.into_iter().try_for_each(|slug| {
            state
                .get_task(&slug)
                .ok_or_else(|| Error::task_not_found(&slug))
                .map(|task| task.complete())
        })?;
        state.save()
    }
}
