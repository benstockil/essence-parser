use crate::{lexer::Lexer, parser::Parser, types::TypeInference};

mod ast;
mod error;
mod lexer;
mod name_resolution;
mod parser;
mod span;
mod token;
mod types;

fn main() {
    let file = include_str!(
        "~/uni/modules/vip/conjure-oxide/test-suite/tests/integration/savilerow/solitaire_battleship/input.essence"
    );
    let mut lexer = Lexer::new(file);
    let mut tokens = Vec::new();

    while let Some(next) = lexer.parse_next() {
        tokens.push(next);
    }

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    dbg!(ast);

    let mut inference = TypeInference::new();
    let inference_out = inference.infer_program(ctx, ast);
    dbg!(inference_out);
}

// parse_expression_1(lhs, min_precedence)
//     lookahead := peek next token
//     while lookahead is a binary operator whose precedence is >= min_precedence
//         op := lookahead
//         advance to next token
//         rhs := parse_primary ()
//         lookahead := peek next token
//         while lookahead is a binary operator whose precedence is greater
//                  than op's, or a right-associative operator
//                  whose precedence is equal to op's
//             rhs := parse_expression_1 (rhs, precedence of op + (1 if lookahead precedence is greater, else 0))
//             lookahead := peek next token
//         lhs := the result of applying op with operands lhs and rhs
//     return lhs
