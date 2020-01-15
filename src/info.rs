use i3ipc::reply::Node;
use i3ipc::I3Connection;

use crate::ext::{NodeExt, NodeSearch, Step};

lazy_static! {
    pub static ref STD: StepFormatter = {
        let mut fmt = StepFormatter::new();
        fmt.set("id")
            .set("depth")
            .set("name")
            .set("layout")
            .set("marks");
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
}

impl StepFormatter {
    pub fn new() -> Self {
        StepFormatter {
            indent: true,
            depth: false,
            id: false,
            name: false,
            focus: false,
            layout: false,
            rect: false,
            marks: false,
            floating: false,
        }
    }

    pub fn show_indent(&mut self, v: bool) {
        self.indent = v;
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
            out.push(format!(
                "{}",
                s.n.name.as_ref().unwrap_or(&String::from("None"))
            ));
        }

        if self.id {
            out.push(format!("{}", s.n.id));
        }
        if self.focus {
            out.push(format!(
                "{}",
                match s.n.focused {
                    true => "F",
                    false => "U",
                }
            ));
        }
        if self.layout {
            out.push(format!("{:?}", s.n.layout));
        }
        if self.rect {
            out.push(format!("{:?}", s.n.rect));
        }
        if self.marks {
            out.push(format!("{}", s.n.marks.join(",")));
        }
        if self.floating {
            out.push(format!("{:.1}", s.n.is_floating()));
        }
        out.join(" ")
    }
}

pub fn pretty_print(n: &Node, fmt: &StepFormatter) -> Result<(), String> {
    println!("Tree:");
    for s in n.preorder() {
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
    pretty_print(ws, fmt)
}

pub fn print_disp(conn: &mut I3Connection, fmt: &StepFormatter) -> Result<(), String> {
    let node = conn.get_tree().expect("get_tree 1");
    let d = node.get_current_output().expect("workspace 2");
    pretty_print(d, fmt)
}
