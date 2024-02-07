use std::path::PathBuf;

use clap::Parser;
use command::{handle_command, write_response, Command};

mod command;
mod hosts;

#[derive(Parser)]
#[command(name = "blocker")]
#[command(about = "Block sites through the hosts file", long_about = None)]
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
