use std::{ops::Range, str::Chars};

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Char,
    Str,
}

#[derive(Debug)]
pub enum EscapeError {
    ZeroChars,
    EscapeOnlyChar,
    RawCarrigeReturn,
    MoreThanOneChar,
    LonlyBackSlash,
    TooShortHexEscape,
    InvalidCharInHexEscape,
    InvalidEscape,
    NoBraceInUnicodeEscape,
    NonAsciiCharInByte,
    MultipleSkippedLinesWarning,
}

pub fn unescape_unicode(
    s: &str,
    mode: Mode,
    cb: &mut impl FnMut(Range<usize>, Result<char, EscapeError>),
) {
    match mode {
        Mode::Char => {
            let mut chars = s.chars();
            let res = unescape_char(&mut chars, mode);
            cb(0..(s.len() - chars.as_str().len()), res);
        }
        Mode::Str => unescape_str(s, Mode::Str, cb),
    }
}

fn unescape_char(chars: &mut Chars, mode: Mode) -> Result<char, EscapeError> {
    let c = chars.next().ok_or(EscapeError::ZeroChars)?;
    let res = match c {
        '\\' => scan_escape(chars, mode),
        '\n' | '\t' | '\'' => Err(EscapeError::EscapeOnlyChar),
        '\r' => Err(EscapeError::RawCarrigeReturn),
        _ => ascii_check(c, true),
    }?;
    if chars.next().is_some() {
        return Err(EscapeError::MoreThanOneChar);
    }
    Ok(res)
}

fn ascii_check(c: char, allow_unicode: bool) -> Result<char, EscapeError> {
    if allow_unicode || c.is_ascii() {
        Ok(c)
    } else {
        Err(EscapeError::NonAsciiCharInByte)
    }
}

fn scan_escape<T: From<char> + From<u8>>(chars: &mut Chars, mode: Mode) -> Result<T, EscapeError> {
    let res: char = match chars.next().ok_or(EscapeError::LonlyBackSlash)? {
        '"' => '"',
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        '\\' => '\\',
        '\'' => '\'',
        '0' => '\0',
        'x' => {
            let hi = chars.next().ok_or(EscapeError::TooShortHexEscape)?;
            let hi = hi.to_digit(16).ok_or(EscapeError::InvalidCharInHexEscape)?;

            let lo = chars.next().ok_or(EscapeError::TooShortHexEscape)?;
            let lo = lo.to_digit(16).ok_or(EscapeError::InvalidCharInHexEscape)?;

            let value = ((hi << 4) + lo) as u8;

            return Ok(T::from(value));
        }
        'u' => {
            return scan_unicode(chars, true);
        }
        _ => return Err(EscapeError::InvalidEscape),
    };

    Ok(T::from(res))
}

fn scan_unicode<T: From<char> + From<u8>>(
    chars: &mut Chars,
    allow_unicode: bool,
) -> Result<T, EscapeError> {
    if chars.next() != Some('{') {
        return Err(EscapeError::NoBraceInUnicodeEscape);
    }

    todo!("unicode escape not yet implemented")
}

fn unescape_str<T: From<char> + From<u8>>(
    s: &str,
    mode: Mode,
    cb: &mut impl FnMut(Range<usize>, Result<T, EscapeError>),
) {
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        let start = s.len() - chars.as_str().len() - c.len_utf8();
        let res = match c {
            '\\' => match chars.clone().next() {
                Some('\n') => {
                    skip_ascii_whitespace(&mut chars, start, &mut |range, err| cb(range, Err(err)));
                    continue;
                }
                _ => scan_escape::<T>(&mut chars, mode),
            },
            '"' => Err(EscapeError::EscapeOnlyChar),
            '\r' => Err(EscapeError::RawCarrigeReturn),
            _ => ascii_check(c, true).map(T::from)
        };
        let end = s.len() - chars.as_str().len();
        cb(start..end, res);
    }
}

fn skip_ascii_whitespace(
    chars: &mut Chars,
    start: usize,
    cb: &mut impl FnMut(Range<usize>, EscapeError),
) {
    let tail = chars.as_str();
    let first_non_space = tail
        .bytes()
        .position(|b| b.is_ascii_whitespace())
        .unwrap_or(tail.len());
    if tail[1..first_non_space].contains('\n') {
        let end = start + first_non_space + 1;
        cb(start..end, EscapeError::MultipleSkippedLinesWarning);
    }
    let tail = &tail[first_non_space..];
    *chars = tail.chars();
}
