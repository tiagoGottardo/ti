use std::cmp::{max, min};

pub const TOP_SPACE_SIZE: usize = 1;
pub const LEFT_SPACE_SIZE: usize = 6;
pub const BOTTOM_SPACE_SIZE: usize = 1;

use crate::{
    HIDE_CURSOR, SHOW_CURSOR,
    app::{App, Mode},
    cell::{Cell, Color, Theme},
    document::Pos,
    terminal::get_terminal_size,
};

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
            cells: vec![Cell::default(); width * height],
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

        let mut file_name = if let Some((_, right)) = doc.file_path.rsplit_once('/') {
            right.to_owned()
        } else {
            doc.file_path.clone()
        };
        file_name.insert(0, ' ');

        for i in 0..render_buffer.width {
            let char = file_name.chars().nth(i).unwrap_or(' ');
            let mut cell = bar_cell(char);

            if i < file_name.chars().count() {
                cell.fg = Color::FileName;
                cell.bold = true;
                cell.italic = true;
            }

            render_buffer.cells[i] = cell;
        }

        for (row, line) in doc
            .lines
            .iter()
            .skip(viewport.top_row)
            .take(viewport.height)
            .enumerate()
        {
            let screen_row = row + TOP_SPACE_SIZE;
            let line_row = row + viewport.top_row;
            let current_line = line_row == cursor.row;

            if current_line {
                for col in 0..render_buffer.width {
                    render_buffer.cells[screen_row * render_buffer.width + col].bg =
                        Color::CurrentLine;
                }
            }

            let num = min(
                9999,
                if current_line {
                    line_row + 1
                } else {
                    line_row.abs_diff(cursor.row)
                },
            );

            for (i, char) in format!("{:>4}", num).chars().enumerate() {
                let mut cell = Cell::new(char);

                if current_line {
                    cell.fg = Color::CurrentLineNumber;
                    cell.bg = Color::CurrentLine;
                    cell.bold = true;
                }

                render_buffer.cells[screen_row * render_buffer.width + i] = cell;
            }

            if line.len() == 0 {
                if let Mode::Visual(landmark) = mode {
                    let pos = Pos {
                        row: row + viewport.top_row,
                        col: 0,
                    };

                    let start = min(*landmark, cursor.to_pos());
                    let end = max(*landmark, cursor.to_pos());

                    if is_visual_selected(pos, start, end) {
                        render_buffer.cells
                            [(row + TOP_SPACE_SIZE) * render_buffer.width + LEFT_SPACE_SIZE]
                            .selected();
                    }
                }
            }

            for (i, char) in line
                .chars()
                .skip(viewport.left_column)
                .map(|ch| {
                    if ch == ' ' && line.len() > 0 {
                        '·'
                    } else {
                        ch
                    }
                })
                .enumerate()
            {
                let mut cell = Cell::new(char);

                if char == '·' {
                    cell.fg = Color::Whitespace;
                }

                if current_line {
                    cell.bg = Color::CurrentLine;
                }

                if let Mode::Visual(landmark) = mode {
                    let pos = Pos {
                        row: line_row,
                        col: i + viewport.left_column,
                    };

                    let start = min(*landmark, cursor.to_pos());
                    let end = max(*landmark, cursor.to_pos());

                    if is_visual_selected(pos, start, end) {
                        cell.selected();
                    }
                }

                render_buffer.cells[screen_row * render_buffer.width + LEFT_SPACE_SIZE + i] = cell;
            }
        }

        let bottom_bar_str = format!(" {} | {}", mode, doc.file_path,);

        let cursor_pos_str = format!("{}:{} ", cursor.row + 1, cursor.col + 1);

        let bottom_bar_row = viewport.height + TOP_SPACE_SIZE;
        let file_name_start = format!(" {} | ", mode).chars().count();
        let file_name_end = file_name_start + doc.file_path.chars().count();

        for i in 0..render_buffer.width {
            let char = if let Some(ch) = bottom_bar_str.chars().nth(i) {
                ch
            } else if i >= render_buffer.width - cursor_pos_str.chars().count() {
                cursor_pos_str
                    .chars()
                    .nth(i - (render_buffer.width - cursor_pos_str.chars().count()))
                    .unwrap_or(' ')
            } else {
                ' '
            };

            let mut cell = bar_cell(char);

            if (file_name_start..file_name_end).contains(&i) {
                cell.fg = Color::FileName;
                cell.bold = true;
                cell.italic = true;
            }

            render_buffer.cells[bottom_bar_row * render_buffer.width + i] = cell;
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

    pub fn patch(diff: Vec<(Pos, Cell)>, theme: &Theme) -> String {
        let mut render = String::new();

        render.push_str(HIDE_CURSOR);

        for (pos, cell) in diff {
            render.push_str(&cell.build(
                Pos {
                    row: pos.row + 1,
                    col: pos.col + 1,
                },
                &theme,
            ));
        }

        render.push_str(SHOW_CURSOR);

        render
    }
}

fn bar_cell(char: char) -> Cell {
    Cell {
        char,
        fg: Color::BarForeground,
        bg: Color::BarBackground,
        bold: true,
        italic: false,
    }
}

fn is_visual_selected(pos: Pos, start: Pos, end: Pos) -> bool {
    start <= pos && pos <= end
}

#[cfg(test)]
mod tests {
    use super::is_visual_selected;
    use crate::document::Pos;

    #[test]
    fn visual_selection_uses_columns_on_single_line() {
        let start = Pos { row: 3, col: 4 };
        let end = Pos { row: 3, col: 8 };

        assert!(!is_visual_selected(Pos { row: 3, col: 3 }, start, end));
        assert!(is_visual_selected(Pos { row: 3, col: 4 }, start, end));
        assert!(is_visual_selected(Pos { row: 3, col: 8 }, start, end));
        assert!(!is_visual_selected(Pos { row: 3, col: 9 }, start, end));
    }
}
