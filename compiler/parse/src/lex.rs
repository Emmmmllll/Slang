use ast::{
    token::{LitKind, Token, TokenKind},
    tokenstream::TokenStream,
};
use lex::{unescape::Mode, Cursor};
use source_idx::{BytePos, SrcData, Symbol};

use crate::tokentrees;

pub struct StringReader<'a> {
    cursor: lex::Cursor<'a>,
    src: &'a str,
    pos: BytePos,
    start_pos: BytePos,
}

pub fn parse_token_trees(mut source: &str, mut start_pos: BytePos) -> Option<TokenStream> {
    if let Some(shebang_len) = lex::strip_shebang(source) {
        source = &source[shebang_len..];
        start_pos = start_pos + BytePos::from_usize(shebang_len);
    }

    let cursor = Cursor::new(source);

    let string_reader = StringReader {
        cursor,
        src: source,
        pos: start_pos,
        start_pos,
    };
    let (stream, res) = tokentrees::TokenTreesReader::parse_all_token_trees(string_reader);
    res?;
    Some(stream)
}

impl<'a> StringReader<'a> {
    pub fn next_token(&mut self) -> (Token, bool) {
        let mut preceeded_by_whitespace = false;
        loop {
            // let str_before = self.cursor.as_str();
            let token = self.cursor.next_token();
            let start = self.pos;
            self.pos = self.pos + BytePos(token.len);

            let kind = match token.kind {
                lex::TokenKind::LineComment => TokenKind::Comment(ast::token::CommentType::Line),
                lex::TokenKind::BlockComment => TokenKind::Comment(ast::token::CommentType::Block),
                lex::TokenKind::Whitespace => {
                    preceeded_by_whitespace = true;
                    continue;
                }
                lex::TokenKind::Ident => self.ident(start),
                lex::TokenKind::Literal { kind } => {
                    let (kind, symbol) =
                        self.lexer_literal(start, start + BytePos(token.len), kind);
                    TokenKind::Literal(ast::token::Lit { kind, symbol })
                }
                lex::TokenKind::Semi => TokenKind::Semi,
                lex::TokenKind::Comma => TokenKind::Comma,
                lex::TokenKind::Dot => TokenKind::Dot,
                lex::TokenKind::OpenParen => {
                    TokenKind::OpenDelim(ast::token::Delimiter::Parenthesis)
                }
                lex::TokenKind::CloseParen => {
                    TokenKind::CloseDelim(ast::token::Delimiter::Parenthesis)
                }
                lex::TokenKind::OpenBrace => TokenKind::OpenDelim(ast::token::Delimiter::Brace),
                lex::TokenKind::CloseBrace => TokenKind::CloseDelim(ast::token::Delimiter::Brace),
                lex::TokenKind::OpenBracket => TokenKind::OpenDelim(ast::token::Delimiter::Bracket),
                lex::TokenKind::CloseBracket => {
                    TokenKind::CloseDelim(ast::token::Delimiter::Bracket)
                }
                lex::TokenKind::At => TokenKind::At,
                lex::TokenKind::Hashtag => TokenKind::Hashtag,
                lex::TokenKind::Tilde => TokenKind::Tilde,
                lex::TokenKind::Question => TokenKind::Question,
                lex::TokenKind::Colon => TokenKind::Colon,
                lex::TokenKind::Dollar => TokenKind::Dollar,
                lex::TokenKind::Eq => TokenKind::Eq,
                lex::TokenKind::Bang => TokenKind::Not,
                lex::TokenKind::Lt => TokenKind::Lt,
                lex::TokenKind::Gt => TokenKind::Gt,
                lex::TokenKind::Minus => TokenKind::BinOp(ast::token::BinOpToken::Minus),
                lex::TokenKind::And => TokenKind::BinOp(ast::token::BinOpToken::And),
                lex::TokenKind::Or => TokenKind::BinOp(ast::token::BinOpToken::Or),
                lex::TokenKind::Plus => TokenKind::BinOp(ast::token::BinOpToken::Plus),
                lex::TokenKind::Star => TokenKind::BinOp(ast::token::BinOpToken::Star),
                lex::TokenKind::Slash => TokenKind::BinOp(ast::token::BinOpToken::Slash),
                lex::TokenKind::Peak => TokenKind::BinOp(ast::token::BinOpToken::Peak),
                lex::TokenKind::Percent => TokenKind::BinOp(ast::token::BinOpToken::Percent),
                lex::TokenKind::Unknown => {
                    eprintln!("Unknown token occured");
                    continue;
                }
                lex::TokenKind::Eof => TokenKind::Eof,
            };
            let src_data = self.make_src_data(start, self.pos);
            return (Token::new(kind, src_data), preceeded_by_whitespace);
        }
    }

    #[inline]
    fn src_index(&self, index: BytePos) -> usize {
        (index - self.start_pos).to_usize()
    }
    fn str_from(&self, start: BytePos) -> &str {
        self.str_from_to(start, self.pos)
    }

    fn str_from_to(&self, start: BytePos, end: BytePos) -> &str {
        &self.src[self.src_index(start)..self.src_index(end)]
    }

    fn make_src_data(&self, lo: BytePos, hi: BytePos) -> SrcData {
        SrcData::with_root_ctxt(lo, hi)
    }

    fn lexer_literal(
        &self,
        start: BytePos,
        end: BytePos,
        kind: lex::LiteralKind,
    ) -> (LitKind, Symbol) {
        match kind {
            lex::LiteralKind::Char => self.lexer_unicode(LitKind::Char, Mode::Char, start, end, 1, 1),
            lex::LiteralKind::Str => self.lexer_unicode(LitKind::Str, Mode::Str, start, end, 1, 1),
            lex::LiteralKind::Int { base } => {
                let mut kind = LitKind::Int;
                if let lex::Base::Binary | lex::Base::Octal = base {
                    let base = base as u32;
                    let s = self.str_from_to(start + BytePos(2), end);
                    for (idx, c) in s.char_indices() {
                        let _src_data = self.make_src_data(
                            start + BytePos::from_usize(2 + idx),
                            start + BytePos::from_usize(2 + idx + c.len_utf8()),
                        );
                        if c != '_' && c.to_digit(base).is_none() {
                            eprintln!("Invalid Digit Literal");
                            kind = LitKind::Err;
                        }
                    }
                }
                (kind, self.symbol_from_to(start, end))
            }
            lex::LiteralKind::Float { base } => {
                let mut kind = LitKind::Float;
                if let lex::Base::Binary | lex::Base::Octal | lex::Base::Hex = base {
                    eprintln!("Unsupported Float Base");
                    kind = LitKind::Err;
                }
                (kind, self.symbol_from_to(start, end))
            }
        }
    }

    fn symbol_from_to(&self, start: BytePos, end: BytePos) -> Symbol {
        Symbol::get_or_store(self.str_from_to(start, end))
    }

    fn lexer_unicode(
        &self,
        kind: LitKind,
        mode: Mode,
        start: BytePos,
        end: BytePos,
        prefix_len: u32,
        postfix_len: u32,
    ) -> (LitKind, Symbol) {
        let content_start = start + BytePos(prefix_len);
        let content_end = end - BytePos(postfix_len);
        let lit_content = self.str_from_to(content_start, content_end);
        lex::unescape::unescape_unicode(lit_content, mode, &mut |range, res| {
            if let Err(err) = res {
                let (line, col) = get_line_of_char(start.to_usize(), self.src).unwrap();
                eprintln!(
                    "Unicode Escape Error: {:?} @ pos:{}:{} + {:?}",
                    err, line, col, range
                )
            }
        });
        let sym = if let LitKind::Err = kind {
            self.symbol_from_to(start, end)
        } else {
            Symbol::get_or_store(lit_content)
        };
        (kind, sym)
    }

    fn ident(&self, start: BytePos) -> TokenKind {
        let string = self.str_from(start);
        use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};
        let sym = match is_nfc_quick(string.chars()) {
            IsNormalized::Yes => Symbol::get_or_store(string),
            _ => {
                let normalized_str: String = string.chars().nfc().collect();
                Symbol::get_or_store(&normalized_str)
            }
        };
        let src_data = self.make_src_data(start, self.pos);
        // TODO: store ident
        TokenKind::Ident(sym)
    }
}

#[test]
fn parse_tt() {
    let source = include_str!("../../../mock.sl");
    let token_stream = parse_token_trees(source, BytePos(0)).unwrap();
    println!("{:#?}", token_stream);
}

fn get_line_of_char(idx: usize, src: &str) -> Option<(usize, usize)> {
    let preceding = src.get(..idx)?;
    let mut line_idx_offset = 0;
    let lines = preceding
        .lines()
        .inspect(|line| line_idx_offset += line.len())
        .count();
    Some((lines, idx - line_idx_offset))
}
