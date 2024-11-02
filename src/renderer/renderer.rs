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
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Fill(1),
            ]);

            let [ content_area, modeline, commandline ] = vertical.areas(frame.area());
            let [ numline, _, content ] = horizontal.areas(content_area);

            for (num, line) in buffer.content.iter().enumerate() {
                if buffer.cursor.y == num {
                    lines.push(format_line(line, Some(buffer.cursor.x)));

                    nums.push(format_line(&format!("{:<3}", num + 1), None).style(Style::default().fg(Color::DarkGray)));
                } else {
                    lines.push(format_line(line, None));

                    nums.push(format_line(&format!("{:>3}", num + 1), None).style(Style::default().fg(Color::DarkGray)));
                }
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
                Paragraph::new(format!("{} {}", buffer.mode, buffer.title))
                    .style(Style::default().bg(Color::DarkGray)),
                modeline,
            );
            frame.render_widget(
                Paragraph::new(""),
                commandline,
            );
        })?;

        Ok(())
    }
}

fn format_line(line: &str, cursor_x: Option<usize>) -> Line<'static> {
    let mut spans: Vec<Span> = Vec::new();
    let mut line_str = line;

    if line.len() == 0 {
        line_str = " ";
    }

    for (num, c) in line_str.chars().enumerate() {
        if cursor_x.is_some() && cursor_x == Some(num) {
            spans.push(Span::styled(c.to_string(), Style::default().fg(Color::Black).bg(Color::Yellow)));
        } else {
            spans.push(Span::from(c.to_string()));
        }
    }

    Line::from(spans)
}
