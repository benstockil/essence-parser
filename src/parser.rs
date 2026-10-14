use std::collections::VecDeque;

use crate::{
    ast::{
        Identifier, Program,
        domain::{Domain, DomainAttribute, IntRange, NameDomainPair},
        expression::{
            Aggregate, AggregateScope, AggregateType, AggregateVariable, Expression,
            FunctionLiteralPair, Index, NameValuePair, UnaryOperator,
        },
        statement::{
            LettingDeclaration, LettingDeclarationValue, Statement, VariableDeclaration,
            VariableDeclarationDomain,
        },
    },
    error::{ParseError, ParseErrorKind},
    span::Span,
    token::{Token, TokenType},
};

macro_rules! expect {
    {$self:expr, $( $token:pat ),* $(,)?} => {
        $(
            let __macro_get_t = $self.next()?;
            let Token { ty: $token, .. } = __macro_get_t else {
                return Err(ParseError::expected(stringify!($token), __macro_get_t));
            };
        )*
    };
}

macro_rules! maybe {
    ($self:expr, $token:pat) => {{
        let exists = matches!($self.lookahead(0), Some(Token { ty: $token, .. }));
        if exists {
            $self.next()?;
        }
        exists
    }};
}

macro_rules! lookahead_maybe {
    ($self:expr, $token:pat) => {
        matches!($self.lookahead(0), Some(Token { ty: $token, .. }))
    };
}

type Result<T> = std::result::Result<T, ParseError>;

pub struct Parser {
    tokens: VecDeque<Token>,
    current_span: Span,
}

impl Parser {
    pub fn new(tokens: impl Into<VecDeque<Token>>) -> Self {
        Self {
            tokens: tokens.into(),
            current_span: Span::default(),
        }
    }
    fn error(&self, kind: ParseErrorKind) -> ParseError {
        ParseError {
            span: self.current_span,
            kind,
        }
    }

    pub fn next(&mut self) -> Result<Token> {
        self.tokens
            .pop_front()
            .ok_or(self.error(ParseErrorKind::UnexpectedEof))
    }

    pub fn lookahead(&self, tokens: usize) -> Option<&Token> {
        self.tokens.get(tokens)
    }

    pub fn parse(&mut self) -> Result<Program<Identifier>> {
        let mut statements = Vec::new();
        while self.lookahead(0).is_some() {
            statements.push(self.parse_statement()?);
        }

        Ok(Program(statements))
    }

    fn parse_letting_declaration(&mut self) -> Result<LettingDeclaration<Identifier>> {
        let idents = self.parse_comma_separated_list(Self::parse_ident)?;
        expect!(self, TokenType::Be);

        let value_token = self.next()?;
        let value = match value_token.ty {
            TokenType::Domain => LettingDeclarationValue::Domain(self.parse_domain()?),
            TokenType::NewTypeEnum => {
                let variants = self.parse_comma_separated_list(Self::parse_ident)?;
                LettingDeclarationValue::Enum { variants }
            }
            // TokenType::NewTypeOfSize => LettingDeclarationValue::Unnamed(self.parse_expression()?),
            _ => LettingDeclarationValue::Expr(self.parse_expression()?),
        };

        Ok(LettingDeclaration { idents, value })
    }

    fn parse_ident(&mut self) -> Result<Identifier> {
        expect!(self, TokenType::Identifier(i));
        Ok(Identifier(i))
    }

    fn parse_variable_declaration(&mut self) -> Result<VariableDeclaration<Identifier>> {
        let idents = self.parse_comma_separated_list(Self::parse_ident)?;

        let domain_token = self.next()?;
        let domain = match domain_token.ty {
            TokenType::Colon => VariableDeclarationDomain::Domain(self.parse_domain()?),
            TokenType::NewTypeEnum => VariableDeclarationDomain::Enum,
            _ => return Err(ParseError::expected(": or `new type enum`", domain_token)),
        };

        Ok(VariableDeclaration { idents, domain })
    }

    fn parse_statement(&mut self) -> Result<Statement<Identifier>> {
        let stmt_token = self.next()?;
        let stmt = match stmt_token.ty {
            TokenType::Language(l) => Statement::Language(l),
            TokenType::Letting => {
                let declarations =
                    self.parse_comma_separated_list(Self::parse_letting_declaration)?;
                Statement::Letting { declarations }
            }
            TokenType::Given => {
                let declarations =
                    self.parse_comma_separated_list(Self::parse_variable_declaration)?;
                Statement::Given { declarations }
            }
            TokenType::SuchThat => {
                let constraints = self.parse_comma_separated_list(Self::parse_expression)?;
                Statement::SuchThat { constraints }
            }
            TokenType::Where => {
                let constraints = self.parse_comma_separated_list(Self::parse_expression)?;
                Statement::Where { constraints }
            }
            TokenType::Find => {
                let declarations =
                    self.parse_comma_separated_list(Self::parse_variable_declaration)?;
                Statement::Given { declarations }
            }
            _ => return Err(ParseError::expected("beginning of statement", stmt_token)),
        };

        Ok(stmt)
    }

    fn parse_comma_separated_list<T>(
        &mut self,
        parser: fn(&mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        let mut instances = Vec::new();
        loop {
            instances.push((parser)(self)?);

            if !maybe!(self, TokenType::Comma) {
                break;
            }
        }

        Ok(instances)
    }

    fn parse_domain(&mut self) -> Result<Domain<Identifier>> {
        let domain_token = self.next()?;

        let domain = match domain_token.ty {
            TokenType::Identifier(i) => Domain::Identifier(Identifier(i)),
            TokenType::Bool => Domain::Bool,
            TokenType::Int => {
                let ranges = if maybe!(self, TokenType::LParen) {
                    let r = self.parse_comma_separated_list(Self::parse_range)?;
                    expect!(self, TokenType::RParen);
                    r
                } else {
                    Vec::new()
                };

                Domain::Int { ranges }
            }
            TokenType::Set => {
                let attributes = self.parse_attributes()?;
                expect!(self, TokenType::Of);
                let domain = self.parse_domain()?;
                Domain::Set {
                    attributes,
                    member_domain: Box::new(domain),
                }
            }
            TokenType::MSet => {
                let attributes = self.parse_attributes()?;
                expect!(self, TokenType::Of);
                let domain = self.parse_domain()?;
                Domain::Set {
                    attributes,
                    member_domain: Box::new(domain),
                }
            }
            TokenType::Tuple => {
                expect!(self, TokenType::LParen);
                let member_domains = self.parse_comma_separated_list(Self::parse_domain)?;
                expect!(self, TokenType::LParen);
                Domain::Tuple { member_domains }
            }
            TokenType::Matrix => {
                expect!(self, TokenType::IndexedBy, TokenType::LBracket);
                let indexed_by = self.parse_comma_separated_list(Self::parse_domain)?;
                expect!(self, TokenType::RBracket, TokenType::Of);
                let domain = self.parse_domain()?;
                Domain::Matrix {
                    indexed_by,
                    element_domain: Box::new(domain),
                }
            }
            TokenType::Record => {
                expect!(self, TokenType::LBrace);
                let pairs = self.parse_comma_separated_list(Self::parse_name_domain_pair)?;
                expect!(self, TokenType::RBrace);
                Domain::Record { pairs }
            }
            TokenType::Variant => {
                expect!(self, TokenType::LBrace);
                let pairs = self.parse_comma_separated_list(Self::parse_name_domain_pair)?;
                expect!(self, TokenType::RBrace);
                Domain::Record { pairs }
            }
            TokenType::Sequence => {
                let attributes = self.parse_attributes()?;
                expect!(self, TokenType::Of);
                let domain = self.parse_domain()?;
                Domain::Sequence {
                    attributes,
                    member_domain: Box::new(domain),
                }
            }
            TokenType::Function => {
                let attributes = self.parse_attributes()?;
                let input_domain = self.parse_domain()?;
                expect!(self, TokenType::LongRightArrow);
                let output_domain = self.parse_domain()?;
                Domain::Function {
                    attributes,
                    input_domain: Box::new(input_domain),
                    output_domain: Box::new(output_domain),
                }
            }
            TokenType::Relation => {
                let attributes = self.parse_comma_separated_list(Self::parse_attribute)?;
                expect!(self, TokenType::Of, TokenType::LParen);
                let mut domains = Vec::new();
                loop {
                    domains.push(self.parse_domain()?);
                    if !maybe!(self, TokenType::Asterisk) {
                        break;
                    }
                }
                Domain::Relation {
                    attributes,
                    domains,
                }
            }
            TokenType::Partition => {
                let attributes = self.parse_attributes()?;
                expect!(self, TokenType::From);
                let domain = self.parse_domain()?;
                Domain::Partition {
                    attributes,
                    member_domain: Box::new(domain),
                }
            }
            TokenType::Permutation => {
                let attributes = self.parse_attributes()?;
                expect!(self, TokenType::Of);
                let domain = self.parse_domain()?;
                Domain::Permutation {
                    attributes,
                    member_domain: Box::new(domain),
                }
            }
            _ => return Err(ParseError::expected("domain", domain_token)),
        };
        Ok(domain)
    }

    fn parse_range(&mut self) -> Result<IntRange<Identifier>> {
        let mut lower_bound = None;

        if !maybe!(self, TokenType::DoublePeriod) {
            lower_bound = Some(self.parse_expression()?);
        }

        if !maybe!(self, TokenType::DoublePeriod) {
            let Some(value) = lower_bound else {
                return Err(ParseError::expected(
                    "expression or ..",
                    // TODO: ??
                    self.lookahead(0).unwrap().clone(),
                ));
            };
            return Ok(IntRange::Single(value));
        }

        if lookahead_maybe!(self, TokenType::Comma | TokenType::LParen) {
            return Ok(IntRange::Range {
                lower_bound,
                upper_bound: None,
            });
        }

        let upper_bound = Some(self.parse_expression()?);
        Ok(IntRange::Range {
            lower_bound,
            upper_bound,
        })
    }

    fn parse_attributes(&mut self) -> Result<Vec<DomainAttribute<Identifier>>> {
        if !maybe!(self, TokenType::LBrace) {
            return Ok(Vec::new());
        }

        let attributes = self.parse_comma_separated_list(Self::parse_attribute)?;
        expect!(self, TokenType::RBrace);

        Ok(attributes)
    }

    fn parse_attribute(&mut self) -> Result<DomainAttribute<Identifier>> {
        expect!(self, TokenType::Identifier(key));
        let value = if maybe!(self, TokenType::Comma | TokenType::LParen) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        Ok(DomainAttribute { key, value })
    }

    fn parse_name_domain_pair(&mut self) -> Result<NameDomainPair<Identifier>> {
        expect!(self, TokenType::Identifier(name), TokenType::Colon);
        let domain = self.parse_domain()?;
        Ok(NameDomainPair {
            name: Identifier(name),
            domain,
        })
    }

    fn parse_atom(&mut self) -> Result<Expression<Identifier>> {
        let first = self.next()?;
        let atom = match first.ty {
            TokenType::Identifier(i) => Expression::Variable(Identifier(i)),
            TokenType::IntegerValue(v) => Expression::IntegerValue(v),
            TokenType::True => Expression::BooleanValue(true),
            TokenType::False => Expression::BooleanValue(false),
            TokenType::Ampersand => todo!(),
            TokenType::Not => {
                let inner = self.parse_expression()?;
                Expression::UnOp(UnaryOperator::Not, Box::new(inner))
            }
            TokenType::LParen => {
                let first = self.parse_expression()?;

                let expr = if maybe!(self, TokenType::Comma) {
                    let mut members = self.parse_comma_separated_list(Self::parse_expression)?;
                    members.insert(0, first);
                    Expression::TupleLiteral { members }
                } else {
                    first
                };

                expect!(self, TokenType::RParen);
                expr
            }
            TokenType::LBrace => {
                let members = self.parse_comma_separated_list(Self::parse_expression)?;
                expect!(self, TokenType::RBrace);
                Expression::SetLiteral { members }
            }
            TokenType::LBracket => {
                let members = self.parse_comma_separated_list(Self::parse_expression)?;
                let domain = if maybe!(self, TokenType::Colon) {
                    Some(self.parse_domain()?)
                } else {
                    None
                };
                expect!(self, TokenType::RBracket);
                Expression::MatrixLiteral { members, domain }
            }
            TokenType::ForAll
            | TokenType::Exists
            | TokenType::Sum
            | TokenType::Min
            | TokenType::Max => Expression::Aggregate(self.parse_aggregate(&first.ty)?),

            TokenType::MSet => {
                expect!(self, TokenType::LParen);
                let members = self.parse_comma_separated_list(Self::parse_expression)?;
                expect!(self, TokenType::RParen);
                Expression::MultiSetLiteral { members }
            }
            TokenType::Sequence => {
                let members = self.parse_paren_expr_list()?;
                Expression::SequenceLiteral { members }
            }
            TokenType::Tuple => {
                let members = self.parse_paren_expr_list()?;
                Expression::TupleLiteral { members }
            }
            TokenType::Function => {
                expect!(self, TokenType::LParen);
                let pairs = self.parse_comma_separated_list(Self::parse_function_literal_pair)?;
                expect!(self, TokenType::RParen);
                Expression::FunctionLiteral { pairs }
            }
            TokenType::Relation => {
                expect!(self, TokenType::LParen);
                let pairs = self.parse_comma_separated_list(Self::parse_paren_expr_list)?;
                expect!(self, TokenType::RParen);
                Expression::RelationLiteral { pairs }
            }
            TokenType::Partition => {
                expect!(self, TokenType::LParen);
                let partitions =
                    self.parse_comma_separated_list(Self::parse_partition_set_literal)?;
                expect!(self, TokenType::RParen);
                Expression::PartitionLiteral { partitions }
            }
            TokenType::Permutation => {
                expect!(self, TokenType::LParen);
                let cycles = self.parse_comma_separated_list(Self::parse_paren_expr_list)?;
                expect!(self, TokenType::RParen);
                Expression::PermutationLiteral { cycles }
            }
            TokenType::Record => {
                expect!(self, TokenType::LBrace);
                let pairs = self.parse_comma_separated_list(Self::parse_name_value_pair)?;
                expect!(self, TokenType::RBrace);
                Expression::RecordLiteral { pairs }
            }
            TokenType::Variant => {
                expect!(self, TokenType::LBrace);
                let pair = self.parse_name_value_pair()?;
                expect!(self, TokenType::RBrace);
                Expression::VariantLiteral {
                    pair: Box::new(pair),
                }
            }
            _ => return Err(ParseError::expected("expression", first)),
        };
        Ok(atom)
    }

    fn parse_name_value_pair(&mut self) -> Result<NameValuePair<Identifier>> {
        expect!(self, TokenType::Identifier(name), TokenType::Equals);
        let value = self.parse_expression()?;
        Ok(NameValuePair {
            name: Identifier(name),
            value,
        })
    }

    fn parse_partition_set_literal(&mut self) -> Result<Vec<Expression<Identifier>>> {
        expect!(self, TokenType::LBrace);
        let members = self.parse_comma_separated_list(Self::parse_expression)?;
        expect!(self, TokenType::RBrace);
        Ok(members)
    }

    fn parse_paren_expr_list(&mut self) -> Result<Vec<Expression<Identifier>>> {
        expect!(self, TokenType::LParen);
        let exprs = self.parse_comma_separated_list(Self::parse_expression)?;
        expect!(self, TokenType::RParen);
        Ok(exprs)
    }

    fn parse_function_literal_pair(&mut self) -> Result<FunctionLiteralPair<Identifier>> {
        let input = self.parse_expression()?;
        expect!(self, TokenType::LongRightArrow);
        let output = self.parse_expression()?;
        Ok(FunctionLiteralPair { input, output })
    }

    fn parse_index(&mut self) -> Result<Index<Identifier>> {
        let index = if maybe!(self, TokenType::DoublePeriod) {
            Index::Null
        } else {
            Index::Value(Box::new(self.parse_expression()?))
        };

        Ok(index)
    }

    fn parse_expression(&mut self) -> Result<Expression<Identifier>> {
        self.parse_expr_climb(0)
    }

    fn parse_expr_climb(&mut self, min_precedence: u64) -> Result<Expression<Identifier>> {
        let mut result = self.parse_atom()?;

        if maybe!(self, TokenType::LBracket) {
            let indices = self.parse_comma_separated_list(Self::parse_index)?;
            expect!(self, TokenType::RBracket);
            result = Expression::Indexing(Box::new(result), indices);
        }

        if maybe!(self, TokenType::LParen) {
            let arguments = self.parse_comma_separated_list(Self::parse_expression)?;
            expect!(self, TokenType::RParen);
            result = Expression::FunctionCall {
                function: Box::new(result),
                arguments,
            };
        }

        while let Some(op) = self.lookahead(0).and_then(|t| t.to_binop())
            && op.precedence() >= min_precedence
        {
            self.next()?;
            let next_min_precedence =
                op.precedence() + if op.is_right_associative() { 0 } else { 1 };
            let rhs = self.parse_expr_climb(next_min_precedence)?;

            result = Expression::BinOp(op, Box::new(result), Box::new(rhs));
        }

        Ok(result)
    }

    fn parse_aggregate(&mut self, ty: &TokenType) -> Result<Aggregate<Identifier>> {
        let ty = match ty {
            TokenType::ForAll => AggregateType::ForAll,
            TokenType::Exists => AggregateType::Exists,
            TokenType::Sum => AggregateType::Sum,
            TokenType::Min => AggregateType::Min,
            TokenType::Max => AggregateType::Max,
            _ => unreachable!(),
        };

        let variables = self.parse_comma_separated_list(Self::parse_quantifier_variable)?;

        let scope_token = self.next()?;
        let scope = match scope_token.ty {
            TokenType::In => AggregateScope::Collection(Box::new(self.parse_expression()?)),
            TokenType::Colon => AggregateScope::Domain(self.parse_domain()?),
            _ => {
                return Err(ParseError::expected("`in` or `:`", scope_token));
            }
        };

        expect!(self, TokenType::Period);
        let condition = self.parse_expression()?;

        Ok(Aggregate {
            ty,
            variables,
            scope,
            expr: Box::new(condition),
        })
    }

    fn parse_quantifier_variable(&mut self) -> Result<AggregateVariable> {
        let token = self.next()?;
        let variable = match token.ty {
            TokenType::Identifier(i) => AggregateVariable::Variable(Identifier(i)),
            TokenType::LParen => {
                let members = self.parse_comma_separated_list(Self::parse_quantifier_variable)?;
                expect!(self, TokenType::RParen);
                AggregateVariable::Tuple(members)
            }
            TokenType::Underscore => AggregateVariable::Placeholder,
            _ => {
                return Err(ParseError::expected("quantifier variable", token));
            }
        };

        Ok(variable)
    }
}
