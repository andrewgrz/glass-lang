//! AstArena is an arena for keep the ast nodes in linear memory
//! It replaces using boxes randomly on the heap. Ast Nodes will contain
//! links to the other nodes via the indexes which are returned when adding
//! a new node. Check out `ExprArena` for a concrete example of using

use std::marker::PhantomData;
use crate::span::Span;

/// An arena for AstNodes. Generic over the Ast Type and matching Id
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