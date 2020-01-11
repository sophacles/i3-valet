extern crate env_logger;
extern crate i3ipc;

#[macro_use]
extern crate log;

use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::vec::IntoIter;

use i3ipc::reply;
use i3ipc::reply::{Floating, Node};

pub mod collapse;
pub mod floats;
pub mod info;

pub trait NodeSearch {
    fn is_floating(&self) -> bool;
    fn search_focus_path<P: Fn(&Node) -> bool>(&self, p: P) -> Option<&Node>;
    fn postorder(&self) -> Traversal;
    fn preorder(&self) -> Traversal;
    fn has_children(&self) -> bool;

    fn get_current_workspace(&self) -> Option<&Node> {
        self.search_focus_path(|n| n.nodetype == reply::NodeType::Workspace)
    }

    fn get_current_output(&self) -> Option<&Node> {
        self.search_focus_path(|n| n.nodetype == reply::NodeType::Output)
    }

    fn get_current_window(&self) -> Option<&Node> {
        self.search_focus_path(|n| n.focused && n.is_floating())
    }
}

impl NodeSearch for Node {
    fn is_floating(&self) -> bool {
        match self.floating {
            Floating::AutoOff | Floating::UserOff => false,
            Floating::AutoOn | Floating::UserOn => true,
        }
    }

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

    fn has_children(&self) -> bool {
        !self.nodes.is_empty()
    }

    fn postorder(&self) -> Traversal {
        Traversal::new(self, Order::PostOrder)
    }

    fn preorder(&self) -> Traversal {
        Traversal::new(self, Order::PreOrder)
    }
}

pub enum Order {
    PreOrder,
    PostOrder,
}

#[derive(Debug, Clone, Copy)]
pub struct Step<'a> {
    pub d: usize,
    pub n: &'a Node,
}

impl<'a> PartialEq for Step<'a> {
    fn eq(&self, other: &Step<'a>) -> bool {
        self.d == other.d
    }
}

impl<'a> PartialOrd for Step<'a> {
    fn partial_cmp(&self, other: &Step<'a>) -> Option<Ordering> {
        let res = self.d.partial_cmp(&other.d);
        res
    }
}

pub struct Traversal<'a> {
    pub order: Order,
    pub size: usize,
    walked: IntoIter<Step<'a>>,
}

impl<'a> Traversal<'a> {
    pub fn new(n: &'a Node, order: Order) -> Self {
        let it = match order {
            Order::PreOrder => preorder(n, 0),
            Order::PostOrder => postorder(n, 0),
        };
        let size = it.len();
        let walked = it.into_iter();
        Self {
            order,
            size,
            walked,
        }
    }
}

impl<'a> Iterator for Traversal<'a> {
    type Item = Step<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.walked.next()
    }
}

fn preorder<'a>(n: &'a Node, d: usize) -> Vec<Step<'a>> {
    let res = vec![Step { d, n }];
    if n.has_children() {
        n.nodes.iter().fold(res, |mut res, c| {
            res.extend(preorder(c, d + 1));
            res
        })
    } else {
        res
    }
}

fn postorder<'a>(n: &'a Node, d: usize) -> Vec<Step<'a>> {
    if n.has_children() {
        let mut res = n.nodes.iter().fold(vec![], |mut acc, c| {
            acc.extend(postorder(c, d + 1));
            acc
        });
        res.push(Step { d, n });
        res
    } else {
        vec![Step { d, n }]
    }
}
