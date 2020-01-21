#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate i3ipc;

#[macro_use]
extern crate log;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use i3ipc::{
    event::{BindingEventInfo, Event},
    I3Connection, I3EventListener, Subscription,
};

use i3_valet::{
    collapse::clean_current_workspace,
    floats::{teleport_float, Loc, Positioning},
    info, output,
    workspace::alloc_workspsace,
};

fn handle_binding_event(e: BindingEventInfo, conn: &mut I3Connection) -> Result<(), String> {
    debug!("Saw BindingEvent: {:#?}", e);
    // TODO: make this just do multiple things
    let mut args: Vec<&str> = e.binding.command.split_whitespace().collect();
    match args.remove(0) {
        "nop" => {
            let cl = args.join(" ");
            let m = make_args()
                .setting(AppSettings::NoBinaryName)
                .get_matches_from_safe(args.iter().take_while(|x| **x != ";"))
                .map_err(|e| format!("Cannot parse: {} => {}", cl, e.message))?;
            dispatch(m, conn)
        }
        _ => Ok(()),
    }
}

fn listener(command_conn: &mut I3Connection) -> Result<(), String> {
    let mut listener = I3EventListener::connect().unwrap();

    let subs = [Subscription::Binding];
    listener.subscribe(&subs).unwrap();

    for evt in listener.listen() {
        if let Err(res) = match evt.map_err(|_| "Connection died, i3 is most likey termnating")? {
            Event::BindingEvent(e) => handle_binding_event(e, command_conn),
            _ => unreachable!("Can't happen, but here we are"),
        } {
            warn!("Encountered Error in listener: {}", res);
        }
    }
    Ok(())
}

fn make_args<'a, 'b>() -> App<'a, 'b> {
    App::new("i3-valet")
        .version("0.1")
        .author("sophacles@gmail.com")
        .about("tend to your windows")
        .subcommand(SubCommand::with_name("fix").about("clean up the window tree"))
        .subcommand(
            SubCommand::with_name("loc")
            .about("Move a floating window to anchor point")
                .arg(
                    Arg::with_name("how")
                        .help("Positioning of window.\n'abs' is relative to the output\n'rel' is relative to the content area\n")
                        .index(1)
                        .required(true)
                        .possible_values(&["abs", "rel"]),
                )
                .arg(
                    Arg::with_name("where")
                        .help("Anchor point to position window\n")
                        .index(2)
                        .required(true)
                        .possible_values(&["nw", "ne", "sw", "se", "bot", "top", "left", "right"]),
                ),
        )
        .subcommand(
            SubCommand::with_name("print")
            .about("Print window information")
            .arg(
                Arg::with_name("target")
                    .help("what to print")
                    .index(1)
                    .required(true)
                    .possible_values(&["tree", "rects", "window"]),
            ),
        )
        .subcommand(
            SubCommand::with_name("workspace")
            .about("Workspace commands")
            .arg(
                // TODO: replace me with subsubcommands
                Arg::with_name("target")
                    .help("what do do")
                    .index(1)
                    .required(true)
                    .possible_values(&["alloc"]),
            ),
        )
        .subcommand(
            SubCommand::with_name("output")
            .about("Output commands")
            // TODO: replace me with subsubcommands
            .arg(
                Arg::with_name("cmd")
                    .help("where to go")
                    .index(1)
                    .required(true)
                    .possible_values(&["focus", "move-ws"]),
                    )
            .arg(
                Arg::with_name("target")
                    .help("where to go")
                    .index(2)
                    .required(true)
                    .possible_values(&["next", "prev"]),
            ),
        )
        .subcommand(SubCommand::with_name("listen").about("connect to i3 socket and wait for events"))
}

// TODO: make this happy with all the options and stuff
fn dispatch(m: ArgMatches, conn: &mut I3Connection) -> Result<(), String> {
    match m.subcommand_name() {
        Some("fix") => clean_current_workspace(conn),
        Some("loc") => {
            let m = m.subcommand.unwrap().matches;
            teleport_float(
                conn,
                value_t!(m.value_of("where"), Loc).expect("possible values broke!"),
                value_t!(m.value_of("how"), Positioning).expect("possible_values broke!"),
            )
        }
        Some("print") => {
            let m = m.subcommand.unwrap().matches;
            match m.value_of("target").unwrap() {
                "tree" => info::print_ws(conn, &info::STD),
                "rects" => info::print_disp(conn, &info::RECT),
                "window" => info::print_window(conn, &info::WINDOW),
                _ => unreachable!("stupid possible_values failed"),
            }
        }
        Some("workspace") => {
            let m = m.subcommand.unwrap().matches;
            match m.value_of("target").unwrap() {
                "alloc" => alloc_workspsace(conn),
                _ => unreachable!("stupid possible_values failed"),
            }
        }
        Some("output") => {
            let m = m.subcommand.unwrap().matches;
            match m.value_of("cmd").unwrap() {
                "focus" => match m.value_of("target").unwrap() {
                    "next" => output::focus_next(conn),
                    "prev" => output::focus_prev(conn),
                    _ => unreachable!("stupid possible_values failed"),
                },
                "move-ws" => match m.value_of("target").unwrap() {
                    "next" => output::workspace_to_next(conn),
                    "prev" => output::workspace_to_prev(conn),
                    _ => unreachable!("stupid possible_values failed"),
                },
                _ => unreachable!("stupid possible_values failed"),
            }
        }
        Some("listen") => Err(format!("Cannot dispatch listen: cli command only.")),
        None => info::print_ws(conn, &info::STD),
        Some(f) => Err(format!("Unknown command: {}", f)),
    }
}

fn main() {
    env_logger::init();

    let mut conn = I3Connection::connect().expect("i3connect");

    let app = make_args();
    let parsed = app.get_matches();
    if let Err(res) = match parsed.subcommand_name() {
        Some("listen") => listener(&mut conn),
        _ => dispatch(parsed, &mut conn),
    } {
        eprint!("Error running command: {}\n", res);
        std::process::exit(1);
    }
    println!("Goodbye?!");
}
