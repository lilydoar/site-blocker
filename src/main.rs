use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use clap::{Parser, Subcommand};

const CLI_NAME: &str = "blocker";

fn main() {
    let cli = Cli::parse();

    let blocker = match Blocker::new(default_hosts(), default_blocked_sites()) {
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
        Ok(blocker) => blocker,
    };

    let resp = match cli.command {
        Command::List => blocker.list(),
        Command::Add { site } => blocker.add(&site),
        Command::Remove { site } => blocker.remove(&site),
    };

    match resp {
        Err(e) => eprintln!("Error: {}", e),
        Ok(resp) => match resp {
            CommandResponse::List(sites) => {
                for site in sites {
                    println!("{}", site);
                }
            }
            CommandResponse::Add(resp) => match resp {
                AddResponse::AlreadyExists(site) => {
                    println!("{} is already in the block list", site)
                }
                AddResponse::Added(site) => println!("{} added to the block list", site),
            },
            CommandResponse::Remove(resp) => match resp {
                RemoveResponse::NotFound(site) => println!("{} is not in the block list", site),
                RemoveResponse::Removed(site) => println!("{} removed from the block list", site),
            },
        },
    }
}

#[derive(Parser)]
#[command(name = CLI_NAME)]
#[command(about = "Block sites through the hosts file", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "List all sites blocked sites")]
    List,
    #[command(about = "Add a site to the block list")]
    Add { site: String },
    #[command(about = "Remove a site from the block list", name = "rm")]
    Remove { site: String },
}

enum CommandResponse {
    List(Vec<String>),
    Add(AddResponse),
    Remove(RemoveResponse),
}

enum AddResponse {
    AlreadyExists(String),
    Added(String),
}

enum RemoveResponse {
    NotFound(String),
    Removed(String),
}

struct Blocker {
    hosts: PathBuf,
    blocked_sites: PathBuf,
}

fn default_hosts() -> PathBuf {
    PathBuf::from("/etc/hosts")
}

fn default_blocked_sites() -> PathBuf {
    directories::ProjectDirs::from("", "", CLI_NAME)
        .expect("Could not find the home directory")
        .data_dir()
        .join("sites.txt")
}

fn validate_paths(hosts: &PathBuf, blocked_sites: &PathBuf) -> Result<(), std::io::Error> {
    if !hosts.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Hosts file not found at {}", hosts.display()),
        ));
    }

    if let Some(parent_dir) = blocked_sites.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }

    if !blocked_sites.exists() {
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(blocked_sites)?;
    }

    Ok(())
}

impl Blocker {
    pub fn new(hosts: PathBuf, blocked_sites: PathBuf) -> Result<Self, std::io::Error> {
        validate_paths(&hosts, &blocked_sites)?;

        Ok(Blocker {
            hosts,
            blocked_sites,
        })
    }

    pub fn list(&self) -> Result<CommandResponse, std::io::Error> {
        let sites = self.blocked_sites()?;
        Ok(CommandResponse::List(sites))
    }

    pub fn add(&self, site: &str) -> Result<CommandResponse, std::io::Error> {
        if self.blocked_sites()?.contains(&site.to_string()) {
            return Ok(CommandResponse::Add(AddResponse::AlreadyExists(
                site.to_string(),
            )));
        }

        let mut file = OpenOptions::new().append(true).open(&self.blocked_sites)?;
        writeln!(file, "{}\n", site)?;

        Ok(CommandResponse::Add(AddResponse::Added(site.to_string())))
    }

    pub fn remove(&self, site: &str) -> Result<CommandResponse, std::io::Error> {
        if !self.blocked_sites()?.contains(&site.to_string()) {
            return Ok(CommandResponse::Remove(RemoveResponse::NotFound(
                site.to_string(),
            )));
        }

        // Remove the site from the blocked sites file

        Ok(CommandResponse::Remove(RemoveResponse::Removed(
            site.to_string(),
        )))
    }

    fn blocked_sites(&self) -> Result<Vec<String>, std::io::Error> {
        let file = File::open(&self.blocked_sites)?;
        BufReader::new(file).lines().collect()
    }

    fn sync_with_hosts(&self) -> Result<(), std::io::Error> {
        // Update the hosts file to ensure that it matches the blocked sites file

        Ok(())
    }
}
