use crate::{
    ast::expression::{BinaryOperator, Comparison},
    span::Span,
};

#[derive(Debug, Clone)]
pub struct Token {
    pub ty: TokenType,
    pub span: Span,
}

impl Token {
    pub fn to_binop(&self) -> Option<BinaryOperator> {
        let binop = match self.ty {
            TokenType::Plus => BinaryOperator::Add,
            TokenType::Minus => BinaryOperator::Subtract,
            TokenType::Asterisk => BinaryOperator::Multiply,
            TokenType::Divide => BinaryOperator::Divide,
            TokenType::Modulo => BinaryOperator::Modulo,
            TokenType::Exponent => BinaryOperator::Exponent,
            TokenType::Equals => BinaryOperator::Equals,
            TokenType::NotEquals => BinaryOperator::NotEquals,
            TokenType::Or => BinaryOperator::Or,
            TokenType::And => BinaryOperator::And,
            TokenType::Implies => BinaryOperator::Implies,
            TokenType::Iff => BinaryOperator::Iff,
            TokenType::LessThanEq => BinaryOperator::Compare {
                ty: Comparison::LessThanEq,
            },
            TokenType::GreaterThanEq => BinaryOperator::Compare {
                ty: Comparison::GreaterThanEq,
            },
            TokenType::LessThan => BinaryOperator::Compare {
                ty: Comparison::LessThan,
            },
            TokenType::GreaterThan => BinaryOperator::Compare {
                ty: Comparison::GreaterThan,
            },
            TokenType::LessThanEqLex => BinaryOperator::LexCompare {
                ty: Comparison::LessThanEq,
            },
            TokenType::GreaterThanEqLex => BinaryOperator::LexCompare {
                ty: Comparison::GreaterThanEq,
            },
            TokenType::LessThanLex => BinaryOperator::LexCompare {
                ty: Comparison::LessThan,
            },
            TokenType::GreaterThanLex => BinaryOperator::LexCompare {
                ty: Comparison::GreaterThan,
            },
            _ => return None,
        };
        Some(binop)
    }
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Identifier(String),
    IntegerValue(u64),
    True,
    False,

    // Symbols
    Ampersand,
    Colon,
    DoubleColon,
    LongRightArrow,
    LeftArrow,
    Period,
    DoublePeriod,
    Comma,
    Underscore,

    // Arithmetic
    Plus,
    Minus,
    Asterisk,
    Divide,
    Modulo,
    Exponent,

    // Boolean
    Equals,
    NotEquals,
    Not,
    Or,
    And,
    Implies,
    Iff,

    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    // Comparison
    LessThanEq,
    GreaterThanEq,
    LessThan,
    GreaterThan,

    LessThanEqLex,
    GreaterThanEqLex,
    LessThanLex,
    GreaterThanLex,

    // Keywords
    Language(String),
    Letting,
    Given,
    SuchThat,
    Where,
    Find,
    Be,
    Of,
    From,
    In,
    Domain,
    IndexedBy,
    NewTypeEnum,

    // Quantifiers
    ForAll,
    Exists,
    Sum,
    Min,
    Max,

    // Set operators
    Union,
    Intersect,
    Subset,
    SubsetEq,
    Supset,
    SupsetEq,

    // Sequence operators
    Substring,
    Subsequence,

    // Dominance
    Maximising,
    Minimising,

    // Domains
    Int,
    Bool,
    Set,
    MSet,
    Tuple,
    Matrix,
    Record,
    Variant,
    Sequence,
    Function,
    Relation,
    Partition,
    Permutation,
}

impl TokenType {
    pub fn at(self, line: u32, col: u32, length: u32) -> Token {
        Token {
            span: Span { line, col, length },
            ty: self,
        }
    }
}
