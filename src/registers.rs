use nix::libc::user_regs_struct;
use owo_colors::OwoColorize;
use ratatui::{
    prelude::{Line, Span, Style},
    style::Modifier,
};

/// Struct used to manipulate registers data from https://docs.rs/libc/0.2.147/libc/struct.user_regs_struct.html
#[derive(Debug)]
pub struct RegistersData {
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
            orig_rax: registers.orig_rax,
            rdi: registers.rdi,
            rsi: registers.rsi,
            rdx: registers.rdx,
            rax: registers.rax,
        }
    }

    /// Returns a good string which shows the output for a line
    pub fn output(&self) -> String {
        format!(
            "{}({:x}, {:x}, {:x}, ...) = {:x}",
            self.orig_rax.bold(),
            self.rdi,
            self.rsi,
            self.rdx,
            self.rax
        )
    }

    /// Returns a good line for TUI
    pub fn output_ui(&self) -> Line {
        Line::from(vec![
            Span::styled(
                format!("{}", self.orig_rax),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "({:x}, {:x}, {:x}, ...) = {:x}",
                    self.rdi, self.rsi, self.rdx, self.rax
                ),
                Style::default(),
            ),
        ])
    }
}
