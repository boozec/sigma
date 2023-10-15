use nix::{
    sys::{
        ptrace,
        signal::Signal,
        wait::{waitpid, WaitStatus},
    },
    unistd::Pid,
};
use std::{os::unix::process::CommandExt, process::Command};

pub fn exec(command: &mut Command) -> anyhow::Result<Pid> {
    unsafe {
        command.pre_exec(|| ptrace::traceme().map_err(|e| e.into()));
    }
    let child = command.spawn()?;
    Ok(Pid::from_raw(child.id() as i32))
}

pub fn trace(pid: Pid) -> anyhow::Result<()> {
    let mut have_to_print = true;
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
