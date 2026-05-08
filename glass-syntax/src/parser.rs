//! The parser code. Currently generated with chumsky

use crate::ast::{ExprArena, ExprAst, ExprId, Literal};
use chumsky::prelude::*;
use std::cell::RefCell;

pub fn module_parser<'src>(
    arena: &RefCell<ExprArena>,
) -> impl Parser<'src, &'src str, ExprId, extra::Err<Rich<'src, char>>> {
    let int = text::int(10).map_with(|s: &str, extra| {
        arena.borrow_mut().new_node(
            ExprAst::Literal(Literal::Int(s.parse::<i64>().unwrap())),
            extra.span(),
        )
    });

    let float = text::int(10)
        .then(just('.'))
        .then(text::int(10).or_not())
        .to_slice()
        .map_with(|s: &str, extra| {
            arena.borrow_mut().new_node(
                ExprAst::Literal(Literal::Float(s.parse::<f64>().unwrap())),
                extra.span(),
            )
        });

    let atom = choice((float, int)).padded();

    atom
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_parse(input: &str, expected: ExprAst) {
        let arena = RefCell::new(ExprArena::new());

        let parse_result = module_parser(&arena).parse(input).into_result();
        let inner_arena = arena.into_inner();

        assert!(parse_result.is_ok());

        let result = inner_arena.get_node(parse_result.unwrap()).unwrap();
        assert_eq!(expected, *result);
    }

    #[test]
    fn test_parse_int() {
        test_parse("1", ExprAst::Literal(Literal::Int(1)));
        test_parse("10", ExprAst::Literal(Literal::Int(10)));
        test_parse("20", ExprAst::Literal(Literal::Int(20)));
        test_parse("944560", ExprAst::Literal(Literal::Int(944560)));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_parse_float() {
        test_parse("1.1", ExprAst::Literal(Literal::Float(1.1)));
        test_parse("2.23", ExprAst::Literal(Literal::Float(2.23)));
        test_parse("3.23", ExprAst::Literal(Literal::Float(3.23)));
        test_parse("4.23", ExprAst::Literal(Literal::Float(4.23)));
        test_parse("5.23", ExprAst::Literal(Literal::Float(5.23)));
        test_parse("6.23", ExprAst::Literal(Literal::Float(6.23)));
        test_parse("7.23", ExprAst::Literal(Literal::Float(7.23)));
        test_parse("8.23", ExprAst::Literal(Literal::Float(8.23)));
        test_parse("9.23", ExprAst::Literal(Literal::Float(9.23)));
        test_parse("0.23", ExprAst::Literal(Literal::Float(0.23)));
        test_parse("10.23", ExprAst::Literal(Literal::Float(10.23)));
        test_parse(
            "10.239999999999999",
            ExprAst::Literal(Literal::Float(10.239999999999999)),
        );
        test_parse(
            "10.2399999999999989",
            ExprAst::Literal(Literal::Float(10.2399999999999989)),
        );
    }
}
