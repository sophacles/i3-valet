use clap::ValueEnum;
use i3ipc::{reply::Node, I3Connection};

use crate::ext::NodeSearch;

fn mark_name(wsname: &str, name: &str) -> String {
    format!("{}_{}", wsname, name)
}

fn unmark(target: Option<&Node>, mark: &str) -> Result<String, String> {
    let cmd = match target {
        Some(n) => format!("[con_id={}] unmark {}", n.id, mark),
        None => format!("unmark {}", mark),
    };
    Ok(cmd)
}

fn mark(target: Option<&Node>, mark: &str) -> Result<String, String> {
    let cmd = match target {
        Some(n) => format!("[con_id={}] mark --add {}", n.id, mark),
        None => format!("mark --add {}", mark),
    };
    Ok(cmd)
}

fn swap_mark(mark: &str) -> Result<String, String> {
    Ok(format!("swap container with mark {}", mark))
}

pub fn make_main(conn: &mut I3Connection) -> Result<Vec<String>, String> {
    let node = conn.get_tree().map_err(|_| "get_tree 1")?;
    let ws = node
        .get_current_workspace()
        .ok_or("No current workspace?!")?;

    mark(None, &mark_name(ws.name.as_ref().unwrap(), "main")).map(|s| vec![s])
}

pub fn swap_main(conn: &mut I3Connection) -> Result<Vec<String>, String> {
    let node = conn.get_tree().map_err(|_| "get_tree 1")?;
    let ws = node
        .get_current_workspace()
        .ok_or("No current workspace?!")?;

    let cur_window = ws.get_current_window().ok_or("nothing focused?")?;

    let main_mark = mark_name(ws.name.as_ref().unwrap(), "main");
    let last_mark = mark_name(ws.name.as_ref().unwrap(), "last");

    let main = ws.find_mark(&main_mark).ok_or("No main, aborting")?;

    let mut res = Vec::with_capacity(5);
    if cur_window.id == main.id {
        match ws.find_mark(&last_mark) {
            Some(n) => {
                res.push(swap_mark(&last_mark)?);
                res.push(unmark(None, &last_mark)?);
                res.push(unmark(None, &main_mark)?);
                res.push(mark(Some(n), &main_mark)?);
                res.push(mark(Some(cur_window), &last_mark)?);
            }
            None => {
                return Err("No last window, aborting".to_string());
            }
        }
    } else {
        res.push(swap_mark(&main_mark)?);
        res.push(unmark(None, &last_mark)?);
        res.push(unmark(None, &main_mark)?);
        res.push(mark(Some(cur_window), &main_mark)?);
        res.push(mark(Some(main), &last_mark)?);
    }

    Ok(res)
}

pub fn focus_main(conn: &mut I3Connection) -> Result<Vec<String>, String> {
    let wslist = conn
        .get_workspaces()
        .map_err(|_| "Cannot get workspace list!")?;

    Ok(wslist
        .workspaces
        .iter()
        .filter_map(|ws| {
            if ws.focused {
                Some(format!("[con_mark={}] focus", mark_name(&ws.name, "main")))
            } else {
                None
            }
        })
        .collect())
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum LayoutAction {
    Set,
    Swap,
    Focus,
}

pub fn run_main(action: LayoutAction, conn: &mut I3Connection) -> Result<Vec<String>, String> {
    match action {
        LayoutAction::Set => make_main(conn),
        LayoutAction::Swap => swap_main(conn),
        LayoutAction::Focus => focus_main(conn),
    }
}
