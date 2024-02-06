use clap::Subcommand;

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
