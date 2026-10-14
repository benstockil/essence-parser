use crate::token::{Token, TokenType};

pub struct Lexer {
    source: Vec<char>,
    current_char: usize,
    line: u32,
    col: u32,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            source: input.chars().collect(),
            current_char: 0,
            line: 0,
            col: 0,
        }
    }

    fn current(&self) -> Option<char> {
        self.source.get(self.current_char).copied()
    }

    fn lookahead(&self, chars: usize) -> &[char] {
        let start = self.current_char;
        let end = usize::min(start + chars, self.source.len());
        &self.source[start..end]
    }

    fn advance(&mut self) -> Option<char> {
        self.current_char += 1;

        let current = self.current()?;

        if current == '\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }

        Some(current)
    }

    pub fn parse_next(&mut self) -> Option<Token> {
        let mut ready = false;
        while !ready {
            ready = true;

            // skip whitespace
            if self.current()?.is_whitespace() {
                ready = false;
                self.advance();
            }

            // skip comments
            if self.current()? == '$' {
                ready = false;
                while self.current()? != '\n' {
                    self.advance();
                }
                self.advance();
            }
        }

        let current = self.current()?;
        let l = self.line;
        let c = self.col;

        let mut advance_one = true;

        let token = match current {
            '(' => TokenType::LParen.at(l, c, 1),
            ')' => TokenType::RParen.at(l, c, 1),
            '[' => TokenType::LBracket.at(l, c, 1),
            ']' => TokenType::RBracket.at(l, c, 1),
            '{' => TokenType::LBrace.at(l, c, 1),
            '}' => TokenType::RBrace.at(l, c, 1),
            '&' => TokenType::Ampersand.at(l, c, 1),
            ',' => TokenType::Comma.at(l, c, 1),
            '+' => TokenType::Plus.at(l, c, 1),
            '%' => TokenType::Modulo.at(l, c, 1),
            '=' => TokenType::Equals.at(l, c, 1),
            '_' => TokenType::Underscore.at(l, c, 1),

            '\\' => {
                if self.advance() == Some('/') {
                    TokenType::Or.at(l, c, 2)
                } else {
                    // TODO: invalid token
                    todo!()
                }
            }

            '/' => {
                if self.advance() == Some('\\') {
                    TokenType::And.at(l, c, 2)
                } else {
                    advance_one = false;
                    TokenType::Divide.at(l, c, 1)
                }
            }

            '*' => {
                if self.advance() == Some('*') {
                    TokenType::Exponent.at(l, c, 2)
                } else {
                    advance_one = false;
                    TokenType::Asterisk.at(l, c, 1)
                }
            }

            '.' => {
                if self.advance() == Some('.') {
                    TokenType::DoublePeriod.at(l, c, 2)
                } else {
                    advance_one = false;
                    TokenType::Period.at(l, c, 1)
                }
            }

            '!' => {
                if self.advance() == Some('=') {
                    TokenType::NotEquals.at(l, c, 2)
                } else {
                    advance_one = false;
                    TokenType::Not.at(l, c, 1)
                }
            }

            ':' => match self.advance() {
                Some(':') => TokenType::DoubleColon.at(l, c, 2),
                _ => {
                    advance_one = false;
                    TokenType::Colon.at(l, c, 1)
                }
            },

            '-' => match self.advance() {
                Some('-') if self.lookahead(2) == ['-', '>'] => {
                    self.advance();
                    TokenType::LongRightArrow.at(l, c, 3)
                }
                Some('>') => TokenType::Implies.at(l, c, 2),
                _ => {
                    advance_one = false;
                    TokenType::Minus.at(l, c, 1)
                }
            },

            '<' | '>' => 'comp: {
                let mut eq = false;
                let mut lex = false;

                let next = self.advance();
                if current == '<' && next == Some('-') {
                    break 'comp if self.advance() == Some('>') {
                        TokenType::Iff.at(l, c, 3)
                    } else {
                        TokenType::LeftArrow.at(l, c, 2)
                    };
                }

                if next == Some('=') {
                    eq = true;
                    self.advance();
                }

                if self.lookahead(3) == ['l', 'e', 'x'] {
                    lex = true;
                    self.advance();
                    self.advance();
                    self.advance();
                }

                match (current, eq, lex) {
                    ('<', false, false) => TokenType::LessThan.at(l, c, 1),
                    ('<', true, false) => TokenType::LessThanEq.at(l, c, 2),
                    ('>', false, false) => TokenType::GreaterThan.at(l, c, 1),
                    ('>', true, false) => TokenType::GreaterThanEq.at(l, c, 2),
                    ('<', false, true) => TokenType::LessThanLex.at(l, c, 4),
                    ('<', true, true) => TokenType::LessThanEqLex.at(l, c, 5),
                    ('>', false, true) => TokenType::GreaterThanLex.at(l, c, 4),
                    ('>', true, true) => TokenType::GreaterThanEqLex.at(l, c, 5),
                    _ => unreachable!(),
                }
            }

            x if x.is_ascii_digit() => {
                let mut value = String::from(x);
                while let Some(next) = self.advance() {
                    if !(next.is_ascii_digit()) {
                        break;
                    }
                    value.push(next);
                }

                advance_one = false;

                TokenType::IntegerValue(value.parse().unwrap()).at(l, c, value.len() as u32)
            }

            x if x.is_alphabetic() => 'ident: {
                let mut ident = String::from(x);
                while let Some(next) = self.advance() {
                    if next == '\'' {
                        ident.push(next);
                        self.advance();
                        break;
                    }

                    let partials = ["such", "indexed", "new"];

                    if next == ' ' && ident == "language" {
                        let mut lang = String::new();
                        while let Some(next) = self.advance()
                            && next != '\n'
                        {
                            lang.push(next);
                        }
                        let length = 9 + lang.len() as u32;
                        break 'ident TokenType::Language(lang).at(l, c, length);
                    }

                    if !(next == '_'
                        || next.is_alphabetic()
                        || (next == ' ' && partials.contains(&ident.as_str())))
                    {
                        break;
                    }

                    ident.push(next);
                }

                advance_one = false;

                match ident.as_str() {
                    // Constants
                    "true" => TokenType::True.at(l, c, 4),
                    "false" => TokenType::False.at(l, c, 5),

                    // Keywords
                    "letting" => TokenType::Letting.at(l, c, 6),
                    "given" => TokenType::Given.at(l, c, 5),
                    "where" => TokenType::Where.at(l, c, 5),
                    "such that" => TokenType::SuchThat.at(l, c, 9),
                    "find" => TokenType::Find.at(l, c, 4),
                    "be" => TokenType::Be.at(l, c, 2),
                    "of" => TokenType::Of.at(l, c, 2),
                    "from" => TokenType::From.at(l, c, 4),
                    "in" => TokenType::In.at(l, c, 2),
                    "domain" => TokenType::Domain.at(l, c, 6),
                    "indexed by" => TokenType::IndexedBy.at(l, c, 10),
                    "new type enum" => TokenType::NewTypeEnum.at(l, c, 10),

                    // Quantifiers
                    "forAll" => TokenType::ForAll.at(l, c, 6),
                    "exists" => TokenType::Exists.at(l, c, 6),
                    "sum" => TokenType::Sum.at(l, c, 3),
                    "min" => TokenType::Min.at(l, c, 3),
                    "max" => TokenType::Max.at(l, c, 3),

                    // Set operators
                    "union" => TokenType::Union.at(l, c, 5),
                    "intersect" => TokenType::Intersect.at(l, c, 9),
                    "subset" => TokenType::Subset.at(l, c, 6),
                    "subsetEq" => TokenType::SubsetEq.at(l, c, 8),
                    "supset" => TokenType::Supset.at(l, c, 6),
                    "supsetEq" => TokenType::SupsetEq.at(l, c, 8),

                    // Sequence operators
                    "substring" => TokenType::Substring.at(l, c, 9),
                    "subsequence" => TokenType::Subsequence.at(l, c, 11),

                    // Dominance
                    "maximising" => TokenType::Maximising.at(l, c, 10),
                    "minimising" => TokenType::Minimising.at(l, c, 10),

                    // Domains
                    "int" => TokenType::Int.at(l, c, 3),
                    "bool" => TokenType::Bool.at(l, c, 4),
                    "set" => TokenType::Set.at(l, c, 3),
                    "mset" => TokenType::MSet.at(l, c, 4),
                    "tuple" => TokenType::Tuple.at(l, c, 5),
                    "matrix" => TokenType::Matrix.at(l, c, 6),
                    "record" => TokenType::Record.at(l, c, 6),
                    "variant" => TokenType::Variant.at(l, c, 7),
                    "sequence" => TokenType::Sequence.at(l, c, 8),
                    "function" => TokenType::Function.at(l, c, 8),
                    "relation" => TokenType::Relation.at(l, c, 8),
                    "partition" => TokenType::Partition.at(l, c, 9),
                    "permutation" => TokenType::Permutation.at(l, c, 11),
                    x => TokenType::Identifier(x.to_string()).at(l, c, x.len() as u32),
                }
            }

            _ => {
                dbg!(current);
                todo!() // TODO: invalid
            }
        };

        if advance_one {
            self.advance();
        }

        Some(token)
    }
}
