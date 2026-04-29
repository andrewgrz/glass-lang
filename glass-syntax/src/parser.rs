//! The parser code. Currently generated with chumsky

use crate::ast::{ExprArena, ExprAst, Literal};
use chumsky::prelude::*;
use std::result;

#[derive(Debug)]
enum Expr<'src> {
    Num(f64),
    Var(&'src str),

    Neg(Box<Expr<'src>>),
    Add(Box<Expr<'src>>, Box<Expr<'src>>),
    Sub(Box<Expr<'src>>, Box<Expr<'src>>),
    Mul(Box<Expr<'src>>, Box<Expr<'src>>),
    Div(Box<Expr<'src>>, Box<Expr<'src>>),

    Call(&'src str, Vec<Expr<'src>>),
    Let {
        name: &'src str,
        rhs: Box<Expr<'src>>,
        then: Box<Expr<'src>>,
    },
    Fn {
        name: &'src str,
        args: Vec<&'src str>,
        body: Box<Expr<'src>>,
        then: Box<Expr<'src>>,
    },
}

pub fn module_parser<'src>() -> impl Parser<'src, &'src str, ExprAst, extra::Err<Rich<'src, char>>>
{
    let mut arena = ExprArena::new();

    let int =
        text::int(10).map(|s: &str| ExprAst::Literal(Literal::Int(s.parse::<i64>().unwrap())));

    let float = text::int(10)
        .then(just('.'))
        .then(text::int(10).or_not())
        .to_slice()
        .map(|s: &str| ExprAst::Literal(Literal::Float(s.parse::<f64>().unwrap())));

    let atom = choice((float, int)).padded();

    atom
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_int() {
        // Our parser expects only numbers
        assert_eq!(
            module_parser().parse("1").into_result(),
            Ok(ExprAst::Literal(Literal::Int(1)))
        );
        assert_eq!(
            module_parser().parse("123").into_result(),
            Ok(ExprAst::Literal(Literal::Int(123)))
        );
        assert_eq!(
            module_parser().parse("     123      ").into_result(),
            Ok(ExprAst::Literal(Literal::Int(123)))
        );
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_parse_float() {
        // Our parser expects only numbers
        assert_eq!(
            module_parser().parse("1.23").into_result(),
            Ok(ExprAst::Literal(Literal::Float(1.23)))
        );

        assert_eq!(
            module_parser().parse("1.1").into_result(),
            Ok(ExprAst::Literal(Literal::Float(1.1)))
        );
        assert_eq!(
            module_parser().parse("3.14159").into_result(),
            Ok(ExprAst::Literal(Literal::Float(3.14159)))
        );

        assert_eq!(
            module_parser().parse("3.").into_result(),
            Ok(ExprAst::Literal(Literal::Float(3.)))
        );
    }
}
