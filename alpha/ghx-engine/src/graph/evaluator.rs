//! Evaluatie van grafen in topologische volgorde.

use std::collections::{BTreeMap, BTreeSet, HashMap};
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
    let topology = Topology::sort(graph).map_err(EvaluationError::from)?;

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

    let mut result = EvaluationResult::default();

    for node_id in topology.order {
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

        let mut pin_names: BTreeSet<String> = node.inputs.keys().cloned().collect();
        if let Some(connections) = incoming.get(&node_id) {
            pin_names.extend(connections.keys().cloned());
        }

        let mut input_values = Vec::with_capacity(pin_names.len());
        for pin in pin_names {
            let value = if let Some(connections) = incoming
                .get(&node_id)
                .and_then(|node_map| node_map.get(&pin))
            {
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
            } else if let Some(default) = node.inputs.get(&pin) {
                default.clone()
            } else {
                return Err(EvaluationError::MissingInput { node_id, pin });
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
    use super::{EvaluationError, evaluate};
    use crate::components::ComponentRegistry;
    use crate::graph::Graph;
    use crate::graph::node::Node;

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
}
