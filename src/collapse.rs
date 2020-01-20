extern crate log;

use std::collections::HashSet;

use i3ipc::{reply::Node, I3Connection};

use crate::ext::{i3_command, NodeSearch};

fn find_candidate(root: &Node) -> Vec<(&Node, i64)> {
    let mut leaves_seen = HashSet::new();
    let mut res: Vec<(&Node, i64)> = Vec::with_capacity(2);
    for s in root.preorder() {
        println!("Walk to: id({})", s.n.id);
        // skip root since it's a workspace and that gets messy
        // with marking and moving to mark...
        if s.d == 0 {
            continue;
        }

        for child in s.n.nodes.iter() {
            let mut n = child;
            while n.nodes.len() == 1 {
                n = &n.nodes[0];
            }

            // First time we encounter a leaf we have traversed from
            // the "highest" point - the one closest to root - so we
            // track the seen leaves, and ignore when they are found
            // again, since it's an ignored command issuee otherwise
            if n.id != child.id && !leaves_seen.contains(&n.id) {
                leaves_seen.insert(n.id);
                res.push((n, s.n.id));
            }
        }
    }
    res
}

pub fn clean_current_workspace(conn: &mut I3Connection) -> Result<(), String> {
    loop {
        //crate::info::print_ws(conn, &info::STD);
        let node = conn.get_tree().map_err(|e| format!("Get tree: {:?}", e))?;
        let ws = node
            .get_current_workspace()
            .expect("No current workspace!?");
        for (candidate, to) in find_candidate(ws) {
            let cmd = format!(
                "[con_id={}] mark i3v; [con_id={}] move container to mark i3v; unmark i3v",
                to, candidate.id
            );
            i3_command(&cmd, conn)?;
        }
        return Ok(());
    }
}
