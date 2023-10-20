use crate::cli::Args;
use crate::registers::RegistersData;
use byteorder::{LittleEndian, WriteBytesExt};
use nix::{
    sys::{
        ptrace,
        signal::Signal,
        wait::{waitpid, WaitStatus},
    },
    unistd::Pid,
};
use std::{
    fs::File,
    io::{self, Write},
    os::{raw::c_void, unix::process::CommandExt},
    process::{Command, Stdio},
    str,
};

/// Exec the `command` value tracing it with `ptrace` lib
pub fn exec(command: &str) -> anyhow::Result<()> {
    let params: Vec<&str> = command.split(' ').collect();

    let mut command = Command::new(params[0]);
    command.args(params[1..].iter());
    command.stdout(Stdio::null());

    unsafe {
        command.pre_exec(|| ptrace::traceme().map_err(|e| e.into()));
    }

    command.exec();

    Ok(())
}

/// Attach a ptrace status to a `pid`
pub fn attach(pid: Pid) -> anyhow::Result<()> {
    ptrace::attach(pid)?;

    Ok(())
}

/// Trace a process with `pid` ID and returns a list of `RegistersData`
pub fn trace(pid: Pid, args: &Args) -> anyhow::Result<Vec<RegistersData>> {
    // First wait for the parent process
    _ = waitpid(pid, None)?;

    // FIXME: file writing on attachment
    // If `file_to_print` is not None, create a new file with that value for redirecting all the
    // output (also in stdout)
    let mut f = None;
    if let Some(filename) = &args.file_to_print {
        f = Some(File::create(filename)?);
    }

    let mut lines: Vec<RegistersData> = Vec::new();

    // Since you have to do 2 syscalls (start and end) you have to alternate the print value,
    // because it could be equals except for the `rax` register.
    let mut have_to_print = true;

    let filters: Vec<&str> = match &args.filter {
        Some(filter) => filter.split(',').collect::<Vec<&str>>(),
        None => vec![],
    };
    while let Some(reg) = trace_next(pid)? {
        have_to_print ^= true;
        if have_to_print {
            if !filters.is_empty() && !filters.contains(&reg.name()) {
                continue;
            }

            if let Some(ref mut f) = f {
                writeln!(f, "{}", reg.output(pid))?;
            }

            if args.no_tui {
                writeln!(io::stdout(), "{}", reg.output(pid))?;
            }

            lines.push(reg);
        }
    }
    Ok(lines)
}

/// Read memory and returns a string.
/// Thank you https://github.com/JakWai01/lurk/blob/e3a3d6c026bbf818fe1329f8d458be544c3c5ebc/src/arch/mod.rs#L66
pub fn read_memory(pid: Pid, address: u64) -> String {
    let mut string = String::new();

    let mut count = 0;
    let word_size = 8;

    loop {
        let address = unsafe { (address as *mut c_void).offset(count) };

        match ptrace::read(pid, address) {
            Ok(read) => {
                let mut bytes: Vec<u8> = vec![];
                bytes.write_i64::<LittleEndian>(read).unwrap_or_else(|err| {
                    panic!("Failed to write {read} as i64 LittleEndian: {err}");
                });

                if !bytes
                    .iter()
                    .filter(|&b| *b == 0x0)
                    .collect::<Vec<&u8>>()
                    .is_empty()
                {
                    break;
                }

                bytes.iter().for_each(|b| {
                    string.push(*b as char);
                });

                count += word_size;
            }
            Err(_) => break,
        };
    }

    if string.len() > 24 {
        string = string[..24].to_string();
        string.push_str("...");
    }

    string = string.replace('\n', "\\n");

    format!("\"{string}\"")
}

/// Get the next step for a ptrace process
pub fn trace_next(pid: Pid) -> anyhow::Result<Option<RegistersData>> {
    ptrace::syscall(pid, None)?;
    let status = waitpid(pid, None).unwrap();

    match status {
        // Match the stopped value for a process
        WaitStatus::Stopped(pid, signal) => {
            match signal {
                Signal::SIGTRAP => {
                    let reg = RegistersData::new(ptrace::getregs(pid)?);
                    return Ok(Some(reg));
                }
                _ => {}
            };
        }
        _ => {}
    };

    Ok(None)
}

/// Kill a process traced by ptrace
pub fn trace_kill(pid: Pid) -> anyhow::Result<()> {
    let _ = ptrace::kill(pid);
    Ok(())
}
