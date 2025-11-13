//! Evaluatie van grafen in topologische volgorde.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use crate::components::{ComponentError, ComponentRegistry, OutputMap};
use crate::graph::Graph;
use crate::graph::node::NodeId;
use crate::graph::topo::{Topology, TopologyError};
use crate::graph::value::Value;

/// Resultaat van een evaluatie-run.
#[derive(Debug, Default, Clone)]
pub struct EvaluationResult {
    /// Uitgangen per node.
    pub node_outputs: HashMap<NodeId, BTreeMap<String, Value>>,
    /// Verzameling van renderbare geometrie-waarden.
    pub geometry: Vec<Value>,
}

/// Voorbereide metadata die hergebruik van topologie en verbindingen mogelijk maakt.
#[derive(Debug, Clone, Default)]
pub struct EvaluationPlan {
    order: Vec<NodeId>,
    incoming: HashMap<NodeId, HashMap<String, Vec<(NodeId, String)>>>,
    pin_order: HashMap<NodeId, Vec<String>>,
}

impl EvaluationPlan {
    /// Bouwt een evaluatieplan op basis van een graph.
    pub fn new(graph: &Graph) -> Result<Self, EvaluationError> {
        let topology = Topology::sort(graph)?;

        let mut incoming: HashMap<NodeId, HashMap<String, Vec<(NodeId, String)>>> = HashMap::new();
        for wire in graph.wires() {
            incoming
                .entry(wire.to_node)
                .or_default()
                .entry(wire.to_pin.0.clone())
                .or_default()
                .push((wire.from_node, wire.from_pin.0.clone()));
        }

        for per_node in incoming.values_mut() {
            for connections in per_node.values_mut() {
                connections.sort();
            }
        }

        let mut pin_order = HashMap::new();
        for node in graph.nodes() {
            let mut pins: Vec<String> = node.input_order().to_vec();
            if let Some(connections) = incoming.get(&node.id) {
                let mut extra: Vec<String> = connections.keys().cloned().collect();
                extra.sort();
                for pin in extra {
                    if !pins.iter().any(|existing| existing == &pin) {
                        pins.push(pin);
                    }
                }
            }
            pin_order.insert(node.id, pins);
        }

        Ok(Self {
            order: topology.order,
            incoming,
            pin_order,
        })
    }

    #[must_use]
    pub fn order(&self) -> &[NodeId] {
        &self.order
    }

    fn incoming_connections(&self, node: NodeId, pin: &str) -> Option<&Vec<(NodeId, String)>> {
        self.incoming
            .get(&node)
            .and_then(|node_map| node_map.get(pin))
    }

    fn pins(&self, node: NodeId) -> &[String] {
        self.pin_order
            .get(&node)
            .map(Vec::as_slice)
            .unwrap_or_else(|| &[])
    }
}

/// Fouttype voor evaluatieproblemen.
#[derive(Debug)]
pub enum EvaluationError {
    /// Topologiesortering is mislukt.
    Topology(TopologyError),
    /// De node heeft geen bijbehorend component.
    ComponentNotFound {
        node_id: NodeId,
        guid: Option<String>,
        name: Option<String>,
        nickname: Option<String>,
    },
    /// Een vereiste input ontbreekt.
    MissingInput { node_id: NodeId, pin: String },
    /// Een output van een afhankelijke node ontbreekt.
    MissingDependencyOutput {
        node_id: NodeId,
        dependency: NodeId,
        pin: String,
    },
    /// Het component gaf een foutmelding tijdens evaluatie.
    ComponentFailed {
        node_id: NodeId,
        component: String,
        source: ComponentError,
    },
    /// De node kon niet teruggevonden worden in de graph (inconsistentie).
    UnknownNode(NodeId),
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Topology(err) => write!(f, "topologiesortering mislukt: {err}"),
            Self::ComponentNotFound {
                node_id,
                guid,
                name,
                nickname,
            } => write!(
                f,
                "geen component gevonden voor node {} (guid={:?}, name={:?}, nickname={:?})",
                node_id.0, guid, name, nickname
            ),
            Self::MissingInput { node_id, pin } => {
                write!(f, "node {} mist verplichte input `{pin}`", node_id.0)
            }
            Self::MissingDependencyOutput {
                node_id,
                dependency,
                pin,
            } => write!(
                f,
                "node {} mist output `{pin}` van afhankelijke node {}",
                node_id.0, dependency.0
            ),
            Self::ComponentFailed {
                node_id,
                component,
                source,
            } => write!(
                f,
                "component `{component}` (node {}) faalde: {}",
                node_id.0, source
            ),
            Self::UnknownNode(node_id) => {
                write!(f, "node {} bestaat niet in de graph", node_id.0)
            }
        }
    }
}

impl std::error::Error for EvaluationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ComponentFailed { source, .. } => Some(source),
            Self::Topology(err) => Some(err),
            _ => None,
        }
    }
}

impl From<TopologyError> for EvaluationError {
    fn from(error: TopologyError) -> Self {
        Self::Topology(error)
    }
}

/// Evalueert een graph met behulp van de opgegeven componentregistry.
pub fn evaluate(
    graph: &Graph,
    registry: &ComponentRegistry,
) -> Result<EvaluationResult, EvaluationError> {
    let plan = EvaluationPlan::new(graph)?;
    evaluate_with_plan(graph, registry, &plan)
}

/// Evalueert een graph met behulp van een vooraf opgebouwd evaluatieplan.
pub fn evaluate_with_plan(
    graph: &Graph,
    registry: &ComponentRegistry,
    plan: &EvaluationPlan,
) -> Result<EvaluationResult, EvaluationError> {
    let mut result = EvaluationResult::default();

    for &node_id in plan.order() {
        let node = graph
            .node(node_id)
            .ok_or(EvaluationError::UnknownNode(node_id))?;

        let component = registry.resolve(
            node.guid.as_deref(),
            node.name.as_deref(),
            node.nickname.as_deref(),
        );

        let component = component.ok_or_else(|| EvaluationError::ComponentNotFound {
            node_id,
            guid: node.guid.clone(),
            name: node.name.clone(),
            nickname: node.nickname.clone(),
        })?;

        let pins = plan.pins(node_id);
        let mut input_values = Vec::with_capacity(pins.len());

        for pin in pins {
            let value = if let Some(connections) = plan.incoming_connections(node_id, pin) {
                let mut values = Vec::with_capacity(connections.len());
                for (from_node, from_pin) in connections {
                    let outputs = result.node_outputs.get(from_node).ok_or_else(|| {
                        EvaluationError::MissingDependencyOutput {
                            node_id,
                            dependency: *from_node,
                            pin: from_pin.clone(),
                        }
                    })?;

                    let value = outputs.get(from_pin).ok_or_else(|| {
                        EvaluationError::MissingDependencyOutput {
                            node_id,
                            dependency: *from_node,
                            pin: from_pin.clone(),
                        }
                    })?;
                    values.push(value.clone());
                }

                if values.len() == 1 {
                    values.into_iter().next().unwrap()
                } else {
                    Value::List(values)
                }
            } else if let Some(default) = node.inputs.get(pin) {
                default.clone()
            } else {
                return Err(EvaluationError::MissingInput {
                    node_id,
                    pin: pin.clone(),
                });
            };

            input_values.push(value);
        }

        let outputs = component
            .evaluate(&input_values, &node.meta)
            .map_err(|error| EvaluationError::ComponentFailed {
                node_id,
                component: component.name().to_owned(),
                source: error,
            })?;

        let stored_outputs = merge_outputs(node.outputs.clone(), outputs);
        collect_geometry(&stored_outputs, &mut result.geometry);
        result.node_outputs.insert(node_id, stored_outputs);
    }

    Ok(result)
}

fn merge_outputs(
    mut existing: BTreeMap<String, Value>,
    new_outputs: OutputMap,
) -> BTreeMap<String, Value> {
    for (pin, value) in new_outputs {
        existing.insert(pin, value);
    }
    existing
}

fn collect_geometry(outputs: &BTreeMap<String, Value>, geometry: &mut Vec<Value>) {
    for value in outputs.values() {
        collect_value_geometry(value, geometry);
    }
}

fn collect_value_geometry(value: &Value, geometry: &mut Vec<Value>) {
    match value {
        Value::Point(_) | Value::CurveLine { .. } | Value::Surface { .. } => {
            geometry.push(value.clone());
        }
        Value::List(values) => {
            for value in values {
                collect_value_geometry(value, geometry);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{EvaluationError, EvaluationPlan, evaluate};
    use crate::components::ComponentRegistry;
    use crate::graph::Graph;
    use crate::graph::node::{Node, NodeId};
    use crate::graph::value::Value;
    use crate::graph::wire::Wire;

    #[test]
    fn evaluates_empty_graph() {
        let graph = Graph::new();
        let registry = ComponentRegistry::default();
        let result = evaluate(&graph, &registry).expect("lege graph evalueert");
        assert!(result.node_outputs.is_empty());
        assert!(result.geometry.is_empty());
    }

    #[test]
    fn missing_component_yields_error() {
        let mut graph = Graph::new();
        let node_id = graph
            .add_node(Node::new(crate::graph::node::NodeId::new(0)))
            .unwrap();
        let registry = ComponentRegistry::default();

        let err = evaluate(&graph, &registry).expect_err("component ontbreekt");
        match err {
            EvaluationError::ComponentNotFound {
                node_id: err_node, ..
            } => {
                assert_eq!(err_node, node_id);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn evaluation_plan_preserves_declared_input_order() {
        let mut graph = Graph::new();

        let mut node = Node::new(NodeId::new(0));
        node.add_input_pin("C");
        node.add_input_pin("N");
        node.add_input_pin("K");
        let node_id = graph.add_node(node).unwrap();

        let mut source = Node::new(NodeId::new(1));
        source.set_output("Out", Value::Number(5.0));
        let source_id = graph.add_node(source).unwrap();

        graph
            .add_wire(Wire::new(source_id, "Out", node_id, "Extra"))
            .unwrap();

        let plan = EvaluationPlan::new(&graph).expect("kan plan bouwen");
        let pins = plan.pins(node_id);
        let expected = ["C", "N", "K", "Extra"].map(String::from);
        assert_eq!(pins, &expected);
    }
}
