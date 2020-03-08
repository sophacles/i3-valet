use std::cmp::{Ord, Ordering};

use super::node_ext::NodeExt;
use i3ipc::reply::{Node, NodeType};

/// An extension trait to i3rpc-rs Node that adds searching functionality
///
/// NodeSearch adds some methodes to Node that provide various forms of tree traversal and some
/// pre-implemented common searches.
pub trait NodeSearch {
    /// A postorder traversal of the Node and its children as an iterator with
    /// Item = Step
    fn postorder(&self) -> PostOrder;

    /// A preorder traversal of the Node and its children as an iterator with
    /// Item = Step
    fn preorder(&self) -> PreOrder;

    /// Search through the focus path returning the first Node that matches the predicate function.
    fn search_focus_path<P: Fn(&Node) -> bool>(&self, p: P) -> Option<&Node>;

    /// Returns the Node of the currently focused workspace
    fn get_current_workspace(&self) -> Option<&Node> {
        self.search_focus_path(|n| n.nodetype == NodeType::Workspace)
    }

    /// Returns the Node of the currently focused output
    fn get_current_output(&self) -> Option<&Node> {
        self.search_focus_path(|n| n.nodetype == NodeType::Output)
    }

    /// Returns the "content area" of the currently focused output
    /// (the content area has the rectangle that windows will be in basically: output - bars)
    fn get_content_area(&self) -> Option<&Node> {
        self.search_focus_path(|n| n.name.as_ref().map_or(false, |v| v == "content"))
    }

    /// Returns the node of the currently focused window
    fn get_current_window(&self) -> Option<&Node> {
        self.search_focus_path(|n| n.focused)
    }

    /// Find a node with the provided mark
    fn find_mark(&self, mark: &str) -> Option<&Node> {
        for s in self.preorder() {
            if s.n.marks.contains(&mark.to_string()) {
                return Some(s.n);
            }
        }
        None
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

/// A struct describing how the direction of movement to get to the
/// current Node in a Step. TODO: describe better
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Move {
    Up,
    Down,
    Sibling,
}

impl Move {
    fn new(cur: &usize, last: &usize) -> Move {
        match cur.cmp(&last) {
            Ordering::Greater => Move::Down,
            Ordering::Equal => Move::Sibling,
            Ordering::Less => Move::Up,
        }
    }
}

/// The output of traversal iterators. It contains the node, as well as info about it's place in
/// the tree. This info may be useful for window rearrangement etc
#[derive(Debug, Clone, Copy)]
pub struct Step<'a> {
    /// Depth of the node relative to the search root
    pub d: usize,
    /// The direction traveled from previous node to get here
    pub m: Move,
    /// The current node
    pub n: &'a Node,
}

/// A post-order iterator over the tree of Nodes
pub struct PostOrder<'a> {
    // The tuple is (
    //       idx of child to traverse on pop,
    //       depth of node,
    //       node
    // )
    stack: Vec<(usize, usize, &'a Node)>,
}

impl<'a> PostOrder<'a> {
    fn new(n: &'a Node) -> Self {
        let mut stack = Vec::with_capacity(16);
        stack.push((0, 0, n));
        PostOrder { stack }
    }
}

impl<'a> Iterator for PostOrder<'a> {
    type Item = Step<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (get_child_idx, mut depth, container) = self.stack.pop()?;

        // if no more children to traverse, we're the step!
        let mut n = match container.nodes.get(get_child_idx) {
            Some(n) => n,
            None => {
                return Some(Step {
                    d: depth,
                    m: Move::Up,
                    n: container,
                })
            }
        };

        // push ourself back on first, and update the idx of child
        // to look at on next iter
        self.stack.push((get_child_idx + 1, depth, container));

        depth += 1;

        // push the ith child, since thats the branch to go down
        while n.has_children() {
            // 1 here, not 0 since we're taking the 0 path on the ride down
            self.stack.push((1, depth, n));
            n = &n.nodes[0];
            depth += 1;
        }

        // n is now the childless bottom, so we return it.
        let top = &self.stack[self.stack.len() - 1];

        // The current node was reached from a parent (top).
        // We got to the current node by the previous value of top.0
        // top.0 is the index of the child to follow the next iteration.
        // So we added 1 when it got pushed.
        // so to get here we were top.0 - 1. If that number is 0, we're first
        // child and cannot be a sibling.
        // get_child (top.0) is only 0 for root explicitly, after that it's a least 1
        let m = match top.1 == depth || top.0 > 1 {
            true => Move::Sibling,
            false => Move::Down,
        };

        Some(Step { d: depth, m, n })
    }
}

/// A pre-order iterator over the tree of Nodes
pub struct PreOrder<'a> {
    // Tuple is: ( depth of node, node )
    stack: Vec<(usize, &'a Node)>,
    // Track last depth to tell the step what directino we moved
    last_d: usize,
}

impl<'a> PreOrder<'a> {
    fn new(n: &'a Node) -> Self {
        // thats super nested... but a nice power of 2
        let mut stack = Vec::with_capacity(16);
        stack.push((0, n));
        PreOrder { last_d: 0, stack }
    }
}

impl<'a> Iterator for PreOrder<'a> {
    type Item = Step<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (d, container) = self.stack.pop()?;
        let new_d = d + 1;

        // Push in reverse order so we get the last
        // child last. Stacks are tricky like that
        for n in container.nodes.iter().rev() {
            self.stack.push((new_d, n));
        }

        let m = Move::new(&d, &self.last_d);
        self.last_d = d;
        Some(Step { d, m, n: container })
    }
}
