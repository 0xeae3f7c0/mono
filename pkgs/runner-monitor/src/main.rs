// runner-monitor: Terminal monitor for CI runner hosts (commit platform, pkgs/runner-monitor)
//! Terminal monitor for a self-hosted CI runner host: live CPU and memory
//! gauges (ratatui). Press `q` to quit.

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Gauge};
use ratatui::{DefaultTerminal, Frame};
use sysinfo::System;

/// Usage snapshot, as fractions in 0.0..=1.0.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Usage {
    cpu: f64,
    memory: f64,
}

impl Usage {
    fn sample(sys: &mut System) -> Self {
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        let memory = if sys.total_memory() == 0 {
            0.0
        } else {
            sys.used_memory() as f64 / sys.total_memory() as f64
        };
        Self {
            cpu: (sys.global_cpu_usage() as f64 / 100.0).clamp(0.0, 1.0),
            memory: memory.clamp(0.0, 1.0),
        }
    }
}

fn draw(frame: &mut Frame, usage: Usage) {
    let [cpu, mem] =
        Layout::vertical([Constraint::Length(3), Constraint::Length(3)]).areas(frame.area());
    let gauge = |title: &'static str, ratio: f64, color| {
        Gauge::default()
            .block(Block::bordered().title(title))
            .gauge_style(Style::default().fg(color))
            .ratio(ratio)
    };
    frame.render_widget(gauge(" CPU ", usage.cpu, Color::Green), cpu);
    frame.render_widget(gauge(" Memory ", usage.memory, Color::Cyan), mem);
}

fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut sys = System::new().map_err(std::io::Error::other)?;
    loop {
        let usage = Usage::sample(&mut sys);
        terminal.draw(|frame| draw(frame, usage))?;
        if event::poll(Duration::from_millis(500))?
            && let Event::Key(key) = event::read()?
            && key.code == KeyCode::Char('q')
        {
            return Ok(());
        }
    }
}

fn main() -> std::io::Result<()> {
    ratatui::run(run)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn renders_both_gauges() {
        let mut terminal = Terminal::new(TestBackend::new(30, 6)).unwrap();
        terminal
            .draw(|f| {
                draw(
                    f,
                    Usage {
                        cpu: 0.5,
                        memory: 0.25,
                    },
                )
            })
            .unwrap();
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(text.contains("CPU") && text.contains("Memory"));
        assert!(text.contains("50%") && text.contains("25%"));
    }

    #[test]
    fn sample_is_a_fraction() {
        let usage = Usage::sample(&mut System::new().unwrap());
        assert!((0.0..=1.0).contains(&usage.cpu) && (0.0..=1.0).contains(&usage.memory));
    }
}
