const ROW_DISTANCE: usize = 4;
const COL_DISTANCE: usize = 6;

use std::cmp::{max, min};

use crate::{
    cursor::Cursor,
    document::Document,
    render_buffer::{BOTTOM_SPACE_SIZE, LEFT_SPACE_SIZE, TOP_SPACE_SIZE},
    terminal::get_terminal_size,
};

pub struct Viewport {
    pub top_row: usize,
    pub left_column: usize,
    pub width: usize,
    pub height: usize,
}

impl Viewport {
    pub fn new() -> Self {
        let (width, height) = get_terminal_size();

        Self {
            top_row: 0,
            left_column: 0,
            width: width - LEFT_SPACE_SIZE,
            height: height - (TOP_SPACE_SIZE + BOTTOM_SPACE_SIZE),
        }
    }

    pub fn fit(&mut self, cursor: &Cursor, doc: &Document) {
        if self.top_row + ROW_DISTANCE > cursor.row {
            self.top_row = max(cursor.row as isize - ROW_DISTANCE as isize, 0) as usize;
        }

        if self.top_row + self.height - ROW_DISTANCE < cursor.row {
            self.top_row = min(
                cursor.row + ROW_DISTANCE - self.height,
                doc.lines.len() - self.height - 1,
            );
        }

        if self.left_column + COL_DISTANCE > cursor.col {
            return self.left_column = max(cursor.col as isize - COL_DISTANCE as isize, 0) as usize;
        }

        if self.left_column + self.width - COL_DISTANCE < cursor.col {
            self.left_column = cursor.col + COL_DISTANCE - self.width;
        }
    }
}
