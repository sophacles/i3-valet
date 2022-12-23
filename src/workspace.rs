use clap::ValueEnum;
use i3ipc::I3Connection;

pub fn next_free_workspace(conn: &mut I3Connection) -> Result<i32, String> {
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
    Ok(prev + 1)
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum WorkspaceTarget {
    Alloc,
    MoveNew,
}

pub fn run(target: WorkspaceTarget, conn: &mut I3Connection) -> Result<Vec<String>, String> {
    let ws = next_free_workspace(conn)?;
    let cmd = match target {
        WorkspaceTarget::Alloc => format!("workspace {}", ws),
        WorkspaceTarget::MoveNew => format!("move container to workspace {}; workspace {}", ws, ws),
    };
    Ok(vec![cmd])
}
