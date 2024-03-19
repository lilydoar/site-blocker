use clap::Parser;
use cli::Cli;

mod cli;
mod hosts;

const DEFAULT_LOG_LEVEL: usize = 2;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Err(err) = stderrlog::new()
        .module(module_path!())
        .verbosity(DEFAULT_LOG_LEVEL + cli.verbose as usize)
        .quiet(cli.quiet)
        .color(match cli.no_color {
            true => stderrlog::ColorChoice::Never,
            false => stderrlog::ColorChoice::Auto,
        })
        .init()
    {
        return Err(err.into());
    }

    cli.handle_command()?;

    Ok(())
}






