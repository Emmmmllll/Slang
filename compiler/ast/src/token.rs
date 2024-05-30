use source_idx::{SrcData, Symbol, DUMMY_SRC_DATA};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TokenKind {
    // Expression operators
    /// `=`
    Eq,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `==`
    EqEq,
    /// `!=`
    Ne,
    /// `>=`
    Ge,
    /// `>`
    Gt,
    /// `&&`
    AndAnd,
    /// `||`
    OrOr,
    /// `!`
    Not,
    /// `~`
    Tilde,
    BinOp(BinOpToken),
    BinOpEq(BinOpToken),

    // Constructing symbols
    /// `@`
    At,
    /// `.`
    Dot,
    /// `..`
    DotDot,
    /// `...`
    DotDotDot,
    /// `..=`
    DotDotEq,
    /// `,`
    Comma,
    /// `;`
    Semi,
    /// `:`
    Colon,
    /// `::`
    DoubleColon,
    /// `->`
    RArrow,
    /// `<-`
    LArrow,
    /// `=>`
    FatArrow,
    /// `#`
    Hashtag,
    /// `$`
    Dollar,
    /// `?`
    Question,
    /// `'`
    SingleQote,
    /// An opening delimiter (e.g. `{`)
    OpenDelim(Delimiter),
    /// An closing delimiter (e.g. `{`)
    CloseDelim(Delimiter),

    // Literals
    Literal(Lit),

    // comments
    Comment(CommentType),

    Ident(Symbol),

    /// End of File
    Eof
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub src_data: SrcData,
}

impl TokenKind {
    pub fn lit(kind: LitKind, symbol: Symbol) -> TokenKind {
        TokenKind::Literal(Lit { kind, symbol })
    }

    pub fn split_double_token(&self) -> Option<(TokenKind, TokenKind)> {
        use TokenKind::*;
        use BinOpToken::*;
        Some(match *self {
            Le => (Lt, Eq),
            EqEq => (Eq, Eq),
            Ne => (Not, Eq),
            Ge => (Gt, Eq),
            AndAnd => (BinOp(And), BinOp(And)),
            OrOr => (BinOp(Or), BinOp(Or)),
            BinOpEq(Plus) => (BinOp(Plus), Eq),
            BinOpEq(Minus) => (BinOp(Minus), Eq),
            BinOpEq(Star) => (BinOp(Star), Eq),
            BinOpEq(Slash) => (BinOp(Slash), Eq),
            BinOpEq(Percent) => (BinOp(Percent), Eq),
            BinOpEq(Peak) => (BinOp(Peak), Eq),
            BinOpEq(And) => (BinOp(And), Eq),
            BinOpEq(Or) => (BinOp(Or), Eq),
            BinOpEq(Shl) => (BinOp(Shl), Eq),
            BinOpEq(Shr) => (BinOp(Shr), Eq),
            DotDot => (Dot, Dot),
            DotDotDot => (Dot, DotDot),
            DoubleColon => (Colon, Colon),
            RArrow => (BinOp(Minus), Gt),
            LArrow => (Lt, BinOp(Minus)),
            FatArrow => (Eq, Gt),
            _ => return None,
        })
    }
}

impl Token {
    pub fn new(kind: TokenKind, src_data: SrcData) -> Token {
        Token { kind, src_data }
    }

    pub fn dummy() -> Token {
        Token::new(TokenKind::Question, DUMMY_SRC_DATA)
    }

    pub fn glue(&self, next: &Token) -> Option<Token> {
        use TokenKind::*;
        let kind = match (self.kind, next.kind) {
            (Eq, Eq) => EqEq,
            (Eq, Gt) => FatArrow,
            
            (Lt, Eq) => Le,
            (Lt, Lt) => BinOp(BinOpToken::Shl),
            (Lt, Le) => BinOpEq(BinOpToken::Shl),
            (Lt, BinOp(BinOpToken::Minus)) => LArrow,

            (Gt, Eq) => Ge,
            (Gt, Gt) => BinOp(BinOpToken::Shr),
            (Gt, Ge) => BinOpEq(BinOpToken::Shr),

            (Not, Eq) => Ne,

            (BinOp(op), Eq) => BinOpEq(op),
            (BinOp(BinOpToken::And), BinOp(BinOpToken::And)) => AndAnd,
            (BinOp(BinOpToken::Or), BinOp(BinOpToken::Or)) => OrOr,
            (BinOp(BinOpToken::Minus), Gt) => RArrow,
            
            (Dot, Dot) => DotDot,
            (Dot, DotDot) => DotDotDot,
            
            (DotDot, Dot) => DotDotDot,
            (DotDot, Eq) => DotDotEq,
            
            (Colon, Colon) => DoubleColon,

            // TODO: (SingleQote, Ident(name)) => Lifetime



            _ => return None
        };

        Some(Token::new(kind, self.src_data.combine(next.src_data)))
    }
    
    pub fn is_punct(&self) -> bool {
        use TokenKind::*;
        match self.kind {
            Eq | Lt | Le | EqEq | Ne | Ge | Gt | AndAnd | OrOr | Not | Tilde | BinOp(_)
            | BinOpEq(_) | At | Dot | DotDot | DotDotDot | DotDotEq | Comma | Semi | Colon
            | DoubleColon | RArrow | LArrow | FatArrow | Hashtag | Dollar | Question | SingleQote => true,
            
            OpenDelim(_) | CloseDelim(_) | Literal(_) | Comment(_) | Ident(_) | Eof => false,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CommentType {
    Line,
    Block,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BinOpToken {
    // Binary operators
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `^`
    Peak,
    /// `&`
    And,
    /// `|`
    Or,
    /// `<<`
    Shl,
    /// `>>`
    Shr,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Delimiter {
    /// `( ... )`
    Parenthesis,
    /// `{ ... }`
    Brace,
    /// `[ ... ]`
    Bracket,
    /// produced by macros / compiler
    Invisible,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Lit {
    pub kind: LitKind,
    pub symbol: Symbol,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LitKind {
    Char,
    Str,
    Float,
    Int,
    Err
}