//! The parser code. Currently generated with chumsky

use crate::ast::{ExprAst, Literal};
use chumsky::prelude::*;

pub fn module_parser<'src>() -> impl Parser<'src, &'src str, ExprAst, extra::Err<Rich<'src, char>>>
{
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
