use ast::{
    token::{Token, TokenKind},
    tokenstream::{GroupSpacing, Spacing, TokenStream, TokenTree},
};
use source_idx::GroupSrcIdx;

use crate::lex::StringReader;

pub(super) struct TokenTreesReader<'src> {
    string_reader: StringReader<'src>,
    token: Token,
}

impl<'src> TokenTreesReader<'src> {
    pub(super) fn parse_all_token_trees(string_reader: StringReader) -> (TokenStream, Option<()>) {
        let mut tt_reader = TokenTreesReader {
            string_reader,
            token: Token::dummy(),
        };
        let (_spacing, stream, res) = tt_reader.parse_token_trees(false);
        (stream, res)
    }

    fn parse_token_trees(&mut self, starts_in_group: bool) -> (Spacing, TokenStream, Option<()>) {
        let (_, open_spacing) = self.next_token(false);

        let mut buf = Vec::new();
        loop {
            match self.token.kind {
                TokenKind::OpenDelim(delim) => {
                    buf.push(match self.parse_tt_open_delim(delim) {
                        Some(tt) => tt,
                        None => return (open_spacing, TokenStream::new(buf), None),
                    });
                }
                TokenKind::CloseDelim(_) => {
                    return (
                        open_spacing,
                        TokenStream::new(buf),
                        if starts_in_group { Some(()) } else { None },
                    );
                }
                TokenKind::Eof => return (open_spacing, TokenStream::new(buf), if starts_in_group { None } else { Some(()) } ),
                _ => {
                    let (this_tok, this_spacing) = self.next_token(true);
                    buf.push(TokenTree::SingleToken(this_tok, this_spacing))
                }
            }
        }
    }

    fn next_token(&mut self, glue: bool) -> (Token, Spacing) {
        let (this_spacing, next_tok) = loop {
            let (next_tok, preceding_whitespace) = self.string_reader.next_token();
            if preceding_whitespace {
                break (Spacing::Alone, next_tok);
            }
            let mut not_glued = false;
            if glue {
                if let Some(glued) = self.token.glue(&next_tok) {
                    self.token = glued;
                } else {
                    not_glued = true;
                }
            }
            if !glue || not_glued {
                let this_spacing = if next_tok.is_punct() {
                    Spacing::Joint
                } else if next_tok.kind == TokenKind::Eof {
                    Spacing::Alone
                } else {
                    Spacing::JointHidden
                };
                break (this_spacing, next_tok);
            }
        };
        let this_tok = std::mem::replace(&mut self.token, next_tok);
        (this_tok, this_spacing)
    }

    fn parse_tt_open_delim(&mut self, open_delim: ast::token::Delimiter) -> Option<TokenTree> {
        let start_src_data = self.token.src_data;

        let (open_spacing, tts, res) = self.parse_token_trees(true);
        res?;

        let group_src_data = GroupSrcIdx::from_pair(start_src_data, self.token.src_data);
        
        let close_spacing = match self.token.kind {
            // correct delimiter
            TokenKind::CloseDelim(delim) if delim == open_delim => {
                self.next_token(false).1
            }
            // incorrect delimiter
            TokenKind::CloseDelim(_delim) => {
                // TODO: Recover from unclosed delimiters
                Spacing::Alone
            }
            TokenKind::Eof => {
                Spacing::Alone
            }
            _ => unreachable!(),
        };

        let spacing = GroupSpacing::new(open_spacing, close_spacing);
        Some(TokenTree::TokenGroup(group_src_data, spacing, open_delim, tts))
    }
}
