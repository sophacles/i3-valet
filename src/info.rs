use clap::ValueEnum;
use tokio_i3ipc::reply::Node;

use crate::ext::{NodeExt, NodeSearch, NotFound, Step};

use lazy_static::lazy_static;

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum PrintTarget {
    /// print the whole tree
    Tree,
    /// print just the dimentions of windows in the tree
    Rects,
    /// print info about only the current window
    Window,
}

pub fn run(target: PrintTarget, tree: &Node) -> Result<(), NotFound> {
    //let node = conn.get_tree().expect("get_tree 1");
    let (to_print, fmt) = match target {
        PrintTarget::Tree => (tree.get_current_workspace()?, &*STD),
        PrintTarget::Rects => (tree.get_current_output()?, &*RECT),
        PrintTarget::Window => (tree.get_current_window()?, &*WINDOW),
    };

    pretty_print(to_print, fmt);
    Ok(())
}

lazy_static! {
    static ref STD: StepFormatter = {
        let mut fmt = StepFormatter::new();
        fmt.id = true;
        fmt.depth = true;
        fmt.name = true;
        fmt.layout = true;
        fmt.marks = true;
        fmt.moveto = true;
        fmt
    };
    static ref RECT: StepFormatter = {
        let mut fmt = StepFormatter::new();
        fmt.id = true;
        fmt.depth = true;
        fmt.name = true;
        fmt.rect = true;
        fmt
    };
    static ref WINDOW: StepFormatter = {
        let mut fmt = StepFormatter::new();
        fmt.show_indent(false);
        fmt.id = true;
        fmt.floating = true;
        fmt.depth = true;
        fmt.name = true;
        fmt.layout = true;
        fmt.marks = true;
        fmt
    };
}

struct StepFormatter {
    // shorten id since much of it is redundant
    short_id: bool,
    // if depth is on, indent by depth spaces
    indent: bool,
    // track columns
    depth: bool,
    id: bool,
    name: bool,
    focus: bool,
    layout: bool,
    rect: bool,
    marks: bool,
    floating: bool,
    moveto: bool,
}

impl StepFormatter {
    fn new() -> Self {
        StepFormatter {
            indent: true,
            short_id: true,
            depth: false,
            id: false,
            name: false,
            focus: false,
            layout: false,
            rect: false,
            marks: false,
            floating: false,
            moveto: false,
        }
    }

    fn show_indent(&mut self, v: bool) -> &mut StepFormatter {
        self.indent = v;
        self
    }

    #[allow(dead_code)]
    fn short_id(&mut self, v: bool) -> &mut StepFormatter {
        self.short_id = v;
        self
    }

    fn format(&self, s: &Step) -> String {
        let mut out: Vec<String> = vec![];
        if self.depth {
            if self.indent {
                out.push(format!("{}{}", "  ".repeat(s.d), s.d));
            } else {
                out.push(format!("{}", s.d));
            }
        }

        if self.name {
            out.push(
                s.n.name
                    .as_ref()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| String::from("None")),
            );
        }

        if self.id {
            if self.short_id {
                let s = format!("{}", s.n.id);
                let last = &s[s.len() - 5..];
                out.push(last.to_string());
            } else {
                out.push(s.n.id.to_string());
            }
        }
        if self.focus {
            out.push(
                match s.n.focused {
                    true => "F",
                    false => "U",
                }
                .to_string(),
            );
        }
        if self.layout {
            out.push(format!("{:?}", s.n.layout));
        }
        if self.rect {
            out.push(format!("{:?}", s.n.rect));
        }
        if self.marks {
            if let Some(ref marks) = s.n.marks {
                out.push(marks.0.join(","));
            }
        }
        if self.floating {
            out.push(format!("{:.1}", s.n.is_floating()));
        }
        if self.moveto {
            out.push(format!("{:?}", s.m));
        }

        out.join(" ")
    }
}

impl Default for StepFormatter {
    fn default() -> Self {
        Self::new()
    }
}

fn pretty_print(n: &Node, fmt: &StepFormatter) {
    println!("Tree:");
    for s in n.preorder() {
        println!("{}", fmt.format(&s));
    }
}

#[allow(dead_code)]
fn post_print(n: &Node, fmt: &StepFormatter) {
    println!("Tree:");
    for s in n.postorder() {
        println!("{}", fmt.format(&s));
    }
}
