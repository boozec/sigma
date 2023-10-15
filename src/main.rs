mod trace;

use crate::trace::{exec, trace};
use clap::Parser;
use std::process::Command;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    command: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let params = args.command.split(' ').collect::<Vec<&str>>();

    let mut command = Command::new(params[0]);
    if params.len() > 1 {
        for arg in &params[1..] {
            command.arg(arg);
        }
    }
    let pid = exec(&mut command)?;
    trace(pid)?;

    Ok(())
}
