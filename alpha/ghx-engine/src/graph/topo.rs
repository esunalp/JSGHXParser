//! Topologische utilities.

use std::collections::{HashMap, VecDeque};
use std::fmt;

use super::{Graph, node::NodeId};

/// Resultaat van een topologische sortering.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Topology {
    pub order: Vec<NodeId>,
}

/// Fouttype voor topologische sortering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopologyError {
    /// De graph bevat een cyclus. Bevat een pad dat de cyclus illustreert.
    Cycle { cycle: Vec<NodeId> },
}

impl fmt::Display for TopologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cycle { cycle } => {
                if cycle.is_empty() {
                    f.write_str("graph bevat een cyclus")
                } else {
                    let chain = cycle
                        .iter()
                        .map(|NodeId(id)| id.to_string())
                        .collect::<Vec<_>>()
                        .join(" -> ");
                    write!(f, "graph bevat een cyclus: {chain}")
                }
            }
        }
    }
}

impl std::error::Error for TopologyError {}

impl Topology {
    /// Construeert een lege topologie.
    #[must_use]
    pub fn empty() -> Self {
        Self { order: Vec::new() }
    }

    /// Voert een topologische sortering uit met behulp van het Kahn algoritme.
    pub fn sort(graph: &Graph) -> Result<Self, TopologyError> {
        if graph.node_count() == 0 {
            return Ok(Self::empty());
        }

        let mut indegree: HashMap<NodeId, usize> = HashMap::new();
        let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

        for node in graph.nodes() {
            indegree.entry(node.id).or_insert(0);
            adjacency.entry(node.id).or_default();
        }

        for wire in graph.wires() {
            adjacency
                .entry(wire.from_node)
                .or_default()
                .push(wire.to_node);
            *indegree.entry(wire.to_node).or_insert(0) += 1;
        }

        for neighbours in adjacency.values_mut() {
            neighbours.sort();
        }

        let mut zero_indegree: Vec<NodeId> = indegree
            .iter()
            .filter_map(|(node, &count)| (count == 0).then_some(*node))
            .collect();
        zero_indegree.sort();

        let mut queue: VecDeque<NodeId> = zero_indegree.into();
        let mut order = Vec::with_capacity(graph.node_count());

        while let Some(node) = queue.pop_front() {
            order.push(node);
            if let Some(neighbours) = adjacency.get(&node) {
                for neighbour in neighbours {
                    if let Some(count) = indegree.get_mut(neighbour) {
                        *count -= 1;
                        if *count == 0 {
                            queue.push_back(*neighbour);
                        }
                    }
                }
            }
        }

        if order.len() == graph.node_count() {
            return Ok(Self { order });
        }

        let cycle = find_cycle(&adjacency).unwrap_or_default();
        Err(TopologyError::Cycle { cycle })
    }
}

fn find_cycle(adjacency: &HashMap<NodeId, Vec<NodeId>>) -> Option<Vec<NodeId>> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum VisitState {
        Unvisited,
        Visiting,
        Visited,
    }

    fn dfs(
        node: NodeId,
        adjacency: &HashMap<NodeId, Vec<NodeId>>,
        state: &mut HashMap<NodeId, VisitState>,
        stack: &mut Vec<NodeId>,
    ) -> Option<Vec<NodeId>> {
        state.insert(node, VisitState::Visiting);
        stack.push(node);

        if let Some(neighbours) = adjacency.get(&node) {
            for neighbour in neighbours {
                match state
                    .get(neighbour)
                    .copied()
                    .unwrap_or(VisitState::Unvisited)
                {
                    VisitState::Unvisited => {
                        if let Some(cycle) = dfs(*neighbour, adjacency, state, stack) {
                            return Some(cycle);
                        }
                    }
                    VisitState::Visiting => {
                        if let Some(position) = stack.iter().position(|&n| n == *neighbour) {
                            let mut cycle = stack[position..].to_vec();
                            cycle.push(*neighbour);
                            return Some(cycle);
                        }
                    }
                    VisitState::Visited => {}
                }
            }
        }

        stack.pop();
        state.insert(node, VisitState::Visited);
        None
    }

    let mut state: HashMap<NodeId, VisitState> = HashMap::new();
    for node in adjacency.keys() {
        if state.get(node).copied().unwrap_or(VisitState::Unvisited) == VisitState::Unvisited {
            let mut stack = Vec::new();
            if let Some(cycle) = dfs(*node, adjacency, &mut state, &mut stack) {
                return Some(cycle);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{Topology, TopologyError};
    use crate::graph::Graph;
    use crate::graph::node::Node;
    use crate::graph::wire::Wire;

    #[test]
    fn sorts_simple_graph() {
        let mut graph = Graph::new();
        let node_a = graph
            .add_node(Node::new(crate::graph::node::NodeId::new(0)))
            .unwrap();
        let node_b = graph
            .add_node(Node::new(crate::graph::node::NodeId::new(1)))
            .unwrap();
        let node_c = graph
            .add_node(Node::new(crate::graph::node::NodeId::new(2)))
            .unwrap();

        graph.add_wire(Wire::new(node_a, "A", node_b, "B")).unwrap();
        graph.add_wire(Wire::new(node_b, "A", node_c, "B")).unwrap();

        let topology = Topology::sort(&graph).expect("topologie");
        assert_eq!(topology.order, vec![node_a, node_b, node_c]);
    }

    #[test]
    fn detects_cycle() {
        let mut graph = Graph::new();
        let node_a = graph
            .add_node(Node::new(crate::graph::node::NodeId::new(0)))
            .unwrap();
        let node_b = graph
            .add_node(Node::new(crate::graph::node::NodeId::new(1)))
            .unwrap();

        graph.add_wire(Wire::new(node_a, "A", node_b, "B")).unwrap();
        graph.add_wire(Wire::new(node_b, "B", node_a, "A")).unwrap();

        let err = Topology::sort(&graph).expect_err("cycle gedetecteerd");
        match err {
            TopologyError::Cycle { cycle } => {
                assert!(cycle.contains(&node_a));
                assert!(cycle.contains(&node_b));
            }
        }
    }
}
