use std::path::PathBuf;

use clap::Subcommand;
use log::{debug, info};

use crate::hosts::HostsInteractor;

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(visible_alias = "ls")]
    #[command(about = "List blocked sites")]
    List,
    #[command(about = "Add a blocked site")]
    Add {
        #[arg(short, long, required = true)]
        site: Vec<String>,
    },
    #[command(visible_alias = "rm")]
    #[command(about = "Remove a blocked site")]
    Remove {
        #[arg(short, long, required = true)]
        site: Vec<String>,
    },
}

#[derive(Debug)]
pub enum CommandResponse {
    List(Vec<String>),
    Add(Vec<AddResponse>),
    Remove(Vec<RemoveResponse>),
}

#[derive(Debug)]
pub enum AddResponse {
    AlreadyExists(String),
    Added(String),
}

#[derive(Debug)]
pub enum RemoveResponse {
    NotFound(String),
    Removed(String),
}

pub fn handle_command(
    command: Command,
    hosts: &PathBuf,
) -> Result<CommandResponse, std::io::Error> {
    let mut interactor = HostsInteractor::new(hosts)?;

    debug!("handling command: {:?}", command);
    match command {
        Command::List => Ok(CommandResponse::List(interactor.blocked_sites())),
        Command::Add { site } => {
            let responses = site
                .into_iter()
                .filter(|s| validate_site(s))
                .map(|s| match interactor.blocked_sites().contains(&s) {
                    true => AddResponse::AlreadyExists(s),
                    false => {
                        interactor.add_site(&s);
                        AddResponse::Added(s)
                    }
                })
                .collect();
            interactor.write(hosts)?;
            Ok(CommandResponse::Add(responses))
        }
        Command::Remove { site } => {
            let responses = site
                .into_iter()
                .map(|s| match interactor.blocked_sites().contains(&s) {
                    false => RemoveResponse::NotFound(s),
                    true => {
                        interactor.remove_site(&s);
                        RemoveResponse::Removed(s)
                    }
                })
                .collect();
            interactor.write(hosts)?;
            Ok(CommandResponse::Remove(responses))
        }
    }
}

pub fn write_response(response: CommandResponse) {
    debug!("writing response: {:?}", response);
    match response {
        CommandResponse::List(sites) => {
            for site in sites {
                println!("{}", site);
            }
        }
        CommandResponse::Add(responses) => {
            for resp in responses {
                match resp {
                    AddResponse::AlreadyExists(site) => info!("{} is already blocked", site),
                    AddResponse::Added(site) => info!("{} added", site),
                }
            }
        }
        CommandResponse::Remove(responses) => {
            for resp in responses {
                match resp {
                    RemoveResponse::NotFound(site) => info!("{} is not blocked", site),
                    RemoveResponse::Removed(site) => info!("{} removed", site),
                }
            }
        }
    }
}

fn validate_site(site: &str) -> bool {
    if site.is_empty() {
        info!("site cannot be empty");
        return false;
    }

    if site.chars().any(|c| c.is_whitespace()) {
        info!("site cannot contain whitespace");
        return false;
    }

    if !site.chars().all(|c| c.is_ascii()) {
        info!("site must be ascii");
        return false;
    }

    true
}
