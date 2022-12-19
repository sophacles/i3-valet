use clap::ValueEnum;
use i3ipc::{reply::Node, I3Connection};

use crate::ext::{NodeExt, NodeSearch, Step};

lazy_static! {
    pub static ref STD: StepFormatter = {
        let mut fmt = StepFormatter::new();
        fmt.set("id")
            .set("depth")
            .set("name")
            .set("layout")
            .set("marks")
            .set("move");
        fmt
    };
    pub static ref RECT: StepFormatter = {
        let mut fmt = StepFormatter::new();
        fmt.set("id").set("depth").set("name").set("rect");
        fmt
    };
    pub static ref WINDOW: StepFormatter = {
        let mut fmt = StepFormatter::new();
        fmt.show_indent(false);
        fmt.set("id")
            .set("floating")
            .set("depth")
            .set("name")
            .set("layout")
            .set("marks");
        fmt
    };
}

pub struct StepFormatter {
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
    pub fn new() -> Self {
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

    pub fn show_indent(&mut self, v: bool) -> &mut StepFormatter {
        self.indent = v;
        self
    }

    pub fn short_id(&mut self, v: bool) -> &mut StepFormatter {
        self.short_id = v;
        self
    }

    pub fn set(&mut self, what: &str) -> &mut StepFormatter {
        match what {
            "depth" => self.depth = true,
            "id" => self.id = true,
            "name" => self.name = true,
            "focus" => self.focus = true,
            "layout" => self.layout = true,
            "rect" => self.rect = true,
            "marks" => self.marks = true,
            "floating" => self.floating = true,
            "move" => self.moveto = true,
            _ => (),
        }
        self
    }

    pub fn unset(&mut self, what: &str) -> &mut StepFormatter {
        match what {
            "depth" => self.depth = false,
            "id" => self.id = false,
            "name" => self.name = false,
            "focus" => self.focus = false,
            "layout" => self.layout = false,
            "rect" => self.rect = false,
            "marks" => self.marks = false,
            "floating" => self.floating = false,
            "move" => self.moveto = false,
            _ => (),
        }
        self
    }

    pub fn format(&self, s: &Step) -> String {
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
            out.push(s.n.marks.join(","));
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

pub fn pretty_print(n: &Node, fmt: &StepFormatter) -> Result<(), String> {
    println!("Tree:");
    for s in n.preorder() {
        println!("{}", fmt.format(&s));
    }
    Ok(())
}

pub fn post_print(n: &Node, fmt: &StepFormatter) -> Result<(), String> {
    println!("Tree:");
    for s in n.postorder() {
        println!("{}", fmt.format(&s));
    }
    Ok(())
}

pub fn print_window(conn: &mut I3Connection, fmt: &StepFormatter) -> Result<(), String> {
    let node = conn.get_tree().expect("get_tree 1");
    let ws = node.get_current_window().expect("workspace 2");
    pretty_print(ws, fmt)
}

pub fn print_ws(conn: &mut I3Connection, fmt: &StepFormatter) -> Result<(), String> {
    let node = conn.get_tree().expect("get_tree 1");
    let ws = node.get_current_workspace().expect("workspace 2");

    println!("pre:");
    pretty_print(ws, fmt)
}

pub fn print_disp(conn: &mut I3Connection, fmt: &StepFormatter) -> Result<(), String> {
    let node = conn.get_tree().expect("get_tree 1");
    let d = node.get_current_output().expect("workspace 2");
    pretty_print(d, fmt)
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum PrintTarget {
    Tree,
    Rects,
    Window,
}

pub fn run(target: PrintTarget, conn: &mut I3Connection) -> Result<(), String> {
    match target {
        PrintTarget::Tree => print_ws(conn, &STD),
        PrintTarget::Rects => print_disp(conn, &RECT),
        PrintTarget::Window => print_window(conn, &WINDOW),
    }
}
