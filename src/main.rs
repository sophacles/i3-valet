extern crate clap;
extern crate env_logger;
extern crate i3ipc;

#[macro_use]
extern crate log;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use i3ipc::event::Event;
use i3ipc::{I3Connection, I3EventListener, Subscription};

use i3_valet::collapse::clean_current_workspace;
use i3_valet::floats::teleport_float;
use i3_valet::info;

fn listenery_shit(command_conn: &mut I3Connection) {
    let mut listener = I3EventListener::connect().unwrap();

    let subs = [Subscription::Binding, Subscription::Window];
    listener.subscribe(&subs).unwrap();

    for evt in listener.listen() {
        match evt.unwrap() {
            Event::WindowEvent(e) => {
                debug!("Saw WindowEvent: {:#?}", e);
            }
            Event::BindingEvent(e) => {
                debug!("Saw BindingEvent: {:#?}", e);
                let mut args: Vec<String> = e
                    .binding
                    .command
                    .split_whitespace()
                    .map(|s| String::from(s))
                    .collect();
                if args.remove(0) == "nop" {
                    let cl = args.join(" ");
                    match make_args()
                        .setting(AppSettings::NoBinaryName)
                        .get_matches_from_safe(args)
                    {
                        Ok(m) => dispatch(m, command_conn),
                        Err(e) => warn!("Cannot parse: {}\n (error was: {:?})", cl, e),
                    }
                }
            }
            _ => unreachable!("Can't happen, but here we are"),
        }
    }
}

fn make_args<'a, 'b>() -> App<'a, 'b> {
    App::new("i3-valet")
        .version("0.1")
        .author("sophacles@gmail.com")
        .about("tools for i3 wm")
        .subcommand(SubCommand::with_name("fix"))
        .subcommand(
            SubCommand::with_name("loc")
                .arg(
                    Arg::with_name("how")
                        .index(1)
                        .required(true)
                        .possible_values(&["abs", "rel"]),
                )
                .arg(
                    Arg::with_name("where")
                        .index(2)
                        .required(true)
                        .possible_values(&["nw", "ne", "sw", "se", "bot", "top", "left", "right"]),
                ),
        )
        .subcommand(
            SubCommand::with_name("print").arg(
                Arg::with_name("target")
                    .index(1)
                    .required(true)
                    .possible_values(&["tree", "rects", "window"]),
            ),
        )
        .subcommand(SubCommand::with_name("listen"))
}

// TODO: make this happy with all the options and stuff
fn dispatch(m: ArgMatches, conn: &mut I3Connection) {
    match m.subcommand_name() {
        Some("fix") => {
            clean_current_workspace(conn);
        }
        Some("loc") => {
            let m = m.subcommand.unwrap().matches;
            teleport_float(
                conn,
                m.value_of("where").unwrap().to_string().parse().unwrap(),
                m.value_of("how").unwrap().to_string().parse().unwrap(),
            );
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
        Some("listen") => info!("Cannot dispatch listen: cli command only."),
        None => info::print_ws(conn, &info::STD),
        Some(f) => info!("Invalid command: {}", f),
    }
}

fn main() {
    env_logger::init();

    let mut conn = I3Connection::connect().expect("i3connect");
    //dispatch_old(args, &mut conn);

    let app = make_args();
    let parsed = app.get_matches();
    match parsed.subcommand_name() {
        Some("listen") => listenery_shit(&mut conn),
        _ => dispatch(parsed, &mut conn),
    }
}
