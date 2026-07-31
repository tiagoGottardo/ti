use std::cmp::{Ordering, max, min};
use std::{
    env, fs,
    io::{self},
};

use crate::{app::Mode, error::TiError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos {
    pub row: usize,
    pub col: usize,
}

impl Ord for Pos {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.row.cmp(&other.row) {
            Ordering::Equal => self.col.cmp(&other.col),
            o => o,
        }
    }
}

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Document {
    pub file_path: String,
    pub lines: Vec<String>,
}

impl Document {
    pub fn new() -> anyhow::Result<Self> {
        let file_path = env::args()
            .nth(1)
            .ok_or_else(|| TiError("You need to provide the file path!".to_owned()))?;

        let mut lines = fs::read_to_string(&file_path)?;
        if let Some('\n') = lines.chars().last() {
            lines.pop();
        }

        Ok(Self {
            file_path: file_path,
            lines: lines
                .split("\n")
                .map(|line| line.to_owned())
                .collect::<Vec<String>>(),
        })
    }

    pub fn save(&self) -> io::Result<()> {
        let mut lines = self.lines.join("\n");
        lines.push('\n');
        fs::write(&self.file_path, lines)
    }

    pub fn insert(&mut self, pos: Pos, str: &str) -> Pos {
        let parts = str.split("\n").collect::<Vec<_>>();
        let byte_col = Self::char_to_byte_idx(&self.lines[pos.row], pos.col);

        let rest = self.lines[pos.row].drain(byte_col..).collect::<String>();

        self.lines[pos.row].insert_str(byte_col, parts[0]);

        parts.iter().skip(1).enumerate().for_each(|(i, s)| {
            self.lines.insert(pos.row + i + 1, (*s).to_owned());
        });

        let final_row = pos.row + parts.len() - 1;
        let final_pos = Pos {
            row: final_row,
            col: if parts.len() == 1 {
                pos.col + Self::char_len(parts[0])
            } else {
                Self::char_len(parts.last().unwrap())
            },
        };

        self.lines[final_row].push_str(&rest);

        final_pos
    }

    pub fn copy(&mut self, from: Pos, to: Pos) -> String {
        let (start, end) = if from < to { (from, to) } else { (to, from) };

        let mut result = String::new();

        let mut first_line = self.lines[start.row].clone();
        first_line.push('\n');

        if start.row == end.row {
            return first_line
                .chars()
                .skip(start.col)
                .take(end.col + 1 - start.col)
                .collect::<String>();
        }

        result.push_str(&first_line.chars().skip(start.col).collect::<String>());

        for i in start.row + 1..end.row {
            result.push_str(&self.lines[i].clone());
            result.push('\n');
        }

        let mut last_line = self.lines[end.row].clone();
        last_line.push('\n');

        result.push_str(&last_line.chars().take(end.col).collect::<String>());

        result
    }

    pub fn delete(&mut self, from: Pos, to: Pos) -> String {
        let start = min(from, to);
        let end = max(from, to);

        assert!(start.row < self.lines.len());
        assert!(end.row < self.lines.len());
        assert!(start.col <= Self::char_len(&self.lines[start.row]));
        assert!(end.col <= Self::char_len(&self.lines[end.row]));

        if start.row == end.row {
            return self.delete_same_line(start, end);
        }

        let mut result = String::new();
        let start_line_len = Self::char_len(&self.lines[start.row]);

        result.push_str(&Self::char_slice(
            &self.lines[start.row],
            start.col,
            start_line_len,
        ));
        result.push('\n');

        for row in start.row + 1..end.row {
            result.push_str(&self.lines[row]);
            result.push('\n');
        }

        let end_line_len = Self::char_len(&self.lines[end.row]);
        result.push_str(&Self::char_slice(
            &self.lines[end.row],
            0,
            min(end.col + 1, end_line_len),
        ));

        let deletes_end_newline = end.col == end_line_len;
        if deletes_end_newline {
            result.push('\n');
        }

        let prefix = Self::char_slice(&self.lines[start.row], 0, start.col);
        let suffix = if deletes_end_newline {
            self.lines.get(end.row + 1).cloned().unwrap_or_default()
        } else {
            Self::char_slice(&self.lines[end.row], end.col + 1, end_line_len)
        };
        let replacement = format!("{prefix}{suffix}");
        let remove_through = if deletes_end_newline && end.row + 1 < self.lines.len() {
            end.row + 1
        } else {
            end.row
        };

        if replacement.is_empty()
            && deletes_end_newline
            && end.row + 1 == self.lines.len()
            && start.col == 0
            && start.row > 0
        {
            self.lines.drain(start.row..=remove_through);
        } else {
            self.lines[start.row] = replacement;
            self.lines.drain(start.row + 1..=remove_through);
        }

        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        result
    }

    fn delete_same_line(&mut self, start: Pos, end: Pos) -> String {
        let line_len = Self::char_len(&self.lines[start.row]);
        let deletes_newline = end.col == line_len;
        let mut result = Self::char_slice(
            &self.lines[start.row],
            start.col,
            min(end.col + 1, line_len),
        );

        if deletes_newline {
            result.push('\n');
        }

        let prefix = Self::char_slice(&self.lines[start.row], 0, start.col);

        if deletes_newline {
            if start.row + 1 < self.lines.len() {
                let next_line = self.lines.remove(start.row + 1);
                self.lines[start.row] = format!("{prefix}{next_line}");
            } else if prefix.is_empty() && start.row > 0 {
                self.lines.remove(start.row);
            } else {
                self.lines[start.row] = prefix;
            }
        } else {
            let suffix = Self::char_slice(&self.lines[start.row], end.col + 1, line_len);
            self.lines[start.row] = format!("{prefix}{suffix}");
        }

        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        result
    }

    fn char_len(line: &str) -> usize {
        line.chars().count()
    }

    fn char_slice(line: &str, start: usize, end: usize) -> String {
        line.chars().skip(start).take(end - start).collect()
    }

    fn char_to_byte_idx(line: &str, col: usize) -> usize {
        line.char_indices()
            .map(|(idx, _)| idx)
            .nth(col)
            .unwrap_or(line.len())
    }

    pub fn insert_line(&mut self, row: usize) {
        self.lines.insert(row, String::new())
    }

    pub fn col_bound(&self, row: usize, mode: Mode) -> usize {
        let line_len = Self::char_len(&self.lines[row]);

        match mode {
            Mode::Insert | Mode::Visual(_) => line_len,
            _ => max(line_len as isize - 1, 0) as usize,
        }
    }

    pub fn row_bound(&self) -> usize {
        max(self.lines.len() as isize - 1, 0) as usize
    }

    pub fn next_word(&self, pos: Pos) -> Pos {
        let chars = self.indexed_chars();
        let Some(mut idx) = self.pos_idx(pos, &chars) else {
            return pos;
        };

        let state = CharState::from(chars[idx].1);

        while idx < chars.len() - 1 && state.matches(chars[idx].1) {
            idx += 1;
        }

        while idx < chars.len() - 1 && chars[idx].1.is_whitespace() {
            idx += 1;
        }

        chars[idx].0
    }

    pub fn prev_word(&self, pos: Pos) -> Pos {
        let chars = self.indexed_chars();
        let Some(idx) = self.pos_idx(pos, &chars) else {
            return pos;
        };

        if idx == 0 {
            return chars[idx].0;
        }

        let mut idx = idx - 1;

        while idx > 0 && chars[idx].1.is_whitespace() {
            idx -= 1;
        }

        let state = CharState::from(chars[idx].1);

        while idx > 0 && state.matches(chars[idx - 1].1) {
            idx -= 1;
        }

        chars[idx].0
    }

    pub fn last_char_of_next_word(&self, pos: Pos) -> Pos {
        let chars = self.indexed_chars();
        let Some(mut idx) = self.pos_idx(pos, &chars) else {
            return pos;
        };

        if idx < chars.len() - 1 {
            idx += 1;
        }

        while idx < chars.len() - 1 && chars[idx].1.is_whitespace() {
            idx += 1;
        }

        let state = CharState::from(chars[idx].1);

        while idx < chars.len() - 1 && state.matches(chars[idx + 1].1) {
            idx += 1;
        }

        chars[idx].0
    }

    fn indexed_chars(&self) -> Vec<(Pos, char)> {
        let mut chars = Vec::new();

        for (row, line) in self.lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                chars.push((Pos { row, col }, ch));
            }

            if row + 1 < self.lines.len() {
                chars.push((
                    Pos {
                        row,
                        col: line.chars().count(),
                    },
                    '\n',
                ));
            }
        }

        if chars.is_empty() {
            chars.push((Pos { row: 0, col: 0 }, '\n'));
        }

        chars
    }

    fn pos_idx(&self, pos: Pos, chars: &[(Pos, char)]) -> Option<usize> {
        if self.lines.is_empty() {
            return chars.len().checked_sub(1);
        }

        let row = pos.row.min(self.lines.len().saturating_sub(1));
        let col = pos.col.min(self.lines[row].chars().count());
        let bounded_pos = Pos { row, col };

        chars
            .iter()
            .position(|(char_pos, _)| *char_pos == bounded_pos)
            .or_else(|| chars.len().checked_sub(1))
    }

    pub fn snapshot(&self) -> Vec<String> {
        self.lines.clone()
    }

    pub fn restore(&mut self, lines: Vec<String>) {
        self.lines = lines;
    }
}

enum CharState {
    Alphabetic,
    NonAlphabetic,
    WhiteSpace,
}

impl CharState {
    fn from(ch: char) -> Self {
        if ch.is_alphanumeric() {
            Self::Alphabetic
        } else if ch.is_whitespace() {
            Self::WhiteSpace
        } else {
            Self::NonAlphabetic
        }
    }

    fn matches(&self, ch: char) -> bool {
        match self {
            Self::Alphabetic => ch.is_alphanumeric(),
            Self::NonAlphabetic => !ch.is_alphanumeric() && !ch.is_whitespace(),
            Self::WhiteSpace => ch.is_whitespace(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_joinlines_correctly() {
        let mut doc = Document {
            file_path: "monster.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "asdf".to_owned()],
        };

        let pos = Pos { row: 0, col: 4 };

        doc.delete(pos.clone(), pos);

        assert_eq!(doc.lines, vec!["asdfasdf".to_owned()]);
    }

    #[test]
    fn it_delete_content_in_the_same_line_without_join() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "asdf".to_owned()],
        };

        let start = Pos { row: 0, col: 0 };
        let end = Pos { row: 0, col: 2 };

        doc.delete(start, end);

        assert_eq!(doc.lines, vec!["f".to_owned(), "asdf".to_owned()]);
    }

    #[test]
    #[should_panic]
    fn it_should_crash() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "asdf".to_owned()],
        };

        let start = Pos { row: 0, col: 10 };
        let end = Pos { row: 0, col: 2 };

        doc.delete(start, end);
    }

    #[test]
    fn it_delete_content_in_the_same_line_and_join() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "asdf".to_owned()],
        };

        let start = Pos { row: 0, col: 2 };
        let end = Pos { row: 0, col: 4 };

        doc.delete(start, end);

        assert_eq!(doc.lines, vec!["asasdf".to_owned()]);
    }

    #[test]
    fn it_delete_multiple_lines() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec![
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
            ],
        };

        let start = Pos { row: 0, col: 4 };
        let end = Pos { row: 2, col: 4 };

        doc.delete(start, end);

        assert_eq!(doc.lines, vec!["asdfasdf".to_owned()]);
    }

    #[test]
    fn it_delete_multiple_lines_and_drain() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec![
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
            ],
        };

        let start = Pos { row: 0, col: 4 };
        let end = Pos { row: 2, col: 2 };

        doc.delete(start, end);

        assert_eq!(doc.lines, vec!["asdff".to_owned(), "asdf".to_owned()]);
    }

    #[test]
    fn it_drain_and_delete_multiple_lines() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec![
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
            ],
        };

        let start = Pos { row: 0, col: 2 };
        let end = Pos { row: 2, col: 4 };

        doc.delete(start, end);

        assert_eq!(doc.lines, vec!["asasdf".to_owned()]);
    }

    #[test]
    fn it_drain_and_delete_multiple_lines2() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec![
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
            ],
        };

        let start = Pos { row: 0, col: 2 };
        let end = Pos { row: 2, col: 1 };

        doc.delete(start, end);

        assert_eq!(doc.lines, vec!["asdf".to_owned(), "asdf".to_owned()]);
    }

    #[test]
    fn it_insert_inline_content() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "----".to_owned()],
        };

        let pos = doc.insert(Pos { row: 0, col: 2 }, "mise");

        assert_eq!(doc.lines, vec!["asmisedf".to_owned(), "----".to_owned()]);
        assert_eq!(pos, Pos { row: 0, col: 6 })
    }

    #[test]
    fn it_insert_inline_utf8_content() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["açb".to_owned()],
        };

        let pos = doc.insert(Pos { row: 0, col: 2 }, "é");

        assert_eq!(doc.lines, vec!["açéb".to_owned()]);
        assert_eq!(pos, Pos { row: 0, col: 3 })
    }

    #[test]
    fn it_insert_multiple_line_content() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "----".to_owned()],
        };

        let pos = doc.insert(Pos { row: 0, col: 2 }, "mise\n123");

        assert_eq!(
            doc.lines,
            vec!["asmise".to_owned(), "123df".to_owned(), "----".to_owned()]
        );
        assert_eq!(pos, Pos { row: 1, col: 3 })
    }

    #[test]
    fn it_insert_new_line_with_content() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "----".to_owned()],
        };

        let pos = doc.insert(Pos { row: 0, col: 4 }, "\n123");

        assert_eq!(
            doc.lines,
            vec!["asdf".to_owned(), "123".to_owned(), "----".to_owned()]
        );
        assert_eq!(pos, Pos { row: 1, col: 3 })
    }

    #[test]
    fn it_insert_content_and_new_line() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "----".to_owned()],
        };

        let pos = doc.insert(Pos { row: 0, col: 4 }, "123\n");

        assert_eq!(
            doc.lines,
            vec!["asdf123".to_owned(), "".to_owned(), "----".to_owned()]
        );
        assert_eq!(pos, Pos { row: 1, col: 0 })
    }

    #[test]
    fn it_goes_to_next_word_on_same_line() {
        let doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["one two".to_owned()],
        };

        assert_eq!(
            doc.next_word(Pos { row: 0, col: 0 }),
            Pos { row: 0, col: 4 }
        );
    }

    #[test]
    fn it_goes_to_next_word_across_lines() {
        let doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["one".to_owned(), "two".to_owned()],
        };

        assert_eq!(
            doc.next_word(Pos { row: 0, col: 1 }),
            Pos { row: 1, col: 0 }
        );
    }

    #[test]
    fn it_goes_to_previous_word_across_lines() {
        let doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["one".to_owned(), "two".to_owned()],
        };

        assert_eq!(
            doc.prev_word(Pos { row: 1, col: 1 }),
            Pos { row: 1, col: 0 }
        );
        assert_eq!(
            doc.prev_word(Pos { row: 1, col: 0 }),
            Pos { row: 0, col: 0 }
        );
    }

    #[test]
    fn it_goes_to_last_char_of_next_word() {
        let doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["one two".to_owned()],
        };

        assert_eq!(
            doc.last_char_of_next_word(Pos { row: 0, col: 0 }),
            Pos { row: 0, col: 2 }
        );
        assert_eq!(
            doc.last_char_of_next_word(Pos { row: 0, col: 2 }),
            Pos { row: 0, col: 6 }
        );
    }

    #[test]
    fn it_treats_punctuation_as_word_motion_group() {
        let doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["one... two".to_owned()],
        };

        assert_eq!(
            doc.next_word(Pos { row: 0, col: 0 }),
            Pos { row: 0, col: 3 }
        );
        assert_eq!(
            doc.next_word(Pos { row: 0, col: 3 }),
            Pos { row: 0, col: 7 }
        );
        assert_eq!(
            doc.last_char_of_next_word(Pos { row: 0, col: 2 }),
            Pos { row: 0, col: 5 }
        );
    }

    #[test]
    fn it_drain_and_delete_multiple_lines_and_return_buf() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec![
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
            ],
        };

        let start = Pos { row: 0, col: 2 };
        let end = Pos { row: 2, col: 4 };

        assert_eq!(doc.delete(start, end), "df\nasdf\nasdf\n".to_owned());
    }

    #[test]
    fn it_drain_and_delete_multiple_lines_and_return_buf2() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec![
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
            ],
        };

        let start = Pos { row: 0, col: 2 };
        let end = Pos { row: 2, col: 1 };

        assert_eq!(doc.delete(start, end), "df\nasdf\nas".to_owned());
    }

    #[test]
    fn it_drain_and_return_buf() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec![
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
            ],
        };

        let start = Pos { row: 2, col: 1 };
        let end = Pos { row: 2, col: 2 };

        assert_eq!(doc.delete(start, end), "sd".to_owned());
    }

    #[test]
    fn it_deletes_utf8_content_by_character_column() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["açéb".to_owned()],
        };

        let start = Pos { row: 0, col: 1 };
        let end = Pos { row: 0, col: 2 };

        assert_eq!(doc.delete(start, end), "çé".to_owned());
        assert_eq!(doc.lines, vec!["ab".to_owned()]);
    }

    #[test]
    fn it_drain_and_return_buf2() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec![
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
            ],
        };

        let start = Pos { row: 2, col: 1 };
        let end = Pos { row: 2, col: 4 };

        assert_eq!(doc.delete(start, end), "sdf\n".to_owned());
    }

    #[test]
    fn it_deletes_last_empty_line() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec![
                "I".to_owned(),
                "am".to_owned(),
                "back".to_owned(),
                "".to_owned(),
            ],
        };

        let start = Pos { row: 3, col: 0 };
        let end = Pos { row: 3, col: 0 };

        let deleted = doc.delete(start, end);

        assert_eq!(
            doc.lines,
            vec!["I".to_owned(), "am".to_owned(), "back".to_owned()]
        );
        assert_eq!(deleted, "\n".to_owned());
    }

    #[test]
    fn it_deletes_last_two_empty_lines() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec![
                "I".to_owned(),
                "am".to_owned(),
                "back".to_owned(),
                "".to_owned(),
                "".to_owned(),
            ],
        };

        let start = Pos { row: 3, col: 0 };
        let end = Pos { row: 4, col: 0 };

        let deleted = doc.delete(start, end);

        assert_eq!(
            doc.lines,
            vec!["I".to_owned(), "am".to_owned(), "back".to_owned()]
        );
        assert_eq!(deleted, "\n\n".to_owned());
    }
}
