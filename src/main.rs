use clap::CommandFactory;
use clap_complete::CompleteEnv;
use randd_tasks::{Cli, Result, State};
//use color_backtrace::BacktracePrinter;

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    CompleteEnv::with_factory(Cli::command).complete();
    pretty_env_logger::init();
    color_backtrace::install();
    //color_eyre::install()?;
    let cli = Cli::default();
    let state = State::load()?;
    cli.execute(state)?;
    //if let Err(error) = cli.execute(state) {
    //eprintln!("{:#?}", error.backtrace());
    //}
    Ok(())
}
