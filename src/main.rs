use randd_tasks::{Cli, Result, State};
//use color_backtrace::BacktracePrinter;

fn main() -> Result<()> {
    dotenv::dotenv().ok();
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
