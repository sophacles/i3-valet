use i3ipc::I3Connection;

pub fn alloc_workspsace(conn: &mut I3Connection) -> Result<(), String> {
    let ws_reply = conn
        .get_workspaces()
        .map_err(|e| format!("Get workspaces: {:?}", e))?;

    let mut ws_list = ws_reply.workspaces;
    ws_list.sort_by(|a, b| a.num.partial_cmp(&b.num).unwrap());

    let mut prev = 0;
    for ws in ws_list.iter().skip_while(|x| x.num < 1) {
        if ws.num - prev > 1 {
            break;
        }
        prev = ws.num;
    }

    let cmd = format!("workspace {}", prev + 1);
    let r = conn.run_command(&cmd).map_err(|e| format!("{}", e))?;
    debug!("RUN:{}\nGOT: {:?}", cmd, r);
    Ok(())
}
