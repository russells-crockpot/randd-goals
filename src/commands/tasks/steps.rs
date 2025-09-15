use crate::{
    Error, Result, State,
    commands::{ExecutableCommand, completion},
    error::RanddGoalsError,
    task::{TaskBuilder, TaskConfig},
};
use clap::{Args, Subcommand};
use clap_complete::{ArgValueCompleter, PathCompleter};

#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab")]
pub enum StepCommands {
    /// Adds step(s).
    #[command(aliases=["a", "n", "new"])]
    Add(AddStepCommand),
    /// Removes step(s).
    #[command(aliases = ["rm", "delete"])]
    Remove(RemoveStepCommand),
    /// Update a step.
    Update(UpdateStepCommand),
    /// Change the order of step(s) (TODO).
    #[command(aliases = ["m", "mv"])]
    Move(MoveStepCommand),
    /// Defers step(s) (TODO).
    Defer(DeferStepCommand),
    /// Imports step(s) (TODO).
    Import(ImportStepCommand),
}

impl ExecutableCommand for StepCommands {
    fn execute(self, state: State) -> Result<()> {
        match self {
            Self::Add(cmd) => cmd.execute(state),
            Self::Remove(cmd) => cmd.execute(state),
            Self::Update(cmd) => cmd.execute(state),
            Self::Move(cmd) => cmd.execute(state),
            Self::Defer(cmd) => cmd.execute(state),
            Self::Import(cmd) => cmd.execute(state),
        }
    }
}

#[derive(Debug, Args)]
#[command(rename_all = "kebab")]
pub struct AddStepCommand {
    #[arg(short, long)]
    pub description: Option<String>,
    #[arg(short, long)]
    pub place: Option<u32>,
    #[arg(add = ArgValueCompleter::new(completion::all_tasks))]
    pub task: String,
    #[arg()]
    pub steps: Vec<String>,
}

impl ExecutableCommand for AddStepCommand {
    fn execute(self, state: State) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Args)]
#[command(rename_all = "kebab")]
pub struct RemoveStepCommand {
    #[arg(add = ArgValueCompleter::new(completion::all_tasks))]
    pub task: String,
    #[arg()]
    pub steps: Vec<u32>,
}

impl ExecutableCommand for RemoveStepCommand {
    fn execute(self, state: State) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Args)]
#[command(rename_all = "kebab")]
pub struct UpdateStepCommand {
    #[arg(short, long)]
    pub title: Option<String>,
    #[arg(short, long)]
    pub description: Option<String>,
    #[arg(add = ArgValueCompleter::new(completion::all_tasks))]
    pub task: String,
    #[arg()]
    pub step: u32,
}

impl ExecutableCommand for UpdateStepCommand {
    fn execute(self, state: State) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Args)]
#[command(rename_all = "kebab")]
pub struct MoveStepCommand {
    #[arg(add = ArgValueCompleter::new(completion::all_tasks))]
    pub task: String,
    #[arg()]
    pub step: u32,
    #[arg()]
    pub to: u32,
}

impl ExecutableCommand for MoveStepCommand {
    fn execute(self, state: State) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Args)]
#[command(rename_all = "kebab")]
pub struct DeferStepCommand {}

impl ExecutableCommand for DeferStepCommand {
    fn execute(self, state: State) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Args)]
#[command(rename_all = "kebab")]
pub struct ImportStepCommand {}

impl ExecutableCommand for ImportStepCommand {
    fn execute(self, state: State) -> Result<()> {
        todo!()
    }
}
