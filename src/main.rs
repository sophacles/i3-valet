use log::*;

//use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use clap::{Parser, Subcommand};
use i3ipc::{
    event::{BindingEventInfo, Event},
    I3Connection, I3EventListener, Subscription,
};

pub mod collapse;
pub mod ext;
pub mod floats;
pub mod info;
pub mod manage;
pub mod output;
pub mod workspace;

#[derive(Subcommand, Debug)]
enum LayoutCmd {
    Main { action: manage::LayoutAction },
}

#[derive(Subcommand, Debug)]
enum Sub {
    /// clean up the window tree
    Fix,

    /// Move A floating window to anchor point
    Loc {
        /// Positioning of window.
        how: floats::Positioning,
        /// Anchor point to position window
        pos: floats::Loc,
    },

    ///Print window information
    Print {
        /// what to print
        target: info::PrintTarget,
    },

    /// Workspace commands
    Workspace {
        /// what to do
        target: workspace::WorkspaceTarget,
    },

    /// Output commands
    Output {
        change: output::Change,
        dir: output::Direction,
    },

    /// Movement within the layout
    Layout {
        #[command(subcommand)]
        cmd: LayoutCmd,
    },

    /// connect to i3 socket and wait for events
    Listen,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct App {
    #[command(subcommand)]
    cmd: Sub,
}

#[derive(Parser, Debug)]
#[command(no_binary_name(true), author, version, about, long_about = None)]
struct ReceivedCmd {
    #[command(subcommand)]
    cmd: Sub,
}

impl Sub {
    fn dispatch(&self, conn: &mut I3Connection) -> Result<(), String> {
        println!("Dispatching: {:?}", self);
        let cmds = match self {
            Sub::Fix => {
                let tree = conn.get_tree().map_err(|e| format!("Get tree: {:?}", e))?;
                collapse::clean_current_workspace(&tree)
            }
            Sub::Listen => Err("Cannot dispatch listen: cli command only.".to_string()),
            Sub::Loc { pos, how } => {
                let tree = conn.get_tree().map_err(|e| format!("Get tree: {:?}", e))?;
                floats::teleport_float(&tree, *pos, *how)
            }
            Sub::Print { target } => {
                let tree = conn.get_tree().map_err(|e| format!("Get tree: {:?}", e))?;
                info::run(*target, &tree)
            }
            Sub::Workspace { target } => {
                let mut workspaces = conn
                    .get_workspaces()
                    .map_err(|e| format!("Get workspaces: {:?}", e))?;
                workspace::run(*target, &mut workspaces)
            }
            Sub::Output { change, dir } => {
                let workspaces = conn
                    .get_workspaces()
                    .map_err(|e| format!("Get workspaces: {:?}", e))?;

                let outputs = conn
                    .get_outputs()
                    .map_err(|e| format!("Cannot get outputs: {:?}", e))?;
                output::run(*change, *dir, &workspaces, &outputs)
            }
            Sub::Layout { cmd } => match cmd {
                LayoutCmd::Main { action } => {
                    let tree = conn.get_tree().map_err(|e| format!("Get tree: {:?}", e))?;
                    manage::run_main(*action, &tree)
                }
            },
        }?;

        for cmd in cmds {
            ext::i3_command(&cmd, conn)?;
        }
        Ok(())
    }
}

fn main() {
    env_logger::init();

    let app = App::parse();
    if 

    if let Err(res) = match app.cmd {
        Sub::Listen => listener(),
        _ => {
            let mut conn = I3Connection::connect().expect("i3connect");
            app.cmd.dispatch(&mut conn)
        }
    } {
        eprintln!("Error running command: {}", res);
        std::process::exit(1);
    }
    println!("Goodbye?!");
}

fn parse_command_string(cmd: &str) -> Result<ReceivedCmd, String> {
    debug!("parsing command: {}", cmd);
    let mut args = cmd.split_whitespace();
    if let Some("nop") = args.next() {
        return ReceivedCmd::try_parse_from(args)
            .map_err(|e| format!("Error parsing command: {:?}", e));
    }
    Err(format!("Skipping non-valet command: {}", cmd))
}

fn handle_binding_event(e: BindingEventInfo, conn: &mut I3Connection) -> Result<(), String> {
    debug!("Saw BindingEvent: {:#?}", e);
    for subcmd in e.binding.command.split(';') {
        let parsed_cmd = parse_command_string(subcmd)?;
        parsed_cmd.cmd.dispatch(conn)?;
    }
    Ok(())
}

fn listener() -> Result<(), String> {
    let mut listener = I3EventListener::connect().unwrap();

    let subs = [Subscription::Binding];
    listener.subscribe(&subs).unwrap();

    for evt in listener.listen() {
        let evt = evt.map_err(|_| "Connection died, i3 is most likey termnating")?;
        if let Event::BindingEvent(b) = evt {
            let mut command_conn = I3Connection::connect().expect("i3connect");
            if let Err(e) = handle_binding_event(b, &mut command_conn) {
                warn!("Encountered Error in listener: {}", e);
            }
        }
    }
    Ok(())
}
