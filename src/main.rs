mod arch;
mod cli;
mod registers;
mod trace;
mod ui;

use crate::cli::Args;
use crate::trace::{exec, trace};
use crate::ui::UI;

use clap::Parser;
use fork::{fork, Fork};
use nix::unistd::Pid;
use owo_colors::OwoColorize;
use trace::attach;

/// Create a fork of the program and execute the process in the child. Parent gets the pid
/// value and trace it.
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let process: Result<Pid, String> = if args.command.is_some() {
        match fork() {
            Ok(Fork::Child) => return exec(&args.command.unwrap()),
            Ok(Fork::Parent(child)) => Ok(Pid::from_raw(child)),
            Err(err) => Err(format!("fork() failed: {err}")),
        }
    } else if args.attach.is_some() {
        let pid = Pid::from_raw(args.attach.unwrap());

        if attach(pid).is_ok() {
            Ok(pid)
        } else {
            Err(format!("Unable to attach to process `{pid}`"))
        }
    } else {
        Err(format!("You must define a command or a PID to attach"))
    };

    match process {
        Ok(pid) => {
            if !args.no_tui {
                let mut ui = UI::new();

                ui.start(pid, &args)?;
            } else {
                trace(pid, &args)?;
            }
        }
        Err(e) => {
            eprintln!("{}", e.red());
        }
    };

    Ok(())
}
