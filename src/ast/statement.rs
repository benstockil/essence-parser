use crate::ast::{
    Identifier,
    domain::Domain,
    expression::{Expression, Variable},
};

#[derive(Debug, Clone)]
pub enum Statement<V: Variable> {
    Language(String),
    Letting {
        declarations: Vec<LettingDeclaration<V>>,
    },
    Given {
        declarations: Vec<VariableDeclaration<V>>,
    },
    Where {
        constraints: Vec<Expression<V>>,
    },
    SuchThat {
        constraints: Vec<Expression<V>>,
    },
    Find {
        declarations: Vec<VariableDeclaration<V>>,
    },
}

#[derive(Debug, Clone)]
pub struct VariableDeclaration<V: Variable> {
    pub idents: Vec<Identifier>,
    pub domain: VariableDeclarationDomain<V>,
}

#[derive(Debug, Clone)]
pub enum VariableDeclarationDomain<V: Variable> {
    Domain(Domain<V>),
    Enum,
}

#[derive(Debug, Clone)]
pub struct LettingDeclaration<V: Variable> {
    pub idents: Vec<Identifier>,
    pub value: LettingDeclarationValue<V>,
}

#[derive(Debug, Clone)]
pub enum LettingDeclarationValue<V: Variable> {
    Expr(Expression<V>),
    Domain(Domain<V>),
    Enum { variants: Vec<Identifier> },
    Unnamed { size: Expression<V> },
}
