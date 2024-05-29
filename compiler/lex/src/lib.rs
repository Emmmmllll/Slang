pub mod cursor;
pub mod literal;
pub mod token;

pub use cursor::Cursor;
pub use token::{Token, TokenKind};
pub use literal::{LiteralKind, Base};

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
