pub mod cursor;
pub mod literal;
pub mod token;

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
