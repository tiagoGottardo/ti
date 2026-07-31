use std::io::{self, Read, stdin};

const ESC_BYTE: u8 = 0x1b;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Sym(char),
    Alt(char),
    Enter,
    Backspace,
    Tab,
    Escape,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    PageUp,
    PageDown,
    // Delete,
    ScrollUp,
    ScrollDown,
    Unknown,
}

pub fn get_key_pressed() -> io::Result<Key> {
    let mut input = stdin();
    let first = read_byte_blocking(&mut input)?;

    if first == ESC_BYTE {
        return read_escape_sequence(&mut input);
    }

    read_plain_key(first, &mut input)
}

fn read_byte_blocking(input: &mut impl Read) -> io::Result<u8> {
    let mut byte = [0; 1];

    loop {
        match input.read(&mut byte)? {
            0 => continue,
            _ => return Ok(byte[0]),
        }
    }
}

fn read_byte_optional(input: &mut impl Read) -> io::Result<Option<u8>> {
    let mut byte = [0; 1];

    match input.read(&mut byte)? {
        0 => Ok(None),
        _ => Ok(Some(byte[0])),
    }
}

fn read_plain_key(first: u8, input: &mut impl Read) -> io::Result<Key> {
    match first {
        b'\r' | b'\n' => Ok(Key::Enter),
        b'\t' => Ok(Key::Tab),
        0x7f | 0x08 => Ok(Key::Backspace),
        byte if byte.is_ascii() => Ok(Key::Sym(byte as char)),
        byte => read_utf8_key(byte, input),
    }
}

fn read_utf8_key(first: u8, input: &mut impl Read) -> io::Result<Key> {
    let len = match first {
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return Ok(Key::Unknown),
    };

    let mut bytes = vec![first];
    while bytes.len() < len {
        let Some(byte) = read_byte_optional(input)? else {
            return Ok(Key::Unknown);
        };
        bytes.push(byte);
    }

    Ok(std::str::from_utf8(&bytes)
        .ok()
        .and_then(|s| s.chars().next())
        .map(Key::Sym)
        .unwrap_or(Key::Unknown))
}

fn read_escape_sequence(input: &mut impl Read) -> io::Result<Key> {
    let Some(first) = read_byte_optional(input)? else {
        return Ok(Key::Escape);
    };

    match first {
        b'[' => read_csi_sequence(input),
        b'O' => read_ss3_sequence(input),
        byte => read_plain_key(byte, input).map(|key| match key {
            Key::Sym(ch) => Key::Alt(ch),
            _ => Key::Unknown,
        }),
    }
}

fn read_ss3_sequence(input: &mut impl Read) -> io::Result<Key> {
    let Some(final_byte) = read_byte_optional(input)? else {
        return Ok(Key::Escape);
    };

    Ok(match final_byte {
        b'H' => Key::Home,
        b'F' => Key::End,
        b'A' => Key::ArrowUp,
        b'B' => Key::ArrowDown,
        b'C' => Key::ArrowRight,
        b'D' => Key::ArrowLeft,
        _ => Key::Unknown,
    })
}

fn read_csi_sequence(input: &mut impl Read) -> io::Result<Key> {
    let mut seq = Vec::new();

    while let Some(byte) = read_byte_optional(input)? {
        seq.push(byte);

        if (0x40..=0x7e).contains(&byte) {
            break;
        }
    }

    Ok(parse_csi_sequence(&seq))
}

fn parse_csi_sequence(seq: &[u8]) -> Key {
    match seq {
        [b'A'] => Key::ArrowUp,
        [b'B'] => Key::ArrowDown,
        [b'C'] => Key::ArrowRight,
        [b'D'] => Key::ArrowLeft,
        [b'H'] => Key::Home,
        [b'F'] => Key::End,
        [b'1' | b'7', b'~'] => Key::Home,
        [b'4' | b'8', b'~'] => Key::End,
        // [b'3', b'~'] => Key::Delete,
        [b'5', b'~'] => Key::PageUp,
        [b'6', b'~'] => Key::PageDown,
        [b'<', rest @ ..] => parse_sgr_mouse(rest),
        [b'M', rest @ ..] => parse_x10_mouse(rest),
        _ => Key::Unknown,
    }
}

fn parse_sgr_mouse(seq: &[u8]) -> Key {
    let Some((&final_byte, body)) = seq.split_last() else {
        return Key::Unknown;
    };

    if final_byte != b'M' && final_byte != b'm' {
        return Key::Unknown;
    }

    let Some(button_code) = body
        .split(|byte| *byte == b';')
        .next()
        .and_then(|part| std::str::from_utf8(part).ok())
        .and_then(|part| part.parse::<u8>().ok())
    else {
        return Key::Unknown;
    };

    scroll_key(button_code)
}

fn parse_x10_mouse(seq: &[u8]) -> Key {
    if seq.len() < 3 {
        return Key::Unknown;
    }

    scroll_key(seq[0].saturating_sub(32))
}

fn scroll_key(button_code: u8) -> Key {
    if button_code & 64 == 0 {
        return Key::Unknown;
    }

    match button_code & 1 {
        0 => Key::ScrollUp,
        _ => Key::ScrollDown,
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn read_key(bytes: &[u8]) -> Key {
        let mut input = Cursor::new(bytes);
        let first = read_byte_blocking(&mut input).unwrap();

        if first == ESC_BYTE {
            read_escape_sequence(&mut input).unwrap()
        } else {
            read_plain_key(first, &mut input).unwrap()
        }
    }

    #[test]
    fn it_reads_alt_ascii_key() {
        assert_eq!(read_key(b"\x1bj"), Key::Alt('j'));
    }

    #[test]
    fn it_reads_alt_utf8_key() {
        assert_eq!(read_key("\x1bé".as_bytes()), Key::Alt('é'));
    }

    #[test]
    fn it_ignores_alt_non_symbol_key() {
        assert_eq!(read_key(b"\x1b[1;3D"), Key::Unknown);
    }

    #[test]
    fn it_keeps_plain_escape() {
        assert_eq!(read_key(b"\x1b"), Key::Escape);
    }
}
