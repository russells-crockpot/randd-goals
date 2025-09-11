use crate::{Result, State};
use clap::Parser;

pub mod config;
pub mod tasks;
use tasks::TaskCommands;
pub mod today;
pub use today::TodayCommands;
mod completion;

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
    Tasks(TaskCommands),
    #[command(subcommand)]
    Today(TodayCommands),
}

pub trait ExecutableCommand {
    fn execute(self, state: State) -> Result<()>;
}

impl ExecutableCommand for Commands {
    fn execute(self, state: State) -> Result<()> {
        match self {
            Self::Tasks(cmd) => cmd.execute(state),
            Self::Today(cmd) => cmd.execute(state),
        }
    }
}
