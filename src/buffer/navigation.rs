use crate::buffer::{Buffer, Mode};

pub trait Navigation {
    fn move_cursor(&mut self, x: i32, y: i32);
    fn move_cursor_to_top(&mut self);
    fn move_cursor_to_bot(&mut self);
}

impl Navigation for Buffer {
    fn move_cursor(&mut self, x: i32, y: i32) {
        match self.mode {
            Mode::Normal | Mode::Visual { .. } => {
                // Sets the new y value.
                // Clamp is used to make sure it doesn't exceed the length of the line or 0.
                let new_y =
                    (self.cursor.y as i32 + y).clamp(0, self.content.len() as i32 - 1) as usize;
                self.cursor.y = new_y;

                // Adjusts the viewport to match the cursor position.
                self.viewport.adjust(self.cursor.y, self.content.len());

                // Checks if cursor is moved horiozontally.
                // If not, it checks if x is larger than the current lines length and adjusts accordingly.
                if x != 0 {
                    let current_line_len = self.content[self.cursor.y].len();
                    let new_x =
                        (self.cursor.x as i32 + x).clamp(0, current_line_len as i32) as usize;

                    self.cursor.x = new_x;
                    self.cursor.desired_x = new_x;
                } else {
                    let current_line_len = self.content[self.cursor.y].len();
                    self.cursor.x = self.cursor.desired_x.min(current_line_len);
                }
            },
            Mode::Command => {
                let prefix_len: i32 = self.command_line.prefix.len() as i32;
                let input_len: i32 = self.command_line.input.len() as i32;
                let new_x =
                    (self.cursor.x as i32 + x)
                    .clamp(prefix_len, prefix_len + input_len) as usize;

                self.command_line.cursor.x = new_x;
                self.command_line.cursor.desired_x = new_x;
            },
            _ => {}
        }
    }

    fn move_cursor_to_top(&mut self) {
        self.cursor.x = 0;
        self.cursor.y = 0;

        self.viewport.adjust(self.cursor.y, self.content.len());
    }

    fn move_cursor_to_bot(&mut self) {
        self.cursor.x = 0;
        self.cursor.y = self.content.len() - 1;

        self.viewport.adjust(self.cursor.y, self.content.len());
    }
}
