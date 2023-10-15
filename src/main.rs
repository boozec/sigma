mod trace;

use crate::trace::{exec, trace};
use clap::Parser;
use fork::{fork, Fork};
use nix::unistd::Pid;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Command to execute from ptrace
    command: String,
    /// Write the output to a file instead of the standard output
    #[arg(short = 'f', long = "file")]
    file_to_print: Option<String>,
}

/// Create a fork of the program and execute the process in the child. Parent gets the pid
/// value and trace it.
fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let pid = match fork() {
        Ok(Fork::Child) => return exec(&args.command),
        Ok(Fork::Parent(child)) => Pid::from_raw(child as i32),
        Err(err) => panic!("fork() failed: {err}"),
    };

    trace(pid, args.file_to_print)?;

    Ok(())
}
