use crate::ast::{
    Identifier,
    expression::{Expression, Variable},
};

#[derive(Debug, Clone)]
pub enum Domain<V: Variable> {
    Identifier(Identifier),
    Bool,
    Int {
        ranges: Vec<IntRange<V>>,
    },
    Tuple {
        member_domains: Vec<Self>,
    },
    Matrix {
        indexed_by: Vec<Self>,
        element_domain: Box<Self>,
    },
    Record {
        pairs: Vec<NameDomainPair<V>>,
    },
    Variant {
        pairs: Vec<NameDomainPair<V>>,
    },
    Set {
        attributes: Vec<DomainAttribute<V>>,
        member_domain: Box<Self>,
    },
    MultiSet {
        attributes: Vec<DomainAttribute<V>>,
        member_domain: Box<Self>,
    },
    Sequence {
        attributes: Vec<DomainAttribute<V>>,
        member_domain: Box<Self>,
    },
    Function {
        attributes: Vec<DomainAttribute<V>>,
        input_domain: Box<Self>,
        output_domain: Box<Self>,
    },
    Relation {
        attributes: Vec<DomainAttribute<V>>,
        domains: Vec<Self>,
    },
    Partition {
        attributes: Vec<DomainAttribute<V>>,
        member_domain: Box<Self>,
    },
    Permutation {
        attributes: Vec<DomainAttribute<V>>,
        member_domain: Box<Self>,
    },
}

#[derive(Debug, Clone)]
pub enum IntRange<V: Variable> {
    Single(Expression<V>),
    Range {
        lower_bound: Option<Expression<V>>,
        upper_bound: Option<Expression<V>>,
    },
}

#[derive(Debug, Clone)]
pub struct DomainAttribute<V: Variable> {
    pub key: String,
    pub value: Option<Expression<V>>,
}

#[derive(Debug, Clone)]
pub struct NameDomainPair<V: Variable> {
    pub name: Identifier,
    pub domain: Domain<V>,
}
