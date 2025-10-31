//! Verbindingen tussen nodes.

use super::node::NodeId;

/// Placeholder voor een verbinding tussen twee nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Wire {
    pub from: NodeId,
    pub to: NodeId,
}
