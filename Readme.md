# i3-valet

A collection of tools for i3 that assist in window, workspace and output
operations. `i3-valet` can be run directly from the command line or as an i3
sidekick that listens for keybinding events over ipc.

## Building & Installation

`i3-valet` is written in Rust and has only been tested against the latest
stable version. It can be build with `cargo build --release` and the binary can
be placed somehere in `$PATH`.

(note: this is still a newly public tool, if there's interest I'm glad to work
with folks on better distribution)

## Using i3-valet

`i3-valet` operates by running commands (called Actions). These actions will
connect to i3-ipc and query information and then send commands back to i3.
Whether run via keybinding in `listen` mode or from the cli, the actions have
consistent names and arguments so you can test out what an action will do
without having to muck with your i3config.

The available actions are:
```
  fix        clean up the window tree
  loc        Move A floating window to anchor point
  print      Print information about the current tree or window
  workspace  Workspace commands
  output     Movement between relative outputs
  layout     Window layout helpers
```

Information about actions and arguments can be found via the help subcommand, e.g.:

```
$ i3-valet help run output
Movement between relative outputs.

This assumes outputs are linear and cycles through them in order

Usage: i3-valet run output <CHANGE> <DIR>

Arguments:
  <CHANGE>
          Possible values:
          - move-ws:  move workspace to a different output
          - move-win: move workspace to a different output
          - focus:    focus a different output

  <DIR>
          [possible values: next, prev]

Options:
  -h, --help
          Print help information (use `-h` for a summary)
```

### CLI 

To run an action from the command line simply use `i3-valet run
<ACTION>`. For example to focus the next output:

```
$ i3-valet run output focus next
```

### Listen

Listen mode can be entered by running `i3-valet listen`. In this mode the
program will connect to i3 and subscribe to keybind events. When an event for
an i3-valet action arrives, it will do that action.  Listen mode will exit the
program if the connection to i3 is closed so it's recommended to run it from
i3-config like this:

```
exec_always --no-startup-id i3-valet listen
```

So that if i3 is restarted `i3-valet` will also restart.

To configure keybindings use the `nop` command followed by an action just like
the action on a command line. For example to configure a mode for moving floats
using the `loc` action:

```
mode "move" {
    bindsym u     nop loc rel nw
    bindsym i     nop loc rel top
    bindsym o     nop loc rel ne
    bindsym n     nop loc rel sw
    bindsym m     nop loc rel bot
    bindsym comma nop loc rel se
}
bindsym $mod+m mode "move"
```
**How it works**
The listen functionality takes advantage of the `nop` command in the i3 config
- this command tells i3 that anything after the `nop` is to be ignored. When i3
creates events for pressed keybinding, it puts the entire command into the
event anyway.

For example this binding in i3config:
```
bindsym $mod+t nop foo bar
```
will report `nop foo bar` as the command when `$mod+t` is pressed.


## Examples
TODO: Figure out how to record sessions highlighting various actions

## Contributing

Contributions welome! Just open a PR or Issue and we can hash it out.

Areas where some help would be greatly appreciated:

* Testing and fixing for sway
* Doc improvements
* New actions
