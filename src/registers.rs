use chrono::{DateTime, Local};
use nix::libc::user_regs_struct;
use owo_colors::OwoColorize;
use ratatui::{
    prelude::{Line, Span, Style},
    style::Modifier,
};

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
use crate::arch::linux::x86_64::syscall_name;
#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
use crate::arch::syscall_name;

/// Struct used to manipulate registers data from https://docs.rs/libc/0.2.147/libc/struct.user_regs_struct.html
#[derive(Debug)]
pub struct RegistersData {
    timestamp: DateTime<Local>,
    orig_rax: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rax: u64,
}

impl RegistersData {
    /// Create new `RegistersData` from an `user_regs_struct`'C structure
    pub fn new(registers: user_regs_struct) -> RegistersData {
        RegistersData {
            timestamp: Local::now(),
            orig_rax: registers.orig_rax,
            rdi: registers.rdi,
            rsi: registers.rsi,
            rdx: registers.rdx,
            rax: registers.rax,
        }
    }

    /// Get date in ISO 8601 / RFC 3339 date & time string format
    pub fn date(&self) -> String {
        self.timestamp.format("%+").to_string()
    }

    /// Returns a good string which shows the output for a line
    pub fn output(&self) -> String {
        format!(
            "[{}]: {}({:x}, {:x}, {:x}, ...) = {:x}",
            self.date(),
            syscall_name(self.orig_rax).bold(),
            self.rdi,
            self.rsi,
            self.rdx,
            self.rax
        )
    }

    /// Returns a good line for TUI
    pub fn output_ui(&self) -> Line {
        Line::from(vec![
            Span::raw(format!("[{}]: ", self.date())),
            Span::styled(
                format!("{}", syscall_name(self.orig_rax)),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                "({:x}, {:x}, {:x}, ...) = {:x}",
                self.rdi, self.rsi, self.rdx, self.rax
            )),
        ])
    }
}
