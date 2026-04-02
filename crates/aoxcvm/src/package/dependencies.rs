//! Package dependency graph validation helpers.

use std::collections::{BTreeMap, BTreeSet};

/// Canonical package identifier.
pub type PackageId = String;

/// Directed dependency graph keyed by package id.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DependencyGraph {
    edges: BTreeMap<PackageId, BTreeSet<PackageId>>,
}

impl DependencyGraph {
    pub fn add_package(&mut self, package: PackageId) {
        self.edges.entry(package).or_default();
    }

    pub fn add_dependency(&mut self, package: PackageId, dependency: PackageId) {
        self.edges.entry(package).or_default().insert(dependency);
    }

    /// Fail-closed cycle detection.
    pub fn has_cycle(&self) -> bool {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Mark {
            Visiting,
            Visited,
        }

        fn visit(
            node: &str,
            graph: &BTreeMap<PackageId, BTreeSet<PackageId>>,
            marks: &mut BTreeMap<PackageId, Mark>,
        ) -> bool {
            if matches!(marks.get(node), Some(Mark::Visiting)) {
                return true;
            }
            if matches!(marks.get(node), Some(Mark::Visited)) {
                return false;
            }

            marks.insert(node.to_string(), Mark::Visiting);
            if let Some(deps) = graph.get(node) {
                for dep in deps {
                    if visit(dep, graph, marks) {
                        return true;
                    }
                }
            }
            marks.insert(node.to_string(), Mark::Visited);
            false
        }

        let mut marks = BTreeMap::new();
        self.edges
            .keys()
            .any(|node| visit(node, &self.edges, &mut marks))
    }
}

#[cfg(test)]
mod tests {
    use crate::package::dependencies::DependencyGraph;

    #[test]
    fn acyclic_graph_passes() {
        let mut graph = DependencyGraph::default();
        graph.add_package("pkg.a".to_string());
        graph.add_dependency("pkg.a".to_string(), "pkg.b".to_string());
        graph.add_dependency("pkg.b".to_string(), "pkg.c".to_string());

        assert!(!graph.has_cycle());
    }

    #[test]
    fn cycle_is_detected() {
        let mut graph = DependencyGraph::default();
        graph.add_dependency("pkg.a".to_string(), "pkg.b".to_string());
        graph.add_dependency("pkg.b".to_string(), "pkg.a".to_string());

        assert!(graph.has_cycle());
    }
}
