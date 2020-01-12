extern crate env_logger;
extern crate i3ipc;

#[macro_use]
extern crate log;

use std::env;
//use std::thread;

//use i3ipc::event::inner::{Binding, WindowChange};
use i3ipc::event::Event;
//use i3ipc::reply;
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
                    dispatch(args, command_conn);
                }
            }
            _ => unreachable!("Can't happen, but here we are"),
        }
    }
}

fn do_fix(conn: &mut I3Connection) {
    let mut fmt = info::StepFormatter::new();
    let fmt = fmt
        .set("id")
        .set("depth")
        .set("name")
        .set("layout")
        .set("marks");

    info::print_ws(conn, fmt);
    info!("----------------------------------------------------------");
    info!("Cleaning!");
    if let Ok(n) = clean_current_workspace(conn) {
        info!("DID {} things!", n);
    }
    info!("----------------------------------------------------------");

    info::print_ws(conn, fmt);
}

fn do_move(conn: &mut I3Connection, arg: String, honor_bar: bool) {
    teleport_float(conn, arg.parse().unwrap(), honor_bar);
}

fn dispatch(mut args: Vec<String>, conn: &mut I3Connection) {
    let cmd = args.remove(0);
    match cmd.as_str() {
        "fix" => do_fix(conn),
        "loc" => match args[0].as_str() {
            "abs" => do_move(conn, args[1].to_owned(), false),
            "rel" => do_move(conn, args[1].to_owned(), true),
            _ => do_move(conn, args[0].to_owned(), true),
        },
        "print" => match args[0].as_str() {
            "tree" => info::print_ws(conn, &info::STD),
            "rects" => info::print_disp(conn, &info::RECT),
            "window" => info::print_window(conn, &info::WINDOW),
            _ => warn!("BAD INPUT: {} {:?}", cmd, args),
        },
        "EXP" => listenery_shit(conn),
        _ => warn!("BAD INPUT: {} {:?}", cmd, args),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let args = match args.len() <= 1 {
        true => vec!["print".to_string(), "tree".to_string()],
        false => args[1..].to_vec(),
    };

    env_logger::init();

    let mut conn = I3Connection::connect().expect("i3connect");
    dispatch(args, &mut conn);
}
