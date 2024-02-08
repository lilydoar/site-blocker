use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

pub struct HostsInteractor {
    lines: Vec<HostsLine>,
}

impl HostsInteractor {
    pub fn new(hosts: &PathBuf) -> Result<Self, std::io::Error> {
        let lines = hosts_file_lines(&hosts)?
            .into_iter()
            .map(HostsLine::from)
            .collect();

        Ok(Self { lines })
    }

    pub fn blocked_sites(&self) -> Vec<String> {
        self.lines
            .iter()
            .filter_map(|line| line.directs_to_localhost())
            .collect()
    }

    pub fn add_site(mut self, site: &str) -> Self {
        if !self.blocked_sites().contains(&site.to_string()) {
            self.lines
                .push(HostsLine::Entry("127.0.0.1".to_string(), site.to_string()));
        }

        self
    }

    pub fn remove_site(mut self, site: &str) -> Self {
        let index = self
            .lines
            .iter()
            .position(|line| line.directs_to_localhost() == Some(site.to_string()));

        if let Some(index) = index {
            let _ = self.lines.remove(index);
        }

        self
    }

    pub fn write(&self, hosts: &PathBuf) -> Result<(), std::io::Error> {
        let mut file = match File::create(hosts) {
            Ok(file) => file,
            Err(err) => match err.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("{} Help: Try using 'sudo'", err),
                    ));
                }
                _ => return Err(err),
            },
        };
        file.write_all(
            self.lines
                .iter()
                .map(|line| format!("{}\n", String::from(line)))
                .collect::<Vec<_>>()
                .join("")
                .as_bytes(),
        )
    }
}

fn hosts_file_lines(hosts: &PathBuf) -> Result<Vec<String>, std::io::Error> {
    let file = match File::open(hosts) {
        Ok(file) => file,
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Opening hosts file {}: {}", hosts.display(), err),
                ));
            }
            _ => return Err(err),
        },
    };
    BufReader::new(file).lines().collect()
}

enum HostsLine {
    Empty,
    Comment(String),
    Entry(String, String),
    Invalid(String),
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
                    _ => return HostsLine::Invalid(line.to_string()),
                };
                if parts.next().is_some() {
                    return HostsLine::Invalid(line.to_string());
                }
                HostsLine::Entry(ip.to_string(), domain.to_string())
            }
        }
    }
}

impl From<&HostsLine> for String {
    fn from(line: &HostsLine) -> Self {
        match line {
            HostsLine::Empty => String::new(),
            HostsLine::Comment(text) => text.to_string(),
            HostsLine::Entry(ip, domain) => format!("{}\t{}", ip, domain),
            HostsLine::Invalid(text) => text.clone(),
        }
    }
}

impl HostsLine {
    fn directs_to_localhost(&self) -> Option<String> {
        match self {
            HostsLine::Entry(_, domain) if domain == "localhost" => None,
            HostsLine::Entry(ip, domain) if ip == "127.0.0.1" || ip == "::1" => {
                Some(domain.clone())
            }
            _ => None,
        }
    }
}
