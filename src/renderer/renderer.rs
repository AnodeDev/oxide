use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Layout};
use ratatui::Terminal;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::style::{Color, Style};

use anyhow;

use crate::buffer::Buffer;

pub struct Renderer<B: Backend> {
    terminal: Terminal<B>,
}

impl<B: Backend> Renderer<B> {
    pub fn new(terminal: Terminal<B>) -> Self {
        Renderer { terminal }
    }

    pub fn render(&mut self, buffer: &Buffer) -> anyhow::Result<()> {
        let mut lines: Vec<Line> = Vec::new();
        let mut nums: Vec<Line> = Vec::new();

        self.terminal.draw(|frame| {
            let vertical = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ]);
            let horizontal = Layout::horizontal([
                Constraint::Length(4),
                Constraint::Length(1),
                Constraint::Fill(1),
            ]);

            let [ content_area, modeline, commandline ] = vertical.areas(frame.area());
            let [ numline, _, content ] = horizontal.areas(content_area);

            for (num, line) in buffer.content.iter().enumerate() {
                if buffer.cursor.1 == num {
                    lines.push(format_line(line, Some(buffer.cursor.1)));
                } else {
                    lines.push(format_line(line, None));
                }

                nums.push(format_line(&format!("{:>4}", num + 1), None).style(Style::default().fg(Color::DarkGray)));
            }

            frame.render_widget(
                Paragraph::new(lines),
                content,
            );
            frame.render_widget(
                Paragraph::new(nums),
                numline,
            );
            frame.render_widget(
                Block::new().style(Style::default().bg(Color::Red)),
                modeline,
            );
            frame.render_widget(
                Block::new().style(Style::default().bg(Color::Green)),
                commandline,
            );
        })?;

        Ok(())
    }
}

fn format_line(line: &str, cursor_x: Option<usize>) -> Line<'static> {
    let mut spans: Vec<Span> = Vec::new();

    for (num, c) in line.chars().enumerate() {
        if cursor_x.is_some() && cursor_x == Some(num) {
            spans.push(Span::styled(c.to_string(), Style::default().bg(Color::White).fg(Color::Black)));
        } else {
            spans.push(Span::from(c.to_string()));
        }
    }

    Line::from(spans)
}
