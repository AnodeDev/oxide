/// The visible part of the buffer content
pub struct Viewport {
    pub top: usize,
    pub height: usize,
}

impl Viewport {
    pub fn new(height: usize) -> Self {
        Viewport {
            top: 0,
            height,
        }
    }

    pub fn bottom(&self) -> usize {
        self.top + self.height
    }

    pub fn adjust(&mut self, cursor_y: usize, content_len: usize) {
        if cursor_y < self.top {
            self.top = cursor_y;
        } else if cursor_y >= self.bottom() {
            self.top = cursor_y.saturating_sub(self.height) + 1;
        }

        if self.bottom() > content_len {
            self.top = content_len.saturating_sub(self.height);
        }
    }
}

