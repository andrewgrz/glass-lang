use crate::typecheck_engine::UTypeHead::UNumeric;
use glass_syntax::ast::{BinOp, ExprArena, ExprAst, ExprId, Literal};
use std::collections::HashMap;
use std::{error, fmt};

type ID = usize;

/// Opaque handle — the type of a value (positive / producer)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Value(ID);

/// Opaque handle — a type constraint (negative / consumer)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Use(ID);

#[derive(Debug, PartialEq)]
pub struct TypeError(String);
impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl error::Error for TypeError {}

/// Value type heads — what a value IS
#[derive(Debug, Clone, PartialEq)]
pub enum VTypeHead {
    VBool,
    VInt,
    VFloat,
    VString,
    VChar,
    VFunc { args: Vec<Use>, ret: Value },
    VObj { fields: HashMap<String, Value> },
    VCase { tag: String, val: Value },
}

/// Use type heads — what a use site EXPECTS
#[derive(Debug, Clone, PartialEq)]
pub enum UTypeHead {
    UBool,
    UInt,
    UFloat,
    UString,
    UChar,
    UFunc {
        args: Vec<Value>,
        ret: Use,
    },
    UObj {
        field: String,
        val: Use,
    },
    UCase {
        cases: HashMap<String, Use>,
    },

    /// Marker: "must be some numeric type"
    UNumeric,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeNode {
    Var,
    Value(VTypeHead),
    Use(UTypeHead),
}

pub struct Bindings {
    m: HashMap<String, Value>,
}
impl Bindings {
    /// Create a new bindings
    fn new() -> Self {
        Self { m: HashMap::new() }
    }

    /// retrieve an optional value from the bindings
    /// returns none if the value is not in the bindings
    fn get(&self, k: &str) -> Option<Value> {
        self.m.get(k).copied()
    }

    /// Add a new binding
    ///
    /// `k` is the binding names
    /// `v` is the value we are binding to
    fn insert(&mut self, k: String, v: Value) {
        self.m.insert(k.clone(), v);
    }

    /// execute the next code in the child scope, used when going into a deeper scope
    fn in_child_scope<T>(&mut self, cb: impl FnOnce(&mut Self) -> T) -> T {
        let mut child_scope = Bindings { m: self.m.clone() };
        cb(&mut child_scope)
    }
}

/// The engine in the backend for managing the types during typechecking
#[derive(Debug, Default)]
pub struct TypeCheckerEngine {
    r: crate::reachability::Reachability,
    types: Vec<TypeNode>,
}

impl TypeCheckerEngine {
    pub fn new() -> Self {
        Self {
            r: Default::default(),
            types: Vec::new(),
        }
    }

    /// Internal function to create a new value in the engine
    fn new_val(&mut self, val_type: VTypeHead) -> Value {
        let i = self.r.add_node();
        assert_eq!(i, self.types.len());
        self.types.push(TypeNode::Value(val_type));
        Value(i)
    }

    /// Internal function to create a new use in the engine
    fn new_use(&mut self, constraint: UTypeHead) -> Use {
        let i = self.r.add_node();
        assert_eq!(i, self.types.len());
        self.types.push(TypeNode::Use(constraint));
        Use(i)
    }

    /// Retrieve the type node for the value
    pub fn get_type_node(&self, v: Value) -> Option<&TypeNode> {
        self.types.get(v.0)
    }

    /// Retrieve the type head for the value
    ///
    /// # Panics
    /// Function panics if the provided value is not in the types
    pub fn value_head(&self, val: Value) -> Option<&VTypeHead> {
        if let Some(v) = self.get_type_node(val) {
            return match v {
                TypeNode::Value(head) => Some(head),
                TypeNode::Var => None,    // It's a variable, not yet resolved
                TypeNode::Use(_) => None, // shouldn't happen — Value shouldn't wrap a Use node
            };
        };
        panic!("ICE: invalid value passed to value head: {:?}", val);
    }

    /// the core of the typechecking algo. flows the types backwards and forwards as needed
    pub fn flow(&mut self, lhs: Value, rhs: Use) -> Result<(), TypeError> {
        let mut pending_edges = vec![(lhs, rhs)];
        let mut type_pairs_to_check = Vec::new();
        while let Some((lhs, rhs)) = pending_edges.pop() {
            self.r.add_edge(lhs.0, rhs.0, &mut type_pairs_to_check);

            // Check if adding that edge resulted in any new type pairs needing to be checked
            while let Some((lhs, rhs)) = type_pairs_to_check.pop() {
                if let TypeNode::Value(lhs_head) = &self.types[lhs] {
                    if let TypeNode::Use(rhs_head) = &self.types[rhs] {
                        check_heads(lhs_head, rhs_head, &mut pending_edges)?;
                    }
                }
            }
        }
        assert!(pending_edges.is_empty() && type_pairs_to_check.is_empty());
        Ok(())
    }

    /// Create a type variable — a single node that bridges Value and Use.
    /// Reading it yields a Value. Writing to it consumes through a Use.
    /// Transitivity of flow ensures: anything that flows into the Use
    /// side is compatible with anything the Value side flows to.
    pub fn var(&mut self) -> (Value, Use) {
        let i = self.r.add_node();
        assert_eq!(i, self.types.len());
        self.types.push(TypeNode::Var);
        (Value(i), Use(i))
    }

    /// insert a bool value to the engine
    pub fn bool_value(&mut self) -> Value {
        self.new_val(VTypeHead::VBool)
    }
    /// insert a bool use to the engine
    pub fn bool_use(&mut self) -> Use {
        self.new_use(UTypeHead::UBool)
    }

    /// insert an int value to the engine
    pub fn int_value(&mut self) -> Value {
        self.new_val(VTypeHead::VInt)
    }
    /// insert an int use to the engine
    pub fn int_use(&mut self) -> Use {
        self.new_use(UTypeHead::UInt)
    }

    /// insert a float value to the engine
    pub fn float_value(&mut self) -> Value {
        self.new_val(VTypeHead::VFloat)
    }
    /// insert a float use to the engine
    pub fn float_use(&mut self) -> Use {
        self.new_use(UTypeHead::UFloat)
    }

    /// insert a generic number use to the engine
    pub fn numeric_use(&mut self) -> Use {
        self.new_use(UTypeHead::UNumeric)
    }
}

/// check_heads is called whenever a value type head flows to a use type head, in order to ensure the types are compatible.
pub fn check_heads(
    lhs: &VTypeHead,
    rhs: &UTypeHead,
    out: &mut Vec<(Value, Use)>,
) -> Result<(), TypeError> {
    use UTypeHead::*;
    use VTypeHead::*;

    match (lhs, rhs) {
        (&VBool, &UBool) => Ok(()),
        (&VInt, &UInt) => Ok(()),
        (&VInt, &UNumeric) => Ok(()),
        (&VFloat, &UFloat) => Ok(()),
        (&VFloat, &UNumeric) => Ok(()),
        _ => Err(TypeError("Unexpected types".to_string())),
    }
}

/// Runs the typecheck on an expr
pub fn check_expr(
    engine: &mut TypeCheckerEngine,
    bindings: &mut Bindings,
    expr_arena: &ExprArena,
    expr_id: &ExprId,
) -> Result<Value, TypeError> {
    let expr = expr_arena
        .get_node(*expr_id)
        .expect("ICE: ExprId not found");

    match expr {
        ExprAst::Literal(val) => Ok(match val {
            Literal::Bool(_) => engine.bool_value(),
            Literal::Int(_) => engine.int_value(),
            Literal::Float(_) => engine.float_value(),
            _ => unimplemented!(),
        }),
        ExprAst::Variable(name) => bindings
            .get(name.as_str())
            .ok_or_else(|| TypeError(format!("Undefined variable {}", name))),
        ExprAst::BinOp { op, lhs, rhs } => {
            let lhs_val = check_expr(engine, bindings, expr_arena, lhs)?;
            let rhs_val = check_expr(engine, bindings, expr_arena, rhs)?;

            match op {
                BinOp::Add | BinOp::Sub => {
                    // Fast path: both operand types are already concrete.
                    // This catches literal arithmetic immediately with precise errors.
                    if let (Some(lhs_head), Some(rhs_head)) =
                        (engine.value_head(lhs_val), engine.value_head(rhs_val))
                    {
                        use VTypeHead::*;

                        return match (lhs_head, rhs_head) {
                            (VInt, VInt) => Ok(engine.int_value()),
                            (VFloat, VFloat) => Ok(engine.float_value()),
                            _ => Err(TypeError(format!(
                                "Incompatible types for binary op {:?}",
                                op
                            ))),
                        };
                    }
                }
            }

            // Fallback path: at least one operand is a variable.
            // Use the constraint-based approach so generic code works.
            let (result_val, result_use) = engine.var();
            engine.flow(lhs_val, result_use)?;
            engine.flow(rhs_val, result_use)?;

            let num_use = engine.numeric_use();
            engine.flow(result_val, num_use)?;
            Ok(result_val)
        }
    }
}

#[cfg(test)]
mod check_heads_tests {
    use super::*;

    #[test]
    fn test_bool() {
        assert!(check_heads(&VTypeHead::VBool, &UTypeHead::UBool, &mut Vec::new()).is_ok());
    }

    #[test]
    fn test_bool_mismatch() {
        assert!(check_heads(&VTypeHead::VBool, &UTypeHead::UInt, &mut Vec::new()).is_err());
    }

    #[test]
    fn test_int() {
        assert!(check_heads(&VTypeHead::VInt, &UTypeHead::UInt, &mut Vec::new()).is_ok());
    }
}

#[cfg(test)]
mod check_expr_tests {
    use super::*;
    use glass_syntax::ast::Literal::Bool;
    use glass_syntax::span::SpanFactory;

    struct AstHelper {
        pub expr_arena: ExprArena,
        pub span_factory: SpanFactory,
    }

    impl AstHelper {
        fn new() -> AstHelper {
            AstHelper {
                expr_arena: ExprArena::new(),
                span_factory: SpanFactory::new("example.gs"),
            }
        }

        fn add_expr(&mut self, expr: ExprAst) -> ExprId {
            self.expr_arena
                .new_node(expr.clone(), self.span_factory.span(0, 1))
        }

        fn bool(&mut self, b: bool) -> ExprId {
            self.add_expr(ExprAst::Literal(Bool(b)))
        }

        fn int(&mut self, i: i64) -> ExprId {
            self.add_expr(ExprAst::Literal(Literal::Int(i)))
        }

        fn float(&mut self, f: f64) -> ExprId {
            self.add_expr(ExprAst::Literal(Literal::Float(f)))
        }

        fn add(&mut self, a: ExprId, b: ExprId) -> ExprId {
            self.add_expr(ExprAst::BinOp {
                lhs: a,
                rhs: b,
                op: BinOp::Add,
            })
        }
    }

    fn check_expr_helper(expr_arena: &ExprArena, expr_id: &ExprId) -> Result<Value, TypeError> {
        let mut engine = TypeCheckerEngine::new();
        let mut bindings = Bindings::new();

        check_expr(&mut engine, &mut bindings, expr_arena, expr_id)
    }

    #[test]
    fn test_literal_bool() {
        let mut ast = AstHelper::new();
        let expr_id = ast.bool(true);
        assert!(check_expr_helper(&ast.expr_arena, &expr_id).is_ok());
    }

    #[test]
    fn test_literal_int() {
        let mut ast = AstHelper::new();
        let expr_id = ast.int(1);
        assert!(check_expr_helper(&ast.expr_arena, &expr_id).is_ok());
    }

    #[test]
    fn test_int_add() {
        let mut ast = AstHelper::new();
        let a = ast.int(1);
        let b = ast.int(2);
        let op = ast.add(a, b);
        assert!(check_expr_helper(&ast.expr_arena, &op).is_ok());
    }

    #[test]
    fn test_int_float_add() {
        let mut ast = AstHelper::new();
        let a = ast.int(1);
        let b = ast.float(2.2);
        let op = ast.add(a, b);
        assert!(check_expr_helper(&ast.expr_arena, &op).is_err());
    }

    #[test]
    fn test_float_int_add() {
        let mut ast = AstHelper::new();
        let a = ast.float(1.1);
        let b = ast.int(2);
        let op = ast.add(a, b);
        assert!(check_expr_helper(&ast.expr_arena, &op).is_err());
    }

    #[test]
    fn test_float_add() {
        let mut ast = AstHelper::new();
        let a = ast.float(1.1);
        let b = ast.float(2.2);
        let op = ast.add(a, b);
        assert!(check_expr_helper(&ast.expr_arena, &op).is_ok());
    }
}
