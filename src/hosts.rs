use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

pub struct HostsInteractor {
    pub blocked_sites: Vec<String>,
}

impl HostsInteractor {
    pub fn new(hosts: PathBuf) -> Result<Self, std::io::Error> {
        let blocked_sites = get_blocked_sites_from_hosts_file(&hosts)?;
        Ok(Self { blocked_sites })
    }

    pub fn blocked_sites(&self) -> &Vec<String> {
        &self.blocked_sites
    }

    pub fn add_site(&self, _site: &str) -> Result<(), std::io::Error> {
        todo!()
    }

    pub fn remove_site(&self, _site: &str) -> Result<(), std::io::Error> {
        todo!()
    }
}

fn get_blocked_sites_from_hosts_file(hosts: &PathBuf) -> Result<Vec<String>, std::io::Error> {
    Ok(hosts_file_lines(hosts)?
        .into_iter()
        .filter_map(|line| HostsLine::from(line).blocked_site())
        .collect())
}

fn hosts_file_lines(hosts: &PathBuf) -> Result<Vec<String>, std::io::Error> {
    BufReader::new(match File::open(hosts) {
        Ok(file) => file,
        Err(err) => {
            return Err(std::io::Error::new(
                err.kind(),
                format!("Opening hosts file {}: {}", hosts.display(), err),
            ));
        }
    })
    .lines()
    .collect()
}

enum HostsLine {
    Empty,
    Comment(String),
    Entry(String, String),
    Other(String),
}

impl From<String> for HostsLine {
    fn from(line: String) -> Self {
        match line.trim() {
            "" => HostsLine::Empty,
            line if line.starts_with('#') => HostsLine::Comment(line.to_string()),
            line => {
                let mut parts = line.split_whitespace();
                let (ip, domain) = match (parts.next(), parts.next()) {
                    (Some(ip), Some(domain)) => (ip, domain),
                    _ => return HostsLine::Other(line.to_string()),
                };
                if parts.next().is_some() {
                    return HostsLine::Other(line.to_string());
                }
                HostsLine::Entry(ip.to_string(), domain.to_string())
            }
        }
    }
}

impl HostsLine {
    fn blocked_site(&self) -> Option<String> {
        if let HostsLine::Entry(ip, domain) = self {
            if domain == "localhost" {
                return None;
            }

            return match ip.as_str() {
                "127.0.0.1" | "::1" => Some(domain.clone()),
                _ => None,
            };
        }

        None
    }
}
