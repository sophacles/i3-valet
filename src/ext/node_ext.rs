use i3ipc::reply::{Floating, Node};

pub trait NodeExt {
    fn is_floating(&self) -> bool;
    fn has_children(&self) -> bool;
}

impl NodeExt for Node {
    fn is_floating(&self) -> bool {
        match self.floating {
            Floating::AutoOff | Floating::UserOff => false,
            Floating::AutoOn | Floating::UserOn => true,
        }
    }

    fn has_children(&self) -> bool {
        !self.nodes.is_empty()
    }
}
