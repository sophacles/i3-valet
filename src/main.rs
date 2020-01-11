extern crate env_logger;
extern crate i3ipc;

#[macro_use]
extern crate log;

use std::env;
//use std::thread;

//use i3ipc::event::inner::{Binding, WindowChange};
use i3ipc::event::Event;
//use i3ipc::reply;
use i3ipc::reply::Node;
use i3ipc::{I3Connection, I3EventListener, Subscription};

use i3_valet::collapse::clean_current_workspace;
use i3_valet::floats::teleport_float;
use i3_valet::NodeSearch;

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

fn node_str(node: &Node) -> String {
    format!(
        "Node: {:.1}({}) \"{:.10}\" ({:?}) -- {:?}",
        node.focused,
        node.id,
        node.name.as_ref().unwrap_or(&String::from("None")),
        node.layout,
        node.marks,
    )
}

fn print_ws(conn: &mut I3Connection) {
    let node = conn.get_tree().expect("get_tree 1");
    let ws = node.get_current_workspace().expect("workspace 2");
    ws.pretty_print(node_str);
}

fn print_disp(conn: &mut I3Connection) {
    let node = conn.get_tree().expect("get_tree 1");
    let ws = node.get_current_output().expect("workspace 2");
    ws.pretty_print(node_str);
}

fn do_fix(conn: &mut I3Connection) {
    print_ws(conn);
    info!("----------------------------------------------------------");
    info!("Cleaning!");
    if let Ok(n) = clean_current_workspace(conn) {
        info!("DID {} things!", n);
    }
    info!("----------------------------------------------------------");

    print_ws(conn);
}

fn do_move(conn: &mut I3Connection, arg: String, honor_bar: bool) {
    teleport_float(conn, arg.parse().unwrap(), honor_bar);
}

fn dispatch(mut args: Vec<String>, conn: &mut I3Connection) {
    let cmd = args.remove(0);
    match cmd.as_str() {
        "fix" => do_fix(conn),
        "loc" => do_move(conn, args[0].to_owned(), false),
        "loc_bar" => do_move(conn, args[0].to_owned(), true),
        "EXP" => listenery_shit(conn),
        _ => warn!("BAD INPUT: {} {:?}", cmd, args),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let print: bool = args.len() <= 1;

    env_logger::init();

    let mut conn = I3Connection::connect().expect("i3connect");
    if print {
        print_disp(&mut conn);
        return;
    }
    dispatch(args[1..].to_vec(), &mut conn);
}
