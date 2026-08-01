use std::{env, fs, io, path::PathBuf};

use crate::document::Pos;

#[derive(Clone, PartialEq, Copy, Debug, Eq, Hash)]
pub enum Color {
    Accent,
    Cursor,
    Foreground,
    Background,
    SelectionForeground,
    SelectionBackground,
    CurrentLine,
    CurrentLineNumber,
    BarForeground,
    BarBackground,
    FileName,
    Whitespace,
    Color0,
    Color1,
    Color2,
    Color3,
    Color4,
    Color5,
    Color6,
    Color7,
    Color8,
    Color9,
    Color10,
    Color11,
    Color12,
    Color13,
    Color14,
    Color15,
}

#[derive(Clone, PartialEq, Copy, Debug)]
pub enum ThemeColor {
    Default,
    Rgb(u8, u8, u8),
}

#[derive(Clone, Debug)]
pub struct Theme {
    accent: ThemeColor,
    cursor: ThemeColor,
    foreground: ThemeColor,
    background: ThemeColor,
    selection_foreground: ThemeColor,
    selection_background: ThemeColor,
    current_line: ThemeColor,
    current_line_number: ThemeColor,
    bar_foreground: ThemeColor,
    bar_background: ThemeColor,
    file_name: ThemeColor,
    whitespace: ThemeColor,
    colors: [ThemeColor; 16],
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: ThemeColor::Rgb(127, 127, 255),
            cursor: ThemeColor::Default,
            foreground: ThemeColor::Rgb(235, 219, 178),
            background: ThemeColor::Rgb(40, 40, 40),
            selection_foreground: ThemeColor::Rgb(251, 241, 199),
            selection_background: ThemeColor::Rgb(80, 73, 69),
            current_line: ThemeColor::Rgb(60, 56, 54),
            current_line_number: ThemeColor::Rgb(250, 189, 47),
            bar_foreground: ThemeColor::Rgb(235, 219, 178),
            bar_background: ThemeColor::Rgb(80, 73, 69),
            file_name: ThemeColor::Rgb(250, 189, 47),
            whitespace: ThemeColor::Rgb(102, 92, 84),
            colors: [
                ThemeColor::Rgb(40, 40, 40),
                ThemeColor::Rgb(204, 36, 29),
                ThemeColor::Rgb(152, 151, 26),
                ThemeColor::Rgb(215, 153, 33),
                ThemeColor::Rgb(69, 133, 136),
                ThemeColor::Rgb(177, 98, 134),
                ThemeColor::Rgb(104, 157, 106),
                ThemeColor::Rgb(168, 153, 132),
                ThemeColor::Rgb(146, 131, 116),
                ThemeColor::Rgb(251, 73, 52),
                ThemeColor::Rgb(184, 187, 38),
                ThemeColor::Rgb(250, 189, 47),
                ThemeColor::Rgb(131, 165, 152),
                ThemeColor::Rgb(211, 134, 155),
                ThemeColor::Rgb(142, 192, 124),
                ThemeColor::Rgb(235, 219, 178),
            ],
        }
    }
}

impl Theme {
    fn get(&self, color: Color) -> ThemeColor {
        match color {
            Color::Accent => self.accent,
            Color::Cursor => self.cursor,
            Color::Foreground => self.foreground,
            Color::Background => self.background,
            Color::SelectionForeground => self.selection_foreground,
            Color::SelectionBackground => self.selection_background,
            Color::CurrentLine => self.current_line,
            Color::CurrentLineNumber => self.current_line_number,
            Color::BarForeground => self.bar_foreground,
            Color::BarBackground => self.bar_background,
            Color::FileName => self.file_name,
            Color::Whitespace => self.whitespace,
            Color::Color0 => self.colors[0],
            Color::Color1 => self.colors[1],
            Color::Color2 => self.colors[2],
            Color::Color3 => self.colors[3],
            Color::Color4 => self.colors[4],
            Color::Color5 => self.colors[5],
            Color::Color6 => self.colors[6],
            Color::Color7 => self.colors[7],
            Color::Color8 => self.colors[8],
            Color::Color9 => self.colors[9],
            Color::Color10 => self.colors[10],
            Color::Color11 => self.colors[11],
            Color::Color12 => self.colors[12],
            Color::Color13 => self.colors[13],
            Color::Color14 => self.colors[14],
            Color::Color15 => self.colors[15],
        }
    }

    fn set(&mut self, key: &str, color: ThemeColor) {
        match key {
            "accent" => self.accent = color,
            "cursor" => self.cursor = color,
            "foreground" | "fg" => self.foreground = color,
            "background" | "bg" => self.background = color,
            "selectionforeground" | "selectionfg" => self.selection_foreground = color,
            "selectionbackground" | "selectionbg" => self.selection_background = color,
            "currentline" | "cursorline" => self.current_line = color,
            "currentlinenumber" | "cursorlinenumber" => self.current_line_number = color,
            "barforeground" | "barfg" | "statusforeground" | "statusfg" => {
                self.bar_foreground = color
            }
            "barbackground" | "barbg" | "statusbackground" | "statusbg" => {
                self.bar_background = color
            }
            "filename" | "fileforeground" | "filefg" => self.file_name = color,
            "whitespace" | "space" | "spaces" | "listchars" => self.whitespace = color,
            key if key.starts_with("color") => {
                if let Ok(index) = key["color".len()..].parse::<usize>() {
                    if let Some(slot) = self.colors.get_mut(index) {
                        *slot = color;
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Clone, PartialEq, Copy, Debug)]
pub struct Cell {
    pub char: char,
    pub fg: Color,
    pub bg: Color,
    pub italic: bool,
    pub bold: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            char: ' ',
            fg: Color::Foreground,
            bg: Color::Background,
            italic: false,
            bold: false,
        }
    }
}

impl Cell {
    pub fn new(char: char) -> Self {
        Self {
            char,
            ..Default::default()
        }
    }

    pub fn build(&self, pos: Pos, theme: &Theme) -> String {
        let mut style = String::new();
        if self.bold {
            style.push_str("\x1b[1m");
        }
        if self.italic {
            style.push_str("\x1b[3m");
        }

        style.push_str(&ansi_color(theme.get(self.fg), true));
        style.push_str(&ansi_color(theme.get(self.bg), false));

        format!("\x1b[{};{}H{}{}\x1b[0m", pos.row, pos.col, style, self.char)
    }

    pub fn selected(&mut self) -> &mut Self {
        self.fg = Color::SelectionForeground;
        self.bg = Color::SelectionBackground;
        self
    }
}

pub fn get_theme() -> io::Result<Theme> {
    let mut theme = Theme::default();
    let path = theme_path();

    let Ok(contents) = fs::read_to_string(path) else {
        return Ok(theme);
    };

    for line in contents.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        let key = normalize_key(key);
        let value = clean_value(value);

        if let Some(color) = parse_color(&value) {
            theme.set(&key, color);
        }
    }

    Ok(theme)
}

fn theme_path() -> PathBuf {
    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(config_home).join("ti/colors.toml");
    }

    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join(".config/ti/colors.toml");
    }

    PathBuf::from("colors.toml")
}

fn normalize_key(key: &str) -> String {
    key.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn clean_value(value: &str) -> String {
    let mut cleaned = String::new();
    let mut quote = None;

    for ch in value.chars() {
        match (ch, quote) {
            ('#', None) => break,
            ('"' | '\'', None) => quote = Some(ch),
            (quote_char, Some(active_quote)) if quote_char == active_quote => quote = None,
            _ => cleaned.push(ch),
        }
    }

    cleaned.trim().to_owned()
}

fn parse_color(value: &str) -> Option<ThemeColor> {
    if value.eq_ignore_ascii_case("none") || value.eq_ignore_ascii_case("default") {
        return Some(ThemeColor::Default);
    }

    parse_rgb(value).map(|(r, g, b)| ThemeColor::Rgb(r, g, b))
}

fn parse_rgb(value: &str) -> Option<(u8, u8, u8)> {
    let value = value.strip_prefix('#')?;
    if value.len() != 6 || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }

    let r = u8::from_str_radix(&value[0..2], 16).ok()?;
    let g = u8::from_str_radix(&value[2..4], 16).ok()?;
    let b = u8::from_str_radix(&value[4..6], 16).ok()?;

    Some((r, g, b))
}

fn ansi_color(color: ThemeColor, foreground: bool) -> String {
    match color {
        ThemeColor::Default if foreground => "\x1b[39m".to_owned(),
        ThemeColor::Default => "\x1b[49m".to_owned(),
        ThemeColor::Rgb(r, g, b) if foreground => format!("\x1b[38;2;{r};{g};{b}m"),
        ThemeColor::Rgb(r, g, b) => format!("\x1b[48;2;{r};{g};{b}m"),
    }
}

#[cfg(test)]
mod tests {
    use super::{ThemeColor, clean_value, normalize_key, parse_color};

    #[test]
    fn parses_neovim_style_default_colors() {
        assert_eq!(parse_color("NONE"), Some(ThemeColor::Default));
        assert_eq!(parse_color("default"), Some(ThemeColor::Default));
    }

    #[test]
    fn parses_upper_and_lowercase_hex_colors() {
        assert_eq!(parse_color("#0a1B2c"), Some(ThemeColor::Rgb(10, 27, 44)));
    }

    #[test]
    fn cleans_quoted_values_before_parsing() {
        assert_eq!(clean_value("\"#AABBCC\" # comment"), "#AABBCC");
        assert_eq!(clean_value("'NONE'"), "NONE");
    }

    #[test]
    fn normalizes_theme_keys() {
        assert_eq!(normalize_key("selection_foreground"), "selectionforeground");
        assert_eq!(normalize_key("Color_12"), "color12");
    }
}
