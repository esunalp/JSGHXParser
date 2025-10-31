//! Verbindingen tussen nodes.

use super::node::NodeId;

/// Pin binnen een node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PinId(pub String);

impl From<&str> for PinId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for PinId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// Verbinding tussen een output-pin en een input-pin.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Wire {
    pub from_node: NodeId,
    pub from_pin: PinId,
    pub to_node: NodeId,
    pub to_pin: PinId,
}

impl Wire {
    #[must_use]
    pub fn new<F, T, PF, PT>(from_node: F, from_pin: PF, to_node: T, to_pin: PT) -> Self
    where
        F: Into<NodeId>,
        T: Into<NodeId>,
        PF: Into<PinId>,
        PT: Into<PinId>,
    {
        Self {
            from_node: from_node.into(),
            from_pin: from_pin.into(),
            to_node: to_node.into(),
            to_pin: to_pin.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NodeId, PinId, Wire};

    #[test]
    fn wire_holds_all_fields() {
        let wire = Wire::new(NodeId::new(1), "A", NodeId::new(2), "B");
        assert_eq!(wire.from_node, NodeId::new(1));
        assert_eq!(wire.to_node, NodeId::new(2));
        assert_eq!(wire.from_pin, PinId("A".to_owned()));
        assert_eq!(wire.to_pin, PinId("B".to_owned()));
    }
}
