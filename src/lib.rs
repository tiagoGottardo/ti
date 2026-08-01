pub mod app;
pub mod cell;
pub mod cursor;
pub mod document;
pub mod error;
pub mod key;
pub mod render_buffer;
pub mod terminal;
pub mod undo;
pub mod viewport;

pub const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H";
pub const HIDE_CURSOR: &str = "\x1b[?25l";
pub const SHOW_CURSOR: &str = "\x1b[?25h";
pub const ENABLE_MOUSE: &str = "\x1b[?1000h\x1b[?1006h";
pub const DISABLE_MOUSE: &str = "\x1b[?1000l\x1b[?1006l";

#[macro_export]
macro_rules! prin {
    ($($arg:tt)*) => {{
        print!($($arg)*);
        std::io::Write::flush(&mut std::io::stdout()).expect("Error on flush prin!");
    }};
}
