use crate::{
    cli::Args,
    registers::RegistersData,
    trace::{trace, trace_kill, trace_next},
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use nix::{sys::wait::waitpid, unistd::Pid};
use ratatui::{prelude::*, widgets::*};
use std::io::{self, stdout};

pub struct UI {
    height: usize,
    max_lines: usize,
    scroll: usize,
    lines: Vec<RegistersData>,
}

impl UI {
    pub fn new() -> UI {
        UI {
            height: 0,
            max_lines: 0,
            scroll: 0,
            lines: vec![],
        }
    }

    pub fn add_line(&mut self, registers: RegistersData) {
        self.lines.push(registers);
        self.max_lines = self.lines.len() + 1;
    }

    pub fn get_paragraph(&self, pid: Pid) -> Paragraph {
        let lines: Vec<Line> = self.lines.iter().map(|x| x.output_ui()).collect();
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .border_style(Style::default().fg(Color::Yellow))
                    .title(format!("[{pid}]"))
                    .title(
                        block::Title::from(format!(
                            "[lines {}-{}]",
                            self.scroll,
                            self.scroll + self.height
                        ))
                        .position(block::Position::Bottom)
                        .alignment(Alignment::Right),
                    )
                    .borders(Borders::ALL),
            )
            .scroll((self.scroll as u16, 0));

        paragraph
    }

    pub fn start(&mut self, pid: Pid, args: &Args) -> anyhow::Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let mut have_to_print = true;
        let mut have_to_trace = args.command.is_none();
        let mut should_quit = false;

        if args.command.is_some() {
            let registers = trace(pid, args)?;
            for register in registers {
                self.add_line(register);
            }
        } else {
            // First wait for the parent process
            _ = waitpid(pid, None)?;
        }

        let filters: Vec<&str> = match &args.filter {
            Some(filter) => filter.split(",").collect::<Vec<&str>>(),
            None => vec![],
        };
        while !should_quit {
            if have_to_trace {
                if let Some(reg) = trace_next(pid)? {
                    have_to_print ^= true;
                    if have_to_print {
                        if !filters.is_empty() && !filters.contains(&reg.rax()) {
                            continue;
                        }
                        self.add_line(reg);
                    }
                } else {
                    have_to_trace = false;
                }
            }

            self.height = terminal.get_frame().size().height as usize;
            terminal.draw(|frame| {
                let size = frame.size();

                frame.render_widget(self.get_paragraph(pid), size);
            })?;

            should_quit = handle_events(self)?;
        }

        // FIXME: avoid this kill without Rust errors
        let _ = trace_kill(pid);

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;

        Ok(())
    }
}

fn handle_events(ui: &mut UI) -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(true);
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        if ui.scroll < (ui.max_lines - ui.height + 1) {
                            ui.scroll += 1;
                        }
                    }
                    KeyCode::Char('J') | KeyCode::Char('G') => {
                        ui.scroll = ui.max_lines - ui.height + 1;
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if ui.scroll > 1 {
                            ui.scroll -= 1;
                        }
                    }
                    KeyCode::Char('K') | KeyCode::Char('0') => {
                        ui.scroll = ui.height;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(false)
}
