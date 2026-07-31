use std::cmp::{max, min};

pub const TOP_SPACE_SIZE: usize = 1;
pub const LEFT_SPACE_SIZE: usize = 6;
pub const BOTTOM_SPACE_SIZE: usize = 1;

use crate::{
    HIDE_CURSOR, SHOW_CURSOR,
    app::{App, Mode},
    document::Pos,
    terminal::get_terminal_size,
};

#[derive(Clone, PartialEq, Copy, Debug)]
pub struct Cell {
    pub char: char,
    pub highlight: bool,
}

impl Cell {
    pub fn new(char: char) -> Self {
        Self {
            char,
            highlight: false,
        }
    }
}

#[derive(Clone)]
pub struct RenderBuffer {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
}

impl RenderBuffer {
    pub fn new() -> Self {
        let (width, height) = get_terminal_size();

        Self {
            width,
            height,
            cells: vec![
                Cell {
                    char: ' ',
                    highlight: false
                };
                width * height
            ],
        }
    }

    pub fn from(
        App {
            doc,
            viewport,
            mode,
            cursor,
            ..
        }: &App,
    ) -> Self {
        let mut render_buffer = Self::new();

        let file_name = if let Some((_, right)) = doc.file_path.rsplit_once('/') {
            right.to_owned()
        } else {
            doc.file_path.clone()
        };

        render_buffer.cells[0].highlight = true;
        for i in 1..render_buffer.width {
            render_buffer.cells[i] = Cell {
                char: if let Some(char) = file_name.chars().nth(i - 1) {
                    char
                } else {
                    ' '
                },
                highlight: true,
            };
        }

        for (row, line) in doc
            .lines
            .iter()
            .skip(viewport.top_row)
            .take(viewport.height)
            .enumerate()
        {
            let num = min(
                9999,
                if row + viewport.top_row == cursor.row {
                    row + viewport.top_row + 1
                } else {
                    (row + viewport.top_row).abs_diff(cursor.row)
                },
            );

            for (i, char) in format!("{:>4}", num).chars().enumerate() {
                render_buffer.cells[(row + TOP_SPACE_SIZE) * render_buffer.width + i] = Cell {
                    char,
                    highlight: false,
                };
            }

            for (col, char) in line
                .chars()
                .skip(viewport.left_column)
                .map(|ch| if ch != ' ' { ch } else { '·' })
                .enumerate()
            {
                let highlight = match mode {
                    Mode::Visual(landmark) => {
                        let pos = Pos { row, col };

                        let mut start = min(*landmark, cursor.to_pos());
                        let mut end = max(*landmark, cursor.to_pos());

                        start.row -= viewport.top_row;
                        end.row -= viewport.top_row;

                        start <= pos && pos <= end
                    }
                    _ => false,
                };

                render_buffer.cells
                    [(row + TOP_SPACE_SIZE) * render_buffer.width + (col + LEFT_SPACE_SIZE)] =
                    Cell { char, highlight };
            }
        }

        let bottom_bar_str = format!(
            "{} | {} | {}:{}",
            mode,
            doc.file_path,
            cursor.row + 1,
            cursor.col + 1
        );

        render_buffer.cells[(viewport.height + TOP_SPACE_SIZE) * render_buffer.width].highlight =
            true;
        for i in 1..render_buffer.width {
            render_buffer.cells[(viewport.height + TOP_SPACE_SIZE) * render_buffer.width + i] =
                Cell {
                    char: if let Some(char) = bottom_bar_str.chars().nth(i - 1) {
                        char
                    } else {
                        ' '
                    },
                    highlight: true,
                };
        }

        render_buffer
    }

    pub fn diff(&self, old: &Self) -> Vec<(Pos, Cell)> {
        let mut diff = vec![];

        for (i, cell) in self.cells.iter().enumerate() {
            if &old.cells[i] != cell {
                let col = i % self.width;
                let row = i / self.width;
                diff.push((Pos { row, col }, *cell));
            }
        }

        diff
    }

    pub fn patch(diff: Vec<(Pos, Cell)>) -> String {
        let mut render = String::new();

        render.push_str(HIDE_CURSOR);

        for (pos, cell) in diff {
            let gray = if cell.char == '·' { "90" } else { "0" };
            let highlighted = if cell.highlight { ";40" } else { "" };

            render.push_str(&format!(
                "\x1b[{};{}H\x1b[{}{}m{}\x1b[0m",
                pos.row + 1,
                pos.col + 1,
                gray,
                highlighted,
                cell.char,
            ));
        }

        render.push_str(SHOW_CURSOR);

        render
    }
}
