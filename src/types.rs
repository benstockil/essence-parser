use std::collections::HashMap;

use ena::unify::{EqUnifyValue, InPlaceUnificationTable};

use crate::ast::{
    Identifier, Program,
    domain::Domain,
    expression::{
        AggregateScope, AggregateType, AggregateVariable, BinaryOperator, Expression,
        TypedVariableId, UnaryOperator, VariableId,
    },
    statement::{
        LettingDeclaration, LettingDeclarationValue, Statement, VariableDeclaration,
        VariableDeclarationDomain,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar(u32);

impl ena::unify::UnifyKey for TypeVar {
    type Value = Option<TypeVar>;

    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(u: u32) -> Self {
        Self(u)
    }

    fn tag() -> &'static str {
        "TypeVar"
    }
}

impl EqUnifyValue for TypeVar {}

#[derive(Debug, Clone)]
pub enum EssenceType {
    TypeVar(TypeVar),
    Int,
    Bool,
    Set(Box<EssenceType>),
    MultiSet(Box<EssenceType>),
    Sequence(Box<EssenceType>),
    Tuple(Vec<EssenceType>),
    Matrix(Box<EssenceType>),
    Function {
        input: Box<EssenceType>,
        output: Box<EssenceType>,
    },
    Relation(Vec<EssenceType>),
    Partition(Box<EssenceType>),
    Permutation(Box<EssenceType>),
    Record(Vec<(Identifier, EssenceType)>),
    Variant(Identifier, Box<EssenceType>),
    Domain,
}

#[derive(Debug, Clone)]
pub struct TypeInference {
    unification_table: InPlaceUnificationTable<TypeVar>,
}

impl TypeInference {
    pub fn new() -> Self {
        Self {
            unification_table: InPlaceUnificationTable::new(),
        }
    }

    fn fresh_ty_var(&mut self) -> TypeVar {
        self.unification_table.new_key(None)
    }

    pub fn infer_program(
        &mut self,
        ctx: &mut im::HashMap<VariableId, EssenceType>,
        program: Program<VariableId>,
    ) -> GenOut<Program<TypedVariableId>> {
        let (constraints, typed_stmts): (Vec<_>, Vec<_>) = program
            .0
            .into_iter()
            .map(|s| {
                let out = self.infer_statement(ctx, s);
                (out.constraints, out.typed_ast)
            })
            .unzip();

        GenOut::new(
            constraints.into_iter().flatten().collect(),
            Program(typed_stmts),
        )
    }

    pub fn infer_statement(
        &mut self,
        ctx: &mut im::HashMap<VariableId, EssenceType>,
        stmt: Statement<VariableId>,
    ) -> GenOut<Statement<TypedVariableId>> {
        match stmt {
            Statement::Language(l) => GenOut::new(vec![], Statement::Language(l)),
            Statement::Letting { declarations } => {
                let (decl_outs, _decl_tys): (Vec<_>, Vec<_>) = declarations
                    .into_iter()
                    .map(|d| {
                        let (constraints, ast, ty) = match d.value {
                            LettingDeclarationValue::Expr(expr) => {
                                let (expr_out, expr_ty) = self.infer_expr(ctx.clone(), expr);
                                (
                                    expr_out.constraints,
                                    LettingDeclarationValue::Expr(expr_out.typed_ast),
                                    expr_ty,
                                )
                            }
                            LettingDeclarationValue::Domain(domain) => {
                                let (domain_out, domain_ty) =
                                    self.infer_domain(ctx.clone(), domain);
                                (
                                    domain_out.constraints,
                                    LettingDeclarationValue::Domain(domain_out.typed_ast),
                                    domain_ty,
                                )
                            }
                            LettingDeclarationValue::Enum { variants } => (
                                vec![],
                                LettingDeclarationValue::Enum { variants },
                                EssenceType::Domain,
                            ),
                            LettingDeclarationValue::Unnamed { size } => {
                                let size_out = self.check_expr(ctx.clone(), size, EssenceType::Int);
                                (
                                    size_out.constraints,
                                    LettingDeclarationValue::Unnamed {
                                        size: size_out.typed_ast,
                                    },
                                    EssenceType::Domain,
                                )
                            }
                        };

                        (
                            GenOut::new(
                                constraints,
                                LettingDeclaration {
                                    idents: d.idents,
                                    value: ast,
                                },
                            ),
                            ty,
                        )
                    })
                    .unzip();

                let (constraints, declarations): (Vec<_>, Vec<_>) = decl_outs
                    .into_iter()
                    .map(|o| (o.constraints, o.typed_ast))
                    .unzip();

                GenOut::new(
                    constraints.into_iter().flatten().collect(),
                    Statement::Letting { declarations },
                )
            }
            Statement::Given { declarations } => {
                let (decl_outs, _decl_tys): (Vec<_>, Vec<_>) = declarations
                    .into_iter()
                    .map(|d| {
                        let (constraints, ast, ty) = match d.domain {
                            VariableDeclarationDomain::Domain(domain) => {
                                let (out, ty) = self.infer_domain(ctx.clone(), domain);
                                (
                                    out.constraints,
                                    VariableDeclarationDomain::Domain(out.typed_ast),
                                    ty,
                                )
                            }
                            VariableDeclarationDomain::Enum => {
                                (vec![], VariableDeclarationDomain::Enum, EssenceType::Domain)
                            }
                        };

                        (
                            GenOut::new(
                                constraints,
                                VariableDeclaration {
                                    idents: d.idents,
                                    domain: ast,
                                },
                            ),
                            ty,
                        )
                    })
                    .unzip();

                let (constraints, declarations): (Vec<_>, Vec<_>) = decl_outs
                    .into_iter()
                    .map(|o| (o.constraints, o.typed_ast))
                    .unzip();

                GenOut::new(
                    constraints.into_iter().flatten().collect(),
                    Statement::Given { declarations },
                )
            }
            Statement::Where { constraints: exprs } => {
                let (constraints, typed_exprs): (Vec<_>, Vec<_>) = exprs
                    .into_iter()
                    .map(|expr| {
                        let out = self.check_expr(ctx.clone(), expr, EssenceType::Bool);
                        (out.constraints, out.typed_ast)
                    })
                    .unzip();

                GenOut::new(
                    constraints.into_iter().flatten().collect(),
                    Statement::Where {
                        constraints: typed_exprs,
                    },
                )
            }
            Statement::SuchThat { constraints: exprs } => {
                let (constraints, typed_exprs): (Vec<_>, Vec<_>) = exprs
                    .into_iter()
                    .map(|expr| {
                        let out = self.check_expr(ctx.clone(), expr, EssenceType::Bool);
                        (out.constraints, out.typed_ast)
                    })
                    .unzip();

                GenOut::new(
                    constraints.into_iter().flatten().collect(),
                    Statement::SuchThat {
                        constraints: typed_exprs,
                    },
                )
            }
            Statement::Find { declarations } => {
                let (decl_outs, _decl_tys): (Vec<_>, Vec<_>) = declarations
                    .into_iter()
                    .map(|d| {
                        let (constraints, ast, ty) = match d.domain {
                            VariableDeclarationDomain::Domain(domain) => {
                                let (out, ty) = self.infer_domain(ctx.clone(), domain);
                                (
                                    out.constraints,
                                    VariableDeclarationDomain::Domain(out.typed_ast),
                                    ty,
                                )
                            }
                            VariableDeclarationDomain::Enum => {
                                (vec![], VariableDeclarationDomain::Enum, EssenceType::Domain)
                            }
                        };

                        (
                            GenOut::new(
                                constraints,
                                VariableDeclaration {
                                    idents: d.idents,
                                    domain: ast,
                                },
                            ),
                            ty,
                        )
                    })
                    .unzip();

                let (constraints, declarations): (Vec<_>, Vec<_>) = decl_outs
                    .into_iter()
                    .map(|o| (o.constraints, o.typed_ast))
                    .unzip();

                GenOut::new(
                    constraints.into_iter().flatten().collect(),
                    Statement::Find { declarations },
                )
            }
        }
    }

    fn infer_expr(
        &mut self,
        ctx: im::HashMap<VariableId, EssenceType>,
        expr: Expression<VariableId>,
    ) -> (GenOut<Expression<TypedVariableId>>, EssenceType) {
        match expr {
            Expression::Variable(v) => {
                let ty = &ctx[&v];
                (
                    GenOut::new(vec![], Expression::Variable(TypedVariableId(v, ty.clone()))),
                    ty.clone(),
                )
            }
            Expression::IntegerValue(i) => (
                GenOut::new(vec![], Expression::IntegerValue(i)),
                EssenceType::Int,
            ),
            Expression::BooleanValue(b) => (
                GenOut::new(vec![], Expression::BooleanValue(b)),
                EssenceType::Bool,
            ),
            Expression::UnOp(op, expr) => match op {
                UnaryOperator::Not => {
                    let input_out = self.check_expr(ctx, *expr, EssenceType::Bool);
                    (
                        GenOut::new(
                            input_out.constraints,
                            Expression::UnOp(op, Box::new(input_out.typed_ast)),
                        ),
                        EssenceType::Bool,
                    )
                }
                UnaryOperator::Absolute | UnaryOperator::Negative => {
                    let input_out = self.check_expr(ctx, *expr, EssenceType::Int);
                    (
                        GenOut::new(
                            input_out.constraints,
                            Expression::UnOp(op, Box::new(input_out.typed_ast)),
                        ),
                        EssenceType::Int,
                    )
                }
                UnaryOperator::Cardinality => {
                    let set_ty =
                        EssenceType::Set(Box::new(EssenceType::TypeVar(self.fresh_ty_var())));
                    let input_out = self.check_expr(ctx, *expr, set_ty);
                    (
                        GenOut::new(
                            input_out.constraints,
                            Expression::UnOp(op, Box::new(input_out.typed_ast)),
                        ),
                        EssenceType::Int,
                    )
                }
            },
            Expression::BinOp(op, left, right) => match op {
                BinaryOperator::Add
                | BinaryOperator::Subtract
                | BinaryOperator::Divide
                | BinaryOperator::Multiply
                | BinaryOperator::Modulo
                | BinaryOperator::Exponent => {
                    let left_out = self.check_expr(ctx.clone(), *left, EssenceType::Int);
                    let right_out = self.check_expr(ctx, *right, EssenceType::Int);
                    (
                        GenOut::new(
                            left_out
                                .constraints
                                .into_iter()
                                .chain(right_out.constraints)
                                .collect(),
                            Expression::BinOp(
                                op,
                                Box::new(left_out.typed_ast),
                                Box::new(right_out.typed_ast),
                            ),
                        ),
                        EssenceType::Int,
                    )
                }

                BinaryOperator::And
                | BinaryOperator::Or
                | BinaryOperator::Implies
                | BinaryOperator::Iff
                | BinaryOperator::Equals
                | BinaryOperator::NotEquals => {
                    let left_out = self.check_expr(ctx.clone(), *left, EssenceType::Bool);
                    let right_out = self.check_expr(ctx, *right, EssenceType::Bool);
                    (
                        GenOut::new(
                            left_out
                                .constraints
                                .into_iter()
                                .chain(right_out.constraints)
                                .collect(),
                            Expression::BinOp(
                                op,
                                Box::new(left_out.typed_ast),
                                Box::new(right_out.typed_ast),
                            ),
                        ),
                        EssenceType::Bool,
                    )
                }

                BinaryOperator::Compare { ty } => todo!(),
                BinaryOperator::LexCompare { ty } => todo!(),
                BinaryOperator::Union | BinaryOperator::Intersect => {
                    let set_ty =
                        EssenceType::Set(Box::new(EssenceType::TypeVar(self.fresh_ty_var())));

                    let left_out = self.check_expr(ctx.clone(), *left, set_ty.clone());
                    let right_out = self.check_expr(ctx, *right, set_ty.clone());

                    (
                        GenOut::new(
                            left_out
                                .constraints
                                .into_iter()
                                .chain(right_out.constraints)
                                .collect(),
                            Expression::BinOp(
                                op,
                                Box::new(left_out.typed_ast),
                                Box::new(right_out.typed_ast),
                            ),
                        ),
                        set_ty,
                    )
                }

                BinaryOperator::In => {
                    let elem_ty = EssenceType::TypeVar(self.fresh_ty_var());
                    let set_ty = EssenceType::Set(Box::new(elem_ty.clone()));

                    let left_out = self.check_expr(ctx.clone(), *left, elem_ty);
                    let right_out = self.check_expr(ctx, *right, set_ty);

                    (
                        GenOut::new(
                            left_out
                                .constraints
                                .into_iter()
                                .chain(right_out.constraints)
                                .collect(),
                            Expression::BinOp(
                                op,
                                Box::new(left_out.typed_ast),
                                Box::new(right_out.typed_ast),
                            ),
                        ),
                        EssenceType::Bool,
                    )
                }

                BinaryOperator::Subset
                | BinaryOperator::SubsetEq
                | BinaryOperator::Supset
                | BinaryOperator::SupsetEq => {
                    let set_ty =
                        EssenceType::Set(Box::new(EssenceType::TypeVar(self.fresh_ty_var())));

                    let left_out = self.check_expr(ctx.clone(), *left, set_ty.clone());
                    let right_out = self.check_expr(ctx, *right, set_ty);

                    (
                        GenOut::new(
                            left_out
                                .constraints
                                .into_iter()
                                .chain(right_out.constraints)
                                .collect(),
                            Expression::BinOp(
                                op,
                                Box::new(left_out.typed_ast),
                                Box::new(right_out.typed_ast),
                            ),
                        ),
                        EssenceType::Bool,
                    )
                }
                BinaryOperator::Substring | BinaryOperator::Subsequence => {
                    let seq_ty =
                        EssenceType::Sequence(Box::new(EssenceType::TypeVar(self.fresh_ty_var())));

                    let left_out = self.check_expr(ctx.clone(), *left, seq_ty.clone());
                    let right_out = self.check_expr(ctx, *right, seq_ty);

                    (
                        GenOut::new(
                            left_out
                                .constraints
                                .into_iter()
                                .chain(right_out.constraints)
                                .collect(),
                            Expression::BinOp(
                                op,
                                Box::new(left_out.typed_ast),
                                Box::new(right_out.typed_ast),
                            ),
                        ),
                        EssenceType::Bool,
                    )
                }
            },
            Expression::Aggregate(aggregate) => {
                let ret_ty = match aggregate.ty {
                    AggregateType::ForAll | AggregateType::Exists => EssenceType::Bool,
                    AggregateType::Sum | AggregateType::Min | AggregateType::Max => {
                        EssenceType::Int
                    }
                };

                let expr_out = self.check_expr(ctx, *aggregate.expr, ret_ty);

                todo!()

                // let (variables_out, variables_ty) = aggregate
                //     .variables
                //     .into_iter()
                //     .map(|v| match v {
                //         AggregateVariable::Placeholder => {}
                //         AggregateVariable::Variable(identifier) => self.infer_expr(ctx.clone(), v),
                //         AggregateVariable::Tuple(variables) => todo!(),
                //     })
                //     .unzip();
                //
                // let collection_ty = match aggregate.scope {
                //     AggregateScope::Collection(expr) => self.infer_expr(ctx, *expr),
                //     AggregateScope::Domain(domain) => self.get_domain_ty(ctx, domain),
                // };

                // (GenOut::new())
            }
            Expression::Indexing(expression, items) => todo!(),
            Expression::FunctionCall {
                function,
                mut arguments,
            } => {
                // TODO: handle multi-args
                assert_eq!(arguments.len(), 1);
                let arg = arguments.pop().unwrap();

                let (arg_out, arg_ty) = self.infer_expr(ctx.clone(), arg);

                let ret_ty = EssenceType::TypeVar(self.fresh_ty_var());
                let fun_ty = EssenceType::Function {
                    input: Box::new(arg_ty),
                    output: Box::new(ret_ty.clone()),
                };

                let fun_out = self.check_expr(ctx, *function, fun_ty);

                (
                    GenOut::new(
                        arg_out
                            .constraints
                            .into_iter()
                            .chain(fun_out.constraints)
                            .collect(),
                        Expression::FunctionCall {
                            function: Box::new(fun_out.typed_ast),
                            arguments: vec![arg_out.typed_ast],
                        },
                    ),
                    ret_ty,
                )
            }
            Expression::TupleLiteral { members } => {
                let (outs, types): (Vec<_>, Vec<_>) = members
                    .into_iter()
                    .map(|e| self.infer_expr(ctx.clone(), e))
                    .unzip();

                let (constraints, typed_members): (Vec<_>, _) = outs
                    .into_iter()
                    .map(|o| (o.constraints, o.typed_ast))
                    .unzip();

                let constraints = constraints.into_iter().flatten().collect();

                (
                    GenOut::new(
                        constraints,
                        Expression::TupleLiteral {
                            members: typed_members,
                        },
                    ),
                    EssenceType::Tuple(types),
                )
            }
            Expression::SetLiteral { members } => {
                let member_ty_var = self.fresh_ty_var();

                let (constraints, typed_members): (Vec<_>, Vec<_>) = members
                    .into_iter()
                    .map(|e| {
                        let (out, _) = self.infer_expr(ctx.clone(), e);
                        (out.constraints, out.typed_ast)
                    })
                    .unzip();

                let constraints = constraints.into_iter().flatten().collect();

                (
                    GenOut::new(
                        constraints,
                        Expression::SetLiteral {
                            members: typed_members,
                        },
                    ),
                    EssenceType::Set(Box::new(EssenceType::TypeVar(member_ty_var))),
                )
            }
            Expression::MultiSetLiteral { mut members } => match members.len() {
                0 => {
                    let ty =
                        EssenceType::MultiSet(Box::new(EssenceType::TypeVar(self.fresh_ty_var())));
                    (
                        GenOut::new(vec![], Expression::MultiSetLiteral { members: vec![] }),
                        ty,
                    )
                }
                1 => {
                    let (member_out, member_ty) = self.infer_expr(ctx, members.pop().unwrap());
                    (
                        GenOut::new(
                            member_out.constraints,
                            Expression::MultiSetLiteral {
                                members: vec![member_out.typed_ast],
                            },
                        ),
                        EssenceType::MultiSet(Box::new(member_ty)),
                    )
                }
                _ => {
                    let member_ty = EssenceType::TypeVar(self.fresh_ty_var());
                    let (outs, types): (Vec<_>, Vec<_>) = members
                        .into_iter()
                        .map(|e| self.infer_expr(ctx.clone(), e))
                        .unzip();

                    let (constraints, typed_members): (Vec<_>, Vec<_>) = outs
                        .into_iter()
                        .map(|o| (o.constraints, o.typed_ast))
                        .unzip();

                    let constraints: Vec<_> = constraints.into_iter().flatten().collect();
                    // (GenOut::new())
                    todo!()
                }
            },
            Expression::SequenceLiteral { members } => todo!(),
            Expression::MatrixLiteral { members, domain } => todo!(),
            Expression::FunctionLiteral { pairs } => todo!(),
            Expression::RelationLiteral { pairs } => todo!(),
            Expression::PermutationLiteral { cycles } => todo!(),
            Expression::RecordLiteral { pairs } => todo!(),
            Expression::VariantLiteral { pair } => todo!(),
            Expression::PartitionLiteral { partitions } => todo!(),
        }
    }

    fn check_expr(
        &mut self,
        ctx: im::HashMap<VariableId, EssenceType>,
        expr: Expression<VariableId>,
        ty: EssenceType,
    ) -> GenOut<Expression<TypedVariableId>> {
        match (expr, ty) {
            (Expression::IntegerValue(i), EssenceType::Int) => {
                GenOut::new(vec![], Expression::IntegerValue(i))
            }
            (Expression::BooleanValue(b), EssenceType::Bool) => {
                GenOut::new(vec![], Expression::BooleanValue(b))
            }
            (expr, expected_ty) => {
                let (mut out, actual_ty) = self.infer_expr(ctx, expr);
                out.constraints
                    .push(TypeConstraint::TypeEqual(expected_ty, actual_ty));
                out
            }
        }
    }

    fn infer_domain(
        &self,
        ctx: im::HashMap<VariableId, EssenceType>,
        domain: Domain<VariableId>,
    ) -> (GenOut<Domain<TypedVariableId>>, EssenceType) {
        match domain {
            Domain::Identifier(ident) => todo!(),
            Domain::Bool => todo!(),           //EssenceType::Bool,
            Domain::Int { ranges } => todo!(), //EssenceType::Int,
            Domain::Tuple { member_domains } => todo!(),
            // EssenceType::Tuple(
            //     member_domains
            //         .into_iter()
            //         .map(|d| self.get_domain_ty(ctx, d))
            //         .collect(),
            // ),
            Domain::Matrix {
                indexed_by,
                element_domain,
            } => todo!(), //EssenceType::Matrix(Box::new(self.get_domain_ty(ctx, *element_domain))),
            Domain::Record { pairs } => todo!(),
            Domain::Variant { pairs } => todo!(),
            Domain::Set {
                attributes,
                member_domain,
            } => todo!(),
            Domain::MultiSet {
                attributes,
                member_domain,
            } => todo!(),
            Domain::Sequence {
                attributes,
                member_domain,
            } => todo!(),
            Domain::Function {
                attributes,
                input_domain,
                output_domain,
            } => todo!(),
            Domain::Relation {
                attributes,
                domains,
            } => todo!(),
            Domain::Partition {
                attributes,
                member_domain,
            } => todo!(),
            Domain::Permutation {
                attributes,
                member_domain,
            } => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenOut<T> {
    // Set of constraints to be solved
    constraints: Vec<TypeConstraint>,
    // Ast where all variables are annotated with their type
    typed_ast: T,
}

impl<T> GenOut<T> {
    fn new(constraints: Vec<TypeConstraint>, typed_ast: T) -> Self {
        Self {
            constraints,
            typed_ast,
        }
    }

    fn empty(typed_ast: T) -> Self {
        Self::new(vec![], typed_ast)
    }
}

#[derive(Debug, Clone)]
pub enum TypeConstraint {
    TypeEqual(EssenceType, EssenceType),
}

// pub struct EssenceValue {
//     pub ty: EssenceType,
//     pub value: Expression,
// }
// pub struct SymbolTable {
//     symbols: HashMap<Identifier, EssenceValue>,
// }

// impl SymbolTable {
//     pub fn get_value(&self, ident: &Identifier) -> Option<&EssenceValue> {
//         self.symbols.get(ident)
//     }
//
//     // pub fn get_function(&self, ident: Identifier) -> Option<EssenceValue> {
//     //
//     // }
// }

// impl Expression {
//     fn member_ty(&self, ctx: &SymbolTable) -> Option<EssenceType> {
//         match self.return_type(ctx)? {
//             EssenceType::Set(ty)
//             | EssenceType::MultiSet(ty)
//             | EssenceType::Sequence(ty)
//             | EssenceType::Matrix(ty)
//             | EssenceType::Partition(ty)
//             | EssenceType::Permutation(ty) => Some(*ty),
//             _ => None,
//         }
//     }
//
//     fn return_type(&self, ctx: &SymbolTable) -> Option<EssenceType> {
//         let ty = match self {
//             Expression::Identifier(ident) => ctx.get_value(ident)?.ty.clone(),
//             Expression::IntegerValue(_) => EssenceType::Int,
//             Expression::BooleanValue(_) => EssenceType::Bool,
//             Expression::UnOp(unop, _) => match unop {
//                 UnaryOperator::Not => EssenceType::Bool,
//                 UnaryOperator::Negative | UnaryOperator::Absolute | UnaryOperator::Cardinality => {
//                     EssenceType::Int
//                 }
//             },
//             Expression::BinOp(binop, left, _) => match binop {
//                 BinaryOperator::Add
//                 | BinaryOperator::Subtract
//                 | BinaryOperator::Divide
//                 | BinaryOperator::Multiply
//                 | BinaryOperator::Modulo
//                 | BinaryOperator::Exponent => EssenceType::Int,
//                 BinaryOperator::And
//                 | BinaryOperator::Or
//                 | BinaryOperator::Implies
//                 | BinaryOperator::Iff
//                 | BinaryOperator::Equals
//                 | BinaryOperator::NotEquals
//                 | BinaryOperator::Compare { .. }
//                 | BinaryOperator::LexCompare { .. }
//                 | BinaryOperator::In
//                 | BinaryOperator::Subset
//                 | BinaryOperator::SubsetEq
//                 | BinaryOperator::Supset
//                 | BinaryOperator::SupsetEq
//                 | BinaryOperator::Substring
//                 | BinaryOperator::Subsequence => EssenceType::Bool,
//                 BinaryOperator::Union | BinaryOperator::Intersect => {
//                     let Some(EssenceType::Set(member_ty)) = left.return_type(ctx) else {
//                         todo!();
//                     };
//                     EssenceType::Set(member_ty)
//                 }
//             },
//             Expression::Aggregate(aggregate) => match aggregate.ty {
//                 AggregateType::ForAll | AggregateType::Exists => EssenceType::Bool,
//                 AggregateType::Sum | AggregateType::Min | AggregateType::Max => EssenceType::Int,
//             },
//             Expression::Indexing(expression, _) => expression.member_ty(ctx)?,
//             Expression::FunctionCall {
//                 function,
//                 arguments,
//             } => ctx.get_function(function),
//             Expression::SetLiteral { members } => EssenceType::Set(members),
//             Expression::MultiSetLiteral { members } => EssenceType::MultiSet(),
//             Expression::SequenceLiteral { members } => EssenceType::Sequence(),
//             Expression::MatrixLiteral { members, domain } => EssenceType::Sequence(),
//             Expression::FunctionLiteral { pairs } => EssenceType::Function(),
//             Expression::RelationLiteral { pairs } => EssenceType::Relation(),
//             Expression::PermutationLiteral { pairs } => EssenceType::Permutation(),
//             Expression::RecordLiteral { pairs } => {
//                 let tys = pairs
//                     .iter()
//                     .map(|p| (p.name, p.value.return_type(ctx)))
//                     .collect()?;
//                 EssenceType::Record(tys)
//             }
//             Expression::Variant(pair) => {
//                 let ty = pair.value.return_type(ctx)?;
//                 EssenceType::Variant(pair.name, Box::new(ty))
//             }
//         };
//         Some(ty)
//     }
// }
