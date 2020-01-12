extern crate log;

use i3ipc::reply::Node;
use i3ipc::I3Connection;

use crate::{info, NodeSearch, Step};

pub struct Collapse<'a> {
    pub target: &'a Node,
    pub candidate: &'a Node,
}

#[derive(Debug)]
enum CollapseState<'a> {
    Candidate(Step<'a>),
    Collapsing(Step<'a>),
    Fresh,
}

impl<'a> CollapseState<'a> {
    fn pretty(&'a self) -> String {
        match self {
            CollapseState::Candidate(s) => format!("Candidate({})", s.n.id),
            CollapseState::Collapsing(s) => format!("Collapsing({})", s.n.id),
            CollapseState::Fresh => String::from("Fresh"),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Move {
    Up,
    Down,
    Sibling,
}

impl Move {
    fn new(cur: &Step, last: &Step) -> Move {
        if cur.d > last.d {
            Move::Down
        } else if cur.d == last.d {
            Move::Sibling
        } else if cur.d < last.d {
            Move::Up
        } else {
            panic!("WTF!");
        }
    }
}

// postorder traversal lets us look for changes in depth easily.
// Any window that has no siblings is a candidate to merge up.
// The target node is the first parent that has multiple children, where we wish to reparent the
// candidate.
// The depth limit of 2 is because the current implementation does a simple window move to cause i3
// to collapse the child up. Room for improvement here includes breadcrumbs or similar to have the
// necesarry motions understood and performed all at once.
fn find_candidate(root: &Node) -> Option<&Node> {
    let mut candidate = CollapseState::Fresh;
    let mut prev = Step { d: 0, n: root };

    for cur in root.postorder() {
        let mc = Move::new(&cur, &prev);
        debug!(
            "{}:{} - {:?} ** {}",
            cur.d,
            cur.n.id,
            mc,
            candidate.pretty()
        );
        match candidate {
            CollapseState::Collapsing(c) => {
                if c.d > 2 {
                    debug!("Pushing {}", c.n.id);
                    debug!("collapse: {} <== {}", c.n.id, prev.n.id);
                    return Some(c.n);
                }
            }
            CollapseState::Candidate(c) => {
                if mc == Move::Up {
                    candidate = CollapseState::Collapsing(c);
                }
            }
            _ => (),
        };

        if mc == Move::Down && cur.d > 2 {
            candidate = CollapseState::Candidate(cur.clone());
        } else if mc == Move::Sibling {
            candidate = CollapseState::Fresh;
        }

        prev = cur;
    }
    debug!("No candidate");
    None
}

fn collapse_workspace(ws: &Node, conn: &mut I3Connection) -> Result<usize, String> {
    debug!("In collapse_workspace");
    let mut ops: usize = 0;
    //for x in find_candidates(ws) {
    if let Some(x) = find_candidate(ws) {
        let cmd = format!("[con_id={}] move left", x.id);
        debug!("RUN:{}", cmd);
        let r = conn.run_command(&cmd).map_err(|e| format!("{}", e))?;
        debug!("GOT: {:?}", r);
        ops += 1;
    }
    debug!("collapse done");
    Ok(ops)
}

pub fn clean_current_workspace(conn: &mut I3Connection) -> Result<usize, String> {
    let mut collapse_ops = 0;
    loop {
        info::print_ws(conn, &info::STD);
        let node = conn.get_tree().expect("No tree result!?");
        let ws = node
            .get_current_workspace()
            .expect("No current workspace!?");
        let ops = collapse_workspace(ws, conn)?;
        collapse_ops += ops;
        if ops == 0 {
            return Ok(collapse_ops);
        }
    }
}
