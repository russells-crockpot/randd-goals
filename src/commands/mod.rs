use crate::{Result, State};
use clap::{Args, Parser, Subcommand};

pub mod config;
pub mod tasks;
use tasks::TaskCommands;

#[derive(Debug, Parser)]
#[command(version, author)]
#[command(rename_all = "kebab")]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn execute(self, state: State) -> Result<()> {
        self.command.execute(state)
    }
}

impl Default for Cli {
    fn default() -> Self {
        Self::parse()
    }
}

#[derive(Debug, Parser)]
pub enum Commands {
    #[command(subcommand)]
    Tasks(tasks::TaskCommands),
}

pub trait ExecutableCommand {
    fn execute(self, state: State) -> Result<()>;
}

impl ExecutableCommand for Commands {
    fn execute(self, state: State) -> Result<()> {
        match self {
            Self::Tasks(cmd) => cmd.execute(state),
        }
    }
}
