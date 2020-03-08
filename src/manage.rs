use i3ipc::{reply::Node, I3Connection};

use crate::{
    ext::{i3_command, NodeSearch},
    info,
};

pub fn is_shaped_right(conn: &mut I3Connection) -> Result<(), String> {
    println!("Hello manage!");
    info::print_ws(conn, &info::STD)?;
    let node = conn.get_tree().map_err(|_| "get_tree 1")?;
    let ws = node.get_current_workspace().ok_or("workspace 2")?;
    let mut fmt = info::StepFormatter::new();
    let fmt = fmt.show_indent(false).set("id");

    for n in ws.preorder() {
        print!("{} ", fmt.format(&n));
    }
    print!("\n");
    for n in ws.postorder() {
        print!("{} ", fmt.format(&n));
    }
    i3_command(&format!("[con_id={}] split horizontal", ws.id), conn)?;
    print!("\n");
    Ok(())
}

pub fn shape(conn: &mut I3Connection) -> Result<(), String> {
    swap_main(conn)
}

fn mark_name(wsname: &str, name: &str) -> String {
    format!("{}_{}", wsname, name)
}

fn unmark(target: Option<&Node>, mark: &str, conn: &mut I3Connection) -> Result<(), String> {
    let cmd = match target {
        Some(n) => format!("[con_id={}] unmark {}", n.id, mark),
        None => format!("unmark {}", mark),
    };
    i3_command(&cmd, conn)
}

fn mark(target: Option<&Node>, mark: &str, conn: &mut I3Connection) -> Result<(), String> {
    let cmd = match target {
        Some(n) => format!("[con_id={}] mark --add {}", n.id, mark),
        None => format!("mark --add {}", mark),
    };
    i3_command(&cmd, conn)
}

fn swap_mark(mark: &str, conn: &mut I3Connection) -> Result<(), String> {
    i3_command(&format!("swap container with mark {}", mark), conn)
}

pub fn make_main(conn: &mut I3Connection) -> Result<(), String> {
    let node = conn.get_tree().map_err(|_| "get_tree 1")?;
    let ws = node
        .get_current_workspace()
        .ok_or("No current workspace?!")?;

    mark(None, &mark_name(ws.name.as_ref().unwrap(), "main"), conn)
}

pub fn swap_main(conn: &mut I3Connection) -> Result<(), String> {
    let node = conn.get_tree().map_err(|_| "get_tree 1")?;
    let ws = node
        .get_current_workspace()
        .ok_or("No current workspace?!")?;

    let cur_window = ws.get_current_window().ok_or("nothing focused?")?;

    let main_mark = mark_name(ws.name.as_ref().unwrap(), "main");
    let last_mark = mark_name(ws.name.as_ref().unwrap(), "last");

    let main = ws.find_mark(&main_mark).ok_or("No main, aborting")?;

    if cur_window.id == main.id {
        match ws.find_mark(&last_mark) {
            Some(n) => {
                swap_mark(&last_mark, conn)?;
                unmark(None, &last_mark, conn)?;
                unmark(None, &main_mark, conn)?;
                mark(Some(n), &main_mark, conn)?;
                mark(Some(cur_window), &last_mark, conn)?;
                return Ok(());
            }
            None => {
                return Err("No last window, aborting".to_string());
            }
        }
    } else {
        swap_mark(&main_mark, conn)?;
        unmark(None, &last_mark, conn)?;
        unmark(None, &main_mark, conn)?;
        mark(Some(cur_window), &main_mark, conn)?;
        mark(Some(main), &last_mark, conn)?;
    }

    Ok(())
}

pub fn focus_main(conn: &mut I3Connection) -> Result<(), String> {
    let wslist = conn
        .get_workspaces()
        .map_err(|_| "Cannot get workspace list!")?;
    for ws in wslist.workspaces {
        if ws.focused {
            i3_command(
                &format!("[con_mark={}] focus", mark_name(&ws.name, "main")),
                conn,
            )?;
        }
    }
    Ok(())
}
