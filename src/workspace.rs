use clap::ValueEnum;
use tokio_i3ipc::reply::Workspaces;

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum WorkspaceTarget {
    /// Create and focus a new workspace
    Alloc,
    /// Create a new workspace and move the current focused container to it.
    MoveNew,
}

pub fn run(target: WorkspaceTarget, workspaces: &mut Workspaces) -> Vec<String> {
    let ws = next_free_workspace(workspaces);
    let cmd = match target {
        WorkspaceTarget::Alloc => format!("workspace {}", ws),
        WorkspaceTarget::MoveNew => format!("move container to workspace {}; workspace {}", ws, ws),
    };
    vec![cmd]
}

pub fn next_free_workspace(workspaces: &mut Workspaces) -> i32 {
    workspaces.sort_by(|a, b| a.num.partial_cmp(&b.num).unwrap());

    // go over the workspaces that are present.
    // Workspace 0 doesn't exist, and named workspaces are num = -1
    // After ws 1, any gap we find (where cur.num - prev > 1)
    // we'll break the search, leaving prev set to the last ws before
    // gap. If there's no gaps, we'll fall off the end, with prev being
    // the highest seen ws num. In both cases, adding 1 gets us what we
    // want.
    let mut prev = 0;
    for ws in workspaces.iter().skip_while(|x| x.num < 1) {
        if ws.num - prev > 1 {
            break;
        }
        prev = ws.num;
    }
    prev + 1
}
