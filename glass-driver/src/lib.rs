//! Driver code to manage the execution of a flow

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::Parser;
use chumsky::extra::SimpleState;
use chumsky::prelude::Rich;
use glass_syntax::ast::{ArgAstArena, ExprArena, ExprId};
use glass_syntax::parser::module_parser;
use std::cell::RefCell;
use std::fs;

struct Arenas {
    expr: RefCell<ExprArena>,
    ast: RefCell<ArgAstArena>,
}

pub type PipelineResult = Result<Vec<ExprId>, ()>;

pub fn pipeline(filename: &str) -> PipelineResult {
    let contents = fs::read_to_string(filename).expect("Should have been able to read the file");

    match parse_contents(
        &contents,
        Arenas {
            expr: RefCell::new(ExprArena::new()),
            ast: RefCell::new(ArgAstArena::new()),
        },
    ) {
        Ok(result) => Ok(result),
        Err(errs) => {
            print_errors(filename, &contents, errs);
            Err(())
        }
    }
}

pub fn print_errors(filename: &str, contents: &str, parse_errs: Vec<Rich<char>>) {
    eprintln!("total errors: {}", parse_errs.len());

    parse_errs.into_iter().for_each(|e| {
        Report::build(ReportKind::Error, (filename, e.span().into_range()))
            .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
            .with_message(e.to_string())
            .with_label(
                Label::new((filename, e.span().into_range()))
                    .with_message(e.reason().to_string())
                    .with_color(Color::Red),
            )
            .finish()
            .print((filename, Source::from(contents)))
            .unwrap()
    });
}

fn parse_contents(contents: &str, arenas: Arenas) -> Result<Vec<ExprId>, Vec<Rich<char>>> {
    let mut state = SimpleState::from(Vec::<Rich<char>>::new());

    let parser = module_parser(&arenas.expr, &arenas.ast);

    let (result, errors) = parser
        .parse_with_state(contents, &mut state)
        .into_output_errors();

    let mut all_errors: Vec<_> = errors
        .into_iter()
        .chain(<Vec<Rich<'_, char>> as Clone>::clone(&state))
        .collect();

    all_errors.sort_by_key(|e| e.span().start);

    if all_errors.is_empty() {
        Ok(result.unwrap())
    } else {
        Err(all_errors)
    }
}

#[cfg(test)]
mod tests {
    use crate::pipeline;

    #[test]
    fn test_pipeline() {
        assert!(pipeline("../examples/simple/adder.gls").is_ok());
    }

    #[test]
    fn test_pipeline_with_errors() {
        let result = pipeline("../examples/simple/bad_syntax.gls");
        assert!(result.is_err());
    }
}
