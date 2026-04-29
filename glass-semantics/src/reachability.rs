use std::collections::HashSet;

#[cfg(doc)]
use crate::typecheck_engine::{Use, Value};

/// a set that maintains order to help reduce the O(n^3) problem in reachability in the most common cases
///
/// see here for more information on the impl: <https://blog.polybdenum.com/2020/08/01/subtype-inference-by-example-part-5-incremental-reachability.html>
#[derive(Debug, Default)]
struct OrderedSet<T> {
    v: Vec<T>,
    s: HashSet<T>,
}
impl<T: Eq + std::hash::Hash + Clone> OrderedSet<T> {
    fn insert(&mut self, value: T) -> bool {
        if self.s.insert(value.clone()) {
            self.v.push(value);
            true
        } else {
            false
        }
    }

    fn iter(&self) -> std::slice::Iter<T> {
        self.v.iter()
    }
}

type ID = usize;

/// Reachability, the "Cubic" Part, of the typechecking algo.
///
/// * For each node, the set of [Value] nodes that can reach it (upsets)
/// * For each node, the set of [Use] nodes reachable from it (downsets)
#[derive(Debug, Default)]
pub(crate) struct Reachability {
    upsets: Vec<OrderedSet<ID>>,
    downsets: Vec<OrderedSet<ID>>,
}

impl Reachability {
    pub fn add_node(&mut self) -> ID {
        let i = self.upsets.len();
        self.upsets.push(Default::default());
        self.downsets.push(Default::default());
        i
    }

    pub fn add_edge(&mut self, lhs: ID, rhs: ID, out: &mut Vec<(ID, ID)>) {
        let mut work = vec![(lhs, rhs)];

        while let Some((lhs, rhs)) = work.pop() {
            // Insert returns false if the edge is already present
            if !self.downsets[lhs].insert(rhs) {
                continue;
            }
            self.upsets[rhs].insert(lhs);
            // Inform the caller that a new edge was added
            out.push((lhs, rhs));

            for &lhs2 in self.upsets[lhs].iter() {
                work.push((lhs2, rhs));
            }
            for &rhs2 in self.downsets[rhs].iter() {
                work.push((lhs, rhs2));
            }
        }
    }
}

#[cfg(test)]
mod reachability_tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut reachability = Reachability::default();

        assert_eq!(reachability.upsets.len(), 0);
        assert_eq!(reachability.downsets.len(), 0);

        let node_1 = reachability.add_node();
        assert_eq!(reachability.upsets.len(), 1);
        assert_eq!(reachability.downsets.len(), 1);
        assert_eq!(node_1, 0);
    }

    #[test]
    fn test_add_edge() {
        let mut r = Reachability::default();
        for _ in 0..10 {
            r.add_node();
        }

        // Check simple edges are added
        let mut out = Vec::new();
        r.add_edge(0, 8, &mut out);
        assert_eq!(out, vec![(0, 8)]);

        // check that adding the same edge returns empty
        out.clear();
        r.add_edge(0, 8, &mut out);
        assert_eq!(out, vec![]);

        // check adding a bunch of edges to the struct works as expected
        r.add_edge(0, 3, &mut out);
        r.add_edge(1, 3, &mut out);
        r.add_edge(2, 3, &mut out);
        r.add_edge(4, 5, &mut out);
        r.add_edge(4, 6, &mut out);
        r.add_edge(4, 7, &mut out);
        r.add_edge(6, 7, &mut out);
        r.add_edge(9, 1, &mut out);
        r.add_edge(9, 8, &mut out);

        out.clear();
        r.add_edge(3, 4, &mut out);

        let mut expected = Vec::new();
        for &lhs in &[0, 1, 2, 3, 9] {
            for &rhs in &[4, 5, 6, 7] {
                expected.push((lhs, rhs));
            }
        }

        out.sort_unstable();
        expected.sort_unstable();
        assert_eq!(out, expected);
    }
}
