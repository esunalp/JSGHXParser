//! Parser voor GHX XML-bestanden.

use std::collections::{BTreeMap, HashMap};
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
    let trimmed = strip_xml_preamble(input);
    let prefix = trimmed.chars().take(16).collect::<String>().to_lowercase();

    if prefix.starts_with("<ghx") {
        parse_simple_document(input)
    } else if prefix.starts_with("<archive") {
        parse_archive_document(input)
    } else {
        Err(ParseError::Graph(
            "onbekend GHX-formaat: geen <ghx> of <Archive> root gevonden".to_owned(),
        ))
    }
}

fn strip_xml_preamble(input: &str) -> &str {
    let trimmed = input.trim_start_matches(|c: char| c == '\u{feff}' || c.is_whitespace());
    if let Some(rest) = trimmed.strip_prefix("<?xml") {
        if let Some(idx) = rest.find("?>") {
            return rest[idx + 2..].trim_start();
        }
    }
    trimmed
}

fn parse_simple_document(input: &str) -> ParseResult<Graph> {
    log::debug!("Start parsing vereenvoudigd GHX document");
    let document: SimpleGhxDocument = from_str(input)?;
    let mut graph = Graph::new();
    let mut nodes_by_id: BTreeMap<usize, NodeId> = BTreeMap::new();

    log::debug!("Found {} objects", document.objects.objects.len());
    for object in document.objects.objects {
        let node_id = NodeId::new(object.id);
        log::debug!(
            "Processing object ID: {}, GUID: {:?}, Name: {:?}",
            object.id,
            object.guid,
            object.name
        );
        let mut node = Node::new(node_id);
        node.guid = object.guid;
        node.name = object.name;
        node.nickname = object.nickname;

        if let Some(inputs) = object.inputs {
            for input in inputs.inputs {
                if let Some(value) = input.as_value() {
                    node.set_input(input.name.clone(), value);
                }
            }
        }

        if let Some(outputs) = object.outputs {
            for output in outputs.outputs {
                if let Some(value) = output.as_value() {
                    node.set_output(output.name.clone(), value);
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

fn parse_archive_document(input: &str) -> ParseResult<Graph> {
    log::debug!("Start parsing Archive-structuur GHX document");
    let document: ArchiveDocument = from_str(input)?;

    let definition = document
        .chunks
        .find_case_insensitive("Definition")
        .ok_or_else(|| ParseError::Graph("Definition chunk ontbreekt".to_owned()))?;
    let definition_objects = definition
        .find_case_insensitive("DefinitionObjects")
        .ok_or_else(|| ParseError::Graph("DefinitionObjects chunk ontbreekt".to_owned()))?;

    let object_chunks: Vec<&RawChunk> = definition_objects
        .children()
        .filter(|chunk| chunk.name.eq_ignore_ascii_case("Object"))
        .collect();

    let mut graph = Graph::new();
    let mut output_lookup: HashMap<String, (NodeId, String)> = HashMap::new();
    let mut pending_wires: Vec<PendingWire> = Vec::new();

    for (idx, chunk) in object_chunks.into_iter().enumerate() {
        let parsed = parse_archive_object(chunk, idx)?;
        let node_id = graph.add_node(parsed.node)?;

        if let Some(instance_guid) = parsed.instance_guid.as_ref() {
            if let Some(pin) = parsed.default_output_pin.as_ref() {
                output_lookup.insert(instance_guid.clone(), (node_id, pin.clone()));
            }
        }

        for (guid, pin) in parsed
            .output_guids
            .into_iter()
            .zip(parsed.output_pins.into_iter())
        {
            output_lookup.insert(guid, (node_id, pin));
        }

        for pending in parsed.pending_inputs {
            pending_wires.push(PendingWire {
                target_node: node_id,
                target_pin: pending.pin,
                sources: pending.sources,
            });
        }
    }

    for pending in pending_wires {
        for source in pending.sources {
            let (from_node, from_pin) = output_lookup
                .get(&source)
                .cloned()
                .ok_or_else(|| ParseError::Graph(format!("onbekende bronreferentie: {source}")))?;
            graph.add_wire(Wire::new(
                from_node,
                from_pin,
                pending.target_node,
                pending.target_pin.clone(),
            ))?;
        }
    }

    Ok(graph)
}

fn parse_archive_object(chunk: &RawChunk, index: usize) -> ParseResult<ArchiveObjectParseResult> {
    let mut node = Node::new(NodeId::new(index));

    let component_guid_norm = chunk.item_value("GUID").and_then(normalize_guid_str);
    if let Some(norm) = component_guid_norm.as_ref() {
        node.guid = Some(format!("{{{norm}}}"));
    }

    let container = chunk
        .find_case_insensitive("Container")
        .ok_or_else(|| ParseError::Graph("Object mist Container chunk".to_owned()))?;

    let instance_guid_norm = container
        .item_value("InstanceGuid")
        .and_then(normalize_guid_str);

    let name = container
        .item_value("Name")
        .or_else(|| chunk.item_value("Name"))
        .map(str::to_owned);
    node.name = name;

    let nickname = container.item_value("NickName").and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    });
    node.nickname = nickname;

    let is_slider = component_guid_norm.as_deref().map_or(false, |guid| {
        guid == "57da07bd-ecab-415d-9d86-af36d7073abc"
            || guid == "5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b"
    });
    let is_panel = component_guid_norm
        .as_deref()
        .map_or(false, |guid| guid == "59e0b89a-e487-49f8-bab8-b5bab16be14c");

    if is_slider {
        apply_slider_meta(container, &mut node);
    }

    if is_panel {
        if let Some(user_text) = container.item_value("UserText").map(str::to_owned) {
            node.insert_meta("userText", user_text.clone());
            node.set_output("Output", Value::Text(user_text));
        } else {
            node.set_output("Output", Value::Null);
        }
    }

    let mut output_guids = Vec::new();
    let mut output_pins = Vec::new();

    for (output_index, output_chunk) in container
        .children()
        .filter(|child| child.name.eq_ignore_ascii_case("param_output"))
        .enumerate()
    {
        let info = parse_param_chunk(output_chunk, "out", output_index);
        let pin_name = info.pin_name.clone();
        node.set_output(pin_name.clone(), Value::Null);
        if let Some(guid) = info.instance_guid {
            output_guids.push(guid);
        }
        output_pins.push(pin_name);
    }

    let mut pending_inputs = Vec::new();
    for (input_index, input_chunk) in container
        .children()
        .filter(|child| child.name.eq_ignore_ascii_case("param_input"))
        .enumerate()
    {
        let info = parse_param_chunk(input_chunk, "in", input_index);
        if let Some(default_value) = info.default_value.clone() {
            node.set_input(info.pin_name.clone(), default_value);
        }
        if !info.sources.is_empty() {
            pending_inputs.push(PendingInput {
                pin: info.pin_name,
                sources: info.sources,
            });
        }
    }

    let default_output_pin = if !output_pins.is_empty() {
        output_pins.first().cloned()
    } else if is_slider {
        if !node.outputs.contains_key("OUT") {
            node.set_output("OUT", Value::Null);
        }
        Some("OUT".to_owned())
    } else if is_panel {
        Some("Output".to_owned())
    } else {
        None
    };

    Ok(ArchiveObjectParseResult {
        node,
        instance_guid: instance_guid_norm,
        output_guids,
        output_pins,
        default_output_pin,
        pending_inputs,
    })
}

fn apply_slider_meta(container: &RawChunk, node: &mut Node) {
    let mut value = None;
    let mut min = None;
    let mut max = None;
    let mut step = None;

    for slider_chunk in container
        .children()
        .filter(|child| child.name.to_lowercase().contains("slider"))
    {
        if let Some(raw_value) = slider_chunk.item_value("Value") {
            if value.is_none() {
                value = parse_f64(raw_value);
            }
        }
        if let Some(raw_min) = slider_chunk.item_value("Min") {
            if min.is_none() {
                min = parse_f64(raw_min);
            }
        }
        if let Some(raw_max) = slider_chunk.item_value("Max") {
            if max.is_none() {
                max = parse_f64(raw_max);
            }
        }
        if let Some(raw_step) = slider_chunk
            .item_value("Step")
            .or_else(|| slider_chunk.item_value("Increment"))
            .or_else(|| slider_chunk.item_value("Interval"))
        {
            if step.is_none() {
                step = parse_f64(raw_step);
            }
        }
    }

    let value = value.unwrap_or(0.0);
    if let Some(min) = min {
        node.insert_meta("min", min);
    }
    if let Some(max) = max {
        node.insert_meta("max", max);
    }

    let mut final_step = step.unwrap_or_else(|| {
        if let (Some(min), Some(max)) = (min, max) {
            let range = max - min;
            if range > 0.0 { range / 100.0 } else { 0.1 }
        } else {
            0.1
        }
    });

    if final_step <= 0.0 || !final_step.is_finite() {
        final_step = 0.1;
    }

    node.insert_meta("value", value);
    node.insert_meta("step", final_step);
    node.set_output("OUT", Value::Number(value));
}

fn parse_param_chunk(chunk: &RawChunk, fallback_prefix: &str, fallback_index: usize) -> ParamInfo {
    let index = chunk.index.unwrap_or(fallback_index);

    let pin_name = chunk
        .item_value("NickName")
        .or_else(|| chunk.item_value("Name"))
        .or_else(|| chunk.item_value("Description"))
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{fallback_prefix}{index}"));

    let sources = chunk
        .item_values("Source")
        .into_iter()
        .filter_map(normalize_guid_str)
        .collect();

    let default_value = parse_persistent_value(chunk);
    let instance_guid = chunk
        .item_value("InstanceGuid")
        .and_then(normalize_guid_str);

    ParamInfo {
        pin_name,
        instance_guid,
        sources,
        default_value,
    }
}

fn parse_persistent_value(chunk: &RawChunk) -> Option<Value> {
    let persistent = chunk.find_case_insensitive("PersistentData")?;
    let branch = persistent
        .children()
        .find(|child| child.name.eq_ignore_ascii_case("Branch"))?;
    let item_chunk = branch
        .children()
        .find(|child| child.name.eq_ignore_ascii_case("Item"))?;
    let value_item = item_chunk.items.items.first()?;
    let text = value_item.text.as_deref()?.trim();
    if text.is_empty() {
        return None;
    }

    let type_name = value_item
        .type_name
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();

    if type_name.contains("point") {
        if let Some(point) = parse_point_value(text) {
            return Some(Value::Point(point));
        }
    }

    if type_name.contains("vector") {
        if let Some(vector) = parse_point_value(text) {
            return Some(Value::Vector(vector));
        }
    }

    if type_name.contains("double")
        || type_name.contains("single")
        || type_name.contains("int")
        || type_name.contains("number")
    {
        if let Some(number) = parse_f64(text) {
            return Some(Value::Number(number));
        }
    }

    if type_name.contains("bool") {
        let normalized = text.to_ascii_lowercase();
        return Some(Value::Boolean(
            normalized == "true" || normalized == "1" || normalized == "yes",
        ));
    }

    Some(Value::Text(text.to_owned()))
}

fn parse_point_value(text: &str) -> Option<[f64; 3]> {
    let parts: Vec<Option<f64>> = text.split(',').map(parse_f64).collect();
    if parts.len() != 3 {
        return None;
    }
    let x = parts[0]?;
    let y = parts[1]?;
    let z = parts[2]?;
    Some([x, y, z])
}

fn parse_f64(value: &str) -> Option<f64> {
    let normalized = value.trim().replace(',', ".");
    normalized.parse::<f64>().ok()
}

fn normalize_guid_str(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let without_braces = trimmed.trim_matches(|c| c == '{' || c == '}');
    let lowered = without_braces.to_ascii_lowercase();
    if lowered.is_empty() {
        None
    } else {
        Some(lowered)
    }
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
struct SimpleGhxDocument {
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
    fn as_value(&self) -> Option<Value> {
        let raw = self.value()?;
        if let Ok(number) = raw.parse::<f64>() {
            return Some(Value::Number(number));
        }

        let lowercase = raw.trim().to_lowercase();
        if lowercase == "true" {
            return Some(Value::Boolean(true));
        }
        if lowercase == "false" {
            return Some(Value::Boolean(false));
        }

        Some(Value::Text(raw))
    }
}

impl GhxSlider {
    fn output_pin(&self) -> Option<String> {
        self.output_pin.clone()
    }
}

#[derive(Debug, Default, Deserialize)]
struct ArchiveDocument {
    #[serde(default)]
    items: RawItems,
    #[serde(default)]
    chunks: RawChunks,
}

#[derive(Debug, Default, Deserialize)]
struct RawChunks {
    #[serde(default, rename = "chunk")]
    chunks: Vec<RawChunk>,
}

#[derive(Debug, Default, Deserialize)]
struct RawItems {
    #[serde(default, rename = "item")]
    items: Vec<RawItem>,
}

#[derive(Debug, Default, Deserialize)]
struct RawChunk {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@index")]
    index: Option<usize>,
    #[serde(default)]
    items: RawItems,
    #[serde(default)]
    chunks: RawChunks,
}

#[derive(Debug, Deserialize)]
struct RawItem {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@index")]
    index: Option<usize>,
    #[serde(rename = "@type_name")]
    type_name: Option<String>,
    #[serde(rename = "@type_code")]
    type_code: Option<String>,
    #[serde(rename = "$text")]
    text: Option<String>,
}

#[derive(Debug)]
struct ArchiveObjectParseResult {
    node: Node,
    instance_guid: Option<String>,
    output_guids: Vec<String>,
    output_pins: Vec<String>,
    default_output_pin: Option<String>,
    pending_inputs: Vec<PendingInput>,
}

#[derive(Debug)]
struct PendingWire {
    target_node: NodeId,
    target_pin: String,
    sources: Vec<String>,
}

#[derive(Debug)]
struct PendingInput {
    pin: String,
    sources: Vec<String>,
}

#[derive(Debug)]
struct ParamInfo {
    pin_name: String,
    instance_guid: Option<String>,
    sources: Vec<String>,
    default_value: Option<Value>,
}

impl RawChunks {
    fn find_case_insensitive(&self, name: &str) -> Option<&RawChunk> {
        self.chunks
            .iter()
            .find(|chunk| chunk.name.eq_ignore_ascii_case(name))
    }

    fn children(&self) -> impl Iterator<Item = &RawChunk> {
        self.chunks.iter()
    }
}

impl RawChunk {
    fn find_case_insensitive(&self, name: &str) -> Option<&RawChunk> {
        self.chunks.find_case_insensitive(name)
    }

    fn children(&self) -> impl Iterator<Item = &RawChunk> {
        self.chunks.children()
    }

    fn item_value(&self, name: &str) -> Option<&str> {
        self.items
            .items
            .iter()
            .find(|item| item.name.eq_ignore_ascii_case(name))
            .and_then(|item| item.text.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    fn item_values(&self, name: &str) -> Vec<&str> {
        self.items
            .items
            .iter()
            .filter(|item| item.name.eq_ignore_ascii_case(name))
            .filter_map(|item| item.text.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::parse_str;
    use crate::graph::node::MetaValue;
    use crate::graph::value::Value;

    #[test]
    fn parses_point_with_default_value() {
        let xml = include_str!("../../../../tools/ghx-samples/point_default.ghx");
        let graph = parse_str(xml).expect("graph with default point parsed");
        assert_eq!(graph.node_count(), 1);
        let point_node = graph.nodes().first().unwrap();
        let input_value = point_node.inputs.get("P").unwrap();
        match input_value {
            Value::Point(p) => {
                assert!((p[0] - 10.5).abs() < f64::EPSILON);
                assert!((p[1] - 20.0).abs() < f64::EPSILON);
                assert!((p[2] - -5.2).abs() < f64::EPSILON);
            }
            _ => panic!("Expected a Point value, got {:?}", input_value),
        }
    }

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
        let has_curve_wire = graph
            .wires()
            .iter()
            .any(|wire| wire.to_node == line_node.id && wire.to_pin.0 == "A");
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
        let has_curve_input = graph
            .wires()
            .iter()
            .any(|wire| wire.to_node == extrude_node.id && wire.to_pin.0 == "Curve");
        let has_distance_input = graph
            .wires()
            .iter()
            .any(|wire| wire.to_node == extrude_node.id && wire.to_pin.0 == "Distance");
        assert!(has_curve_input, "extrude node should have curve input wire");
        assert!(
            has_distance_input,
            "extrude node should have distance input wire"
        );

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

    #[test]
    fn parses_archive_style_graph() {
        let xml = include_str!("../../../web/lijntest.ghx");
        let graph = parse_str(xml).expect("archive graph parsed");
        assert!(graph.node_count() >= 13);
        assert!(graph.wire_count() > 0);

        let slider = graph
            .nodes()
            .iter()
            .find(|node| node.guid.as_deref() == Some("{57da07bd-ecab-415d-9d86-af36d7073abc}"))
            .expect("slider node present");
        let value = match slider.meta("value") {
            Some(MetaValue::Number(number)) => *number,
            _ => panic!("slider meta value missing"),
        };
        assert!(value > 0.0);

        let panel = graph
            .nodes()
            .iter()
            .find(|node| node.guid.as_deref() == Some("{59e0b89a-e487-49f8-bab8-b5bab16be14c}"))
            .expect("panel node present");
        let panel_text = match panel.meta("userText") {
            Some(MetaValue::Text(text)) => text.clone(),
            _ => panic!("panel user text missing"),
        };
        assert!(!panel_text.is_empty());
    }
}
