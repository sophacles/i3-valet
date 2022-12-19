mod node_ext;
mod node_search;

use i3ipc::I3Connection;

pub use node_ext::NodeExt;
pub use node_search::{Move, NodeSearch, Step};

pub fn i3_command(cmd: &str, conn: &mut I3Connection) -> Result<(), String> {
    log::debug!("Sending i3 command: {}", cmd);
    conn.run_command(cmd)
        .map_err(|e| format!("Command: {} got: {:?}", cmd, e))?;
    Ok(())
}
