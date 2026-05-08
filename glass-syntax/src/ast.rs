//! Contains all the ast code for the language
//!
//! Top level code for a file is ModuleAst (not added yet)

use crate::ast_arena::AstArena;

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
    BinOp {
        lhs: ExprId,
        op: BinOp,
        rhs: ExprId,
    },
    Variable(String),
    Let {
        name: String,
        rhs: ExprId,
    },
    FuncDef {
        name: String,
        args: Vec<ArgAst>,
        body: ExprId,
    },

    FuncCall {
        name: String,
        args: Vec<ExprId>,
    },
}

impl<'a> ExprAst {
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

/// The Argument ast
#[derive(Debug, PartialEq, Clone)]
pub struct ArgAst {
    name: String,
    arg_type: Option<String>,
}

impl ArgAst {
    pub fn new(name: String) -> ArgAst {
        ArgAst {
            name,
            arg_type: None,
        }
    }

    pub fn new_with_arg_type(name: String, arg_type: Option<String>) -> ArgAst {
        ArgAst { name, arg_type }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArgId(usize);

impl From<usize> for ArgId {
    fn from(n: usize) -> Self {
        ArgId(n)
    }
}

impl From<ArgId> for usize {
    fn from(id: ArgId) -> Self {
        id.0
    }
}

/// An AST arena for ArgAst's
pub type ArgAstArena = AstArena<ArgAst, ArgId>;

#[derive(Debug, PartialEq, Clone)]
pub enum BinOp {
    Add,
    Sub,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chumsky::span::SimpleSpan;

    #[test]
    fn test_expr_arena() {
        // Create a new arena
        let mut arena = ExprArena::new();

        // Add some new nodes to the arena
        let a_span = SimpleSpan {
            start: 0,
            end: 1,
            context: (),
        };
        let a = arena.new_node(ExprAst::new_int(1), a_span.clone());

        let b_span = SimpleSpan {
            start: 1,
            end: 2,
            context: (),
        };
        let b = arena.new_node(ExprAst::new_int(2), b_span.clone());

        // The bin op21
        let bin_op = ExprAst::BinOp {
            lhs: a,
            op: BinOp::Add,
            rhs: b,
        };
        let c_span = SimpleSpan {
            start: 4,
            end: 5,
            context: (),
        };
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
