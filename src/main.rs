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
use trace::attach;

/// Create a fork of the program and execute the process in the child. Parent gets the pid
/// value and trace it.
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let pid = if args.command.is_some() {
        match fork() {
            Ok(Fork::Child) => return exec(&args.command.unwrap()),
            Ok(Fork::Parent(child)) => Pid::from_raw(child),
            Err(err) => panic!("fork() failed: {err}"),
        }
    } else if args.attach.is_some() {
        let pid = Pid::from_raw(args.attach.unwrap());

        if attach(pid).is_ok() {
            pid
        } else {
            panic!("Unable to attach to process {pid}");
        }
    } else {
        panic!("You must define a command or a PID to attach");
    };

    if !args.no_tui {
        let mut ui = UI::new();

        ui.start(pid, &args)?;
    } else {
        trace(pid, &args)?;
    }

    Ok(())
}
