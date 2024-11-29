use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Terminal;

use std::io::Stdout;
use std::path::Path;

use crate::buffer::{Buffer, CommandLineState, Cursor, Mode};
use crate::renderer::Error;

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
}

impl Renderer {
    pub fn new(terminal: Terminal<CrosstermBackend<Stdout>>) -> Self {
        Renderer { terminal }
    }

    // Renders the current buffer
    pub fn render(&mut self, buffer: &Buffer) -> Result<()> {
        let mut lines: Vec<Line> = Vec::new();
        let mut nums: Vec<Line> = Vec::new();
        let mut formatted_cmd_content: Vec<Line> = Vec::new();
        let visual_mode_on = match buffer.mode {
            // Checks if visual mode is on
            Mode::Visual => true,
            _ => false,
        };

        // Creates the buffer areas
        let draw_state = self.terminal.draw(|frame| {
            let vertical = if buffer.mode == Mode::Command {
                Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Length(1 + buffer.command_line.content.len() as u16),
                ])
            } else {
                Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
            };
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
            let modeline_left_divide =
                Layout::horizontal([Constraint::Length(9), Constraint::Fill(1)]);

            // Sets the buffer areas
            let [content_area, modeline, commandline] = vertical.areas(frame.area());
            let [numline, _, content] = horizontal.areas(content_area);
            let [modeline_left, _modeline_center, _modeline_right] =
                modeline_divide.areas(modeline);
            let [modeline_a, modeline_b] = modeline_left_divide.areas(modeline_left);

            // Iterates over the visible buffer
            for (num, line) in buffer
                .content
                .iter()
                .enumerate()
                .skip(buffer.viewport.top)
                .take(buffer.viewport.bottom() - buffer.viewport.top)
            {
                // Checks if the current line is the one with the cursor
                if buffer.cursor.y == num && buffer.mode != Mode::Command {
                    lines.push(format_line(
                        line,
                        num,
                        &buffer.cursor,
                        visual_mode_on,
                        buffer.visual_start,
                        buffer.visual_end,
                        true,
                    ));
                } else {
                    lines.push(format_line(
                        line,
                        num,
                        &buffer.cursor,
                        visual_mode_on,
                        buffer.visual_start,
                        buffer.visual_end,
                        false,
                    ));
                }

                // Adds the line numbers and pushes them to the right
                nums.push(
                    format_line(
                        &format!("{:>3}", num + 1),
                        num,
                        &buffer.cursor,
                        false,
                        buffer.visual_start,
                        buffer.visual_end,
                        false,
                    )
                    .style(Style::default().fg(Color::DarkGray)),
                );
            }

            if buffer.mode == Mode::Command {
                let content: &Vec<String> = match buffer.command_line.state {
                    CommandLineState::FindFile => {
                        let mut content = Vec::new();

                        for path in &buffer.command_line.content {
                            let suffix = if Path::new(path).is_dir() { "/" } else { "" };

                            if let Some(file_name) = Path::new(path).file_name() {
                                content.push(format!(
                                    "{}{}",
                                    file_name.to_string_lossy().into_owned(),
                                    suffix
                                ));
                            }
                        }

                        &content.clone()
                    }
                    _ => &buffer.command_line.content,
                };

                formatted_cmd_content = format_cmd_content(content, &buffer.command_line.cursor);

                formatted_cmd_content.push(format_line(
                    &format!(
                        "{}{}{}",
                        buffer.command_line.prefix,
                        buffer.command_line.input,
                        buffer.command_line.suffix,
                    ),
                    0,
                    &buffer.command_line.cursor,
                    false,
                    None,
                    None,
                    true,
                ));
            } else if buffer.command_line.state == CommandLineState::Error {
                formatted_cmd_content.push(format_error(buffer.command_line.prefix.clone()));
            }

            // Renders the buffer
            frame.render_widget(Paragraph::new(lines), content);
            frame.render_widget(Paragraph::new(nums), numline);
            frame.render_widget(
                Block::new().style(Style::default().bg(Color::DarkGray)),
                modeline,
            );
            frame.render_widget(
                Paragraph::new(Line::from(buffer.mode.to_string())).centered(),
                modeline_a,
            );
            frame.render_widget(
                Paragraph::new(Line::from(buffer.title.to_string())),
                modeline_b,
            );
            frame.render_widget(Paragraph::new(formatted_cmd_content), commandline);
        });

        match draw_state {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::DrawError),
        }
    }

    // Returns the terminal size
    pub fn get_terminal_size(&self) -> ratatui::layout::Size {
        match self.terminal.size() {
            Ok(size) => size,
            Err(_) => todo!(),
        }
    }
}

// Formats the line
// TODO: Reduce the amount of parameters, or take only the necessary parts of the parameters
fn format_line(
    line: &str,
    line_num: usize,
    cursor: &Cursor,
    visual_mode_on: bool,
    visual_start_opt: Option<Cursor>,
    visual_end_opt: Option<Cursor>,
    cursor_line: bool,
) -> Line<'static> {
    let mut spans: Vec<Span> = Vec::new();
    let cursor_style = Style::new().fg(Color::Black).bg(Color::Yellow);
    let highlight_style = Style::default().bg(Color::LightRed);
    let line_str = format!("{} ", line);
    let mut is_highlighted = false;

    // Iterates over the characters of the line
    for (num, c) in line_str.chars().enumerate() {
        let span = Span::from(c.to_string());

        // Highlights if current character is selected
        if visual_mode_on {
            is_highlighted =
                check_cursor_for_visual(line_num, num, visual_start_opt, visual_end_opt);
        }

        // Highlights if current character matches the cursor position
        if cursor_line && cursor.x == num {
            spans.push(span.style(cursor_style));
        } else if is_highlighted {
            spans.push(span.style(highlight_style));
        } else {
            spans.push(span);
        }

        is_highlighted = false;
    }

    Line::from(spans)
}

fn format_error(line: String) -> Line<'static> {
    let style = Style::new().fg(Color::Red);

    Line::styled(line, style)
}

fn format_cmd_content(content: &Vec<String>, cursor: &Cursor) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();
    let cursor_style = Style::new().fg(Color::Black).bg(Color::Yellow);

    for (num, line) in content.iter().enumerate() {
        let line = Line::from(line.to_string());

        if num == cursor.y {
            lines.push(line.style(cursor_style));
        } else {
            lines.push(line);
        }
    }

    lines
}

// Checks if the current character position is highlighted
fn check_cursor_for_visual(
    line_num: usize,
    c_num: usize,
    visual_start: Option<Cursor>,
    visual_end: Option<Cursor>,
) -> bool {
    if let (Some(start), Some(end)) = (visual_start, visual_end) {
        // Sets the top and bottom cursor
        let (top, bottom) = if start.y < end.y {
            (start, end)
        } else if start.y == end.y && start.x < end.x {
            (start, end)
        } else if start.y == end.y && start.x > end.x {
            (end, start)
        } else {
            (end, start)
        };

        if line_num >= top.y && line_num <= bottom.y {
            if line_num == top.y && line_num == bottom.y {
                // Single line selection
                let (left, right) = if start.x <= end.x {
                    (start.x, end.x)
                } else {
                    (end.x, start.x)
                };
                return c_num >= left && c_num <= right;
            } else if line_num == top.y {
                // First line of multi-line selection
                return c_num >= top.x;
            } else if line_num == bottom.y {
                // Last line of multi-line selection
                return c_num <= bottom.x;
            } else {
                // Middle lines of multi-line selection
                return true;
            }
        }
    }

    false
}
