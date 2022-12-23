use clap::ValueEnum;
use i3ipc::reply::{Outputs, Workspaces};

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
fn neighbor(
    which: Direction,
    workspaces: &Workspaces,
    outputs: &Outputs,
) -> Result<String, String> {
    // find current workspace since that will
    // also tell us the current output
    //let ws_reply = conn
    //  .get_workspaces()
    //  .map_err(|e| format!("Get workspaces: {:?}", e))?;

    let current_ws = workspaces
        .workspaces
        .iter()
        .find(|ws| ws.focused)
        .ok_or("No focused workspace?")?;

    let current_output = current_ws.output.clone();

    // get the list of active output names.
    // active seems to mean "can display things"
    //let output_list = conn
    //    .get_outputs()
    //    .map_err(|e| format!("Cannot get outputs: {:?}", e))?;

    let mut output_list: Vec<String> = outputs
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

pub fn run(
    change: Change,
    dir: Direction,
    workspaces: &Workspaces,
    outputs: &Outputs,
) -> Result<Vec<String>, String> {
    let target = neighbor(dir, workspaces, outputs)?;

    let cmd = match change {
        Change::Focus => format!("focus output {}", target),
        Change::MoveWs => format!("move workspace to output {}", target),
        Change::MoveWin => format!("move window to output {}", target),
    };
    Ok(vec![cmd])
}
