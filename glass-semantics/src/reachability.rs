use std::collections::HashSet;

type ID = usize;
#[derive(Debug, Default, Clone)]
pub(crate) struct Reachability {
    upsets: Vec<HashSet<ID>>,
    downsets: Vec<HashSet<ID>>,
}

impl Reachability {
    pub fn add_node(&mut self) -> ID {
        let i = self.upsets.len();

        let mut set = HashSet::with_capacity(1);
        set.insert(i);

        self.upsets.push(set.clone());
        self.downsets.push(set);
        i
    }

    pub fn add_edge(&mut self, lhs: ID, rhs: ID, out: &mut Vec<(ID, ID)>) {
        if self.downsets[lhs].contains(&rhs) {
            return;
        }

        // Get all ancestors of lhs, including lhs itself
        let mut lhs_set: Vec<ID> = self.upsets[lhs].iter().cloned().collect();
        lhs_set.sort_unstable();

        // Get all descendents of rhs, including rhs itself
        let mut rhs_set: Vec<ID> = self.downsets[rhs].iter().cloned().collect();
        rhs_set.sort_unstable();

        for &lhs2 in &lhs_set {
            for &rhs2 in &rhs_set {
                if self.downsets[lhs2].insert(rhs2) {
                    self.upsets[rhs2].insert(lhs2);
                    out.push((lhs2, rhs2));
                }
            }
        }
    }
}
