use i3ipc::I3Connection;

enum Direction {
    Next,
    Prev,
}

// TODO: clean me up
fn neighbor(which: Direction, conn: &mut I3Connection) -> Result<String, String> {
    let ws_reply = conn
        .get_workspaces()
        .map_err(|e| format!("Get workspaces: {:?}", e))?;

    let current_ws = ws_reply
        .workspaces
        .into_iter()
        .find(|ws| ws.focused)
        .ok_or("No focused workspace?")?;

    let current_output = current_ws.output;

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

pub fn focus_next_output(conn: &mut I3Connection) -> Result<(), String> {
    let target = neighbor(Direction::Next, conn)?;

    let cmd = format!("focus output {}", target);
    let r = conn.run_command(&cmd).map_err(|e| format!("{}", e))?;
    debug!("RUN:{}\nGOT: {:?}", cmd, r);
    Ok(())
}

pub fn focus_prev_output(conn: &mut I3Connection) -> Result<(), String> {
    let target = neighbor(Direction::Prev, conn)?;

    let cmd = format!("focus output {}", target);
    let r = conn.run_command(&cmd).map_err(|e| format!("{}", e))?;
    debug!("RUN:{}\nGOT: {:?}", cmd, r);
    Ok(())
}
