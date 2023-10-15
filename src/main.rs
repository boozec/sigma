mod trace;

use crate::trace::{exec, trace};
use clap::Parser;
use fork::{fork, Fork};
use nix::unistd::Pid;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    command: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let pid = match fork() {
        Ok(Fork::Child) => return exec(&args.command),
        Ok(Fork::Parent(child)) => Pid::from_raw(child as i32),
        Err(err) => panic!("fork() failed: {err}"),
    };

    trace(pid)?;

    Ok(())
}
