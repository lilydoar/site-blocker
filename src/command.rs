use std::path::PathBuf;

use clap::Subcommand;

use crate::hosts::HostsInteractor;

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "List blocked sites")]
    List,
    #[command(about = "Add a blocked site")]
    Add { site: String },
    #[command(name = "rm")]
    #[command(about = "Remove a blocked site")]
    Remove { site: String },
}

pub enum CommandResponse {
    List(Vec<String>),
    Add(AddResponse),
    Remove(RemoveResponse),
}

pub enum AddResponse {
    AlreadyExists(String),
    Added(String),
}

pub enum RemoveResponse {
    NotFound(String),
    Removed(String),
}

pub fn handle_command(command: Command, hosts: PathBuf) -> Result<CommandResponse, std::io::Error> {
    let interactor = HostsInteractor::new(hosts)?;

    match command {
        Command::List => Ok(CommandResponse::List(interactor.blocked_sites())),
        Command::Add { site } => match interactor.blocked_sites().contains(&site) {
            true => Ok(CommandResponse::Add(AddResponse::AlreadyExists(site))),
            false => {
                interactor.add_site(&site).write()?;
                Ok(CommandResponse::Add(AddResponse::Added(site)))
            }
        },
        Command::Remove { site } => match interactor.blocked_sites().contains(&site) {
            false => Ok(CommandResponse::Remove(RemoveResponse::NotFound(site))),
            true => {
                interactor.remove_site(&site).write()?;
                Ok(CommandResponse::Remove(RemoveResponse::Removed(site)))
            }
        },
    }
}

pub fn write_response(response: CommandResponse) {
    match response {
        CommandResponse::List(sites) => {
            for site in sites {
                println!("{}", site);
            }
        }
        CommandResponse::Add(resp) => match resp {
            AddResponse::AlreadyExists(site) => println!("{} is already in the block list", site),
            AddResponse::Added(site) => println!("{} added to the block list", site),
        },
        CommandResponse::Remove(resp) => match resp {
            RemoveResponse::NotFound(site) => println!("{} is not in the block list", site),
            RemoveResponse::Removed(site) => println!("{} removed from the block list", site),
        },
    }
}