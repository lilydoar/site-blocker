use std::{path::PathBuf, process::ExitCode};

use clap::{crate_description, crate_name, crate_version, ArgAction, Parser};
use command::{handle_command, write_response, Command};
use log::error;

mod command;
mod hosts;

const DEFAULT_LOG_LEVEL: usize = 2;

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Err(err) = stderrlog::new()
        .module(module_path!())
        .verbosity(DEFAULT_LOG_LEVEL + cli.verbose as usize)
        .quiet(cli.quiet)
        .init()
    {
        eprintln!("Error: failed to initialize logging: {}", err);
        return ExitCode::FAILURE;
    }

    let hosts = PathBuf::from(cli.hosts_file);
    let response = match handle_command(cli.command, &hosts) {
        Ok(response) => response,
        Err(err) => {
            error!("{}", err);
            return ExitCode::FAILURE;
        }
    };
    write_response(response);
    ExitCode::SUCCESS
}

#[derive(Parser)]
#[command(name = crate_name!())]
#[command(about = crate_description!(), long_about = None, version = crate_version!())]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(long, env, default_value = "/etc/hosts")]
    #[arg(help = "Set a custom hosts file path")]
    hosts_file: PathBuf,
    #[arg(short, long, action = ArgAction::SetTrue)]
    #[arg(help = "Suppress log output")]
    quiet: bool,
    #[arg(short, long, action = ArgAction::Count)]
    #[arg(help = "Set the log level (repeat for more logs)")]
    verbose: u8,
}
