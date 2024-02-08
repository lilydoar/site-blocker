use std::path::PathBuf;

use clap::{crate_description, crate_name, crate_version, Parser};
use command::{handle_command, write_response, Command};

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

fn main() {
    let hosts = PathBuf::from("/etc/hosts"); // TODO: Load from config file, env var, or CLI arg
    let response = match handle_command(Cli::parse().command, hosts) {
        Ok(response) => response,
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

    write_response(response);
}
