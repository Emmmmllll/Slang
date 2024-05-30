pub mod cursor;
pub mod literal;
pub mod token;
pub mod unescape;

pub use cursor::Cursor;
pub use token::{Token, TokenKind};
pub use literal::{LiteralKind, Base};

/// allows files to start with a shebang '#!/bin/slangc' which can be useful on Unix based systems
/// though it is ignored by the compiler
pub fn strip_shebang(src: &str) -> Option<usize> {
    if let Some(shebang) = src.strip_prefix("#!") {
        let next_non_whitespace_token = tokenize(shebang).map(|t| t.kind).find(|t|{
            !matches!(
                t,
                TokenKind::Whitespace
            )
        });
        // the next token is an `[` it can be an crate attribut and is valid code
        // e.g. `#![example_attribute_here]`
        if next_non_whitespace_token != Some(TokenKind::OpenBracket) {
            // it must be a shebang at this point
            return Some(2 + shebang.lines().next().unwrap_or_default().len());
        }
    }
    None
}

pub fn tokenize(src: &str) -> impl Iterator<Item = Token> + '_ {
    let mut cursor = Cursor::new(src);
    std::iter::from_fn(move || {
        let next_token = cursor.next_token();
        if next_token.kind != TokenKind::Eof { Some(next_token) } else { None }
    })
}

#[cfg(test)]
mod test {
    #[test]
    fn lex() {
        let file = include_str!("../../../mock.sl");
        let mut cursor = crate::cursor::Cursor::new(file);
        loop {
            let token = cursor.next_token();
            println!("{:?}", token);
            if let crate::token::TokenKind::Eof = token.kind {
                break;
            }
        }
    }
}
