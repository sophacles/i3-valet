use std::str::FromStr;

use i3ipc::reply::Node;
use i3ipc::I3Connection;

use crate::NodeSearch;

#[derive(Debug)]
pub enum Loc {
    NW,
    NE,
    SW,
    SE,
    Top,
    Bottom,
}

impl FromStr for Loc {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nw" => Ok(Loc::NW),
            "ne" => Ok(Loc::NE),
            "sw" => Ok(Loc::SW),
            "se" => Ok(Loc::SE),
            "top" => Ok(Loc::Top),
            "bot" => Ok(Loc::Bottom),
            _ => Err(format!("Bad input: {}", s)),
        }
    }
}
//            x    y    w    h
type Rect = (i32, i32, i32, i32);

struct Mon {
    bounds: Rect,
}

impl Mon {
    pub fn from_node(node: &Node) -> Self {
        Mon { bounds: node.rect }
    }

    pub fn move_to(&self, container: Rect, to: Loc) -> (i32, i32) {
        let (x, y, w, h) = self.bounds;
        let (.., ww, wh) = container;
        match to {
            Loc::NW => (x, y),
            Loc::NE => ((x + w - ww), y),
            Loc::SW => (x, (y + h - wh)),
            Loc::SE => ((x + w - ww), (y + h - wh)),
            Loc::Top => ((x + w / 2 - ww / 2), y),
            Loc::Bottom => ((x + w / 2 - ww / 2), (y + h - wh)),
        }
    }
}

pub fn teleport_float(conn: &mut I3Connection, to: Loc, honor_bar: bool) -> Option<i64> {
    println!("Teleport floating to: {:?}", to);

    let tree = conn.get_tree().ok()?;
    let target: &Node = match honor_bar {
        true => tree.get_content_area()?,
        false => tree.get_current_output()?,
    };

    let current_window = target.get_current_window()?;

    let cur_display = Mon::from_node(target);
    let (x, y) = cur_display.move_to(current_window.rect, to);

    let cmd = format!("move position {} {}", x, y);
    let r = conn.run_command(&cmd).map_err(|e| format!("{}", e));
    debug!("RUN:{}\nGOT: {:?}", cmd, r);
    Some(current_window.id)
}
