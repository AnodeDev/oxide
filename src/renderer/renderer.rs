use ratatui::prelude::*;
use ratatui::layout::{Constraint, Layout};
use ratatui::Terminal;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::style::{Color, Style};

use anyhow;

use std::io::Stdout;
use std::cell::Ref;

use crate::buffer::{Buffer, Cursor, Mode};

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
        let visual_mode_on = match buffer.mode {
            Mode::Visual => true,
            _ => false,
        };

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
                    lines.push(format_line(line, num, &buffer.cursor, visual_mode_on, buffer.visual_start, buffer.visual_end, true));
                } else {
                    lines.push(format_line(line, num, &buffer.cursor, visual_mode_on, buffer.visual_start, buffer.visual_end, false));
                }
                nums.push(format_line(&format!("{:>3}", num + 1), num, &buffer.cursor, false, buffer.visual_start, buffer.visual_end, false).style(Style::default().fg(Color::DarkGray)));
            }

            if buffer.mode == Mode::Command {
                let mut cursor = buffer.cursor.clone();
                if cursor.x > buffer.commandline.len() {
                    cursor.x = 1;
                } else {
                    cursor.x += 1;
                }
                commandline_line = format_line(&format!(":{}", buffer.commandline), 0, &cursor, false, buffer.visual_start, buffer.visual_end, true)
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

fn format_line(
    line: &str,
    line_num: usize,
    cursor: &Cursor,
    visual_mode_on: bool,
    visual_start_opt: Option<Cursor>,
    visual_end_opt: Option<Cursor>,
    cursor_line: bool) -> Line<'static> {
    let mut spans: Vec<Span> = Vec::new();
    let cursor_style = Style::new().fg(Color::Black).bg(Color::Yellow);
    let highlight_style = Style::default().bg(Color::LightRed);
    let line_str = format!("{} ", line);
    let mut is_highlighted = false;

    for (num, c) in line_str.chars().enumerate() {
        let span = Span::from(c.to_string());

        if visual_mode_on {
            is_highlighted = check_cursor_for_visual(cursor, line_num, num, visual_start_opt, visual_end_opt);
        }

        if cursor_line && cursor.x == num {
            spans.push(span.style(cursor_style));
        } else if is_highlighted {
            spans.push(span.style(highlight_style));
        }else {
            spans.push(span);
        }

        is_highlighted = false;
    }

    Line::from(spans)
}

fn check_cursor_for_visual(cursor: &Cursor, line_num: usize, c_num: usize, visual_start: Option<Cursor>, visual_end: Option<Cursor>) -> bool {
    if let (Some(start), Some(end)) = (visual_start, visual_end) {
        let (top, bottom) = if start.y <= end.y { (start, end) } else { (end, start) };
        
        if line_num >= top.y && line_num <= bottom.y {
            if cursor.y != line_num {
                return true;
            } else if top.y == bottom.y {
                let (left, right) = if start.x <= end.x { (start.x, end.x) } else { (end.x, start.x) };
                return c_num >= left && c_num <= right;
            } else if line_num == top.y {
                return if start.y == top.y { c_num >= start.x } else { c_num >= end.x };
            } else if line_num == bottom.y {
                return if start.y == bottom.y { c_num <= start.x } else { c_num <= end.x };
            } else {
                return true;
            }
        }
    }

    false
}
