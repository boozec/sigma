use nix::{
    sys::{
        ptrace,
        signal::Signal,
        wait::{waitpid, WaitStatus},
    },
    unistd::Pid,
};
use std::{os::unix::process::CommandExt, process::Command};

pub fn exec(command: &String) -> anyhow::Result<()> {
    let params: Vec<&str> = command.split(' ').collect();

    let mut command = Command::new(params[0]);
    command.args(params[1..].iter());

    unsafe {
        command.pre_exec(|| ptrace::traceme().map_err(|e| e.into()));
    }

    command.exec();

    Ok(())
}

pub fn trace(pid: Pid) -> anyhow::Result<()> {
    let mut have_to_print = true;

    // First wait if for the parent process
    _ = waitpid(pid, None)?;

    loop {
        have_to_print ^= true;
        ptrace::syscall(pid, None)?;
        let status = waitpid(pid, None)?;

        match status {
            WaitStatus::Exited(_pid, _) => {
                break;
            }
            WaitStatus::Stopped(pid, signal) => {
                match signal {
                    Signal::SIGTRAP => {
                        let regs = ptrace::getregs(pid)?;
                        if have_to_print {
                            println!(
                                "{}({:x}, {:x}, {:x}, ...) = {:x}",
                                regs.orig_rax, regs.rdi, regs.rsi, regs.rdx, regs.rax,
                            );
                        }
                    }
                    _ => {}
                };
            }
            _ => {}
        };
    }

    Ok(())
}
