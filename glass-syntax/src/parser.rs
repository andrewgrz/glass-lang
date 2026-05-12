//! The parser code. Currently generated with chumsky

use crate::ast::*;
use chumsky::input::MapExtra;
use chumsky::prelude::*;
use std::cell::RefCell;

type Extra<'src> = extra::Err<Rich<'src, char>>;
type MExtra<'src, 'a> = MapExtra<'src, 'a, &'src str, Extra<'src>>;

fn name_parser<'src>() -> impl Parser<'src, &'src str, String, extra::Err<Rich<'src, char>>> {
    text::ident().padded().map(|s: &str| s.to_string())
}

fn expr_parser<'src>(
    expr_arena: &RefCell<ExprArena>,
) -> impl Parser<'src, &'src str, ExprId, extra::Err<Rich<'src, char>>> {
    let int = text::int(10).map_with(|s: &str, extra| {
        expr_arena.borrow_mut().new_node(
            ExprAst::Literal(Literal::Int(s.parse::<i64>().unwrap())),
            extra.span(),
        )
    });

    let float = text::int(10)
        .then(just('.'))
        .then(text::int(10).or_not())
        .to_slice()
        .map_with(|s: &str, extra| {
            expr_arena.borrow_mut().new_node(
                ExprAst::Literal(Literal::Float(s.parse::<f64>().unwrap())),
                extra.span(),
            )
        });

    let variable = name_parser().map_with(|s: String, extra| {
        expr_arena
            .borrow_mut()
            .new_node(ExprAst::Variable(s.to_string()), extra.span())
    });

    choice((float, int, variable)).padded()
}

pub fn module_parser<'src>(
    expr_arena: &RefCell<ExprArena>,
    args_arena: &RefCell<ArgAstArena>,
) -> impl Parser<'src, &'src str, Vec<ExprId>, extra::Err<Rich<'src, char>>> {
    let type_annotation = just(':').ignore_then(name_parser());
    let func_arg = name_parser().then(type_annotation.or_not()).map_with(
        |(name, ty): (String, Option<String>), extra| {
            args_arena.borrow_mut().new_node(
                ArgAst::new_with_arg_type(name.to_string(), ty),
                extra.span(),
            )
        },
    );

    let func_args = just('(')
        .ignore_then(
            func_arg
                .padded()
                .separated_by(just(','))
                .collect::<Vec<ArgId>>(),
        )
        .then_ignore(just(')'));

    let body = just('{')
        .ignore_then(
            expr_parser(expr_arena)
                .separated_by(just(';'))
                .collect::<Vec<ExprId>>(),
        )
        .then_ignore(just('}'));

    let def = just("def")
        .padded()
        .ignore_then(name_parser())
        .then(func_args)
        .padded()
        .then(body)
        .padded()
        .map_with(
            |((name, args), body): ((String, Vec<ArgId>), Vec<ExprId>), extra: &mut MExtra| {
                expr_arena
                    .borrow_mut()
                    .new_node(ExprAst::FuncDef { name, args, body }, extra.span())
            },
        );

    def.padded().repeated().collect::<Vec<ExprId>>()
}

#[cfg(test)]
mod parse_expr_test {
    use super::*;

    fn run_test(input: &str, expected: ExprAst) {
        let arena = RefCell::new(ExprArena::new());

        let parse_result = expr_parser(&arena).parse(input).into_result();
        let inner_arena = arena.into_inner();

        assert!(parse_result.is_ok());

        let result = inner_arena.get_node(parse_result.unwrap()).unwrap();
        assert_eq!(expected, *result);
    }

    #[test]
    fn test_parse_int() {
        run_test("1", ExprAst::Literal(Literal::Int(1)));
        run_test("10", ExprAst::Literal(Literal::Int(10)));
        run_test("20", ExprAst::Literal(Literal::Int(20)));
        run_test("944560", ExprAst::Literal(Literal::Int(944560)));
    }

    #[test]
    fn test_parse_float() {
        run_test("1.1", ExprAst::Literal(Literal::Float(1.1)));
        run_test("2.23", ExprAst::Literal(Literal::Float(2.23)));
        run_test("3.23", ExprAst::Literal(Literal::Float(3.23)));
        run_test("4.23", ExprAst::Literal(Literal::Float(4.23)));
        run_test("5.23", ExprAst::Literal(Literal::Float(5.23)));
        run_test("6.23", ExprAst::Literal(Literal::Float(6.23)));
        run_test("7.23", ExprAst::Literal(Literal::Float(7.23)));
        run_test("8.23", ExprAst::Literal(Literal::Float(8.23)));
        run_test("9.23", ExprAst::Literal(Literal::Float(9.23)));
        run_test("0.23", ExprAst::Literal(Literal::Float(0.23)));
        run_test("10.23", ExprAst::Literal(Literal::Float(10.23)));
        run_test(
            "10.239999999999999",
            ExprAst::Literal(Literal::Float(10.239999999999999)),
        );
        run_test(
            "10.2399999999999989",
            ExprAst::Literal(Literal::Float(10.239_999_999_999_998)),
        );
    }

    #[test]
    fn test_parse_name() {
        assert_eq!(
            name_parser().parse("Test").into_result(),
            Ok("Test".to_string())
        );
        assert_eq!(
            name_parser().parse("test").into_result(),
            Ok("test".to_string())
        );
        assert_eq!(
            name_parser().parse("test5").into_result(),
            Ok("test5".to_string())
        );
        assert_eq!(
            name_parser().parse("test5_test").into_result(),
            Ok("test5_test".to_string())
        );
        assert_eq!(
            name_parser().parse("test5_Test").into_result(),
            Ok("test5_Test".to_string())
        );
    }

    #[test]
    fn test_parse_name_emojis_fail() {
        assert!(name_parser().parse("a😉").into_result().is_err());
        assert!(name_parser().parse("🚀").into_result().is_err());
        assert!(name_parser().parse("💕you").into_result().is_err());
        assert!(name_parser().parse("you💕").into_result().is_err());
        assert!(name_parser().parse("You💕").into_result().is_err());
    }

    #[test]
    fn test_parse_name_failures() {
        assert!(name_parser().parse("1").into_result().is_err());
        assert!(name_parser().parse("2").into_result().is_err());
        assert!(name_parser().parse("4adfasdfs").into_result().is_err());
        assert!(
            name_parser()
                .parse("test with spaces")
                .into_result()
                .is_err()
        );
    }
}

#[cfg(test)]
mod parse_functions_test {
    use super::*;
    use crate::ast::ExprAst::*;

    fn convert(
        expr: ExprAst,
        expr_arena: &ExprArena,
        args_arena: &ArgAstArena,
    ) -> (String, Vec<ArgAst>, Vec<ExprAst>) {
        match expr {
            FuncDef { name, args, body } => (
                name,
                args.iter()
                    .map(|id| args_arena.get_node(*id).unwrap().clone())
                    .collect(),
                body.iter()
                    .map(|id| expr_arena.get_node(*id).unwrap().clone())
                    .collect(),
            ),
            _ => panic!("not a func expr: {:?}", expr),
        }
    }

    fn run_test(input: &str, expected: Vec<(String, Vec<ArgAst>, Vec<ExprAst>)>) {
        let expr_arena = RefCell::new(ExprArena::new());
        let args_arena = RefCell::new(ArgAstArena::new());

        let parse_result = module_parser(&expr_arena, &args_arena)
            .parse(input)
            .into_result();
        let inner_expr_arena = expr_arena.into_inner();
        let inner_args_arena = args_arena.into_inner();

        if parse_result.is_err() {
            dbg!(&parse_result);
        }
        assert!(parse_result.is_ok());

        let result = parse_result
            .unwrap()
            .iter()
            .map(|s| {
                convert(
                    inner_expr_arena.get_node(*s).unwrap().clone(),
                    &inner_expr_arena,
                    &inner_args_arena,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_empty() {
        run_test(
            r#"def test() {}"#,
            vec![("test".to_string(), vec![], vec![])],
        );
    }

    #[test]
    fn test_parse_empty_body() {
        run_test(
            r#"def test(a) {}"#,
            vec![(
                "test".to_string(),
                vec![ArgAst::new("a".to_string())],
                vec![],
            )],
        );
    }

    #[test]
    fn test_parse_simple_1_arg_function() {
        run_test(
            r#"def test(a) {a}"#,
            vec![(
                "test".to_string(),
                vec![ArgAst::new("a".to_string())],
                vec![Variable("a".to_string())],
            )],
        );
    }

    #[test]
    fn test_parse_simple_2_args_function() {
        run_test(
            r#"def test(a, b) {a}"#,
            vec![(
                "test".to_string(),
                vec![ArgAst::new("a".to_string()), ArgAst::new("b".to_string())],
                vec![Variable("a".to_string())],
            )],
        );
    }

    #[test]
    fn test_parse_2_args_2_exprs_function() {
        run_test(
            r#"def test(a, b) {a; b}"#,
            vec![(
                "test".to_string(),
                vec![ArgAst::new("a".to_string()), ArgAst::new("b".to_string())],
                vec![Variable("a".to_string()), Variable("b".to_string())],
            )],
        );
    }

    #[test]
    fn test_parse_2_args_multiple_exprs_function() {
        run_test(
            r#"def test(a, b) {a; 10; c}"#,
            vec![(
                "test".to_string(),
                vec![ArgAst::new("a".to_string()), ArgAst::new("b".to_string())],
                vec![
                    Variable("a".to_string()),
                    ExprAst::Literal(crate::ast::Literal::Int(10)),
                    Variable("c".to_string()),
                ],
            )],
        );
    }

    #[test]
    fn test_parse_multiple_fns() {
        run_test(
            r#"def test(a) {} def second(b) {b}"#,
            vec![
                (
                    "test".to_string(),
                    vec![ArgAst::new("a".to_string())],
                    vec![],
                ),
                (
                    "second".to_string(),
                    vec![ArgAst::new("b".to_string())],
                    vec![Variable("b".to_string())],
                ),
            ],
        );
    }
}
