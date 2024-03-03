use std::{fs::File, io::Write, path::PathBuf};

use log::{debug, info, trace};

const LOCALHOST_IPV4: &str = "127.0.0.1";
const LOCALHOST_IPV6: &str = "::1";

pub struct HostsFile {
    path: PathBuf,
    lines: Vec<HostsLine>,
}

#[derive(Debug, PartialEq)]
enum HostsLine {
    Empty,
    Comment(String),
    BlockedSite(String),
    Other(String),
}

impl HostsFile {
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        trace!("creating hosts file interactor for: {}", path.display());
        let lines = std::fs::read_to_string(&path)?
            .lines()
            .map(HostsLine::from)
            .collect();
        Ok(Self { path, lines })
    }

    pub fn blocked_sites(&self) -> Vec<String> {
        self.lines
            .iter()
            .filter_map(|line| match line {
                HostsLine::BlockedSite(site) => Some(site.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn set(&mut self, sites: Vec<String>) {
        let blocked_sites = self.blocked_sites();

        self.delete(
            blocked_sites
                .iter()
                .filter(|site| !sites.contains(site))
                .cloned()
                .collect(),
        );

        self.add(
            sites
                .into_iter()
                .filter(|site| !blocked_sites.contains(site))
                .collect(),
        );
    }

    pub fn add(&mut self, sites: Vec<String>) {
        let blocked_sites = self.blocked_sites();

        sites
            .iter()
            .filter(|site| blocked_sites.contains(site))
            .for_each(|site| {
                debug!("{} is already blocked", site);
            });

        self.lines.extend(
            sites
                .into_iter()
                .filter(|site| !blocked_sites.contains(site))
                .map(|site| {
                    info!("{} added", site);
                    HostsLine::BlockedSite(site)
                }),
        );
    }

    pub fn delete(&mut self, sites: Vec<String>) {
        sites
            .iter()
            .filter(|site| !self.blocked_sites().contains(site))
            .for_each(|site| {
                debug!("{} is not blocked", site);
            });

        self.lines.retain(|line| match line {
            HostsLine::BlockedSite(site) => match sites.contains(site) {
                true => {
                    info!("{} deleted", site);
                    false
                }
                false => true,
            },
            _ => true,
        });
    }

    pub fn write(&self) -> std::io::Result<()> {
        debug!("writing hosts file: {}", self.path.display());
        let mut file = match File::create(&self.path) {
            Ok(file) => file,
            Err(err) => match err.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("{}. Try using 'sudo'", err),
                    ));
                }
                _ => return Err(err),
            },
        };

        trace!("writing lines: {:?}", self.lines);
        file.write_all(
            self.lines
                .iter()
                .map(String::from)
                .collect::<Vec<String>>()
                .join("\n")
                .as_bytes(),
        )
    }
}

impl From<&str> for HostsLine {
    fn from(line: &str) -> Self {
        match line {
            line if line.is_empty() => HostsLine::Empty,
            line if line.starts_with('#') => HostsLine::Comment(line.to_string()),
            line if line.starts_with(LOCALHOST_IPV4) || line.starts_with(LOCALHOST_IPV6) => {
                let mut parts = line.split_whitespace().peekable();
                match (parts.next(), parts.peek()) {
                    // Do not edit the localhost entry
                    (Some(_), Some(&"localhost")) => HostsLine::Other(line.to_string()),
                    (Some(_), Some(_)) => HostsLine::BlockedSite(parts.collect()),
                    _ => HostsLine::Other(line.to_string()),
                }
            }
            line => HostsLine::Other(line.to_string()),
        }
    }
}

impl From<&HostsLine> for String {
    fn from(line: &HostsLine) -> String {
        match line {
            HostsLine::Empty => String::new(),
            HostsLine::Comment(text) => text.to_string(),
            HostsLine::BlockedSite(site) => format!("{}\t{}", LOCALHOST_IPV4, site),
            HostsLine::Other(text) => text.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hosts_line_from_string() {
        assert_eq!(HostsLine::from(""), HostsLine::Empty);
        assert_eq!(
            HostsLine::from("# This is a comment"),
            HostsLine::Comment("# This is a comment".to_string())
        );
        assert_eq!(
            HostsLine::from("127.0.0.1 example.com"),
            HostsLine::BlockedSite("example.com".to_string())
        );
        assert_eq!(
            HostsLine::from("::1 example.com"),
            HostsLine::BlockedSite("example.com".to_string())
        );
        assert_eq!(
            HostsLine::from("127.0.0.1 localhost"),
            HostsLine::Other("127.0.0.1 localhost".to_string())
        );
        assert_eq!(
            HostsLine::from("::1 localhost"),
            HostsLine::Other("::1 localhost".to_string())
        );
        assert_eq!(
            HostsLine::from("This is not a valid line"),
            HostsLine::Other("This is not a valid line".to_string())
        );
    }

    #[test]
    fn test_string_from_hosts_line() {
        assert_eq!(String::from(&HostsLine::Empty), "".to_string());
        assert_eq!(
            String::from(&HostsLine::Comment("# This is a comment".to_string())),
            "# This is a comment"
        );
        assert_eq!(
            String::from(&HostsLine::BlockedSite("example.com".to_string())),
            "127.0.0.1\texample.com"
        );
        assert_eq!(
            String::from(&HostsLine::Other("This is not a valid line".to_string())),
            "This is not a valid line"
        );
    }

    #[test]
    fn test_add_site() {
        let mut hosts = HostsFile {
            path: PathBuf::new(),
            lines: Vec::new(),
        };
        hosts.add(vec!["example.com".to_string()]);
        assert_eq!(hosts.lines.len(), 1);
        assert_eq!(
            hosts.lines[0],
            HostsLine::BlockedSite("example.com".to_string())
        );
    }

    #[test]
    fn test_remove_site() {
        let mut hosts = HostsFile {
            path: PathBuf::new(),
            lines: vec![HostsLine::BlockedSite("example.com".to_string())],
        };
        hosts.delete(vec!["example.com".to_string()]);
        assert_eq!(hosts.lines.len(), 0);
    }

    #[test]
    fn test_read_and_write() -> Result<(), std::io::Error> {
        let path = tempfile::NamedTempFile::new()?
            .into_temp_path()
            .to_path_buf();

        let hosts = HostsFile {
            path: path.clone(),
            lines: vec![
                HostsLine::Comment("# This is a comment".to_string()),
                HostsLine::Empty,
                HostsLine::BlockedSite("example.com".to_string()),
                HostsLine::Other("This is not a valid line".to_string()),
            ],
        };
        hosts.write()?;

        let loaded_hosts = HostsFile::new(path)?;
        assert_eq!(hosts.lines, loaded_hosts.lines);

        Ok(())
    }
}
