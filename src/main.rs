use std::{path::PathBuf, process::ExitCode};

use clap::{crate_description, crate_name, crate_version, Parser};
use command::{handle_command, write_response, Command};
use log::error;

mod command;
mod hosts;

#[derive(Parser)]
#[command(name = crate_name!())]
#[command(about = crate_description!(), long_about = None)]
#[command(version = crate_version!())]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

fn main() -> ExitCode {
    if let Err(err) = stderrlog::new()
        .module(module_path!())
        .verbosity(log::Level::Info)
        .quiet(false)
        .init()
    {
        eprintln!("Error: failed to initialize logging: {}", err);
        return ExitCode::FAILURE;
    }

    let hosts = PathBuf::from("/etc/hosts"); // TODO: Load from config file, env var, or CLI arg
    let response = match handle_command(Cli::parse().command, &hosts) {
        Ok(response) => response,
        Err(err) => {
            error!("{}", err);
            return ExitCode::FAILURE;
        }
    };
    write_response(response);
    ExitCode::SUCCESS
}
