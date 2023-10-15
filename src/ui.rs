use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use nix::unistd::Pid;
use ratatui::{prelude::*, widgets::*};
use std::io::{self, stdout};

struct UI {
    height: usize,
    max_lines: usize,
    scroll: usize,
}

impl UI {
    fn new() -> UI {
        UI {
            height: 0,
            max_lines: 0,
            scroll: 0,
        }
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

pub fn run_tui(pid: Pid, lines: &str) -> anyhow::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut ui = UI::new();
    ui.max_lines = lines.split('\n').count() + 1;

    let mut should_quit = false;
    while !should_quit {
        ui.height = terminal.get_frame().size().height as usize;
        terminal.draw(move |frame| {
            let size = frame.size();
            frame.render_widget(
                Paragraph::new(lines)
                    .block(
                        Block::default()
                            .border_style(Style::default().fg(Color::Yellow))
                            .title(format!("[{pid}]"))
                            .title(
                                block::Title::from(format!(
                                    "[lines {}-{}]",
                                    ui.scroll,
                                    ui.scroll + ui.height
                                ))
                                .position(block::Position::Bottom)
                                .alignment(Alignment::Right),
                            )
                            .borders(Borders::ALL),
                    )
                    .scroll((ui.scroll as u16, 0)),
                size,
            );
        })?;
        should_quit = handle_events(&mut ui)?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
