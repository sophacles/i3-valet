use log::*;

//use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use clap::{Parser, Subcommand, ValueEnum};
use i3ipc::{
    event::{BindingEventInfo, Event},
    I3Connection, I3EventListener, Subscription,
};

use i3_valet::{
    collapse::clean_current_workspace,
    floats::{teleport_float, Loc, Positioning},
    info,
    manage::{focus_main, make_main, swap_main},
    output,
    workspace::{alloc_workspace, move_to_new_ws},
};

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

#[derive(ValueEnum, Clone, Debug)]
enum OutputArgs {
    Next,
    Prev,
}

#[derive(Subcommand, Debug)]
enum OutputCmd {
    /// move workspace to a different output
    MoveWs { arg: OutputArgs },
    /// move workspace to a different output
    MoveWin { arg: OutputArgs },
    /// focus a different output
    Focus { arg: OutputArgs },
}

#[derive(ValueEnum, Clone, Debug)]
enum PrintTarget {
    Tree,
    Rects,
    Window,
}

#[derive(ValueEnum, Clone, Debug)]
enum WorkspaceTarget {
    Alloc,
    MoveNew,
}

#[derive(ValueEnum, Clone, Debug)]
enum LayoutAction {
    Set,
    Swap,
    Focus,
}

#[derive(Subcommand, Debug)]
enum LayoutCmd {
    Main { action: LayoutAction },
}

#[derive(Subcommand, Debug)]
enum Sub {
    /// clean up the window tree
    Fix,

    /// Move A floating window to anchor point
    Loc {
        /// Positioning of window.
        how: Positioning,
        /// Anchor point to position window
        pos: Loc,
    },

    ///Print window information
    Print {
        /// what to print
        target: PrintTarget,
    },

    /// Workspace commands
    Workspace {
        /// what to do
        target: WorkspaceTarget,
    },

    /// Output commands
    Output {
        #[command(subcommand)]
        cmd: OutputCmd,
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
            Sub::Fix => clean_current_workspace(conn),
            Sub::Listen => Err("Cannot dispatch listen: cli command only.".to_string()),
            Sub::Loc { pos, how } => teleport_float(conn, *pos, *how),
            Sub::Print { target } => match target {
                PrintTarget::Tree => info::print_ws(conn, &info::STD),
                PrintTarget::Rects => info::print_disp(conn, &info::RECT),
                PrintTarget::Window => info::print_window(conn, &info::WINDOW),
            },
            Sub::Workspace { target } => match target {
                WorkspaceTarget::Alloc => alloc_workspace(conn),
                WorkspaceTarget::MoveNew => move_to_new_ws(conn),
            },
            Sub::Output { cmd } => match cmd {
                OutputCmd::Focus { arg } => match arg {
                    OutputArgs::Next => output::focus_next(conn),
                    OutputArgs::Prev => output::focus_prev(conn),
                },
                OutputCmd::MoveWs { arg } => match arg {
                    OutputArgs::Next => output::workspace_to_next(conn),
                    OutputArgs::Prev => output::workspace_to_prev(conn),
                },
                OutputCmd::MoveWin { arg } => match arg {
                    OutputArgs::Next => output::window_to_next(conn),
                    OutputArgs::Prev => output::window_to_prev(conn),
                },
            },
            Sub::Layout { cmd } => match cmd {
                LayoutCmd::Main { action } => match action {
                    LayoutAction::Set => make_main(conn),
                    LayoutAction::Swap => swap_main(conn),
                    LayoutAction::Focus => focus_main(conn),
                },
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
