use std::str::Chars;

pub struct Cursor<'a> {
    remaining_len: usize,
    chars: Chars<'a>,
}

pub(crate) const EOF_CHAR: char = '\0';

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Cursor<'a> {
        Cursor {
            remaining_len: input.len(),
            chars: input.chars(),
        }
    }

    pub fn as_str(&self) -> &'a str {
        self.chars.as_str()
    }

    pub fn take_char(&mut self) -> Option<char> {
        self.chars.next()
    }

    pub fn take_while(&mut self, mut f: impl FnMut(char) -> bool) {
        while f(self.peek_first_char()) && !self.is_eof() {
            self.take_char();
        }
    }

    pub fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    pub fn peek_first_char(&mut self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }
    pub fn peek_second_char(&mut self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }
    pub fn peek_third_char(&mut self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }

    pub fn pos_in_token(&self) -> u32 {
        (self.remaining_len - self.chars.as_str().len()) as u32
    }
    pub fn reset_pos_in_token(&mut self) {
        self.remaining_len = self.chars.as_str().len();
    }
}
