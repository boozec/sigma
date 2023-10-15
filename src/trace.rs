use nix::{
    sys::{
        ptrace,
        signal::Signal,
        wait::{waitpid, WaitStatus},
    },
    unistd::Pid,
};
use std::{fs::File, io::Write, os::unix::process::CommandExt, process::Command};

/// Exec the `command` value tracing it with `ptrace` lib
pub fn exec(command: &str) -> anyhow::Result<()> {
    let params: Vec<&str> = command.split(' ').collect();

    let mut command = Command::new(params[0]);
    command.args(params[1..].iter());

    unsafe {
        command.pre_exec(|| ptrace::traceme().map_err(|e| e.into()));
    }

    command.exec();

    Ok(())
}

/// Trace a process with `pid` ID
pub fn trace(pid: Pid, file_to_print: Option<String>) -> anyhow::Result<Vec<u8>> {
    // Since you have to do 2 syscalls (start and end) you have to alternate the print value,
    // because it could be equals except for the `rax` register.
    let mut have_to_print = true;

    // First wait for the parent process
    _ = waitpid(pid, None)?;

    // If `fiole_to_print` is not None, create a new file with that value for redirecting all the
    // output (also in stdout)
    let mut f = None;
    if let Some(filename) = file_to_print {
        f = Some(File::create(filename)?);
    }

    let mut lines = Vec::new();

    loop {
        have_to_print ^= true;
        ptrace::syscall(pid, None)?;
        let status = waitpid(pid, None)?;

        match status {
            // Break the loop if the process exists
            WaitStatus::Exited(_pid, _) => {
                break;
            }
            // Match the stopped value for a process
            WaitStatus::Stopped(pid, signal) => {
                match signal {
                    Signal::SIGTRAP => {
                        let regs = ptrace::getregs(pid)?;
                        if have_to_print {
                            let output = format!(
                                "{}({:x}, {:x}, {:x}, ...) = {:x}",
                                regs.orig_rax, regs.rdi, regs.rsi, regs.rdx, regs.rax
                            );
                            writeln!(lines, "{output}")?;

                            if let Some(ref mut f) = f {
                                writeln!(f, "{output}")?;
                            }
                        }
                    }
                    _ => {}
                };
            }
            _ => {}
        };
    }

    Ok(lines)
}
