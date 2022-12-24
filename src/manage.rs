use clap::ValueEnum;
use tokio_i3ipc::reply::Node;

use crate::ext::{NodeSearch, NotFound};

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum LayoutAction {
    /// Set the current window as the "main" window
    Set,
    /// Swaps:
    ///
    /// If main is focused, swaps with the previously focused window
    /// If another window is focused, swaps the current window with main
    Swap,
    /// Focus the main window
    Focus,
}

pub fn run_main(action: LayoutAction, tree: &Node) -> Result<Vec<String>, NotFound> {
    match action {
        LayoutAction::Set => make_main(tree),
        LayoutAction::Swap => swap_main(tree),
        LayoutAction::Focus => focus_main(tree),
    }
}

fn mark_name(wsname: &str, name: &str) -> String {
    format!("{}_{}", wsname, name)
}

fn unmark(target: Option<&Node>, mark: &str) -> String {
    match target {
        Some(n) => format!("[con_id={}] unmark {}", n.id, mark),
        None => format!("unmark {}", mark),
    }
}

fn mark(target: Option<&Node>, mark: &str) -> String {
    match target {
        Some(n) => format!("[con_id={}] mark --add {}", n.id, mark),
        None => format!("mark --add {}", mark),
    }
}

fn swap_mark(mark: &str) -> String {
    format!("swap container with mark {}", mark)
}

fn make_main(tree: &Node) -> Result<Vec<String>, NotFound> {
    //let node = conn.get_tree().map_err(|_| "get_tree 1")?;
    let ws = tree.get_current_workspace()?;

    Ok(vec![mark(
        None,
        &mark_name(ws.name.as_ref().unwrap(), "main"),
    )])
}

fn swap_main(tree: &Node) -> Result<Vec<String>, NotFound> {
    let ws = tree.get_current_workspace()?;

    let cur_window = ws.get_current_window()?;

    let main_mark = mark_name(ws.name.as_ref().unwrap(), "main");
    let last_mark = mark_name(ws.name.as_ref().unwrap(), "last");

    let main = ws.find_mark(&main_mark)?;

    let mut res = Vec::with_capacity(5);
    if cur_window.id == main.id {
        let n = ws.find_mark(&last_mark)?;
        res.push(swap_mark(&last_mark));
        res.push(unmark(None, &last_mark));
        res.push(unmark(None, &main_mark));
        res.push(mark(Some(n), &main_mark));
        res.push(mark(Some(cur_window), &last_mark));
    } else {
        res.push(swap_mark(&main_mark));
        res.push(unmark(None, &last_mark));
        res.push(unmark(None, &main_mark));
        res.push(mark(Some(cur_window), &main_mark));
        res.push(mark(Some(main), &last_mark));
    }

    Ok(res)
}

fn focus_main(tree: &Node) -> Result<Vec<String>, NotFound> {
    //let node = conn.get_tree().map_err(|_| "get_tree 1")?;
    let ws = tree.get_current_workspace()?;

    Ok(vec![format!(
        "[con_mark={}] focus",
        mark_name(ws.name.as_ref().unwrap(), "main")
    )])
}
