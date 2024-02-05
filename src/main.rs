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

        self.sync_with_hosts()?;

        Ok(CommandResponse::Add(AddResponse::Added(site.to_string())))
    }

    pub fn remove(&self, site: &str) -> Result<CommandResponse, std::io::Error> {
        if !self.blocked_sites()?.contains(&site.to_string()) {
            return Ok(CommandResponse::Remove(RemoveResponse::NotFound(
                site.to_string(),
            )));
        }

        let file = File::open(&self.blocked_sites)?;
        let mut new_lines = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line?;
            if line == site {
                continue;
            }
            new_lines.push(line);
        }

        let mut file = File::create(&self.blocked_sites)?;
        for line in new_lines {
            writeln!(file, "{}", line)?;
        }

        self.sync_with_hosts()?;

        Ok(CommandResponse::Remove(RemoveResponse::Removed(
            site.to_string(),
        )))
    }

    fn blocked_sites(&self) -> Result<Vec<String>, std::io::Error> {
        let file = File::open(&self.blocked_sites)?;
        BufReader::new(file).lines().collect()
    }

    fn sync_with_hosts(&self) -> Result<(), std::io::Error> {
        Ok(HostsInteractor {
            hosts: self.hosts.clone(),
        }
        .sync_with_blocker(self)?)
    }
}

struct HostsInteractor {
    hosts: PathBuf,
}

impl HostsInteractor {
    fn sync_with_blocker(&self, blocker: &Blocker) -> Result<(), std::io::Error> {
        match self.get_blocker_region()? {
            Some(_) => (),
            None => self.create_blocker_region()?,
        };

        let blocked_sites = blocker.blocked_sites()?;

        let file = File::open(&self.hosts)?;

        let mut new_lines = Vec::new();
        let mut in_region = false;
        for line in BufReader::new(file).lines() {
            let line = line?;

            if line.contains("# END BLOCKER") {
                in_region = false;
                for site in &blocked_sites {
                    new_lines.push(format!("127.0.0.1 {}", site));
                }
            }

            if in_region {
                continue;
            }

            if line.contains("# BEGIN BLOCKER") {
                in_region = true;
            }

            new_lines.push(line);
        }

        let mut file = File::create(&self.hosts)?;
        for line in new_lines {
            writeln!(file, "{}", line)?;
        }

        Ok(())
    }

    fn create_blocker_region(&self) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new().append(true).open(&self.hosts)?;
        writeln!(file, "\n# BEGIN BLOCKER\n# END BLOCKER\n")?;
        Ok(())
    }

    fn get_blocker_region(&self) -> Result<Option<Vec<String>>, std::io::Error> {
        let mut in_region = false;
        let mut lines_in_region = Vec::new();

        let file = File::open(&self.hosts)?;

        for line in BufReader::new(file).lines() {
            let line = line?;

            if line.contains("# BEGIN BLOCKER") {
                in_region = true;
                continue;
            }

            if line.contains("# END BLOCKER") {
                break;
            }

            if in_region {
                lines_in_region.push(line);
            }
        }

        if lines_in_region.is_empty() {
            return Ok(None);
        }

        Ok(Some(lines_in_region))
    }
}
