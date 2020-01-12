use super::node_ext::NodeExt;
use i3ipc::reply::{Node, NodeType};

pub trait NodeSearch {
    fn postorder(&self) -> PostOrder;
    fn preorder(&self) -> PreOrder;
    fn search_focus_path<P: Fn(&Node) -> bool>(&self, p: P) -> Option<&Node>;

    fn get_current_workspace(&self) -> Option<&Node> {
        self.search_focus_path(|n| n.nodetype == NodeType::Workspace)
    }

    fn get_current_output(&self) -> Option<&Node> {
        self.search_focus_path(|n| n.nodetype == NodeType::Output)
    }

    fn get_content_area(&self) -> Option<&Node> {
        self.search_focus_path(|n| n.name.as_ref().map_or(false, |v| v == "content"))
    }

    fn get_current_window(&self) -> Option<&Node> {
        self.search_focus_path(|n| n.focused)
    }
}

impl NodeSearch for Node {
    fn search_focus_path<P: Fn(&Node) -> bool>(&self, p: P) -> Option<&Node> {
        let mut node = self;
        loop {
            if p(node) {
                break Some(node);
            }
            node = node
                .nodes
                .iter()
                .chain(node.floating_nodes.iter())
                .find(|f| f.id == node.focus[0])?;
        }
    }

    fn postorder(&self) -> PostOrder {
        PostOrder::new(self)
    }

    fn preorder(&self) -> PreOrder {
        PreOrder::new(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Step<'a> {
    pub d: usize,
    pub n: &'a Node,
}

pub struct PostOrder<'a> {
    stack: Vec<(usize, Step<'a>)>,
}

impl<'a> PostOrder<'a> {
    fn new(n: &'a Node) -> Self {
        let mut stack = Vec::with_capacity(16);
        stack.push((0, Step { d: 0, n }));
        PostOrder { stack }
    }
}

impl<'a> Iterator for PostOrder<'a> {
    type Item = Step<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (i, s) = self.stack.pop()?;
        let mut n = match s.n.nodes.get(i) {
            Some(n) => n,
            None => return Some(s),
        };

        let mut d = s.d + 1;
        // push ourself on first
        self.stack.push((i + 1, s));
        // push the ith child, since thats the branch to go down
        while n.has_children() {
            // one here, not 0 since we're taking the 0 path on the ride down
            self.stack.push((1, Step { d, n }));
            n = &n.nodes[0];
            d += 1
        }
        // n is now the childless bottom, so we return it.
        Some(Step { d, n })
    }
}

pub struct PreOrder<'a> {
    stack: Vec<Step<'a>>,
}

impl<'a> PreOrder<'a> {
    fn new(n: &'a Node) -> Self {
        // thats super nested... but a nice power of 2
        let mut stack = Vec::with_capacity(16);
        stack.push(Step { d: 0, n });
        PreOrder { stack }
    }
}

impl<'a> Iterator for PreOrder<'a> {
    type Item = Step<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.stack.pop()?;
        let d = res.d + 1;
        for n in res.n.nodes.iter().rev() {
            self.stack.push(Step { d, n });
        }
        Some(res)
    }
}
