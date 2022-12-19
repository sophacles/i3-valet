use log::*;

//use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use clap::{Parser, Subcommand};
use i3ipc::{
    event::{BindingEventInfo, Event},
    I3Connection, I3EventListener, Subscription,
};

use i3_valet::{collapse, floats, info, manage, output, workspace};

fn handle_binding_event(e: BindingEventInfo, conn: &mut I3Connection) -> Result<(), String> {
    debug!("Saw BindingEvent: {:#?}", e);
    // TODO: this is certainly fragile
    println!("Caught command: {}", e.binding.command);
    for subcmd in e.binding.command.split(';') {
        print!("Processing: {} ... ", subcmd);
        let mut args: Vec<&str> = subcmd.split_whitespace().collect();
        if args.is_empty() {
            println!("Skipping!");
            continue;
        }
        match args.remove(0) {
            "nop" => {
                println!("Handling!");
                let app = ReceivedCmd::try_parse_from(args)
                    .map_err(|e| format!("Error parsing command: {:?}", e))?;
                app.cmd.dispatch(conn)?
            }
            _ => {
                println!("Skipping!");
                continue;
            }
        }
    }
    Ok(())
}

fn listener(command_conn: &mut I3Connection) -> Result<(), String> {
    let mut listener = I3EventListener::connect().unwrap();

    let subs = [Subscription::Binding];
    listener.subscribe(&subs).unwrap();

    for evt in listener.listen() {
        if let Err(res) = match evt.map_err(|_| "Connection died, i3 is most likey termnating")? {
            Event::BindingEvent(e) => handle_binding_event(e, command_conn),
            _ => unreachable!("{}", "Can't happen, but here we are"),
        } {
            warn!("Encountered Error in listener: {}", res);
        }
    }
    Ok(())
}

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
        match self {
            Sub::Fix => collapse::clean_current_workspace(conn),
            Sub::Listen => Err("Cannot dispatch listen: cli command only.".to_string()),
            Sub::Loc { pos, how } => floats::teleport_float(conn, *pos, *how),
            Sub::Print { target } => info::run(*target, conn),
            Sub::Workspace { target } => workspace::run(*target, conn),
            Sub::Output { change, dir } => output::run(*change, *dir, conn),
            Sub::Layout { cmd } => match cmd {
                LayoutCmd::Main { action } => manage::run_main(*action, conn),
            },
        }
    }
}

fn main() {
    env_logger::init();

    let mut conn = I3Connection::connect().expect("i3connect");
    let app = App::parse();

    if let Err(res) = match app.cmd {
        Sub::Listen => listener(&mut conn),
        _ => app.cmd.dispatch(&mut conn),
    } {
        eprintln!("Error running command: {}", res);
        std::process::exit(1);
    }
    println!("Goodbye?!");
}
