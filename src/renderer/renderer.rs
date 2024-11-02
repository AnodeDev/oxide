use ratatui::prelude::*;
use ratatui::layout::{Constraint, Layout};
use ratatui::Terminal;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::style::{Color, Style};

use anyhow;

use std::io::Stdout;
use std::cell::{Ref, RefMut};

use crate::buffer::Buffer;

pub struct Renderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Renderer {
    pub fn new(terminal: Terminal<CrosstermBackend<Stdout>>) -> Self {
        Renderer { terminal }
    }

    pub fn render(&mut self, buffer: Ref<Buffer>) -> anyhow::Result<()> {
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
                    lines.push(format_line(line, Some(buffer.cursor.0)));
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

    if let Some(mut x_pos) = cursor_x {
        if x_pos > line.len() - 1 {
            x_pos = line.len() - 1;
        }

        for (num, c) in line.chars().enumerate() {
            if x_pos == num {
                spans.push(Span::styled(c.to_string(), Style::default().fg(Color::Black).bg(Color::Yellow)));
            } else {
                spans.push(Span::from(c.to_string()));
            }
        }
    } else {
        for c in line.chars() {
            spans.push(Span::from(c.to_string()));
        }
    }

    Line::from(spans)
}
