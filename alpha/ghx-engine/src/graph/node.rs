//! Definitie van nodes binnen de Grasshopper graph.

/// Identifier voor een node binnen de graph.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct NodeId(pub usize);

/// Placeholder node representatie.
#[derive(Debug, Clone, Default)]
pub struct Node {
    /// Unieke identifier binnen de graph.
    pub id: NodeId,
    /// Het type component (GUID) dat deze node representeert.
    pub guid: Option<String>,
}
