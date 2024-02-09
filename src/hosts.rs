use std::{
    fmt::{Display, Formatter},
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use log::{debug, trace, warn};

#[derive(Debug)]
pub struct HostsInteractor {
    lines: Vec<HostsLine>,
}

impl HostsInteractor {
    pub fn new(hosts: &PathBuf) -> Result<Self, std::io::Error> {
        debug!("creating hosts interactor for: {}", hosts.display());
        let lines: Vec<HostsLine> = read_hosts_file_lines(&hosts)?
            .into_iter()
            .map(HostsLine::from)
            .collect();

        for (i, line) in lines
            .iter()
            .enumerate()
            .filter(|(_, line)| matches!(line, HostsLine::Invalid(_)))
        {
            warn!("{}:{} Invalid entry: {}", hosts.display(), i, line);
        }

        Ok(Self { lines })
    }

    pub fn blocked_sites(&self) -> Vec<String> {
        self.lines
            .iter()
            .filter_map(|line| line.directs_to_localhost())
            .collect()
    }

    pub fn add_site(mut self, site: &str) -> Self {
        debug!("adding site {} at line {}", site, self.lines.len());
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
            debug!("removing site {} at line {}", site, index);
            let _ = self.lines.remove(index);
        }

        self
    }

    pub fn write(&self, hosts: &PathBuf) -> Result<(), std::io::Error> {
        debug!("writing hosts file: {}", hosts.display());
        let mut file = match File::create(hosts) {
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
        trace!("writing lines:\n{:?}", self.lines);
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

fn read_hosts_file_lines(hosts: &PathBuf) -> Result<Vec<String>, std::io::Error> {
    debug!("reading hosts file: {}", hosts.display());
    let file = match File::open(hosts) {
        Ok(file) => file,
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("{}. Opening hosts file {}", hosts.display(), err),
                ));
            }
            _ => return Err(err),
        },
    };
    BufReader::new(file).lines().collect()
}

#[derive(Debug, PartialEq)]
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

impl Display for HostsLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
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

#[cfg(test)]
mod tests {
    use std::fs::create_dir_all;

    use tempdir::TempDir;

    use super::*;

    #[test]
    fn test_hosts_line_from_string() {
        assert_eq!(HostsLine::from("".to_string()), HostsLine::Empty);
        assert_eq!(
            HostsLine::from("# This is a comment".to_string()),
            HostsLine::Comment("# This is a comment".to_string())
        );
        assert_eq!(
            HostsLine::from("127.0.0.1 localhost".to_string()),
            HostsLine::Entry("127.0.0.1".to_string(), "localhost".to_string())
        );
        assert_eq!(
            HostsLine::from("::1 localhost".to_string()),
            HostsLine::Entry("::1".to_string(), "localhost".to_string())
        );
        assert_eq!(
            HostsLine::from("Not a valid line".to_string()),
            HostsLine::Invalid("Not a valid line".to_string())
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
            String::from(&HostsLine::Entry(
                "127.0.0.1".to_string(),
                "localhost".to_string()
            )),
            "127.0.0.1\tlocalhost"
        );
        assert_eq!(
            String::from(&HostsLine::Invalid("Not a valid line".to_string())),
            "Not a valid line"
        );
    }

    #[test]
    fn test_directs_to_localhost() {
        assert_eq!(
            HostsLine::Entry("127.0.0.1".to_string(), "localhost".to_string())
                .directs_to_localhost(),
            None
        );
        assert_eq!(
            HostsLine::Entry("127.0.0.1".to_string(), "example.com".to_string())
                .directs_to_localhost(),
            Some("example.com".to_string())
        );
        assert_eq!(
            HostsLine::Entry("::1".to_string(), "example.com".to_string()).directs_to_localhost(),
            Some("example.com".to_string())
        );
        assert_eq!(
            HostsLine::Comment("# This is a comment".to_string()).directs_to_localhost(),
            None
        );
    }

    #[test]
    fn test_add_site() {
        let interactor = HostsInteractor { lines: Vec::new() };
        let interactor = interactor.add_site("example.com");
        assert_eq!(interactor.lines.len(), 1);
        assert_eq!(
            interactor.lines[0],
            HostsLine::Entry("127.0.0.1".to_string(), "example.com".to_string())
        );
    }

    #[test]
    fn test_remove_site() {
        let interactor = HostsInteractor {
            lines: vec![HostsLine::Entry(
                "127.0.0.1".to_string(),
                "example.com".to_string(),
            )],
        };
        let interactor = interactor.remove_site("example.com");
        assert_eq!(interactor.lines.len(), 0);
    }

    #[test]
    fn test_read_and_write() -> Result<(), std::io::Error> {
        let hosts = create_mock_hosts_file()?;
        let interactor = HostsInteractor {
            lines: vec![
                HostsLine::Comment("# This is a comment".to_string()),
                HostsLine::Empty,
                HostsLine::Entry("127.0.0.1".to_string(), "localhost".to_string()),
                HostsLine::Entry("::1".to_string(), "localhost".to_string()),
                HostsLine::Invalid("Not a valid line".to_string()),
            ],
        };

        interactor.write(&hosts)?;
        let loaded_interactor = HostsInteractor::new(&hosts)?;
        assert_eq!(interactor.lines, loaded_interactor.lines);

        Ok(())
    }

    fn create_mock_hosts_file() -> Result<PathBuf, std::io::Error> {
        let hosts = TempDir::new("tests")
            .unwrap()
            .path()
            .join("mock_hosts")
            .to_path_buf();

        let parent = hosts.parent().unwrap();
        create_dir_all(&parent)?;

        let mut file = File::create(&hosts)?;
        let contents =
            "# This is a comment\n\n127.0.0.1\tlocalhost\n::1\tlocalhost\nNot a valid line\n";
        file.write_all(contents.as_bytes())?;

        Ok(hosts)
    }
}
