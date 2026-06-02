use crate::ast::Literal;
use crate::ast::{ArgAstArena, ExprArena, ExprAst, ExprId};
use crate::codes::SYNTAX_ERROR;
use crate::diagnostics::Diagnostic;
use crate::lexer::Token::True;
use crate::lexer::{SpannedToken, Tokens, lex};
use chumsky::span::SimpleSpan;

/// Parse a module
pub fn parse_module<'arena>(
    tokens: Tokens,
    expr_arena: &'arena mut ExprArena,
    arg_ast_arena: &'arena mut ArgAstArena,
) -> Vec<Result<ExprId, Diagnostic>> {
    Parser::new(&tokens, expr_arena, arg_ast_arena).parse_module()
}

struct Parser<'arena> {
    tokens: &'arena Tokens,
    expr_arena: &'arena mut ExprArena,
    arg_ast_arena: &'arena mut ArgAstArena,
    p: usize,
}

impl<'arena> Parser<'arena> {
    fn new(
        tokens: &'arena Tokens,
        expr_arena: &'arena mut ExprArena,
        arg_ast_arena: &'arena mut ArgAstArena,
    ) -> Self {
        Self {
            tokens,
            p: 0,
            expr_arena,
            arg_ast_arena,
        }
    }

    fn current(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.p)
    }

    fn next(&mut self) -> Option<&SpannedToken> {
        self.p += 1;
        self.current()
    }

    fn at_end(&self) -> bool {
        self.p >= self.tokens.len()
    }

    fn add_expr_node(&mut self, expr_ast: ExprAst, span: SimpleSpan) -> ExprId {
        self.expr_arena.new_node(expr_ast, span)
    }

    pub fn parse_module(&mut self) -> Vec<Result<ExprId, Diagnostic>> {
        let mut result = Vec::new();

        loop {
            if self.at_end() {
                return result;
            } else {
                result.push(self.parse_expr())
            }
        }
    }

    fn parse_expr(&mut self) -> Result<ExprId, Diagnostic> {
        use crate::lexer::Token::*;

        match self.current() {
            Some(spanned_tok) => match spanned_tok.token() {
                Ok(token) => match *token {
                    Integer(i) => Ok(self
                        .expr_arena
                        .new_node(ExprAst::Literal(Literal::Int(i)), spanned_tok.span())),

                    Float(i) => Ok(self
                        .expr_arena
                        .new_node(ExprAst::Literal(Literal::Float(i)), spanned_tok.span())),

                    True => Ok(self
                        .expr_arena
                        .new_node(ExprAst::Literal(Literal::Bool(true)), spanned_tok.span())),

                    False => Ok(self
                        .expr_arena
                        .new_node(ExprAst::Literal(Literal::Bool(false)), spanned_tok.span())),

                    _ => Err(Diagnostic::new_error(
                        format!(
                            "Unexpected Token: {:?}, Expected Expression",
                            spanned_tok.token().unwrap()
                        ),
                        SYNTAX_ERROR,
                        spanned_tok.span(),
                    )),
                },
                Err(e) => Err(e.clone()),
            },
            None => {
                Err(Diagnostic::new_error(
                    "Unexpected EOF".to_string(),
                    SYNTAX_ERROR,
                    // TODO: FIX ME
                    SimpleSpan {
                        start: 0,
                        end: 1,
                        context: (),
                    },
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    fn runner_helper(input: &str) -> (ExprAst, ExprArena) {
        let mut expr_arena = ExprArena::new();
        let mut args_arena = ArgAstArena::new();
        let tokens = lex(input);
        let mut parser = Parser::new(&tokens, &mut expr_arena, &mut args_arena);

        match parser.parse_expr() {
            Ok(expr_id) => (expr_arena.get_node(expr_id).unwrap().clone(), expr_arena),
            Err(diagnostic) => {
                dbg!(&tokens);
                dbg!(diagnostic);
                panic!("Parse failed");
            }
        }
    }

    fn run_test(input: &str, expected: ExprAst) {
        assert_eq!(expected, runner_helper(input).0);
    }

    #[test]
    fn test_simple_expressions() {
        run_test("1", ExprAst::Literal(Literal::Int(1)));
        run_test("1.23", ExprAst::Literal(Literal::Float(1.23)));
        run_test("true", ExprAst::Literal(Literal::Bool(true)));
        run_test("false", ExprAst::Literal(Literal::Bool(false)));
    }
}
