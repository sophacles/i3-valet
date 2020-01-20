use i3ipc::I3Connection;

use crate::ext::i3_command;

pub fn alloc_workspsace(conn: &mut I3Connection) -> Result<(), String> {
    let ws_reply = conn
        .get_workspaces()
        .map_err(|e| format!("Get workspaces: {:?}", e))?;

    let mut ws_list = ws_reply.workspaces;
    ws_list.sort_by(|a, b| a.num.partial_cmp(&b.num).unwrap());

    // go over the workspaces that are present.
    // Workspace 0 doesn't exist, and named workspaces are num = -1
    // After ws 1, any gap we find (where cur.num - prev > 1)
    // we'll break the search, leaving prev set to the last ws before
    // gap. If there's no gaps, we'll fall off the end, with prev being
    // the highest seen ws num. In both cases, adding 1 gets us what we
    // want.
    let mut prev = 0;
    for ws in ws_list.iter().skip_while(|x| x.num < 1) {
        if ws.num - prev > 1 {
            break;
        }
        prev = ws.num;
    }

    let cmd = format!("workspace {}", prev + 1);
    i3_command(&cmd, conn)
}
