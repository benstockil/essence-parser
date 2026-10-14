use crate::{
    span::Span,
    token::{Token, TokenType},
};

#[derive(Debug, thiserror::Error)]
#[error("{kind} (at {span})")]
pub struct ParseError {
    pub span: Span,
    pub kind: ParseErrorKind,
}

impl ParseError {
    /// Build an [`ParseErrorKind::Expected`] error, taking its span from the
    /// offending token.
    pub fn expected(what: &'static str, token: Token) -> Self {
        ParseError {
            span: token.span,
            kind: ParseErrorKind::Expected(what, token.ty),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseErrorKind {
    #[error("expected {0}, got {1:?}")]
    Expected(&'static str, TokenType),
    #[error("unexpected end of file")]
    UnexpectedEof,
}
