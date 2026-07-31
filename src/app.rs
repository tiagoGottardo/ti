use core::fmt;
use std::cmp::{max, min};

use crate::{
    cursor::Cursor,
    document::{Document, Pos},
    key::{Key, get_key_pressed},
    undo::UndoStack,
    viewport::Viewport,
};

#[derive(Clone)]
pub enum Clipboard {
    Normal(String),
    Line(String),
    None,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Mode {
    Normal,
    Replace,
    Delete,
    Copy,
    Insert,
    Visual(Pos),
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mode = match self {
            Self::Normal | Self::Delete | Self::Copy | Self::Replace => "NORMAL",
            Self::Insert => "INSERT",
            Self::Visual(_) => "VISUAL",
        };

        write!(f, "{mode}")?;

        Ok(())
    }
}

impl Mode {
    pub fn set(&mut self, mode: Mode) -> Mode {
        *self = mode;
        mode
    }
}

pub struct App {
    pub doc: Document,
    pub viewport: Viewport,
    pub cursor: Cursor,
    pub mode: Mode,
    pub undo: UndoStack,
    pub clipboard: Clipboard,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            doc: Document::new()?,
            viewport: Viewport::new(),
            cursor: Cursor::new(),
            mode: Mode::Normal,
            undo: UndoStack::new(),
            clipboard: Clipboard::None,
        })
    }

    pub fn handle_input(&mut self, key: Key) -> anyhow::Result<bool> {
        use Key::*;
        use Mode::*;

        let App {
            cursor,
            doc,
            mode,
            undo,
            clipboard,
            ..
        } = self;

        match (*mode, key) {
            (Normal | Visual(_), Sym('Q')) => return Ok(false),
            (Normal | Visual(_), Sym('W')) => doc.save()?,
            (Normal | Visual(_), Sym('h')) => {
                cursor.bound_col(doc, *mode).left();
            }
            (Normal | Visual(_), Sym('j') | ArrowDown) => {
                cursor.down(doc);
            }
            (Normal | Visual(_), Sym('k') | ArrowUp) => {
                cursor.up();
            }
            (Normal | Visual(_), Sym('l') | ArrowRight) => {
                cursor.right(doc, *mode);
            }
            (Normal, Sym('i')) => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                cursor.bound_col(doc, mode.set(Insert));
            }
            (Normal | Visual(_), Sym('u')) => {
                if let Some(snapshot) = undo.pop() {
                    doc.restore(snapshot.0);
                    *cursor = snapshot.1;
                    mode.set(snapshot.2);
                }
            }
            (Normal, Sym('I')) => {
                mode.set(Insert);
                cursor.go_to_start_of_line(doc);
            }
            (Normal | Visual(_), Sym('w')) => {
                cursor.go_to_pos(doc.next_word(cursor.to_pos()));
            }
            (Normal | Visual(_), Sym('b')) => {
                cursor.go_to_pos(doc.prev_word(cursor.to_pos()));
            }
            (Normal | Visual(_), Sym('e')) => {
                cursor.go_to_pos(doc.last_char_of_next_word(cursor.to_pos()));
            }
            (Normal, Sym('J')) => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);

                let pos = cursor.go_to_end_of_line(doc, Insert).to_pos();

                doc.delete(pos, pos);
                doc.insert(pos, " ");
            }
            (Visual(landmark), Sym('J')) => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);

                let from = min(landmark, cursor.to_pos());
                let to = max(landmark, cursor.to_pos());

                cursor.go_to_pos(from);

                for _ in 0..max(1, to.row - from.row) {
                    let pos = cursor.go_to_end_of_line(doc, Insert).to_pos();

                    doc.delete(pos, pos);
                    doc.insert(pos, " ");
                }

                mode.set(Normal);
            }
            (Normal, Sym('A')) => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                cursor.go_to_end_of_line(doc, mode.set(Insert));
            }
            (Normal, Sym('s')) => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                mode.set(Insert);
                doc.delete(cursor.to_pos(), cursor.to_pos());
            }
            (Normal, Sym('a')) => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                mode.set(Insert);
                cursor.right(doc, self.mode);
            }
            (Normal, Sym('o')) => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                doc.insert(cursor.clone().go_to_end_of_line(doc, Insert).to_pos(), "\n");
                cursor.down(doc).bound_col(doc, mode.set(Insert));
            }
            (Replace, Sym(ch)) => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                if !ch.is_control() {
                    doc.delete(cursor.to_pos(), cursor.to_pos());
                    doc.insert(cursor.to_pos(), &ch.to_string());
                }

                mode.set(Normal);
            }
            (Replace, _) => {
                mode.set(Normal);
            }
            (Normal, Sym('r')) => {
                mode.set(Replace);
            }
            (Normal, Sym('x')) => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                *clipboard = Clipboard::Normal(doc.delete(cursor.to_pos(), cursor.to_pos()));
                cursor.bound_col(doc, *mode);
            }

            (Insert, Escape) => {
                cursor.bound_col(doc, mode.set(Normal));
            }
            (Normal, Sym('v')) => {
                mode.set(Visual(cursor.to_pos()));
            }
            (Visual(_), Escape) => {
                mode.set(Normal);
            }
            (Visual(landmark), Sym('y')) => {
                let cursor_pos = cursor.clone().bound_col(doc, *mode).to_pos();
                *clipboard = Clipboard::Normal(doc.copy(landmark, cursor_pos));
                cursor
                    .go_to_pos(min(landmark, cursor_pos))
                    .bound_row(doc)
                    .bound_col(doc, *mode);
                mode.set(Normal);
            }
            (Visual(landmark), Sym('Y')) => {
                let cursor_pos = cursor.clone().bound_col(doc, *mode).to_pos();

                let (mut start, mut end) = if landmark < cursor_pos {
                    (landmark, cursor_pos)
                } else {
                    (cursor_pos, landmark)
                };

                start.col = 0;
                end.col = doc.col_bound(end.row, *mode);

                *clipboard = Clipboard::Line(doc.copy(start, end));
                cursor.go_to_pos(start).bound_row(doc).bound_col(doc, *mode);
                mode.set(Normal);
            }
            (Visual(landmark), Sym('D')) => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                let cursor_pos = cursor.clone().bound_col(doc, *mode).to_pos();

                let (mut start, mut end) = if landmark < cursor_pos {
                    (landmark, cursor_pos)
                } else {
                    (cursor_pos, landmark)
                };

                start.col = 0;
                end.col = doc.col_bound(end.row, *mode);

                *clipboard = Clipboard::Line(doc.delete(start, end));
                cursor.go_to_pos(start).bound_row(doc).bound_col(doc, *mode);
                mode.set(Normal);
            }
            (Visual(landmark), Sym('d')) => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                let cursor_pos = cursor.clone().bound_col(doc, *mode).to_pos();

                *clipboard = Clipboard::Normal(doc.delete(landmark, cursor_pos));
                cursor
                    .go_to_pos(min(landmark, cursor_pos))
                    .bound_row(doc)
                    .bound_col(doc, *mode);
                mode.set(Normal);
            }
            (Normal, Sym('P')) => {
                undo.push(doc.snapshot(), cursor.clone(), Normal);

                match clipboard {
                    Clipboard::Normal(s) => {
                        cursor.go_to_pos(doc.insert(cursor.clone().to_pos(), &s));
                    }
                    Clipboard::Line(s) => {
                        doc.insert(
                            Pos {
                                row: cursor.row,
                                col: 0,
                            },
                            &s,
                        );
                    }
                    _ => {}
                }
            }
            (Normal, Sym('p')) => {
                undo.push(doc.snapshot(), cursor.clone(), Normal);
                match clipboard {
                    Clipboard::Normal(s) => {
                        let end_pos = doc.insert(cursor.clone().right(doc, Insert).to_pos(), &s);
                        cursor.go_to_pos(end_pos);
                    }
                    Clipboard::Line(s) => {
                        if s.ends_with("\n") {
                            s.pop();
                        }

                        doc.insert(
                            cursor.clone().go_to_end_of_line(doc, Insert).to_pos(),
                            &format!("\n{s}"),
                        );
                        cursor.down(doc);
                    }
                    _ => {}
                }
            }
            (Delete, Sym('d')) => {
                undo.push(doc.snapshot(), cursor.clone(), Normal);
                let start = Pos {
                    row: cursor.row,
                    col: 0,
                };

                let end = cursor.clone().go_to_end_of_line(doc, Insert).to_pos();

                *clipboard = Clipboard::Line(doc.delete(start, end));
                cursor.bound_row(doc);
                mode.set(Normal);
            }
            (Normal, Sym('D')) => {
                undo.push(doc.snapshot(), cursor.clone(), Normal);
                *clipboard = Clipboard::Normal(doc.delete(
                    cursor.to_pos(),
                    cursor.clone().go_to_end_of_line(doc, Normal).to_pos(),
                ));
            }
            (Normal, Sym('Y')) => {
                *clipboard = Clipboard::Normal(doc.copy(
                    cursor.to_pos(),
                    cursor.clone().go_to_end_of_line(doc, Normal).to_pos(),
                ));
            }
            (Copy, Sym('y')) => {
                let start = Pos {
                    row: cursor.row,
                    col: 0,
                };

                let end = cursor.clone().go_to_end_of_line(doc, Insert).to_pos();

                *clipboard = Clipboard::Line(doc.copy(start, end));
                cursor.bound_row(doc);
                mode.set(Normal);
            }
            (Copy, Sym('j')) => {
                if doc.row_bound() <= cursor.row {
                    mode.set(Normal);
                    return Ok(true);
                }

                let start = Pos {
                    row: cursor.row,
                    col: 0,
                };
                let end = cursor
                    .clone()
                    .down(doc)
                    .go_to_end_of_line(doc, Insert)
                    .to_pos();

                *clipboard = Clipboard::Line(doc.copy(start, end));
                cursor.bound_row(doc);
                mode.set(Normal);
            }
            (Copy, Sym('k')) => {
                undo.push(doc.snapshot(), cursor.clone(), Normal);
                if cursor.row == 0 {
                    mode.set(Normal);
                    return Ok(true);
                }

                let start = Pos {
                    row: cursor.row - 1,
                    col: 0,
                };
                let end = cursor.clone().go_to_end_of_line(doc, Insert).to_pos();

                *clipboard = Clipboard::Line(doc.delete(start, end));
                cursor.up().bound_row(doc);
                mode.set(Normal);
            }
            (Delete, Sym('j')) => {
                undo.push(doc.snapshot(), cursor.clone(), Normal);
                if doc.row_bound() <= cursor.row {
                    mode.set(Normal);
                    return Ok(true);
                }

                let start = Pos {
                    row: cursor.row,
                    col: 0,
                };
                let end = cursor
                    .clone()
                    .down(doc)
                    .go_to_end_of_line(doc, Insert)
                    .to_pos();

                *clipboard = Clipboard::Line(doc.delete(start, end));
                cursor.bound_row(doc);
                mode.set(Normal);
            }
            (Delete, Sym('k')) => {
                undo.push(doc.snapshot(), cursor.clone(), Normal);
                if cursor.row == 0 {
                    mode.set(Normal);
                    return Ok(true);
                }

                let start = Pos {
                    row: cursor.row - 1,
                    col: 0,
                };
                let end = cursor.clone().go_to_end_of_line(doc, Insert).to_pos();

                *clipboard = Clipboard::Line(doc.delete(start, end));
                cursor.up().bound_row(doc);
                mode.set(Normal);
            }
            (Copy | Delete, _) => {
                mode.set(Normal);
            }
            (Normal, Sym('d')) => {
                mode.set(Delete);
            }
            (Normal, Sym('y')) => {
                mode.set(Copy);
            }
            (Normal, Sym('g')) => {
                let key = get_key_pressed()?;

                if key != Key::Sym('g') {
                    return self.handle_input(key);
                }

                cursor.go_to_first_line();
            }
            (Normal | Visual(_), Sym('G')) => {
                cursor.go_to_last_char(doc);
            }
            (Normal, Enter) => {
                cursor.down(doc).go_to_start_of_line(doc);
            }
            (Normal, Backspace) => {
                if cursor.col == 0 {
                    cursor.up().go_to_end_of_line(doc, *mode);
                    return Ok(true);
                }

                cursor.left();
            }
            (Insert, Enter) => {
                doc.insert(cursor.to_pos(), "\n");
                cursor.down(doc).bound_col(doc, *mode);
            }
            (Insert, Backspace) => {
                if cursor.row == 0 && cursor.col == 0 {
                    return Ok(true);
                }

                if cursor.col == 0 {
                    cursor.up().go_to_end_of_line(doc, Insert);
                    doc.delete(cursor.to_pos(), cursor.to_pos());
                    return Ok(true);
                }

                cursor.bound_col(doc, *mode).left();
                doc.delete(cursor.to_pos(), cursor.to_pos());
            }
            (Insert, Sym(ch)) if ch.is_control() => {}
            (Insert, Sym(ch)) => {
                doc.insert(cursor.to_pos(), &ch.to_string());
                cursor.right(doc, *mode);
            }
            _ => {}
        }

        Ok(true)
    }
}
