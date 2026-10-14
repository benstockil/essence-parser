use crate::{
    ast::{Identifier, domain::Domain},
    types::EssenceType,
};

pub trait Variable {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct VariableId(usize);

#[derive(Debug, Clone)]
pub struct TypedVariableId(pub VariableId, pub EssenceType);

impl Variable for Identifier {}
impl Variable for VariableId {}
impl Variable for TypedVariableId {}

#[derive(Debug, Clone)]
pub enum Expression<V: Variable> {
    Variable(V),
    IntegerValue(u64),
    BooleanValue(bool),
    UnOp(UnaryOperator, Box<Self>),
    BinOp(BinaryOperator, Box<Self>, Box<Self>),
    Aggregate(Aggregate<V>),
    Indexing(Box<Self>, Vec<Index<V>>),
    FunctionCall {
        function: Box<Expression<V>>,
        arguments: Vec<Expression<V>>,
    },

    SetLiteral {
        members: Vec<Self>,
    },
    MultiSetLiteral {
        members: Vec<Self>,
    },
    SequenceLiteral {
        members: Vec<Self>,
    },
    TupleLiteral {
        members: Vec<Self>,
    },
    MatrixLiteral {
        members: Vec<Self>,
        domain: Option<Domain<V>>,
    },
    FunctionLiteral {
        pairs: Vec<FunctionLiteralPair<V>>,
    },
    RelationLiteral {
        pairs: Vec<Vec<Self>>,
    },
    PartitionLiteral {
        partitions: Vec<Vec<Self>>,
    },
    PermutationLiteral {
        cycles: Vec<Vec<Self>>,
    },
    RecordLiteral {
        pairs: Vec<NameValuePair<V>>,
    },
    VariantLiteral {
        pair: Box<NameValuePair<V>>,
    },
}

#[derive(Debug, Clone)]
pub enum Index<V: Variable> {
    Null,
    Value(Box<Expression<V>>),
}

#[derive(Debug, Clone)]
pub struct Aggregate<V: Variable> {
    pub ty: AggregateType,
    pub variables: Vec<AggregateVariable>,
    pub scope: AggregateScope<V>,
    pub expr: Box<Expression<V>>,
}

#[derive(Debug, Clone)]
pub enum AggregateVariable {
    Placeholder,
    Variable(Identifier),
    Tuple(Vec<AggregateVariable>),
}

#[derive(Debug, Clone)]
pub enum AggregateType {
    ForAll,
    Exists,
    Sum,
    Min,
    Max,
}

#[derive(Debug, Clone)]
pub enum AggregateScope<V: Variable> {
    Collection(Box<Expression<V>>),
    Domain(Domain<V>),
}

#[derive(Debug, Clone)]
pub struct NameValuePair<V: Variable> {
    pub name: Identifier,
    pub value: Expression<V>,
}

#[derive(Debug, Clone)]
pub struct FunctionLiteralPair<V: Variable> {
    pub input: Expression<V>,
    pub output: Expression<V>,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperator {
    Not,
    Negative,
    Absolute,
    Cardinality,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Divide,
    Multiply,
    Modulo,
    Exponent,

    // Boolean
    And,
    Or,
    Implies,
    Iff,
    Equals,
    NotEquals,

    // Comparison
    Compare { ty: Comparison },
    LexCompare { ty: Comparison },

    // Set
    Union,
    Intersect,

    // Set Comparison
    In,
    Subset,
    SubsetEq,
    Supset,
    SupsetEq,

    // Sequence
    Substring,
    Subsequence,
}

impl BinaryOperator {
    // mostly from rust
    pub fn precedence(&self) -> u64 {
        match self {
            BinaryOperator::Exponent => 10,
            BinaryOperator::Divide | BinaryOperator::Multiply | BinaryOperator::Modulo => 9,
            BinaryOperator::Add | BinaryOperator::Subtract => 8,
            BinaryOperator::Equals
            | BinaryOperator::NotEquals
            | BinaryOperator::Compare { .. }
            | BinaryOperator::LexCompare { .. } => 7,
            BinaryOperator::Union => 6,
            BinaryOperator::Intersect => 5,
            BinaryOperator::In
            | BinaryOperator::Subset
            | BinaryOperator::SubsetEq
            | BinaryOperator::Supset
            | BinaryOperator::SupsetEq
            | BinaryOperator::Substring
            | BinaryOperator::Subsequence => 4,
            BinaryOperator::And | BinaryOperator::Or => 3,
            BinaryOperator::Implies => 2,
            BinaryOperator::Iff => 1,
        }
    }

    pub fn is_right_associative(&self) -> bool {
        matches!(self, BinaryOperator::Exponent)
    }
}

// pub enum Associativity {
//     Left,
//     Right,
//     None,
// }

#[derive(Debug, Clone, Copy)]
pub enum Comparison {
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
}
