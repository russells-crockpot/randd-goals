use super::{ExecutableCommand, completion};
use crate::{
    Error, Result, State,
    error::RanddGoalsError,
    task::{TaskBuilder, TaskConfig},
};
use camino::Utf8PathBuf;
use clap::Parser;
use clap_complete::{ArgValueCompleter, PathCompleter};
use cli_table::{Cell, Table};
use std::{collections::BTreeMap, fs, io};

#[derive(Debug, Parser)]
#[command(rename_all = "kebab")]
pub enum TaskCommands {
    /// Add a new task.
    Add(AddTaskCommand),
    /// Add a new task or update it if the task already exists.
    Upsert(UpsertTaskCommand),
    /// Update an existing task.
    Update(UpdateTaskCommand),
    #[command(name = "rm")]
    /// Delete a task.
    Remove(RemoveTaskCommand),
    /// Print a simple list of all tasks.
    List,
    /// Get details about task(s).
    Details(TaskDetailsCommand),
    /// Enable task(s).
    Enable(EnableTaskCommand),
    /// Disable task(s).
    Disable(DisableTaskCommand),
    /// Mark task(s) as complete.
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
    /// The task's slug/id.
    pub slug: Option<String>,
    #[arg(short, long)]
    /// How likely the task is to be chosen.
    pub weight: Option<f64>,
    #[arg(long = "tag")]
    /// Any tags to associate with the task.
    pub tags: Vec<String>,
    #[arg(short, long)]
    /// A more detailed description of the task.
    pub description: Option<String>,
    #[arg(short = 'o', long)]
    /// How many times this task can be completed before it is disabled (unimplemented).
    pub max_occurrences: Option<u32>,
    #[arg(short = 'f', long)]
    /// Minimum number of days before the task can be chosen again (unimplemented).
    pub min_frequency: Option<u32>,
    #[arg()]
    /// The task's title/summary.
    pub task: String,
}

impl_into_task_builder! {
    AddTaskCommand {
        required: (task, tags, description, max_occurrences, min_frequency),
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
    /// How likely the task is to be chosen.
    pub weight: Option<f64>,
    #[arg(long = "tag")]
    /// Any tags to associate with the task.
    pub tags: Vec<String>,
    #[arg(short, long)]
    /// A more detailed description of the task.
    pub description: Option<String>,
    #[arg(short, long)]
    /// The task's title/summary.
    pub task: Option<String>,
    #[arg(short = 'o', long)]
    /// How many times this task can be completed before it is disabled (unimplemented).
    pub max_occurrences: Option<u32>,
    #[arg(short = 'f', long)]
    /// Minimum number of days before the task can be chosen again (unimplemented).
    pub min_frequency: Option<u32>,
    #[arg()]
    //TODO make not required
    /// The task's slug/id.
    pub slug: String,
}

impl_into_task_builder! {
    UpsertTaskCommand {
        required: (slug, tags, description, max_occurrences, min_frequency),
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
    /// How likely the task is to be chosen.
    pub weight: Option<f64>,
    #[arg(long = "tag")]
    /// Any tags to associate with the task.
    pub tags: Vec<String>,
    #[arg(short, long)]
    /// A more detailed description of the task.
    pub description: Option<String>,
    #[arg(short, long)]
    /// The task's title/summary.
    pub task: Option<String>,
    #[arg(short = 'o', long)]
    /// How many times this task can be completed before it is disabled (unimplemented).
    pub max_occurrences: Option<u32>,
    #[arg(short = 'f', long)]
    /// Minimum number of days before the task can be chosen again (unimplemented).
    pub min_frequency: Option<u32>,
    /// The task's slug/id.
    #[arg(add = ArgValueCompleter::new(completion::all_tasks))]
    pub slug: String,
}

impl_into_task_builder! {
    UpdateTaskCommand {
        required: (slug, tags, description, max_occurrences, min_frequency),
        optional: (task),
        copy: (weight),
    }
}

impl ExecutableCommand for UpdateTaskCommand {
    fn execute(self, state: State) -> Result<()> {
        let task = TaskBuilder::from(self).build()?;
        state.update_task(task)?;
        state.save()
    }
}

#[derive(Debug, Parser)]
pub struct EnableTaskCommand {
    #[arg(add = ArgValueCompleter::new(completion::disabled_tasks))]
    /// The task(s) to enable.
    pub tasks: Vec<String>,
}

impl ExecutableCommand for EnableTaskCommand {
    fn execute(self, state: State) -> Result<()> {
        state.enable_tasks(self.tasks)?;
        state.save()
    }
}

#[derive(Debug, Parser)]
//TODO add options
pub struct DisableTaskCommand {
    #[arg(add = ArgValueCompleter::new(completion::enabled_tasks))]
    /// The task(s) to disable.
    pub tasks: Vec<String>,
}

impl ExecutableCommand for DisableTaskCommand {
    fn execute(self, state: State) -> Result<()> {
        state.disable_tasks(self.tasks)?;
        state.save()
    }
}

#[derive(Debug, Parser)]
pub struct TaskDetailsCommand {
    #[arg(add = ArgValueCompleter::new(completion::all_tasks))]
    /// The task(s) to print the details for.
    pub tasks: Vec<String>,
}

impl ExecutableCommand for TaskDetailsCommand {
    fn execute(self, state: State) -> Result<()> {
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
        let stdout = io::stdout();
        serde_yml::to_writer(stdout, &infos)?;
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct RemoveTaskCommand {
    #[arg(add = ArgValueCompleter::new(completion::all_tasks))]
    /// The task(s) to remove.
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
    //TODO make mutually exclusive with positional args
    #[arg(short, long)]
    /// Mark all of today's tasks as complete.
    pub all: bool,
    #[arg(add = ArgValueCompleter::new(completion::uncompleted_tasks))]
    /// The task(s) to complete.
    pub tasks: Vec<String>,
}

impl ExecutableCommand for CompleteTaskCommand {
    fn execute(self, state: State) -> Result<()> {
        let tasks = if self.all {
            state.todays_tasks().into()
        } else {
            self.tasks
        };
        tasks.into_iter().try_for_each(|slug| {
            state
                .get_task(&slug)
                .ok_or_else(|| Error::task_not_found(&slug))
                .map(|task| task.complete())
        })?;
        state.save()
    }
}

#[derive(Debug, Parser)]
pub struct ImportTaskCommand {
    #[arg(short, long)]
    /// Update any tasks that already exist.
    pub update: bool,
    #[arg(add = ArgValueCompleter::new(PathCompleter::file()))]
    /// The csv or yaml file to import tasks from.
    pub file: Utf8PathBuf,
}

impl ExecutableCommand for ImportTaskCommand {
    fn execute(self, mut state: State) -> Result<()> {
        println!("Reading file: {}", self.file);
        let tasks: Vec<TaskConfig> = match self.file.extension() {
            Some(".yml") | Some(".yaml") => {
                let data = fs::read(&self.file)?;
                serde_yml::from_slice(&data)?
            }
            Some(".csv") | Some(".tsv") | Some(".psv") => {
                //TODO handle errors
                csv::Reader::from_path(&self.file)?
                    .into_deserialize()
                    .flatten()
                    .collect()
            }
            Some(ext) => return Err(Error::unsupported_file_type(ext)),
            None => return Err(Error::unsupported_file_type("No extension")),
        };
        println!("Importing {} task(s).", tasks.len());
        if self.update {
            state.upsert_tasks(tasks);
        } else {
            for task in tasks {
                if let Err(Error::RanddGoals {
                    source: RanddGoalsError::TaskAlreadyExists { slug },
                    ..
                }) = state.add_task(task)
                {
                    println!("Task {slug} already exists; skipping...");
                }
            }
        }
        Ok(())
    }
}
