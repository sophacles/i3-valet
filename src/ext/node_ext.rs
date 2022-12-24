use tokio_i3ipc::reply::{Floating, Node};

/// An extenstion trait for i3ipc-rs Nodes with some convenience functions
pub trait NodeExt {
    /// if the window is a floating window
    fn is_floating(&self) -> bool;

    /// if the node has children
    fn has_children(&self) -> bool;
}

impl NodeExt for Node {
    fn is_floating(&self) -> bool {
        if let Some(float_val) = self.floating {
            match float_val {
                Floating::AutoOff | Floating::UserOff => false,
                Floating::AutoOn | Floating::UserOn => true,
            }
        } else {
            false
        }
    }

    fn has_children(&self) -> bool {
        !self.nodes.is_empty()
    }
}
