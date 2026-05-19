//! The parser code. Currently generated with chumsky

use crate::ast::*;
use chumsky::input::MapExtra;
use chumsky::prelude::*;
use std::cell::RefCell;

// Define a state type
use chumsky::extra::SimpleState;

type ParserState<'src> = SimpleState<Vec<Rich<'src, char>>>;
type Extra<'src> = extra::Full<Rich<'src, char>, ParserState<'src>, ()>;
type MExtra<'src, 'a> = MapExtra<'src, 'a, &'src str, Extra<'src>>;

fn name<'src>() -> impl Parser<'src, &'src str, String, Extra<'src>> + Clone {
    text::ident().padded().map(|s: &str| s.to_string())
}

fn expr_parser<'src: 'arena, 'arena>(
    expr_arena: &'arena RefCell<ExprArena>,
) -> impl Parser<'src, &'src str, ExprId, Extra<'src>> + 'arena {
    recursive(|expr| {
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

        let def_call = name()
            .then_ignore(just('('))
            .then(
                expr.clone()
                    .padded()
                    .separated_by(just(','))
                    .collect::<Vec<ExprId>>(),
            )
            .then_ignore(just(')'))
            .map_with(|(name, args), extra| {
                expr_arena
                    .borrow_mut()
                    .new_node(ExprAst::FuncCall { name, args }, extra.span())
            });

        let variable = name().map_with(|s: String, extra| {
            expr_arena
                .borrow_mut()
                .new_node(ExprAst::Variable(s.to_string()), extra.span())
        });

        let atom = choice((float, int, def_call, variable))
            .or(expr.clone().delimited_by(just('('), just(')')))
            .padded();

        let op = |c| just(c).padded();

        let unary = op('-').repeated().foldr_with(atom, |_op, rhs, extra| {
            expr_arena.borrow_mut().new_node(
                ExprAst::UnaryOp {
                    op: BinOp::Neg,
                    rhs,
                },
                extra.span(),
            )
        });

        let product = unary.clone().foldl_with(
            choice((op('*').to(BinOp::Mul), op('/').to(BinOp::Div)))
                .then(unary)
                .repeated(),
            |lhs, (op, rhs), extra| {
                expr_arena
                    .borrow_mut()
                    .new_node(ExprAst::BinOp { lhs, op, rhs }, extra.span())
            },
        );

        let sum = product.clone().foldl_with(
            choice((op('+').to(BinOp::Add), op('-').to(BinOp::Sub)))
                .then(product)
                .repeated(),
            |lhs, (op, rhs), extra| {
                expr_arena
                    .borrow_mut()
                    .new_node(ExprAst::BinOp { lhs, op, rhs }, extra.span())
            },
        );

        let r#let = text::ascii::keyword("let")
            .ignore_then(name())
            .then_ignore(just('='))
            .then(expr)
            .map_with(|(name, rhs), extra| {
                expr_arena
                    .borrow_mut()
                    .new_node(ExprAst::Let { name, rhs }, extra.span())
            });

        choice((r#let, sum))
    })
}

fn type_annotation<'src>() -> impl Parser<'src, &'src str, String, Extra<'src>> {
    just(':').ignore_then(name())
}

/// Parses a module (typically a filename)
pub fn module_parser<'src: 'arena, 'arena>(
    expr_arena: &'arena RefCell<ExprArena>,
    args_arena: &'arena RefCell<ArgAstArena>,
) -> impl Parser<'src, &'src str, Vec<ExprId>, Extra<'src>> {
    let stmt = def(expr_arena, args_arena).padded();
    stmt.repeated().collect::<Vec<ExprId>>()
}

fn def<'src: 'arena, 'arena>(
    expr_arena: &'arena RefCell<ExprArena>,
    args_arena: &'arena RefCell<ArgAstArena>,
) -> impl Parser<'src, &'src str, ExprId, Extra<'src>> + 'arena {
    let func_arg = name().then(type_annotation().or_not()).map_with(
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
                .recover_with(via_parser(
                    none_of(")").repeated().at_least(1).to_slice().map_with(
                        |bad: &str, extra: &mut MExtra| {
                            let span = extra.span();
                            extra
                                .state()
                                .push(Rich::custom(span, format!("invalid function param '{bad}', expected identifier")));
                            ArgId::from(0)
                        },
                    ),
                ))
                .padded()
                .separated_by(just(','))
                .collect::<Vec<ArgId>>(),
        )
        .then_ignore(just(')'));

    let body = just('{')
        .ignore_then(
            expr_parser(expr_arena).recover_with(via_parser(
                none_of("}").repeated().at_least(1).to_slice().map(|_| ExprId::from(0)),
            ))
                .separated_by(just(';'))
                .collect::<Vec<ExprId>>(),
        )
        .then_ignore(just('}'));

    just("def")
        .padded()
        .ignore_then(name().recover_with(via_parser(
            none_of("(").repeated().at_least(1).to_slice().map_with(
                |bad: &str, extra: &mut MExtra| {
                    let span = extra.span();
                    extra
                        .state()
                        .push(Rich::custom(span, format!("invalid function name '{bad}'")));
                    "<error>".to_string()
                },
            ),
        )))
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
        )
}

#[cfg(test)]
mod parse_expr_test {
    use super::*;
    use crate::ast::ExprAst::Variable;

    fn runner_helper(input: &str) -> (ExprAst, ExprArena) {
        let arena = RefCell::new(ExprArena::new());

        let parse_result = expr_parser(&arena).parse(input).into_result();
        let inner_arena = arena.into_inner();

        if parse_result.is_err() {
            dbg!(&parse_result);
        }

        assert!(parse_result.is_ok());

        (
            inner_arena.get_node(parse_result.unwrap()).unwrap().clone(),
            inner_arena,
        )
    }

    fn run_test(input: &str, expected: ExprAst) {
        assert_eq!(expected, runner_helper(input).0);
    }

    fn run_negative(input: &str, rhs_expected: ExprAst) {
        let (result, inner_arena) = runner_helper(input);
        match result {
            ExprAst::UnaryOp { op: _op, rhs } => {
                assert_eq!(rhs_expected, *inner_arena.get_node(rhs).unwrap());
            }
            _ => panic!("Expected UnaryOp"),
        }
    }

    fn run_let(input: &str, name_expected: &str, rhs_expected: ExprAst) {
        let (result, inner_arena) = runner_helper(input);
        match result {
            ExprAst::Let { name, rhs } => {
                assert_eq!(name_expected, &name);
                assert_eq!(rhs_expected, *inner_arena.get_node(rhs).unwrap());
            }
            _ => panic!("Expected Let"),
        }
    }

    fn run_def_call(input: &str, name_expected: &str, args_expected: Vec<ExprAst>) {
        let (result, inner_arena) = runner_helper(input);
        match result {
            ExprAst::FuncCall { name, args } => {
                assert_eq!(name_expected, &name);
                assert_eq!(
                    args_expected,
                    args.iter()
                        .map(|arg| inner_arena.get_node(*arg).unwrap().clone())
                        .collect::<Vec<ExprAst>>()
                );
            }
            _ => panic!("Expected Let"),
        }
    }

    fn run_binop(
        input: &str,
        lhs_expected: ExprAst,
        bin_op_expected: BinOp,
        rhs_expected: ExprAst,
    ) {
        let (result, inner_arena) = runner_helper(input);
        match result {
            ExprAst::BinOp { lhs, op, rhs } => {
                assert_eq!(
                    lhs_expected,
                    *inner_arena.get_node(lhs).unwrap(),
                    "lhs mismatch"
                );
                assert_eq!(bin_op_expected, op);
                assert_eq!(
                    rhs_expected,
                    *inner_arena.get_node(rhs).unwrap(),
                    "rhs mismatch"
                );
            }
            _ => panic!("Expected BinOp"),
        }
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
        assert_eq!(name().parse("Test").into_result(), Ok("Test".to_string()));
        assert_eq!(name().parse("test").into_result(), Ok("test".to_string()));
        assert_eq!(name().parse("test5").into_result(), Ok("test5".to_string()));
        assert_eq!(
            name().parse("test5_test").into_result(),
            Ok("test5_test".to_string())
        );
        assert_eq!(
            name().parse("test5_Test").into_result(),
            Ok("test5_Test".to_string())
        );
    }

    #[test]
    fn test_parse_name_emojis_fail() {
        assert!(name().parse("a😉").into_result().is_err());
        assert!(name().parse("🚀").into_result().is_err());
        assert!(name().parse("💕you").into_result().is_err());
        assert!(name().parse("you💕").into_result().is_err());
        assert!(name().parse("You💕").into_result().is_err());
    }

    #[test]
    fn test_parse_name_failures() {
        assert!(name().parse("1").into_result().is_err());
        assert!(name().parse("2").into_result().is_err());
        assert!(name().parse("4adfasdfs").into_result().is_err());
        assert!(name().parse("test with spaces").into_result().is_err());
    }

    #[test]
    fn test_parse_neg() {
        run_negative("-1", ExprAst::Literal(Literal::Int(1)));
        run_negative("-10", ExprAst::Literal(Literal::Int(10)));
        run_negative("-20", ExprAst::Literal(Literal::Int(20)));
        run_negative("-944.560", ExprAst::Literal(Literal::Float(944.560)));
    }

    #[test]
    fn test_parse_add() {
        let one_expr = ExprAst::Literal(Literal::Int(1));
        run_binop("1+1", one_expr.clone(), BinOp::Add, one_expr.clone());
        run_binop("1 +1", one_expr.clone(), BinOp::Add, one_expr.clone());
        run_binop("1+ 1", one_expr.clone(), BinOp::Add, one_expr.clone());
        run_binop("1 + 1", one_expr.clone(), BinOp::Add, one_expr.clone());
    }

    #[test]
    fn test_parse_subtract() {
        let one_expr = ExprAst::Literal(Literal::Int(1));
        run_binop("1-1", one_expr.clone(), BinOp::Sub, one_expr.clone());
        run_binop("1 -1", one_expr.clone(), BinOp::Sub, one_expr.clone());
        run_binop("1- 1", one_expr.clone(), BinOp::Sub, one_expr.clone());
        run_binop("1 - 1", one_expr.clone(), BinOp::Sub, one_expr.clone());
    }

    #[test]
    fn test_parse_multiply() {
        let one_expr = ExprAst::Literal(Literal::Int(1));
        run_binop("1*1", one_expr.clone(), BinOp::Mul, one_expr.clone());
        run_binop("1 *1", one_expr.clone(), BinOp::Mul, one_expr.clone());
        run_binop("1* 1", one_expr.clone(), BinOp::Mul, one_expr.clone());
        run_binop("1 * 1", one_expr.clone(), BinOp::Mul, one_expr.clone());
    }

    #[test]
    fn test_parse_divide() {
        let one_expr = ExprAst::Literal(Literal::Int(1));
        run_binop("1/1", one_expr.clone(), BinOp::Div, one_expr.clone());
        run_binop("1 /1", one_expr.clone(), BinOp::Div, one_expr.clone());
        run_binop("1/ 1", one_expr.clone(), BinOp::Div, one_expr.clone());
        run_binop("1 / 1", one_expr.clone(), BinOp::Div, one_expr.clone());
    }

    #[test]
    fn test_parse_let() {
        run_let("let a = 10", "a", ExprAst::Literal(Literal::Int(10)));
        run_let("let a=10", "a", ExprAst::Literal(Literal::Int(10)));
        run_let("let a= 10", "a", ExprAst::Literal(Literal::Int(10)));
        run_let("let b = name", "b", Variable("name".to_string()));
    }

    #[test]
    fn test_parse_def_call() {
        run_def_call("add()", "add", vec![]);
        run_def_call("add(1)", "add", vec![ExprAst::Literal(Literal::Int(1))]);
        run_def_call(
            "add(1, 2)",
            "add",
            vec![
                ExprAst::Literal(Literal::Int(1)),
                ExprAst::Literal(Literal::Int(2)),
            ],
        );
        run_def_call("add(a)", "add", vec![Variable("a".to_string())]);
        run_def_call(
            "add(a,b)",
            "add",
            vec![Variable("a".to_string()), Variable("b".to_string())],
        );
        run_def_call(
            "add(a ,b)",
            "add",
            vec![Variable("a".to_string()), Variable("b".to_string())],
        );
        run_def_call(
            "add(a, b)",
            "add",
            vec![Variable("a".to_string()), Variable("b".to_string())],
        );
        run_def_call(
            "add( a, b )",
            "add",
            vec![Variable("a".to_string()), Variable("b".to_string())],
        );
    }

    #[test]
    fn test_parse_precedence_rhs() {
        let (result, inner_arena) = runner_helper("1+2*3");
        match result {
            ExprAst::BinOp { lhs, op, rhs } => {
                assert_eq!(
                    ExprAst::Literal(Literal::Int(1)),
                    *inner_arena.get_node(lhs).unwrap(),
                    "lhs mismatch"
                );
                assert_eq!(BinOp::Add, op);

                match inner_arena.get_node(rhs).unwrap() {
                    ExprAst::BinOp { lhs, op, rhs } => {
                        assert_eq!(
                            ExprAst::Literal(Literal::Int(2)),
                            *inner_arena.get_node(*lhs).unwrap(),
                            "lhs mismatch"
                        );
                        assert_eq!(BinOp::Mul, *op);
                        assert_eq!(
                            ExprAst::Literal(Literal::Int(3)),
                            *inner_arena.get_node(*rhs).unwrap(),
                            "rhs mismatch"
                        );
                    }
                    _ => panic!("Expected BinOp"),
                }
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_parse_precedence_flat() {
        let (result, inner_arena) = runner_helper("1+2-3");
        match result {
            ExprAst::BinOp { lhs, op, rhs } => {
                match inner_arena.get_node(lhs).unwrap() {
                    ExprAst::BinOp { lhs, op, rhs } => {
                        assert_eq!(
                            ExprAst::Literal(Literal::Int(1)),
                            *inner_arena.get_node(*lhs).unwrap(),
                            "lhs mismatch"
                        );
                        assert_eq!(BinOp::Add, *op);
                        assert_eq!(
                            ExprAst::Literal(Literal::Int(2)),
                            *inner_arena.get_node(*rhs).unwrap(),
                            "rhs mismatch"
                        );
                    }
                    _ => panic!("Expected BinOp"),
                }
                assert_eq!(BinOp::Sub, op);
                assert_eq!(
                    ExprAst::Literal(Literal::Int(3)),
                    *inner_arena.get_node(rhs).unwrap(),
                    "lhs mismatch"
                );
            }
            _ => panic!("Expected BinOp"),
        }
    }
}

#[cfg(test)]
fn convert_fn(
    expr: ExprAst,
    expr_arena: &ExprArena,
    args_arena: &ArgAstArena,
) -> (String, Vec<ArgAst>, Vec<ExprAst>) {
    match expr {
        ExprAst::FuncDef { name, args, body } => (
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

#[cfg(test)]
mod parse_functions_test {
    use super::*;
    use crate::ast::ExprAst::*;

    fn run_test(input: &str, expected: (String, Vec<ArgAst>, Vec<ExprAst>)) {
        let expr_arena = RefCell::new(ExprArena::new());
        let args_arena = RefCell::new(ArgAstArena::new());

        let parse_result = def(&expr_arena, &args_arena).parse(input).into_result();
        let inner_expr_arena = expr_arena.into_inner();
        let inner_args_arena = args_arena.into_inner();

        if parse_result.is_err() {
            dbg!(&parse_result);
        }
        assert!(parse_result.is_ok());

        let result = convert_fn(
            inner_expr_arena
                .get_node(parse_result.unwrap())
                .unwrap()
                .clone(),
            &inner_expr_arena,
            &inner_args_arena,
        );

        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_empty() {
        run_test(r#"def test() {}"#, ("test".to_string(), vec![], vec![]));
    }

    #[test]
    fn test_parse_empty_body() {
        run_test(
            r#"def test(a) {}"#,
            (
                "test".to_string(),
                vec![ArgAst::new("a".to_string())],
                vec![],
            ),
        );
    }

    #[test]
    fn test_parse_simple_1_arg_function() {
        run_test(
            r#"def test(a) {a}"#,
            (
                "test".to_string(),
                vec![ArgAst::new("a".to_string())],
                vec![Variable("a".to_string())],
            ),
        );
    }

    #[test]
    fn test_parse_simple_1_arg_with_type_function() {
        run_test(
            r#"def test(a: int) {a}"#,
            (
                "test".to_string(),
                vec![ArgAst::new_with_arg_type(
                    "a".to_string(),
                    Some("int".to_string()),
                )],
                vec![Variable("a".to_string())],
            ),
        );
    }

    #[test]
    fn test_parse_simple_2_args_function() {
        run_test(
            r#"def test(a, b) {a}"#,
            (
                "test".to_string(),
                vec![ArgAst::new("a".to_string()), ArgAst::new("b".to_string())],
                vec![Variable("a".to_string())],
            ),
        );
    }

    #[test]
    fn test_parse_simple_2_arg_with_type_function() {
        run_test(
            r#"def test(a: int, b: int) {a}"#,
            (
                "test".to_string(),
                vec![
                    ArgAst::new_with_arg_type("a".to_string(), Some("int".to_string())),
                    ArgAst::new_with_arg_type("b".to_string(), Some("int".to_string())),
                ],
                vec![Variable("a".to_string())],
            ),
        );
    }

    #[test]
    fn test_parse_2_args_2_exprs_function() {
        run_test(
            r#"def test(a, b) {a; b}"#,
            (
                "test".to_string(),
                vec![ArgAst::new("a".to_string()), ArgAst::new("b".to_string())],
                vec![Variable("a".to_string()), Variable("b".to_string())],
            ),
        );
    }

    #[test]
    fn test_parse_2_args_multiple_exprs_function() {
        run_test(
            r#"def test(a, b) {a; 10; c}"#,
            (
                "test".to_string(),
                vec![ArgAst::new("a".to_string()), ArgAst::new("b".to_string())],
                vec![
                    Variable("a".to_string()),
                    ExprAst::Literal(crate::ast::Literal::Int(10)),
                    Variable("c".to_string()),
                ],
            ),
        );
    }
}

#[cfg(test)]
mod parse_module {
    use super::*;
    use crate::ast::ExprAst::*;

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
                convert_fn(
                    inner_expr_arena.get_node(*s).unwrap().clone(),
                    &inner_expr_arena,
                    &inner_args_arena,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(expected, result);
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
