//! Contains all the ast code for the language
//!
//! Top level code for a file is ModuleAst

use crate::span::Span;
use std::marker::PhantomData;

#[derive(Debug, Default)]
pub struct AstArena<Ast, Id>
where
    Id: From<usize> + Into<usize> + Copy,
{
    nodes: Vec<Ast>,
    spans: Vec<Span>,
    node_id: PhantomData<Id>,
}

impl<Ast, Id> AstArena<Ast, Id>
where
    Id: From<usize> + Into<usize> + Copy,
{
    pub fn new() -> AstArena<Ast, Id> {
        AstArena {
            nodes: Vec::new(),
            spans: Vec::new(),
            node_id: Default::default(),
        }
    }

    /// Add a new node to the arena
    pub fn new_node(&mut self, ast: Ast, span: Span) -> Id {
        let id = self.nodes.len();
        self.nodes.push(ast);
        self.spans.push(span);
        Id::from(id)
    }

    pub fn get_node(&self, id: Id) -> Option<&Ast> {
        self.nodes.get(id.into())
    }

    pub fn get_span(&self, id: Id) -> Option<&Span> {
        self.spans.get(id.into())
    }
}

/// Represents any Literal in the syntax
///
/// Examples are bool, ints, strings
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Bool(bool),
    Int(i64),
    Float(f64),
    Char(char),
    String(String),
}

/// The expr ast
#[derive(Debug, PartialEq, Clone)]
pub enum ExprAst {
    Literal(Literal),
    BinOp { lhs: ExprId, op: BinOp, rhs: ExprId },
    Variable(String),
}

impl ExprAst {
    pub fn new_int(u: i64) -> ExprAst {
        ExprAst::Literal(Literal::Int(u))
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExprId(usize);

impl From<usize> for ExprId {
    fn from(n: usize) -> Self {
        ExprId(n)
    }
}

impl From<ExprId> for usize {
    fn from(id: ExprId) -> Self {
        id.0
    }
}

/// An AST arena for Expr's
pub type ExprArena = AstArena<ExprAst, ExprId>;
#[derive(Debug, PartialEq, Clone)]
pub enum BinOp {
    Add,
    Sub,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::{Location, SpanFactory};

    #[test]
    fn test_expr_arena() {
        // Create a new arena
        let mut arena = ExprArena::new();
        let mut spans = SpanFactory::new("example.gs");

        // Add some new nodes to the arena
        let a_span = spans.span(0, 1);
        let a = arena.new_node(ExprAst::new_int(1), a_span.clone());

        let b_span = spans.span(0, 1);
        let b = arena.new_node(ExprAst::new_int(2), b_span.clone());

        // The bin op21
        let bin_op = ExprAst::BinOp {
            lhs: a,
            op: BinOp::Add,
            rhs: b,
        };
        let c_span = spans.span(0, 3);
        let c = arena.new_node(bin_op.clone(), c_span.clone());

        assert_eq!(&bin_op, arena.get_node(c).expect("couldn't get bin_op"));
        assert_eq!(
            &ExprAst::new_int(1),
            arena.get_node(a).expect("couldn't get a")
        );
        assert_eq!(
            &ExprAst::new_int(2),
            arena.get_node(b).expect("couldn't get b")
        );

        assert_eq!(&a_span, arena.get_span(a).expect("couldn't get a_span"));
        assert_eq!(&b_span, arena.get_span(b).expect("couldn't get b_span"));
        assert_eq!(&c_span, arena.get_span(c).expect("couldn't get c_span"));
    }
}
