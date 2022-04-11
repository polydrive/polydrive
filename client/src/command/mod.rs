pub mod handler;
pub mod pipe;

#[derive(Debug)]
pub enum Command {
    ListFiles,
    Unknown,
}

impl From<Command> for &str {
    fn from(command: Command) -> Self {
        match command {
            Command::ListFiles => "list_files",
            Command::Unknown => "unknown",
        }
    }
}

impl From<&str> for Command {
    fn from(command: &str) -> Self {
        match command {
            "list_files" => Self::ListFiles,
            _ => Self::Unknown,
        }
    }
}
