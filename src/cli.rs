use std::io::{BufRead, Write};

use clap::{
    command, crate_description, crate_name, crate_version, ArgAction, Args, Parser, Subcommand,
};
use log::trace;

use crate::hosts::HostsFile;

const DEFAULT_EDITOR: &str = "vi";
const EDITOR_PROMPT: &str =
    "# Add sites to block. Separate by newline\n# Lines starting with # are ignored\n";

const VALID_SITE_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-.";

#[derive(Parser)]
#[command(name = crate_name!(), about = crate_description!(), version = crate_version!())]
pub struct Cli {
    #[command(subcommand)]
    command: Command,

    #[arg(
        long,
        env = "SITE_BLOCKER_HOSTS_FILE",
        value_name = "FILE",
        default_value = "/etc/hosts",
        help = "Set a custom hosts file path"
    )]
    pub hosts_file: String,

    #[arg(long, action = ArgAction::SetTrue, help = "Disable colored output")]
    pub no_color: bool,

    #[arg(short, long, action = ArgAction::SetTrue, help = "Disable log output")]
    pub quiet: bool,

    #[arg(short, long, action = ArgAction::Count, help = "Set the log level. Repeat for more logs")]
    pub verbose: u8,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "Get blocked sites", alias = "ls")]
    Get,

    #[command(about = "Add blocked sites")]
    Add(SiteOptions),

    #[command(about = "Remove blocked sites", alias = "rm")]
    Delete(SiteOptions),

    #[command(about = "Edit blocked sites through $EDITOR")]
    Edit,
}

#[derive(Args, Debug)]
struct SiteOptions {
    sites: Vec<String>,

    #[arg(short, long, value_name = "FILE")]
    files: Vec<String>,
}

enum Action {
    Noop,
    Get,
    Set(Vec<String>),
    Add(Vec<String>),
    Delete(Vec<String>),
}

#[derive(thiserror::Error, Debug)]
enum SiteError {
    #[error("Empty site")]
    Empty,
    #[error("Site is longer than max length of 255: {0}")]
    TooLong(String),
    #[error("Site contains invalid characters: {0}")]
    InvalidChars(String),
}

impl Cli {
    pub fn handle_command(&self) -> anyhow::Result<()> {
        trace!("handling command: {:?}", self.command);
        let mut hosts = HostsFile::new(self.hosts_file.clone().into())?;
        let action = self
            .command
            .construct_action(hosts.blocked_sites())?
            .validate()?;
        match action {
            Action::Noop => {}
            Action::Get => {
                for site in hosts.blocked_sites() {
                    println!("{}", site);
                }
            }
            Action::Set(sites) => {
                hosts.set(sites);
                hosts.write()?;
            }
            Action::Add(sites) => {
                hosts.add(sites);
                hosts.write()?;
            }
            Action::Delete(sites) => {
                hosts.delete(sites);
                hosts.write()?;
            }
        }
        Ok(())
    }
}

impl Command {
    fn construct_action(&self, blocked_sites: Vec<String>) -> std::io::Result<Action> {
        Ok(match self {
            Command::Get => Action::Get,
            Command::Add(options) => {
                let sites = options.collect_sites()?;
                let sites = match sites.is_empty() {
                    true => read_stdin()?,
                    false => sites,
                };
                match sites.is_empty() {
                    true => Action::Noop,
                    false => Action::Add(sites),
                }
            }
            Command::Delete(options) => {
                let sites = options.collect_sites()?;
                let sites = match sites.is_empty() {
                    true => read_stdin()?,
                    false => sites,
                };
                match sites.is_empty() {
                    true => Action::Noop,
                    false => Action::Delete(sites),
                }
            }
            Command::Edit => {
                let sites = read_editor(blocked_sites)?;
                match sites.is_empty() {
                    true => Action::Noop,
                    false => Action::Set(sites),
                }
            }
        })
    }
}

impl SiteOptions {
    fn collect_sites(&self) -> std::io::Result<Vec<String>> {
        let mut sites = self.sites.clone();

        for file in &self.files {
            sites.extend(std::fs::read_to_string(file)?.lines().map(str::to_string));
        }

        Ok(sites)
    }
}

impl Action {
    fn validate(self) -> Result<Self, SiteError> {
        Ok(match self {
            Action::Noop => Action::Noop,
            Action::Get => Action::Get,
            Action::Set(sites) => Action::Set(validate_sites(sites)?),
            Action::Add(sites) => Action::Add(validate_sites(sites)?),
            Action::Delete(sites) => Action::Delete(validate_sites(sites)?),
        })
    }
}

fn read_stdin() -> std::io::Result<Vec<String>> {
    let mut sites = Vec::new();
    for line in std::io::stdin().lock().lines() {
        sites.extend(line?.split_whitespace().map(str::to_string));
    }
    Ok(sites)
}

fn read_editor(blocked_sites: Vec<String>) -> std::io::Result<Vec<String>> {
    let editor = std::env::var("EDITOR").unwrap_or(DEFAULT_EDITOR.to_string());

    let mut file = tempfile::NamedTempFile::new()?;
    write!(file, "{}", blocked_sites.join("\n"))?;
    write!(file, "\n\n{}", EDITOR_PROMPT)?;

    let path = file.into_temp_path();
    trace!("editing file: {:?} in {}", path, editor);
    let status = std::process::Command::new(editor).arg(&path).status()?;
    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Editor failed",
        ));
    }

    Ok(std::fs::read_to_string(&path)?
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

fn validate_sites(sites: Vec<String>) -> Result<Vec<String>, SiteError> {
    for site in &sites {
        if site.is_empty() {
            return Err(SiteError::Empty);
        }

        if site.len() > 255 {
            return Err(SiteError::TooLong(site.to_string()));
        }

        if !site.chars().all(|c| VALID_SITE_CHARS.contains(c)) {
            return Err(SiteError::InvalidChars(site.to_string()));
        }
    }

    Ok(sites)
}
