//! Kern datastructuren voor het modelleren van Grasshopper grafen.

use std::collections::HashMap;
use std::fmt;

pub mod evaluator;
pub mod internal_expression;
pub mod node;
pub mod topo;
pub mod value;
pub mod wire;

use node::{Node, NodeId};
use wire::Wire;

/// Graph container met indices voor snelle lookups.
#[derive(Debug, Clone, Default)]
pub struct Graph {
    nodes: Vec<Node>,
    wires: Vec<Wire>,
    node_index: HashMap<NodeId, usize>,
    guid_index: HashMap<String, Vec<NodeId>>,
    name_index: HashMap<String, Vec<NodeId>>,
    next_id: usize,
}

impl Graph {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Voeg een node toe aan de graph. Als `node.id` niet gezet is, wordt een nieuw
    /// id uitgegeven.
    pub fn add_node(&mut self, mut node: Node) -> Result<NodeId, GraphError> {
        let id = if node.id == NodeId::default() {
            let assigned = NodeId::new(self.next_id);
            self.next_id += 1;
            node.id = assigned;
            assigned
        } else {
            self.next_id = self.next_id.max(node.id.0 + 1);
            node.id
        };

        if self.node_index.contains_key(&id) {
            return Err(GraphError::DuplicateNode(id));
        }

        let idx = self.nodes.len();
        self.node_index.insert(id, idx);

        if let Some(guid) = node.guid.as_ref() {
            let key = normalize_guid(guid);
            self.guid_index.entry(key).or_default().push(id);
        }

        if let Some(name) = node.name.as_deref() {
            self.index_name(id, name);
        }
        if let Some(nickname) = node.nickname.as_deref() {
            self.index_name(id, nickname);
        }

        self.nodes.push(node);
        Ok(id)
    }

    /// Voeg een verbinding toe tussen twee bestaande nodes.
    pub fn add_wire(&mut self, wire: Wire) -> Result<(), GraphError> {
        if !self.node_index.contains_key(&wire.from_node) {
            return Err(GraphError::UnknownNode(wire.from_node));
        }
        if !self.node_index.contains_key(&wire.to_node) {
            return Err(GraphError::UnknownNode(wire.to_node));
        }

        self.wires.push(wire);
        Ok(())
    }

    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.node_index
            .get(&id)
            .and_then(|idx| self.nodes.get(*idx))
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.node_index
            .get(&id)
            .copied()
            .and_then(move |idx| self.nodes.get_mut(idx))
    }

    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    #[must_use]
    pub fn wires(&self) -> &[Wire] {
        &self.wires
    }

    #[must_use]
    pub fn nodes_with_guid(&self, guid: &str) -> Option<&[NodeId]> {
        self.guid_index
            .get(&normalize_guid(guid))
            .map(|ids| ids.as_slice())
    }

    #[must_use]
    pub fn nodes_with_name(&self, name: &str) -> Option<&[NodeId]> {
        self.name_index
            .get(&normalize_name(name))
            .map(|ids| ids.as_slice())
    }

    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn wire_count(&self) -> usize {
        self.wires.len()
    }

    fn index_name(&mut self, id: NodeId, name: &str) {
        let key = normalize_name(name);
        self.name_index.entry(key).or_default().push(id);
    }
}

/// Fouten die kunnen optreden bij het opbouwen van de graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    DuplicateNode(NodeId),
    UnknownNode(NodeId),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateNode(id) => write!(f, "node {:?} bestaat al in de graph", id.0),
            Self::UnknownNode(id) => write!(f, "node {:?} niet gevonden in graph", id.0),
        }
    }
}

impl std::error::Error for GraphError {}

fn normalize_guid(guid: &str) -> String {
    guid.chars()
        .filter(|c| *c != '{' && *c != '}')
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserting_nodes_creates_indices() {
        let mut graph = Graph::new();
        let mut node = Node::default();
        node.guid = Some("{ABCDEF}".to_string());
        node.name = Some("Number Slider".to_owned());
        node.nickname = Some("Slider".to_owned());

        let id = graph.add_node(node).unwrap();
        assert_eq!(graph.node_count(), 1);
        assert!(graph.node(id).is_some());
        assert_eq!(graph.nodes_with_guid("abcdef").unwrap(), [id]);
        assert_eq!(graph.nodes_with_name("slider").unwrap(), [id]);
    }

    #[test]
    fn duplicate_nodes_error() {
        let mut graph = Graph::new();
        let node = Node::new(NodeId::new(5));
        graph.add_node(node.clone()).unwrap();
        let err = graph.add_node(node).unwrap_err();
        assert!(matches!(err, GraphError::DuplicateNode(id) if id == NodeId::new(5)));
    }

    #[test]
    fn adding_wire_requires_existing_nodes() {
        let mut graph = Graph::new();
        let wire = Wire::new(NodeId::new(0), "A", NodeId::new(1), "B");
        assert!(matches!(
            graph.add_wire(wire.clone()),
            Err(GraphError::UnknownNode(_))
        ));

        let from = Node::new(NodeId::new(0));
        let to = Node::new(NodeId::new(1));
        graph.add_node(from).unwrap();
        graph.add_node(to).unwrap();
        assert!(graph.add_wire(wire).is_ok());
        assert_eq!(graph.wire_count(), 1);
    }
}
