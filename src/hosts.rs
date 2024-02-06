use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

const START_MARKER: &str = "# BLOCKER_START";
const END_MARKER: &str = "# BLOCKER_END";

pub struct HostsInteractor {
    hosts: PathBuf,
    pub blocked_sites: Vec<String>,
}

impl HostsInteractor {
    pub fn new(hosts: PathBuf) -> Result<Self, std::io::Error> {
        match hosts.exists() {
            false => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Hosts file not found",
            )),
            true => {
                let blocked_sites = Self::read_blocked_sites(&hosts)?;
                Ok(Self {
                    hosts,
                    blocked_sites,
                })
            }
        }
    }

    pub fn blocked_sites(&self) -> &Vec<String> {
        &self.blocked_sites
    }

    pub fn add_site(&self, site: &str) -> Result<(), std::io::Error> {
        todo!()
    }

    pub fn remove_site(&self, site: &str) -> Result<(), std::io::Error> {
        todo!()
    }

    fn read_blocked_sites(hosts: &PathBuf) -> Result<Vec<String>, std::io::Error> {
        // Return all sites between the START and END markers
        todo!()
    }

    fn hosts_lines(&self) -> Result<Vec<HostsLine>, std::io::Error> {
        // Retrieve the contents of the hosts file
        let file = File::open(&self.hosts)?;
        BufReader::new(file).lines().collect() // Need to handle the Result portion of this
    }
}

enum HostsLine {
    Comment(String),
    Site(String),
    Empty,
}

impl From<String> for HostsLine {
    fn from(line: String) -> Self {
        match line.trim() {
            "" => HostsLine::Empty,
            line if line.starts_with('#') => HostsLine::Comment(line.to_string()),
            line => HostsLine::Site(line.to_string()),
        }
    }
}

// Retrieve the contents of the hosts file and
// filter out the lines that are not between the START and END markers
//
// let mut in_blocker_region = false;
// let mut sites = Vec::new();
// for line in self.hosts_lines()? {
//     let line = line.trim();
//     if line == START_MARKER {
//         in_blocker_region = true;
//         continue;
//     }
//
//     if line == END_MARKER {
//         break;
//     }
//
//     if in_blocker_region {
//         let site = line.split_whitespace().last().unwrap();
//         sites.push(site.to_string());
//     }
//     // unfinished
// }
//
//
// Ok(sites)
