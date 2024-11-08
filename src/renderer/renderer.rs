use ratatui::prelude::*;
use ratatui::layout::{Constraint, Layout};
use ratatui::Terminal;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::style::{Color, Style};

use anyhow;

use std::io::Stdout;
use std::cell::Ref;

use crate::buffer::{Buffer, Mode};

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
        let mut commandline_line: Line = Line::raw("");

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
            let modeline_divide = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]);
            let modeline_left_divide = Layout::horizontal([
                Constraint::Length(9),
                Constraint::Fill(1),
            ]);


            let [ content_area, modeline, commandline ] = vertical.areas(frame.area());
            let [ numline, _, content ] = horizontal.areas(content_area);
            let [ modeline_left, _modeline_center, _modeline_right ] = modeline_divide.areas(modeline);
            let [ modeline_a, modeline_b ] = modeline_left_divide.areas(modeline_left);

            for (num, line) in buffer.content
                .iter()
                .enumerate()
                .skip(buffer.viewport.top)
                .take(buffer.viewport.bottom() - buffer.viewport.top)
            {
                if buffer.cursor.y == num && buffer.mode != Mode::Command {
                    lines.push(format_line(line, Some(buffer.cursor.x)));
                } else {
                    lines.push(format_line(line, None));
                }
                nums.push(format_line(&format!("{:>3}", num + 1), None).style(Style::default().fg(Color::DarkGray)));
            }

            if buffer.mode == Mode::Command {
                commandline_line = format_line(&format!(":{}", buffer.commandline), Some(buffer.cursor.x + 1));
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
                Block::new()
                    .style(Style::default().bg(Color::DarkGray)),
                modeline,
            );
            frame.render_widget(
                Paragraph::new(Line::from(buffer.mode.to_string()))
                    .centered(),
                modeline_a,
            );
            frame.render_widget(
                Paragraph::new(Line::from(buffer.title.to_string())),
                modeline_b,
            );
            frame.render_widget(
                Paragraph::new(commandline_line),
                commandline,
            );
        })?;

        Ok(())
    }

    pub fn get_terminal_size(&self) -> std::io::Result<ratatui::layout::Size> {
        self.terminal.size()
    }
}

fn format_line(line: &str, cursor_x: Option<usize>) -> Line<'static> {
    let mut spans: Vec<Span> = Vec::new();
    let line_str = format!("{} ", line);

    for (num, c) in line_str.chars().enumerate() {
        if cursor_x.is_some() && cursor_x == Some(num) {
            spans.push(Span::styled(c.to_string(), Style::default().fg(Color::Black).bg(Color::Yellow)));
        } else {
            spans.push(Span::from(c.to_string()));
        }
    }

    Line::from(spans)
}
