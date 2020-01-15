use std::str::FromStr;

use i3ipc::reply::Node;
use i3ipc::I3Connection;

use crate::ext::NodeSearch;

#[derive(Debug)]
pub enum Loc {
    NW,
    NE,
    SW,
    SE,
    Top,
    Bottom,
    Left,
    Right,
}

pub enum Positioning {
    Absolute,
    Relative,
}

impl FromStr for Positioning {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "abs" => Ok(Positioning::Absolute),
            "rel" => Ok(Positioning::Relative),
            _ => Err(format!("Not a valid Position: {}", s)),
        }
    }
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
            "left" => Ok(Loc::Left),
            "right" => Ok(Loc::Right),
            _ => Err(format!("Not a valid Loc: {}", s)),
        }
    }
}
//            x    y    w    h
type Rect = (i32, i32, i32, i32);

struct DisplayArea {
    bounds: Rect,
}

impl DisplayArea {
    pub fn from_node(node: &Node) -> Self {
        DisplayArea { bounds: node.rect }
    }

    pub fn display(tree: &Node) -> Option<Self> {
        Some(DisplayArea::from_node(tree.get_current_output()?))
    }

    pub fn content(tree: &Node) -> Option<Self> {
        Some(DisplayArea::from_node(tree.get_content_area()?))
    }

    pub fn position_window(&self, window: Rect, to: Loc) -> (i32, i32) {
        let (x, y, w, h) = self.bounds;
        let (.., ww, wh) = window;
        match to {
            Loc::NW => (x, y),
            Loc::NE => ((x + w - ww), y),
            Loc::SW => (x, (y + h - wh)),
            Loc::SE => ((x + w - ww), (y + h - wh)),
            Loc::Top => ((x + w / 2 - ww / 2), y),
            Loc::Bottom => ((x + w / 2 - ww / 2), (y + h - wh)),
            Loc::Right => ((x + w - ww), (y + h / 2 - wh / 2)),
            Loc::Left => (x, (y + h / 2 - wh / 2)),
        }
    }
}

pub fn teleport_float(conn: &mut I3Connection, to: Loc, pos: Positioning) -> Option<i64> {
    println!("Teleport floating to: {:?}", to);

    let tree = conn.get_tree().ok()?;
    let current_window = tree.get_current_window()?;

    let current_display = match pos {
        Positioning::Relative => DisplayArea::content(&tree)?,
        Positioning::Absolute => DisplayArea::display(&tree)?,
    };

    let (x, y) = current_display.position_window(current_window.rect, to);

    let cmd = format!("move position {} {}", x, y);
    let r = conn.run_command(&cmd).map_err(|e| format!("{}", e));
    debug!("RUN:{}\nGOT: {:?}", cmd, r);
    Some(current_window.id)
}
