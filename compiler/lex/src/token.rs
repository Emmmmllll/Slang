use crate::{
    cursor::Cursor,
    literal::{Base, LiteralKind},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub len: u32,
}

impl Token {
    pub fn new(kind: TokenKind, len: u32) -> Token {
        Token { kind, len }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// `// comment`
    LineComment,
    /// `/* block comment */`
    BlockComment,
    Whitespace,
    /// any identifier or keyword
    Ident,
    /// literal value e.g. `12u8` , `1.0e-4`, `b"test"`
    Literal {
        kind: LiteralKind,
    },
    /// `;`
    Semi,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,
    /// `[`
    OpenBracket,
    /// `]`
    CloseBracket,
    /// `@`
    At,
    /// `#`
    Hashtag,
    /// `~`
    Tilde,
    /// `?`
    Question,
    /// `:`
    Colon,
    /// `$`
    Dollar,
    /// `=`
    Eq,
    /// `!`
    Bang,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `-`
    Minus,
    /// `&`
    And,
    /// `|`
    Or,
    /// `+`
    Plus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `^`
    Peak,
    /// `%`
    Percent,

    // Unexpexted token
    Unknown,
    // End of input
    Eof,
}

/// True if `c` is considered a whitespace according to Rust language definition.
/// See [Rust language reference](https://doc.rust-lang.org/reference/whitespace.html)
/// for definitions of these classes.
pub fn is_whitespace(c: char) -> bool {
    // This is Pattern_White_Space.
    //
    // Note that this set is stable (ie, it doesn't change with different
    // Unicode versions), so it's ok to just hard-code the values.

    matches!(
        c,
        // Usual ASCII suspects
        '\u{0009}'   // \t
        | '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space

        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
    )
}

pub fn is_id_start(c: char) -> bool {
    c == '_' || unicode_xid::UnicodeXID::is_xid_start(c)
}

pub fn is_id_countinue(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_continue(c)
}

impl<'a> Cursor<'a> {
    // advances curser and returns the token
    pub fn next_token(&mut self) -> Token {
        use TokenKind::*;
        let Some(first_char) = self.take_char() else {
            return Token::new(TokenKind::Eof, 0);
        };
        let token_kind = match first_char {
            '/' => match self.peek_first_char() {
                '/' => self.line_comment(),
                '*' => self.block_comment(),
                _ => Slash,
            },
            c if is_whitespace(c) => self.whitespace(),

            // handle prefixes here
            c if is_id_start(c) => self.ident_or_unhandled_prefix(),

            c @ '0'..='9' => {
                let literal_kind = self.number(c);
                TokenKind::Literal { kind: literal_kind }
            }

            // One-symbol tokens
            ';' => Semi,
            ',' => Comma,
            '.' => Dot,
            '(' => OpenParen,
            ')' => CloseParen,
            '{' => OpenBrace,
            '}' => CloseBrace,
            '[' => OpenBracket,
            ']' => CloseBracket,
            '@' => At,
            '#' => Hashtag,
            '~' => Tilde,
            '?' => Question,
            ':' => Dollar,
            '=' => Eq,
            '!' => Bang,
            '<' => Lt,
            '>' => Gt,
            '-' => Minus,
            '&' => And,
            '|' => Or,
            '+' => Plus,
            '*' => Star,
            '^' => Peak,
            '%' => Percent,

            '\'' => self.char(),

            '"' => {
                if !self.double_quoted_string() {
                    todo!("handle non-terminated string")
                }
                Literal {
                    kind: LiteralKind::Str,
                }
            }
            _ => Unknown,
        };
        let res = Token::new(token_kind, self.pos_in_token());
        self.reset_pos_in_token();
        res
    }

    fn line_comment(&mut self) -> TokenKind {
        self.take_char();

        self.take_while(|c| c != '\n');
        TokenKind::LineComment
    }

    fn block_comment(&mut self) -> TokenKind {
        self.take_char();
        let mut depth = 1usize;
        while let Some(c) = self.take_char() {
            match c {
                '/' if self.peek_first_char() == '*' => {
                    self.take_char();
                    depth += 1;
                }
                '*' if self.peek_first_char() == '/' => {
                    self.take_char();
                    depth -= 1;
                    if depth == 0 {
                        // now the most outer block is closed
                        break;
                    }
                }
                _ => {}
            }
        }
        TokenKind::BlockComment
    }

    fn whitespace(&mut self) -> TokenKind {
        self.take_while(is_whitespace);
        TokenKind::Whitespace
    }

    fn ident_or_unhandled_prefix(&mut self) -> TokenKind {
        use unicode_properties::UnicodeEmoji;

        self.take_while(is_id_countinue);

        match self.peek_first_char() {
            '#' | '"' | '\'' => todo!("handle unkonwn prefix"),
            c if !c.is_ascii() && c.is_emoji_char() => todo!("handle invalid idents"),
            _ => TokenKind::Ident,
        }
    }

    fn number(&mut self, first_digit: char) -> LiteralKind {
        let mut base = Base::Decimal;
        if first_digit == '0' {
            match self.peek_first_char() {
                'b' => {
                    base = Base::Binary;
                    self.take_char();
                    if !self.take_decimal_digits() {
                        todo!("handle empty int")
                    }
                }
                'o' => {
                    base = Base::Octal;
                    self.take_char();
                    if !self.take_decimal_digits() {
                        todo!("handle empty int")
                    }
                }
                'x' => {
                    base = Base::Octal;
                    self.take_char();
                    if !self.take_hexadecimal_digits() {
                        todo!("handle empty int")
                    }
                }
                // Not a base prefix, take the other digits
                '0'..='9' | '_' => {
                    self.take_decimal_digits();
                }
                // also not a prefix, but do nothing
                '.' | 'e' | 'E' => {}

                // just a 0
                _ => return LiteralKind::Int { base },
            }
        } else {
            self.take_decimal_digits();
        }

        match self.peek_first_char() {
            // parse as float with a following dot
            // Attention: do not parse as float if it could be a range or a filed/member access
            '.' if self.peek_second_char() != '.' && !is_id_start(self.peek_second_char()) => {
                self.take_char();
                if self.peek_first_char().is_ascii_digit() {
                    // has decimal digits after `.`
                    self.take_decimal_digits();
                    match self.peek_first_char() {
                        'e' | 'E' => {
                            self.take_char();
                            if !self.take_float_exponent() {
                                todo!("handle empty float exponent")
                            }
                        }
                        _ => {}
                    }
                }
                LiteralKind::Float { base }
            }
            'e' | 'E' => {
                self.take_char();
                if !self.take_float_exponent() {
                    todo!("handle empty float exponent")
                }
                LiteralKind::Float { base }
            }
            _ => LiteralKind::Int { base },
        }
    }

    fn char(&mut self) -> TokenKind {
        if !self.single_queted_string() {
            todo!("Handle non-termitated char")
        }
        TokenKind::Literal {
            kind: LiteralKind::Char,
        }
    }

    fn single_queted_string(&mut self) -> bool {
        // check if literal has only one character
        if self.peek_second_char() == '\'' && self.peek_first_char() != '\\' {
            self.take_char();
            self.take_char();
            return true;
        }

        // Literal has more than one character e.g. '\xff'

        loop {
            match self.peek_first_char() {
                '\'' => {
                    self.take_char();
                    return true;
                }
                '\n' if self.peek_second_char() != '\'' => break,
                crate::cursor::EOF_CHAR if self.is_eof() => break,
                '\\' => {
                    self.take_char();
                    self.take_char();
                }
                _ => {
                    self.take_char();
                }
            }
        }
        // string was not terminated
        false
    }

    fn double_quoted_string(&mut self) -> bool {
        while let Some(c) = self.take_char() {
            match c {
                '"' => {
                    return true;
                }
                '\\' if self.peek_first_char() == '\\' || self.peek_first_char() == '"' => {
                    //skip the next character
                    self.take_char();
                }
                _ => {}
            }
        }
        false
    }

    fn take_decimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            match self.peek_first_char() {
                '_' => {
                    self.take_char();
                }
                '0'..='9' => {
                    has_digits = true;
                    self.take_char();
                }
                _ => break,
            }
        }
        has_digits
    }
    fn take_hexadecimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            match self.peek_first_char() {
                '_' => {
                    self.take_char();
                }
                '0'..='9' | 'a'..='f' | 'A'..='F' => {
                    has_digits = true;
                    self.take_char();
                }
                _ => break,
            }
        }
        has_digits
    }
    fn take_float_exponent(&mut self) -> bool {
        let first = self.peek_first_char();
        if first == '-' || first == '+' {
            self.take_char();
        }
        self.take_decimal_digits()
    }
}
