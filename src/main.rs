use anyhow::Context;
use tokio_i3ipc::{
    event::{BindingData, Event, Subscribe},
    I3,
};
use tokio_stream::StreamExt;

use clap::{Parser, Subcommand, ValueEnum};
use log::*;

pub mod collapse;
pub mod ext;
pub mod floats;
pub mod info;
pub mod manage;
pub mod output;
pub mod workspace;

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

impl LogLevel {
    fn to_filter(self) -> log::LevelFilter {
        use LogLevel::*;
        match self {
            Trace => LevelFilter::Trace,
            Debug => LevelFilter::Debug,
            Info => LevelFilter::Info,
            Warn => LevelFilter::Warn,
            Error => LevelFilter::Error,
            Off => LevelFilter::Off,
        }
    }
}

#[derive(Subcommand, Debug)]
enum LayoutCmd {
    /// Set and focus (etc) a main window and auxilliary windows
    Main { action: manage::LayoutAction },
}

#[derive(Subcommand, Debug)]
enum Action {
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
}

#[derive(Subcommand, Debug)]
enum RunType {
    /// process keybinding events for i3-valet actions to take
    Listen,
    /// run a specific action.
    Run {
        /// dry-run will just print the i3 commands to stdout rather than send them to i3 over the
        /// socket
        #[arg(long)]
        dry_run: bool,

        /// The action to run
        #[command(subcommand)]
        action: Action,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct App {
    /// log level
    #[arg(long, default_value = "off")]
    log: LogLevel,
    #[command(subcommand)]
    how: RunType,
}

#[derive(Parser, Debug)]
#[command(no_binary_name(true), author, version, about, long_about = None)]
struct ReceivedCmd {
    #[command(subcommand)]
    action: Action,
}

impl Action {
    async fn dispatch(&self, conn: &mut I3) -> anyhow::Result<Vec<String>> {
        info!("Dispatching: {:?}", self);
        let cmds = match self {
            Action::Fix => {
                let tree = conn.get_tree().await.context("Get tree for Fix")?;
                collapse::clean_current_workspace(&tree)?
            }
            Action::Loc { pos, how } => {
                let tree = conn.get_tree().await.context("Get tree for Loc")?;
                floats::teleport_float(&tree, *pos, *how)?
            }
            Action::Print { target } => {
                let tree = conn.get_tree().await.context("Get tree for Print")?;
                info::run(*target, &tree).map(|_| vec![])?
            }
            Action::Workspace { target } => {
                let mut workspaces = conn
                    .get_workspaces()
                    .await
                    .context("Get workspaces for Workspace")?;
                workspace::run(*target, &mut workspaces)
            }
            Action::Output { change, dir } => {
                let workspaces = conn
                    .get_workspaces()
                    .await
                    .context("Get workspaces for Ouput")?;

                let outputs = conn.get_outputs().await.context("Get outputs for Output")?;
                output::run(*change, *dir, &workspaces, &outputs)?
            }
            Action::Layout { cmd } => match cmd {
                LayoutCmd::Main { action } => {
                    let tree = conn.get_tree().await.context("Get tree for Layout")?;
                    manage::run_main(*action, &tree)?
                }
            },
        };
        Ok(cmds)

   }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let app = App::parse();

    env_logger::builder()
        .filter_level(app.log.to_filter())
        .init();

    info!("Welcome to i3-valet");

    match app.how {
        RunType::Listen => {
            if let Err(e) = listener().await {
                error!("Fatal error running command: {}", e);
                std::process::exit(1);
            }
        }
        RunType::Run{dry_run, action} => {
            let mut conn = I3::connect().await.expect("i3connect");
            match action.dispatch(&mut conn).await {
                Ok(cmds)=> {
                    if dry_run{
                        for cmd in cmds {
                            println!("{}", cmd);
                        }
                    } else {
                        if let Err(e) = ext::i3_command(cmds, &mut conn).await{
                            eprintln!("Fatal error dispatching command: {:#}", e);
                        }
                    }
                },
                Err(e) =>{
                    eprintln!("Fatal error running command: {:#}", e);
                    std::process::exit(1);
                }
            }
        }
    };

    info!("Exitinging i3-valet");
    Ok(())
}

fn parse_command_string(action: &str) -> anyhow::Result<Option<ReceivedCmd>> {
    debug!("parsing command: {}", action);
    let mut args = action.split_whitespace();
    Ok(if let Some("nop") = args.next() {
        ReceivedCmd::try_parse_from(args).map(Some)?
    } else {
        None
    })
}

async fn handle_binding_event(e: BindingData) {
    trace!("Binding event: {:?}", e);
    for subcmd in e.binding.command.split(';') {
        match parse_command_string(subcmd) {
            Ok(Some(cmd)) => {
                let mut conn = match I3::connect().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        log::error!("couldn't connect while handling binding: {}", e);
                        return;
                    }
                };
                match cmd.action.dispatch(&mut conn).await {
                    Ok(cmds) => if let Err(e)=ext::i3_command(cmds, &mut conn).await{
                        warn!("Error running action '{}': {:#}", subcmd, e);
                    },
                    Err(e) => warn!("Error dispatching action '{}': {:#}", subcmd, e),
                };
            }
            Ok(None) => {
                debug!("Skipping non-i3-valet action: {}", subcmd);
            }

            Err(e) => {
                warn!("Error parsing action '{}': {:#}", subcmd, e);
            }
        };
        debug!("Action completed: {}", subcmd);
    }
}

async fn listener() -> anyhow::Result<()> {
    let mut i3 = I3::connect().await.context("init listener")?;

    i3.subscribe([Subscribe::Binding])
        .await
        .context("couldn't subscribe")?;

    let mut listener = i3.listen();
    while let Some(event) = listener.next().await {
        let evt = event.context("Connection died, i3 is most likey termnating")?;
        if let Event::Binding(ev) = evt {
            tokio::spawn(async { handle_binding_event(ev).await });
        }
    }
    Ok(())
}
