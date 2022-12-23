use clap::ValueEnum;
use i3ipc::reply::Node;

use crate::ext::NodeSearch;

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Loc {
    NW,
    NE,
    SW,
    SE,
    Top,
    #[value(name = "bot")]
    Bottom,
    Left,
    Right,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Positioning {
    /// relative to the output
    #[value(name = "abs")]
    Absolute,
    /// relative to the content area
    #[value(name = "rel")]
    Relative,
}

//            x    y    w    h
type Rect = (i32, i32, i32, i32);

struct DisplayArea(Rect);

impl DisplayArea {
    pub fn from_node(node: &Node) -> Self {
        DisplayArea(node.rect)
    }

    pub fn display(tree: &Node) -> Option<Self> {
        Some(DisplayArea::from_node(tree.get_current_output()?))
    }

    pub fn content(tree: &Node) -> Option<Self> {
        Some(DisplayArea::from_node(tree.get_content_area()?))
    }

    pub fn position_window(&self, window: Rect, to: Loc) -> (i32, i32) {
        let (x, y, w, h) = self.0;
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

fn okers<T>(it: Option<T>, op: &str) -> Result<T, String> {
    it.ok_or(format!("Couldn't find in tree: {}", op))
}

pub fn teleport_float(tree: &Node, to: Loc, pos: Positioning) -> Result<Vec<String>, String> {
    println!("Teleport floating to: {:?}", to);

    //let tree = conn.get_tree().map_err(|e| format!("Get tree: {:?}", e))?;
    let current_window = okers(tree.get_current_window(), "current window")?;

    let current_display = match pos {
        Positioning::Relative => okers(DisplayArea::content(tree), "content")?,
        Positioning::Absolute => okers(DisplayArea::display(tree), "display")?,
    };

    let (x, y) = current_display.position_window(current_window.rect, to);

    let cmd = format!("move position {} {}", x, y);
    Ok(vec![cmd])
}
