use i3ipc::{reply::Node, I3Connection};

use crate::{
    ext::{NodeSearch, Step},
    info,
};

pub fn is_shaped_right(conn: &mut I3Connection) -> Result<(), String> {
    println!("Hello manage!");
    info::print_ws(conn, &info::STD)?;
    let node = conn.get_tree().expect("get_tree 1");
    let ws = node.get_current_workspace().expect("workspace 2");
    let mut fmt = info::StepFormatter::new();
    let fmt = fmt.show_indent(false).set("id");

    for n in ws.preorder() {
        print!("{} ", fmt.format(&n));
    }
    print!("\n");
    for n in ws.postorder() {
        print!("{} ", fmt.format(&n));
    }
    print!("\n");
    //for (pre, post) in ws.preorder().zip(ws.postorder()) { }
    Ok(())
}
