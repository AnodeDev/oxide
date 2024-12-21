use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};
use ratatui::Terminal;

use std::io::Stdout;

use crate::buffer::{Buffer, Minibuffer, Mode};
use crate::renderer::Error;

// ╭──────────────────────────────────────╮
// │ Renderer Consts                      │
// ╰──────────────────────────────────────╯

const CURSOR_STYLE: Style = Style::new()
    .fg(Color::Black)
    .bg(Color::Rgb(0xf2, 0xd5, 0xcf));
const HIGHLIGHT_STYLE: Style = Style::new().bg(Color::Rgb(0x45, 0x47, 0x5a));
const ELEMENT_STYLE: Style = Style::new().bg(Color::Rgb(0x11, 0x11, 0x1b));
const PREFIX_STYLE: Style = Style::new().fg(Color::Black).bg(Color::Blue);
const _ERROR_STYLE: Style = Style::new().fg(Color::Red);

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

    ($line:expr, $line_num:expr, $y_pos:expr) => {{
        let formatted_line = Line::from(String::from($line));

        if $y_pos == $line_num {
            formatted_line.style(CURSOR_STYLE)
        } else {
            formatted_line
        }
    }};

    ($line:expr, $line_num:expr, $start:expr, $cursor:expr) => {{
        let mut spans: Vec<Span> = Vec::new();
        let line_str = format!("{} ", $line);

        // Sets the top and bottom cursor
        let (top, bottom) = if $start.y < $cursor.y
            || ($start.y == $cursor.y && $start.x <= $cursor.x)
        {
            ($start, $cursor)
        } else {
            ($cursor, $start)
        };

        for (num, c) in line_str.chars().enumerate() {
            let span = Span::from(c.to_string());

            let is_selected: bool = if $line_num >= top.y && $line_num <= bottom.y {
                if $line_num == top.y && $line_num == bottom.y {
                    // Single line selection
                    let (left, right) = if $start.x <= $cursor.x {
                        ($start.x, $cursor.x)
                    } else {
                        ($cursor.x, $start.x)
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

macro_rules! format_statusline {
    ($mode: expr, $title: expr, $lines: expr, $cursor: expr) => {{
        let left_line = Line::from(format!(" {} ", $mode)).left_aligned();
        let middle_line = Line::from($title).centered();

        let line_delta = format!("[{}/{}] :{}", $cursor.y + 1, $lines + 1, $cursor.x);
        let line_percentage = if $lines > 0 {
            (($cursor.y as f32 / $lines as f32) * 100_f32).floor()
        } else {
            100.0
        };

        let right_line =
            Line::from(format!(" {}  {}% ", line_delta, line_percentage)).right_aligned();

        (left_line, middle_line, right_line)
    }};
}

macro_rules! format_minibuffer {
    ($prefix:expr, $input:expr, $matched:expr, $x_pos:expr) => {{
        let mut input: Vec<Span> = vec![Span::from($prefix).style(PREFIX_STYLE)];
        let mut spans: Vec<Span> = Vec::new();
        let mut matched: Vec<Span> = Vec::new();
        let line_str = format!("{} ", $input);

        for (num, entry) in $matched.iter().enumerate() {
            let span = if entry != "/" {
                Span::from(format!("{}/", entry))
            } else {
                Span::from(entry.to_string())
            };

            if num == $x_pos {
                matched.push(span.style(CURSOR_STYLE));
            } else {
                matched.push(span);
            }
        }

        for (num, c) in line_str.chars().enumerate() {
            let span = Span::from(c.to_string());

            if $x_pos >= $matched.len() && num == $x_pos - $matched.len() {
                spans.push(span.style(CURSOR_STYLE));
            } else {
                spans.push(span);
            }
        }

        input.append(&mut matched);
        input.append(&mut spans);

        Line::from(input)
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
    statusline: Layout,
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

        let statusline = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]);

        Renderer {
            terminal,
            vertical,
            horizontal,
            statusline,
        }
    }

    pub fn render(&mut self, buffer: &Buffer, minibuffer_opt: Option<&Minibuffer>) -> Result<()> {
        self.terminal.draw(|frame| {
            let mut lines: Vec<Line> = Vec::new();
            let mut nums: Vec<Line> = Vec::new();
            let [buffer_vert, statusline_area, command_line_area] =
                self.vertical.areas(frame.area());
            let [num_line, _, buffer_area] = self.horizontal.areas(buffer_vert);
            let [left_status_area, middle_status_area, right_status_area] =
                self.statusline.areas(statusline_area);

            let visible_buffer_content = buffer
                .content
                .iter()
                .enumerate()
                .skip(buffer.viewport.top)
                .take(buffer.viewport.bottom() - buffer.viewport.top);

            for (num, line) in visible_buffer_content {
                match buffer.mode {
                    Mode::Visual => {
                        if let Some(start) = buffer.visual_start {
                            lines.push(format_line!(line, num, start, buffer.cursor));
                        }
                    }
                    _ => {
                        if buffer.cursor.y == num {
                            lines.push(format_line!(line, buffer.cursor.x));
                        } else {
                            lines.push(format_line!(line));
                        }
                    }
                }

                nums.push(format_line!(format!("{:>3}", num + 1)));
            }


            frame.render_widget(Paragraph::new(lines), buffer_area);
            frame.render_widget(Paragraph::new(nums), num_line);

            if let Some(minibuffer) = minibuffer_opt {
                let [_, minibuffer_area] =
                    Layout::vertical([Constraint::Fill(1), Constraint::Length(minibuffer.content.len() as u16 + 1)])
                        .areas(frame.area());

                let [mb_content_area, mb_input_area] =
                    Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                        .areas(minibuffer_area);

                let [mb_padding, mb_content] = Layout::horizontal([Constraint::Length(1), Constraint::Fill(1)])
                        .areas(mb_content_area);

                let minibuffer_input = format_minibuffer!(minibuffer.prefix.clone(), minibuffer.input, minibuffer.matched_input, minibuffer.cursor.x);
                let mut minibuffer_content: Vec<Line> = Vec::new();

                for (num, entry) in minibuffer.content.iter().enumerate() {
                    minibuffer_content.push(format_line!(entry, num, minibuffer.cursor.y));
                }

                frame.render_widget(Clear, mb_padding);
                frame.render_widget(Clear, mb_content);
                frame.render_widget(Block::new().style(ELEMENT_STYLE), mb_padding);
                frame.render_widget(Paragraph::new(minibuffer_content).style(ELEMENT_STYLE), mb_content);
                frame.render_widget(Paragraph::new(minibuffer_input), mb_input_area);
            } else {
                let (left_status, middle_status, right_status) = format_statusline!(
                    buffer.mode,
                    buffer.title.clone(),
                    buffer.content.len() - 1,
                    buffer.cursor
                );

                frame.render_widget(Block::new().style(ELEMENT_STYLE), statusline_area);
                frame.render_widget(Paragraph::new(left_status), left_status_area);
                frame.render_widget(Paragraph::new(middle_status), middle_status_area);
                frame.render_widget(Paragraph::new(right_status), right_status_area);

                if buffer.mode == Mode::Command {
                    let cmd_input = format_line!(
                        format!(
                            "{}{}",
                            buffer.command_line.prefix, buffer.command_line.input,
                        ),
                        buffer.command_line.cursor.x
                    );

                    frame.render_widget(Paragraph::new(cmd_input), command_line_area);
                }
            }
        })?;

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
