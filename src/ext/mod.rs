mod node_ext;
mod node_search;

use thiserror::Error;
use tokio_i3ipc::I3;

pub use node_ext::NodeExt;
pub use node_search::{Move, NodeSearch, NotFound, Step};

#[derive(Error, Debug)]
#[error("Command: {cmd} got {err}")]
pub struct CommandError {
    cmd: String,
    err: std::io::Error,
}

impl CommandError {
    fn new<T: ToString>(cmd: T, err: std::io::Error) -> Self {
        Self {
            cmd: cmd.to_string(),
            err,
        }
    }
}

pub async fn i3_command(commands: Vec<String>, conn: &mut I3) -> Result<(), CommandError> {
    for cmd in commands{
        log::debug!("Sending i3 command: {}", cmd);
        conn.run_command(cmd.as_str())
            .await
            .map(|_| ())
            .map_err(|e| CommandError::new(cmd, e))?
    }
    Ok(())
}
