use clap::ValueEnum;
use i3ipc::I3Connection;

use crate::ext::i3_command;

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum Direction {
    Next,
    Prev,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Change {
    /// move workspace to a different output
    #[value(name = "move-ws")]
    MoveWs,
    /// move workspace to a different output
    #[value(name = "move-win")]
    MoveWin,
    /// focus a different output
    #[value(name = "focus")]
    Focus,
}

// TODO: clean me up
fn neighbor(which: Direction, conn: &mut I3Connection) -> Result<String, String> {
    // find current workspace since that will
    // also tell us the current output
    let ws_reply = conn
        .get_workspaces()
        .map_err(|e| format!("Get workspaces: {:?}", e))?;

    let current_ws = ws_reply
        .workspaces
        .into_iter()
        .find(|ws| ws.focused)
        .ok_or("No focused workspace?")?;

    let current_output = current_ws.output;

    // get the list of active output names.
    // active seems to mean "can display things"
    let output_list = conn
        .get_outputs()
        .map_err(|e| format!("Cannot get outputs: {:?}", e))?;

    let mut output_list: Vec<String> = output_list
        .outputs
        .iter()
        .filter(|o| o.active)
        .map(|o| o.name.clone())
        .collect();

    if let Direction::Prev = which {
        output_list.reverse();
    }

    let mut found: bool = false;
    // we need outputs to look like this: ABCA so if we're at
    // c we can just wrap.
    for o in output_list.iter().cycle().take(output_list.len() + 1) {
        if found {
            return Ok(o.into());
        }
        if *o == current_output {
            found = true
        }
    }

    Err("There's no output to select?".to_owned())
}

pub fn focus(dir: Direction, conn: &mut I3Connection) -> Result<(), String> {
    let target = neighbor(dir, conn)?;

    let cmd = format!("focus output {}", target);
    i3_command(&cmd, conn)
}

pub fn workspace(dir: Direction, conn: &mut I3Connection) -> Result<(), String> {
    let target = neighbor(dir, conn)?;

    let cmd = format!("move workspace to output {}", target);
    i3_command(&cmd, conn)
}

pub fn window(dir: Direction, conn: &mut I3Connection) -> Result<(), String> {
    let target = neighbor(dir, conn)?;

    let cmd = format!("move window to output {}", target);
    i3_command(&cmd, conn)
}

pub fn run(change: Change, dir: Direction, conn: &mut I3Connection) -> Result<(), String> {
    match change {
        Change::Focus => focus(dir, conn),
        Change::MoveWs => workspace(dir, conn),
        Change::MoveWin => window(dir, conn),
    }
}
