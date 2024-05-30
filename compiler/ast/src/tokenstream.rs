use std::rc::Rc;

use crate::token::{Delimiter, Token};
use source_idx::GroupSrcIdx;

#[derive(Debug)]
pub struct TokenStream( pub Rc<Vec<TokenTree>>);

impl TokenStream {
    pub fn new(tts: Vec<TokenTree>) -> TokenStream {
        TokenStream(Rc::new(tts))
    }
}

#[derive(Debug)]
pub enum TokenTree {
    SingleToken(Token, Spacing),
    TokenGroup(GroupSrcIdx, GroupSpacing, Delimiter, TokenStream)
}

#[derive(Debug)]
pub enum Spacing {
    Alone,
    Joint,
    JointHidden,
}

#[derive(Debug)]
pub struct GroupSpacing {
    open: Spacing,
    close: Spacing,
}

impl GroupSpacing {
    pub fn new(open: Spacing, close: Spacing) -> GroupSpacing {
        GroupSpacing{open, close}
    }
}