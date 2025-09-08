use super::ExecutableCommand;
use crate::{Result, State, config::TaskBuilder};
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
pub enum TaskCommands {
    Add(AddTaskCommand),
    Upsert(UpsertTaskCommand),
    List,
    Details(TaskDetailsCommand),
    Enable(EnableTaskCommand),
    Disable(DisableTaskCommand),
}

impl ExecutableCommand for TaskCommands {
    fn execute(self, state: State) -> Result<()> {
        match self {
            Self::List => todo!(),
            Self::Add(cmd) => cmd.execute(state),
            Self::Upsert(cmd) => cmd.execute(state),
            Self::Details(cmd) => cmd.execute(state),
            Self::Enable(cmd) => cmd.execute(state),
            Self::Disable(cmd) => cmd.execute(state),
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
    pub details: Option<String>,
    #[arg()]
    pub task: String,
}

impl_into_task_builder! {
    AddTaskCommand {
        required: (task, tags, details),
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
    pub details: Option<String>,
    #[arg(short, long)]
    pub task: Option<String>,
    #[arg()]
    pub slug: String,
}

impl_into_task_builder! {
    UpsertTaskCommand {
        required: (slug, tags, details),
        optional: (task),
        copy: (weight),
    }
}

impl ExecutableCommand for UpsertTaskCommand {
    fn execute(self, mut state: State) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Parser)]
pub struct EnableTaskCommand {
    #[arg()]
    pub tasks: String,
}

impl ExecutableCommand for EnableTaskCommand {
    fn execute(self, mut state: State) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Parser)]
//TODO add options
pub struct DisableTaskCommand {
    #[arg()]
    pub tasks: String,
}

impl ExecutableCommand for DisableTaskCommand {
    fn execute(self, mut state: State) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Parser)]
pub struct TaskDetailsCommand {
    #[arg()]
    pub tasks: Vec<String>,
}

impl ExecutableCommand for TaskDetailsCommand {
    fn execute(self, mut state: State) -> Result<()> {
        todo!()
    }
}
