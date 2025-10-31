//! Topologische utilities.

use super::{node::NodeId, Graph};

/// Resultaat van een nog te implementeren topologische sortering.
#[derive(Debug, Default, Clone)]
pub struct Topology {
    pub order: Vec<NodeId>,
}

impl Topology {
    /// Construeert een lege topologie.
    #[must_use]
    pub fn empty() -> Self {
        Self { order: Vec::new() }
    }

    /// Placeholder functie die uiteindelijk een sortering zal uitvoeren.
    #[allow(clippy::unused_self)]
    pub fn sort(_graph: &Graph) -> Self {
        Self::empty()
    }
}
