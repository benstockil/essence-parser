use crate::ast::{expression::Variable, statement::Statement};

pub mod domain;
pub mod expression;
pub mod statement;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Identifier(pub String);

#[derive(Debug, Clone)]
pub struct Program<V: Variable>(pub Vec<Statement<V>>);
