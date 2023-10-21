use chrono::{DateTime, Local};
use nix::{libc::user_regs_struct, unistd::Pid};
use owo_colors::OwoColorize;
use ratatui::{
    prelude::{Line, Span, Style},
    style::{Color, Modifier},
};

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
use crate::arch::linux::x86_64::*;
#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
use crate::arch::syscall_name;
use crate::trace::read_memory;

#[derive(Clone, Debug)]
/// Structure use to monitor what a register has for (argument: value)
struct RegisterOutput {
    /// Value for a register, by default is a number which could be a real value or a memory
    /// address
    value: String,
    /// Argument for a register, eg: "const char *buf"
    argument: &'static str,
}

impl RegisterOutput {
    fn new(address: u64, argument: &'static str) -> Self {
        Self {
            value: address.to_string(),
            argument,
        }
    }
}

/// Struct used to manipulate registers data from https://docs.rs/libc/0.2.147/libc/struct.user_regs_struct.html
#[derive(Debug)]
pub struct RegistersData {
    timestamp: DateTime<Local>,
    orig_rax: u64,
    rdi: RegisterOutput,
    rsi: RegisterOutput,
    rdx: RegisterOutput,
    r10: RegisterOutput,
    r8: RegisterOutput,
    r9: RegisterOutput,
    rax: u64,
}

impl RegistersData {
    /// Create new `RegistersData` from an `user_regs_struct`'C structure
    pub fn new(registers: user_regs_struct) -> RegistersData {
        let (rdi, rsi, rdx, r10, r8, r9) = (
            RegisterOutput::new(registers.rdi, rdi(registers.orig_rax)),
            RegisterOutput::new(registers.rsi, rsi(registers.orig_rax)),
            RegisterOutput::new(registers.rdx, rdx(registers.orig_rax)),
            RegisterOutput::new(registers.r10, r10(registers.orig_rax)),
            RegisterOutput::new(registers.r8, r8(registers.orig_rax)),
            RegisterOutput::new(registers.r9, r9(registers.orig_rax)),
        );

        RegistersData {
            timestamp: Local::now(),
            orig_rax: registers.orig_rax,
            rax: registers.rax,
            rdi,
            rsi,
            rdx,
            r10,
            r8,
            r9,
        }
    }

    /// Get date in ISO 8601 / RFC 3339 date & time string format
    pub fn date(&self) -> String {
        self.timestamp.format("%+").to_string()
    }

    /// Return the rax name as syscall name
    pub fn name(&self) -> &str {
        syscall_name(self.orig_rax)
    }

    /// Returns a good string which shows the output for a line
    pub fn output(&mut self, pid: Pid) -> String {
        let mut output = format!("[{}]: ", self.date());

        if !self.name().is_empty() {
            output.push_str(&format!("{}(", self.name().bold()));
        } else {
            output.push_str(&format!("{}(", self.orig_rax.yellow().bold()));
        }

        let mut has_reg = false;

        let mut regs = [
            &mut self.rdi,
            &mut self.rsi,
            &mut self.rdx,
            &mut self.r10,
            &mut self.r8,
            &mut self.r9,
        ];

        for reg in &mut regs {
            if !reg.argument.is_empty() {
                let output_reg = reg.argument.to_owned() + ":";
                reg.value = if (output_reg.starts_with("const char *")
                    || output_reg.starts_with("char *"))
                    && !reg.value.starts_with("\"")
                {
                    read_memory(pid, reg.value.parse::<u64>().unwrap())
                } else {
                    reg.value.to_string()
                };
                output.push_str(&format!("{} {}, ", output_reg.blue(), reg.value));
                has_reg = true;
            }
        }

        if has_reg {
            output.remove(output.len() - 1);
            output.remove(output.len() - 1);
        }

        output.push_str(&format!(") = 0x{:x}", self.rax)[..]);
        output
    }

    /// Returns a good line for TUI
    pub fn output_ui(&mut self, _pid: Pid) -> Line {
        let mut spans: Vec<Span> = vec![];
        spans.push(Span::raw(format!("[{}]: ", self.date())));
        if !self.name().is_empty() {
            spans.push(Span::styled(
                format!("{}(", self.name()),
                Style::default().add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!("{}(", self.orig_rax),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        let mut regs = [
            &mut self.rdi,
            &mut self.rsi,
            &mut self.rdx,
            &mut self.r10,
            &mut self.r8,
            &mut self.r9,
        ];

        for reg in &mut regs {
            if !reg.argument.is_empty() {
                let output_reg = reg.argument.to_owned() + ":";
                spans.push(Span::styled(
                    format!("{} ", output_reg),
                    Style::default().fg(Color::Blue),
                ));

                //  FIXME: read memory does not work
                // reg.value = if (output_reg.starts_with("const char *")
                //     || output_reg.starts_with("char *"))
                //     && !reg.value.starts_with("\"")
                // {
                //     read_memory(pid, reg.value.parse::<u64>().unwrap())
                // } else {
                //     reg.value.to_string()
                // };
                spans.push(Span::styled(format!("{}, ", reg.value), Style::default()));
            }
        }

        spans.push(Span::styled(
            format!(") = 0x{:x}", self.rax),
            Style::default(),
        ));
        Line::from(spans)
    }
}
