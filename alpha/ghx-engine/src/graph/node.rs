//! Definitie van nodes binnen de Grasshopper graph.

use std::collections::BTreeMap;

use super::value::Value;

/// Identifier voor een node binnen de graph.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default, Ord, PartialOrd)]
pub struct NodeId(pub usize);

impl NodeId {
    #[must_use]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }
}

impl From<usize> for NodeId {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

/// Waarde die meta-informatie over een node beschrijft (bv. slider-ranges).
#[derive(Debug, Clone, PartialEq)]
pub enum MetaValue {
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Text(String),
    List(Vec<MetaValue>),
}

impl From<f64> for MetaValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<i64> for MetaValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<bool> for MetaValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<String> for MetaValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for MetaValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

/// Alias voor een verzameling meta-informatie.
pub type MetaMap = BTreeMap<String, MetaValue>;

/// Node representatie binnen de graph.
#[derive(Debug, Clone)]
pub struct Node {
    /// Unieke identifier binnen de graph.
    pub id: NodeId,
    /// Het type component (GUID) dat deze node representeert.
    pub guid: Option<String>,
    /// Volledige naam van het component in het GHX-bestand.
    pub name: Option<String>,
    /// Nickname/afkorting indien beschikbaar.
    pub nickname: Option<String>,
    /// Ingangswaarden, per pinnickname.
    pub inputs: BTreeMap<String, Value>,
    /// Uitgangswaarden, per pinnickname.
    pub outputs: BTreeMap<String, Value>,
    /// Verdere metadata zoals slider-range of UI hints.
    pub meta: MetaMap,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            id: NodeId::default(),
            guid: None,
            name: None,
            nickname: None,
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
            meta: BTreeMap::new(),
        }
    }
}

impl Node {
    /// Maak een nieuwe node met een meegegeven identifier.
    #[must_use]
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            ..Self::default()
        }
    }

    /// Sla een input-waarde op.
    pub fn set_input<S: Into<String>>(&mut self, pin: S, value: Value) {
        self.inputs.insert(pin.into(), value);
    }

    /// Haal een verwijzing naar een input op.
    pub fn input(&self, pin: &str) -> Option<&Value> {
        self.inputs.get(pin)
    }

    /// Sla een output-waarde op.
    pub fn set_output<S: Into<String>>(&mut self, pin: S, value: Value) {
        self.outputs.insert(pin.into(), value);
    }

    /// Haal een output op.
    pub fn output(&self, pin: &str) -> Option<&Value> {
        self.outputs.get(pin)
    }

    /// Bewaar meta-informatie bij de node.
    pub fn insert_meta<S: Into<String>, V: Into<MetaValue>>(&mut self, key: S, value: V) {
        self.meta.insert(key.into(), value.into());
    }

    /// Haal een meta-item op.
    pub fn meta(&self, key: &str) -> Option<&MetaValue> {
        self.meta.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::{MetaValue, Node, NodeId};
    use crate::graph::value::Value;

    #[test]
    fn store_and_retrieve_inputs_outputs() {
        let mut node = Node::new(NodeId::new(1));
        node.set_input("A", Value::Number(1.0));
        node.set_output("R", Value::Number(2.0));

        assert!(matches!(
            node.input("A"),
            Some(Value::Number(value)) if (value - 1.0).abs() < f64::EPSILON
        ));
        assert!(matches!(
            node.output("R"),
            Some(Value::Number(value)) if (value - 2.0).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn meta_information_roundtrip() {
        let mut node = Node::default();
        node.insert_meta("min", 0.0);
        node.insert_meta("label", "Example");

        assert!(
            matches!(node.meta("min"), Some(MetaValue::Number(v)) if (*v - 0.0).abs() < f64::EPSILON)
        );
        assert!(matches!(node.meta("label"), Some(MetaValue::Text(text)) if text == "Example"));
    }
}
