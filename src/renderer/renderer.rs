use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Terminal;

use std::io::Stdout;

use crate::buffer::{Buffer, Mode};
use crate::renderer::Error;

const CURSOR_STYLE: Style = Style::new().fg(Color::Black).bg(Color::Yellow);
const HIGHLIGHT_STYLE: Style = Style::new().bg(Color::LightRed);
// ╭──────────────────────────────────────╮
// │ Renderer Macros                      │
// ╰──────────────────────────────────────╯

macro_rules! format_line {
    ($line:expr) => {{
        let mut spans: Vec<Span> = Vec::new();
        let line_str = format!("{} ", $line);

        spans.push(Span::raw(line_str));

        Line::from(spans)
    }};

    ($line:expr, $x_pos:expr) => {{
        let mut spans: Vec<Span> = Vec::new();
        let line_str = format!("{} ", $line);

        for (num, c) in line_str.chars().enumerate() {
            let span = Span::from(c.to_string());

            if num == $x_pos {
                spans.push(span.style(CURSOR_STYLE));
            } else {
                spans.push(span);
            }
        }

        Line::from(spans)
    }};

    ($line:expr, $line_num:expr, $visual_start:expr, $cursor:expr) => {{
        let mut spans: Vec<Span> = Vec::new();
        let line_str = format!("{} ", $line);

        // Sets the top and bottom cursor
        let (top, bottom) = if $visual_start.y < $cursor.y {
            ($visual_start, $cursor)
        } else if $visual_start.y == $cursor.y && $cursor.x < $cursor.x {
            ($visual_start, $cursor)
        } else if $visual_start.y == $cursor.y && $visual_start.x > $cursor.x {
            ($cursor, $visual_start)
        } else {
            ($cursor, $visual_start)
        };

        for (num, c) in line_str.chars().enumerate() {
            let span = Span::from(c.to_string());


            let is_selected: bool = 
            if $line_num >= top.y && $line_num <= bottom.y {
                if $line_num == top.y && $line_num == bottom.y {
                    // Single line selection
                    let (left, right) = if $visual_start.x <= $cursor.x {
                        ($visual_start.x, $cursor.x)
                    } else {
                        ($cursor.x, $visual_start.x)
                    };
                    num >= left && num <= right
                } else if $line_num == top.y {
                    // First line of multi-line selection
                    num >= top.x
                } else if $line_num == bottom.y {
                    // Last line of multi-line selection
                    num <= bottom.x
                } else {
                    // Middle lines of multi-line selection
                    true
                }
            } else {
                false
            };

            if $cursor.y == $line_num && $cursor.x == num {
                spans.push(span.style(CURSOR_STYLE));
            } else if is_selected {
                spans.push(span.style(HIGHLIGHT_STYLE));
            } else {
                spans.push(span);
            }
        }

        Line::from(spans)
    }};
}

// ╭──────────────────────────────────────╮
// │ Renderer Types                       │
// ╰──────────────────────────────────────╯

type Result<'a, T> = std::result::Result<T, Error>;

// ╭──────────────────────────────────────╮
// │ Renderer Structs                     │
// ╰──────────────────────────────────────╯

// Handles the rendering of the buffer
pub struct Renderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    vertical: Layout,
    horizontal: Layout,
}

impl Renderer {
    pub fn new(terminal: Terminal<CrosstermBackend<Stdout>>) -> Self {
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

        Renderer {
            terminal,
            vertical,
            horizontal,
        }
    }

    pub fn render_buffer(&mut self, buffer: &Buffer) -> Result<()> {
        self.terminal.draw(|frame| {
            let mut lines: Vec<Line> = Vec::new();
            let mut nums: Vec<Line> = Vec::new();
            let [buffer_vert, _, command_line_area] = self.vertical.areas(frame.area());
            let [num_line, _, buffer_area] = self.horizontal.areas(buffer_vert);

            let iter = buffer.content
                .iter()
                .enumerate()
                .skip(buffer.viewport.top)
                .take(buffer.viewport.bottom() - buffer.viewport.top);

            for (num, line) in iter {
                match buffer.mode {
                    Mode::Visual => {
                        if let Some(start) = buffer.visual_start {
                            lines.push(format_line!(line, num, start, buffer.cursor));
                        }
                    },
                    _ => {
                        if buffer.cursor.y == num {
                            lines.push(format_line!(line, buffer.cursor.x));
                        } else {
                            lines.push(format_line!(line));
                        }
                    },
                }

                nums.push(format_line!(format!("{:>3}", num + 1)));
            }

            frame.render_widget(Paragraph::new(lines), buffer_area);
            frame.render_widget(Paragraph::new(nums), num_line);

            if buffer.mode == Mode::Command {
                log::info!("PREFIX: {}", buffer.command_line.prefix);
                let cmd_content = format_line!(format!("{}{}", buffer.command_line.prefix, buffer.command_line.input), buffer.command_line.cursor.x);
                frame.render_widget(Paragraph::new(cmd_content), command_line_area);
            }
        })?;

        Ok(())
    }

    pub fn render_mini_buffer(&mut self) -> Result<()> {
        Ok(())
    }

    // Returns the terminal size
    pub fn get_terminal_size(&self) -> ratatui::layout::Size {
        match self.terminal.size() {
            Ok(size) => size,
            Err(_) => todo!(),
        }
    }
}

fn format_error(line: String) -> Line<'static> {
    let style = Style::new().fg(Color::Red);

    Line::styled(line, style)
}
