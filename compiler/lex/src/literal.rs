#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Base {
    Binary = 2,
    Octal = 8,
    Decimal = 10,
    Hex = 16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LiteralKind {
    Int { base: Base },
    Float { base: Base },
    Char,
    Str,
}
