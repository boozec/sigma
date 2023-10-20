use chrono::{DateTime, Local};
use nix::libc::user_regs_struct;
use owo_colors::OwoColorize;
use ratatui::{
    prelude::{Line, Span, Style},
    style::{Color, Modifier},
};

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
use crate::arch::linux::x86_64::*;
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
    r10: u64,
    r8: u64,
    r9: u64,
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
            r10: registers.r10,
            r8: registers.r8,
            r9: registers.r9,
            rax: registers.rax,
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
    pub fn output(&self) -> String {
        let mut output = format!("[{}]: ", self.date());

        if self.name().len() > 0 {
            output.push_str(&format!("{}(", self.name().bold()));
        } else {
            output.push_str(&format!("{}(", self.orig_rax.yellow().bold()));
        }

        let mut has_param = false;

        let params = [
            (self.rdi, rdi(self.orig_rax)),
            (self.rsi, rsi(self.orig_rax)),
            (self.rdx, rdx(self.orig_rax)),
            (self.r10, r10(self.orig_rax)),
            (self.r8, r8(self.orig_rax)),
            (self.r9, r9(self.orig_rax)),
        ];

        for param in params {
            if param.1.len() != 0 {
                let output_param = param.1.to_owned() + ":";
                output.push_str(&format!("{} {}, ", output_param.blue(), param.0));
                has_param = true;
            }
        }

        if has_param {
            output.remove(output.len() - 1);
            output.remove(output.len() - 1);
        }

        output.push_str(&format!(") = 0x{:x}", self.rax)[..]);
        output
    }

    /// Returns a good line for TUI
    pub fn output_ui(&self) -> Line {
        let mut spans: Vec<Span> = vec![];
        spans.push(Span::raw(format!("[{}]: ", self.date())));
        if self.name().len() > 0 {
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

        let params = [
            (self.rdi, rdi(self.orig_rax)),
            (self.rsi, rsi(self.orig_rax)),
            (self.rdx, rdx(self.orig_rax)),
            (self.r10, r10(self.orig_rax)),
            (self.r8, r8(self.orig_rax)),
            (self.r9, r9(self.orig_rax)),
        ];

        for param in params {
            if param.1.len() != 0 {
                let output_param = param.1.to_owned() + ":";
                spans.push(Span::styled(
                    format!("{} ", output_param),
                    Style::default().fg(Color::Blue),
                ));
                spans.push(Span::styled(format!("{}, ", param.0), Style::default()));
            }
        }

        spans.push(Span::styled(
            format!(") = 0x{:x}", self.rax),
            Style::default(),
        ));
        Line::from(spans)
    }
}
