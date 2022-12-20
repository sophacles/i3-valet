use std::io;

use tokio_i3ipc::{
    event::{Event, Subscribe},
    I3,
};
use tokio_stream::StreamExt;

use log::*;
//use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use clap::{Parser, Subcommand};
//use i3ipc::{
//    event::{BindingEventInfo, Event},
//    I3Connection, I3EventListener, Subscription,
//};

pub mod collapse;
pub mod ext;
pub mod floats;
pub mod info;
pub mod manage;
pub mod output;
pub mod workspace;

#[derive(Subcommand, Debug)]
enum LayoutCmd {
    /// Set and focus (etc) a main window and auxilliary windows
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
        pos: floats::Pos,
    },

    ///Print information about the current tree or window
    Print {
        /// what to print
        target: info::PrintTarget,
    },

    /// Workspace commands
    Workspace {
        /// what to do
        target: workspace::WorkspaceTarget,
    },

    /// Movement between relative outputs.
    ///
    /// This assumes outputs are linear and cycles through them in order
    Output {
        change: output::Change,
        dir: output::Direction,
    },

    /// Window layout helpers
    Layout {
        #[command(subcommand)]
        cmd: LayoutCmd,
    },

    /// Connect to i3 and handle keyevents for i3-valet
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

    let cmd_res = match app.cmd {
        Sub::Listen => listener(),
        _ => {
            let mut conn = I3Connection::connect().expect("i3connect");
            app.cmd.dispatch(&mut conn)
        }
    };

    if let Err(e) = cmd_res {
        eprintln!("Error running command: {}", e);
        std::process::exit(1);
    }
    println!("Goodbye");
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let mut i3 = I3::connect().await?;
    // this type can be inferred, here is written explicitly:
    //let worksp: reply::Workspaces = i3.get_workspaces().await?;
    //println!("{:#?}", worksp);

    let resp = i3.subscribe([Subscribe::Binding]).await?;

    println!("{:#?}", resp);
    let mut listener = i3.listen();
    while let Some(event) = listener.next().await {
        match event? {
            Event::Workspace(ev) => println!("workspace change event {:?}", ev),
            Event::Window(ev) => println!("window event {:?}", ev),
            Event::Output(ev) => println!("output event {:?}", ev),
            Event::Mode(ev) => println!("mode event {:?}", ev),
            Event::BarConfig(ev) => println!("bar config update {:?}", ev),
            Event::Binding(ev) => println!("binding event {:?}", ev),
            Event::Shutdown(ev) => println!("shutdown event {:?}", ev),
            Event::Tick(ev) => println!("tick event {:?}", ev),
        }
    }
    Ok(())
}
