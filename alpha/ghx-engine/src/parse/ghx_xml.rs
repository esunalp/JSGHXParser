//! Parser voor GHX XML-bestanden.

use std::collections::BTreeMap;
use std::num::{ParseFloatError, ParseIntError};

use crate::graph::node::{Node, NodeId};
use crate::graph::value::Value;
use crate::graph::wire::Wire;
use crate::graph::{Graph, GraphError};

use quick_xml::de::from_str;
use serde::Deserialize;
use thiserror::Error;

/// Result type voor parsing van GHX-bestanden.
pub type ParseResult<T> = Result<T, ParseError>;

/// Beschrijft fouten tijdens het parsen.
#[derive(Debug, Error)]
pub enum ParseError {
    /// Het XML-document kon niet gede-serialiseerd worden.
    #[error("XML parsefout: {0}")]
    Xml(#[from] quick_xml::DeError),
    /// De graph bevat een inconsistente verwijzing.
    #[error("ongeldige graphreferentie: {0}")]
    Graph(String),
    /// Fout tijdens het converteren van numerieke waarden.
    #[error("ongeldige numerieke waarde: {0}")]
    Number(#[from] ParseFloatError),
    /// Fout tijdens het uitlezen van een node-index.
    #[error("ongeldige indexwaarde: {0}")]
    Index(#[from] ParseIntError),
}

impl From<GraphError> for ParseError {
    fn from(err: GraphError) -> Self {
        Self::Graph(err.to_string())
    }
}

/// Leest een GHX-document en converteert het naar een [`Graph`].
pub fn parse_str(input: &str) -> ParseResult<Graph> {
    let document: GhxDocument = from_str(input)?;
    let mut graph = Graph::new();
    let mut nodes_by_id: BTreeMap<usize, NodeId> = BTreeMap::new();

    for object in document.objects.objects {
        let node_id = NodeId::new(object.id);
        let mut node = Node::new(node_id);
        node.guid = object.guid;
        node.name = object.name;
        node.nickname = object.nickname;

        if let Some(inputs) = object.inputs {
            for input in inputs.inputs {
                if let Some(number) = input.as_number()? {
                    node.set_input(input.name.clone(), Value::Number(number));
                }
            }
        }

        if let Some(outputs) = object.outputs {
            for output in outputs.outputs {
                if let Some(number) = output.as_number()? {
                    node.set_output(output.name.clone(), Value::Number(number));
                }
            }
        }

        if let Some(slider) = object.slider {
            node.insert_meta("min", slider.min);
            node.insert_meta("max", slider.max);
            node.insert_meta("step", slider.step);
            node.insert_meta("value", slider.value);
            let output_pin = slider.output_pin().unwrap_or_else(|| "OUT".to_string());
            node.set_output(output_pin, Value::Number(slider.value));
        }

        graph.add_node(node)?;
        nodes_by_id.insert(object.id, node_id);
    }

    for wire in document.wires.wires {
        let (from_node, from_pin) = parse_endpoint(&wire.from, &nodes_by_id)?;
        let (to_node, to_pin) = parse_endpoint(&wire.to, &nodes_by_id)?;
        graph.add_wire(Wire::new(from_node, from_pin, to_node, to_pin))?;
    }

    Ok(graph)
}

fn parse_endpoint(
    reference: &str,
    nodes_by_id: &BTreeMap<usize, NodeId>,
) -> ParseResult<(NodeId, String)> {
    let (node_str, pin) = reference
        .split_once(':')
        .ok_or_else(|| ParseError::Graph(format!("ongeldige pin referentie: {reference}")))?;
    let node_idx: usize = node_str.trim().parse().map_err(ParseError::Index)?;
    let node_id = nodes_by_id
        .get(&node_idx)
        .ok_or_else(|| ParseError::Graph(format!("onbekende node id {node_idx}")))?;
    Ok((*node_id, pin.trim().to_owned()))
}

#[derive(Debug, Deserialize)]
struct GhxDocument {
    #[serde(default)]
    objects: GhxObjects,
    #[serde(default)]
    wires: GhxWires,
}

#[derive(Debug, Default, Deserialize)]
struct GhxObjects {
    #[serde(default, rename = "object")]
    objects: Vec<GhxObject>,
}

#[derive(Debug, Default, Deserialize)]
struct GhxWires {
    #[serde(default, rename = "wire")]
    wires: Vec<GhxWire>,
}

#[derive(Debug, Deserialize)]
struct GhxObject {
    #[serde(rename = "@id")]
    id: usize,
    guid: Option<String>,
    name: Option<String>,
    nickname: Option<String>,
    #[serde(default)]
    slider: Option<GhxSlider>,
    #[serde(default)]
    inputs: Option<GhxInputs>,
    #[serde(default)]
    outputs: Option<GhxOutputs>,
}

#[derive(Debug, Deserialize)]
struct GhxWire {
    #[serde(rename = "@from")]
    from: String,
    #[serde(rename = "@to")]
    to: String,
}

#[derive(Debug, Deserialize)]
struct GhxSlider {
    #[serde(rename = "@min")]
    min: f64,
    #[serde(rename = "@max")]
    max: f64,
    #[serde(rename = "@value")]
    value: f64,
    #[serde(rename = "@step")]
    step: f64,
    #[serde(default, rename = "@output")]
    output_pin: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct GhxInputs {
    #[serde(default, rename = "input")]
    inputs: Vec<GhxPin>,
}

#[derive(Debug, Default, Deserialize)]
struct GhxOutputs {
    #[serde(default, rename = "output")]
    outputs: Vec<GhxPin>,
}

#[derive(Debug, Deserialize)]
struct GhxPin {
    #[serde(rename = "@name")]
    name: String,
    #[serde(default)]
    #[serde(rename = "@type")]
    _pin_type: Option<String>,
    #[serde(default)]
    #[serde(rename = "@value")]
    raw_value: Option<String>,
    #[serde(default)]
    #[serde(rename = "@default")]
    default_value: Option<String>,
    #[serde(default)]
    #[serde(rename = "$value")]
    content: Option<String>,
}

impl GhxPin {
    fn value(&self) -> Option<String> {
        self.raw_value
            .as_ref()
            .or(self.default_value.as_ref())
            .or(self.content.as_ref())
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
    }
}

impl GhxPin {
    fn as_number(&self) -> ParseResult<Option<f64>> {
        if let Some(value) = self.value() {
            Ok(Some(value.parse()?))
        } else {
            Ok(None)
        }
    }
}

impl GhxSlider {
    fn output_pin(&self) -> Option<String> {
        self.output_pin.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::parse_str;
    use crate::graph::node::MetaValue;

    #[test]
    fn parses_minimal_line_graph_with_slider_meta() {
        let xml = include_str!("../../../tools/ghx-samples/minimal_line.ghx");
        let graph = parse_str(xml).expect("graph parsed");
        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.wire_count(), 3);

        let slider = graph
            .nodes()
            .iter()
            .find(|node| node.nickname.as_deref() == Some("Length"))
            .expect("slider node present");

        let extract = |key: &str| match slider.meta(key) {
            Some(MetaValue::Number(number)) => Some(*number),
            Some(MetaValue::Integer(integer)) => Some(*integer as f64),
            _ => None,
        };

        assert_eq!(extract("min"), Some(0.0));
        assert_eq!(extract("max"), Some(10.0));
        assert_eq!(extract("step"), Some(0.5));
        assert_eq!(extract("value"), Some(3.0));

        let line_node = graph
            .nodes()
            .iter()
            .find(|node| node.nickname.as_deref() == Some("Result Line"))
            .expect("line node present");
        let has_curve_wire = graph.wires().iter().any(|wire| {
            wire.to_node == line_node.id && wire.to_pin.0 == "A"
        });
        assert!(has_curve_wire, "line component should receive input wire");
    }

    #[test]
    fn parses_minimal_extrude_graph() {
        let xml = include_str!("../../../tools/ghx-samples/minimal_extrude.ghx");
        let graph = parse_str(xml).expect("graph parsed");
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.wire_count(), 4);

        let extrude_node = graph
            .nodes()
            .iter()
            .find(|node| node.nickname.as_deref() == Some("Extrude Surface"))
            .expect("extrude node present");
        let has_curve_input = graph.wires().iter().any(|wire| {
            wire.to_node == extrude_node.id && wire.to_pin.0 == "Curve"
        });
        let has_distance_input = graph.wires().iter().any(|wire| {
            wire.to_node == extrude_node.id && wire.to_pin.0 == "Distance"
        });
        assert!(has_curve_input, "extrude node should have curve input wire");
        assert!(has_distance_input, "extrude node should have distance input wire");

        let height_slider = graph
            .nodes()
            .iter()
            .find(|node| node.nickname.as_deref() == Some("Height"))
            .expect("height slider present");
        let height_value = match height_slider.meta("value") {
            Some(MetaValue::Number(number)) => *number,
            other => panic!("unexpected slider value meta: {other:?}"),
        };
        assert!((height_value - 2.0).abs() < f64::EPSILON);
    }
}
