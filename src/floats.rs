use clap::ValueEnum;
use tokio_i3ipc::reply::{Node, Rect};

use crate::ext::NodeSearch;

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Pos {
    /// top-left corner
    NW,
    /// top-right corner
    NE,
    /// bottom-left corner
    SW,
    /// bottom-right corner
    SE,
    /// top-center edge
    Top,
    /// bottom-center edge
    #[value(name = "bot")]
    Bottom,
    /// center-left edge
    Left,
    /// center-right edge
    Right,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Positioning {
    /// relative to the output (will overlap bars)
    #[value(name = "abs")]
    Absolute,
    /// relative to the content area (will not overlap bars)
    #[value(name = "rel")]
    Relative,
}

pub fn teleport_float(tree: &Node, to: Pos, pos: Positioning) -> Result<Vec<String>, String> {
    log::info!("Teleport floating to: {:?}", to);

    //let tree = conn.get_tree().map_err(|e| format!("Get tree: {:?}", e))?;
    let current_window = okers(tree.get_current_window(), "current window")?;

    let current_display = match pos {
        Positioning::Relative => okers(DisplayArea::content(tree), "content")?,
        Positioning::Absolute => okers(DisplayArea::display(tree), "display")?,
    };

    let (x, y) = current_display.position_window(&current_window.rect, to);

    let cmd = format!("move position {} {}", x, y);
    Ok(vec![cmd])
}

struct DisplayArea(Rect);

impl DisplayArea {
    fn from_node(node: &Node) -> Self {
        DisplayArea(node.rect.clone())
    }

    fn display(tree: &Node) -> Option<Self> {
        Some(DisplayArea::from_node(tree.get_current_output()?))
    }

    fn content(tree: &Node) -> Option<Self> {
        Some(DisplayArea::from_node(tree.get_content_area()?))
    }

    fn position_window(&self, window: &Rect, to: Pos) -> (isize, isize) {
        let (x, y, w, h) = (self.0.x, self.0.y, self.0.width, self.0.height);
        let (.., ww, wh) = (window.width, window.height);
        match to {
            Pos::NW => (x, y),
            Pos::NE => ((x + w - ww), y),
            Pos::SW => (x, (y + h - wh)),
            Pos::SE => ((x + w - ww), (y + h - wh)),
            Pos::Top => ((x + w / 2 - ww / 2), y),
            Pos::Bottom => ((x + w / 2 - ww / 2), (y + h - wh)),
            Pos::Right => ((x + w - ww), (y + h / 2 - wh / 2)),
            Pos::Left => (x, (y + h / 2 - wh / 2)),
        }
    }
}

fn okers<T>(it: Option<T>, op: &str) -> Result<T, String> {
    it.ok_or(format!("Couldn't find in tree: {}", op))
}
