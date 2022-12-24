mod node_ext;
mod node_search;

use tokio_i3ipc::I3;

pub use node_ext::NodeExt;
pub use node_search::{Move, NodeSearch, Step};

pub async fn i3_command(cmd: &str, conn: &mut I3) -> Result<(), String> {
    log::debug!("Sending i3 command: {}", cmd);
    conn.run_command(cmd)
        .await
        .map(|_| ())
        .map_err(|e| format!("Command: {} got: {:?}", cmd, e))
}
