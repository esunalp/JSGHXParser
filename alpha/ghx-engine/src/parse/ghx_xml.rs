//! Parser voor GHX XML-bestanden.

use std::collections::{BTreeMap, HashMap};
use std::num::{ParseFloatError, ParseIntError};

use crate::graph::node::{MetaValue, Node, NodeId};
use crate::graph::value::Value;
use crate::graph::wire::Wire;
use crate::graph::{Graph, GraphError};

use quick_xml::de::from_str;
use serde::Deserialize;
use thiserror::Error;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

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
    let object_count = document.objects.objects.len();
    log::debug!("Found {} objects", object_count);

    let SimpleGhxDocument { objects, wires } = document;

    let mut graph = Graph::new();
    let mut nodes_by_id: BTreeMap<usize, NodeId> = BTreeMap::new();

    let built_nodes = build_simple_nodes(objects.objects);
    for (object_id, node) in built_nodes {
        let node_id = graph.add_node(node)?;
        nodes_by_id.insert(object_id, node_id);
    }

    for wire in wires.wires {
        let (from_node, from_pin) = parse_endpoint(&wire.from, &nodes_by_id)?;
        let (to_node, to_pin) = parse_endpoint(&wire.to, &nodes_by_id)?;
        graph.add_wire(Wire::new(from_node, from_pin, to_node, to_pin))?;
    }

    Ok(graph)
}

fn build_simple_node(object: GhxObject) -> (usize, Node) {
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
            node.add_input_pin(input.name.clone());
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

    (object.id, node)
}

#[cfg(feature = "parallel")]
fn build_simple_nodes(objects: Vec<GhxObject>) -> Vec<(usize, Node)> {
    objects.into_par_iter().map(build_simple_node).collect()
}

#[cfg(not(feature = "parallel"))]
fn build_simple_nodes(objects: Vec<GhxObject>) -> Vec<(usize, Node)> {
    objects.into_iter().map(build_simple_node).collect()
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

    let parsed_objects = collect_archive_objects(object_chunks)?;

    for parsed in parsed_objects {
        let node_id = graph.add_node(parsed.node)?;

        if let Some(instance_guid) = parsed.instance_guid.as_ref() {
            if let Some(pin) = parsed.default_output_pin.as_ref() {
                output_lookup.insert(instance_guid.clone(), (node_id, pin.clone()));
            }
        }

        for output in parsed.outputs.into_iter() {
            if let Some(guid) = output.guid {
                output_lookup.insert(guid, (node_id, output.pin));
            }
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

#[cfg(feature = "parallel")]
fn collect_archive_objects(
    chunks: Vec<&RawChunk>,
) -> Result<Vec<ArchiveObjectParseResult>, ParseError> {
    chunks
        .into_par_iter()
        .enumerate()
        .map(|(idx, chunk)| parse_archive_object(chunk, idx))
        .collect()
}

#[cfg(not(feature = "parallel"))]
fn collect_archive_objects(
    chunks: Vec<&RawChunk>,
) -> Result<Vec<ArchiveObjectParseResult>, ParseError> {
    chunks
        .into_iter()
        .enumerate()
        .map(|(idx, chunk)| parse_archive_object(chunk, idx))
        .collect()
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

    if let Some(hidden_value) = container.item_value("Hidden") {
        if hidden_value.eq_ignore_ascii_case("true") {
            node.insert_meta("hidden", true);
        }
    }

    let is_slider = component_guid_norm.as_deref().map_or(false, |guid| {
        guid == "57da07bd-ecab-415d-9d86-af36d7073abc"
            || guid == "5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b"
    });
    let is_panel = component_guid_norm
        .as_deref()
        .map_or(false, |guid| guid == "59e0b89a-e487-49f8-bab8-b5bab16be14c");
    let is_value_list = component_guid_norm
        .as_deref()
        .map_or(false, |guid| guid == "00027467-0d24-4fa7-b178-8dc0ac5f42ec");
    let is_colour_swatch = component_guid_norm
        .as_deref()
        .map_or(false, |guid| guid == "9c53bac0-ba66-40bd-8154-ce9829b9db1a");

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

    if is_value_list {
        apply_value_list_meta(container, &mut node);
    }

    if is_colour_swatch {
        apply_colour_swatch_meta(container, &mut node);
    }

    let mut outputs = Vec::new();

    let output_chunks = collect_param_chunks(container, &["param_output", "outputparam"]);
    for (output_index, output_chunk) in output_chunks.into_iter().enumerate() {
        let info = parse_param_chunk(
            output_chunk,
            "out",
            output_index,
            component_guid_norm.as_deref(),
            true,
        );
        let pin_name = info.pin_name.clone();
        node.set_output(pin_name.clone(), Value::Null);
        outputs.push(OutputInfo {
            guid: info.instance_guid,
            pin: pin_name,
        });
    }

    let mut pending_inputs = Vec::new();
    let input_chunks = collect_param_chunks(container, &["param_input", "inputparam"]);
    for (input_index, input_chunk) in input_chunks.into_iter().enumerate() {
        let info = parse_param_chunk(
            input_chunk,
            "in",
            input_index,
            component_guid_norm.as_deref(),
            false,
        );
        node.add_input_pin(info.pin_name.clone());
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

    let default_output_pin = if let Some(first) = outputs.first() {
        Some(first.pin.clone())
    } else if is_slider {
        if !node.outputs.contains_key("OUT") {
            node.set_output("OUT", Value::Null);
        }
        Some("OUT".to_owned())
    } else if is_panel || is_value_list || is_colour_swatch {
        Some("Output".to_owned())
    } else if let Some(param_name) = identify_floating_param(component_guid_norm.as_deref()) {
        if !node.outputs.contains_key(&param_name) {
            node.set_output(&param_name, Value::Null);
        }
        Some(param_name)
    } else {
        None
    };

    Ok(ArchiveObjectParseResult {
        node,
        instance_guid: instance_guid_norm,
        outputs,
        default_output_pin,
        pending_inputs,
    })
}

fn identify_floating_param(guid: Option<&str>) -> Option<String> {
    let guid = guid?;
    match guid {
        // Primitive Types
        "2e3ab970-8545-46bb-836c-1c11e5610bce" => Some("Int".to_owned()),
        "3e8ca6be-fda8-4aaf-b5c0-3c54c8bb7312" => Some("Num".to_owned()),
        "3ede854e-c753-40eb-84cb-b48008f14fd4" => Some("Txt".to_owned()),
        "cb95db89-6165-43b6-9c41-5702bc5bf137" => Some("Bool".to_owned()),
        "15b7afe5-d0d0-43e1-b894-34fcfe3be384" => Some("Domain".to_owned()),
        "90744326-eb53-4a0e-b7ef-4b45f5473d6e" => Some("Domain²".to_owned()),
        "fa36c19d-b108-440c-b33d-a0a4642b45cc" => Some("Domain²".to_owned()),
        "476c0cf8-bc3c-4f1c-a61a-6e91e1f8b91e" => Some("C".to_owned()),
        "81dfff08-0c83-4f1b-a358-14791d642d9e" => Some("Time".to_owned()),
        "203a91c3-287a-43b6-a9c5-ebb96240a650" => Some("Col".to_owned()),
        "bd4a8a18-a3cc-40ba-965b-3be91fee563b" => Some("Matrix".to_owned()),
        "06953bda-1d37-4d58-9b38-4b3c74e54c8f" => Some("Path".to_owned()),
        "56c9c942-791f-4eeb-a4f0-82b93f1c0909" => Some("Path".to_owned()),
        "faf6e3bb-4c84-4cbf-bd88-6d6a0db5667a" => Some("ID".to_owned()),

        // Geometry Types
        "fbac3e32-f100-4292-8692-77240a42fd1a" => Some("Pt".to_owned()),
        "16ef3e75-e315-4899-b531-d3166b42dac9" => Some("Vec".to_owned()),
        "8529dbdf-9b6f-42e9-8e1f-c7a2bde56a70" => Some("Line".to_owned()),
        "1e936df3-0eea-4246-8549-514cb8862b7a" => Some("Mesh".to_owned()),
        "deaf8653-5528-4286-807c-3de8b8dad781" => Some("Srf".to_owned()),
        "d5967b9f-e8ee-436b-a8ad-29fdcecf32d5" => Some("Crv".to_owned()),
        "e02b3da5-543a-46ac-a867-0ba6b0a524de" => Some("Face".to_owned()),
        "04d3eace-deaa-475e-9e69-8f804d687998" => Some("Arc".to_owned()),
        "28f40e48-e739-4211-91bd-f4aefa5965f8" => Some("Transform".to_owned()),
        "3175e3eb-1ae0-4d0b-9395-53fd3e8f8a28" => Some("Field".to_owned()),
        "4f8984c4-7c7a-4d69-b0a2-183cbb330d20" => Some("Pln".to_owned()),
        "6db039c4-cad1-4549-bd45-e31cb0f71692" => Some("TBox".to_owned()),
        "87391af3-35fe-4a40-b001-2bd4547ccd45" => Some("Loc".to_owned()),
        "89cd1a12-0007-4581-99ba-66578665e610" => Some("SubD".to_owned()),
        "919e146f-30ae-4aae-be34-4d72f555e7da" => Some("Brep".to_owned()),
        "a80395af-f134-4d6a-9b89-15edf3161619" => Some("Atom".to_owned()),
        "abf9c670-5462-4cd8-acb3-f1ab0256dbf3" => Some("Rec".to_owned()),
        "ac2bc2cb-70fb-4dd5-9c78-7e1ea97fe278" => Some("Geo".to_owned()),
        "b0851fc0-ab55-47d8-bdda-cc6306a40176" => Some("Grp".to_owned()),
        "b341e2e5-c4b3-49a3-b3a4-b4e6e2054516" => Some("Pipeline".to_owned()),
        "c3407fda-b505-4686-9165-38fe7a9274cf" => Some("Mesh".to_owned()),
        "c9482db6-bea9-448d-98ff-fed6d69a8efc" => Some("Box".to_owned()),
        "d1028c72-ff86-4057-9eb0-36c687a4d98c" => Some("Circle".to_owned()),
        "f91778ca-2700-42fc-8ee6-74049a2292b5" => Some("Geometry Cache".to_owned()),
        "fa20fe95-5775-417b-92ff-b77c13cbf40c" => Some("MPoint".to_owned()),

        _ => None,
    }
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

    node.insert_meta("value", value);
    if let Some(step) = step.and_then(|step| {
        if step.is_finite() && step > 0.0 {
            Some(step)
        } else {
            None
        }
    }) {
        node.insert_meta("step", step);
    }
    node.set_output("OUT", Value::Number(value));
}

fn apply_value_list_meta(container: &RawChunk, node: &mut Node) {
    let mut items = Vec::new();
    let mut selected_index = 0;
    let mut current_output_value = Value::Null;

    // Iterate over ListItem chunks.
    // Note: The exact chunk name for items might need to be checked.
    // In the provided example it's <chunk name="ListItem" index="...">
    let list_items: Vec<&RawChunk> = container
        .children()
        .filter(|c| c.name.eq_ignore_ascii_case("ListItem"))
        .collect();

    for (idx, item_chunk) in list_items.iter().enumerate() {
        let name = item_chunk.item_value("Name").unwrap_or("").to_string();
        let expression = item_chunk.item_value("Expression").unwrap_or("").to_string();
        let selected = item_chunk
            .item_value("Selected")
            .map(|s| s.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        // Store item info.
        // We store the expression as the value, but we try to parse it as a number if possible.
        // However, ValueList usually outputs the expression value.
        // If expression is "0", it could be number or text. Let's match how parse_persistent_value does it roughly,
        // or just keep it simple.
        let value = if let Some(num) = parse_f64(&expression) {
             Value::Number(num)
        } else {
             Value::Text(expression.clone())
        };

        if selected {
            selected_index = idx;
            current_output_value = value.clone();
        }

        // For metadata, we probably want to store the values so the component can use them if needed (e.g. to change selection).
        // But ValueListComponent currently just picks one based on index.
        items.push(match value {
            Value::Number(n) => MetaValue::Number(n),
            Value::Text(t) => MetaValue::Text(t),
            Value::Boolean(b) => MetaValue::Boolean(b),
            _ => MetaValue::Text(expression), // Fallback
        });
    }

    node.insert_meta("ListItems", MetaValue::List(items));
    node.insert_meta("SelectedIndex", selected_index as f64);
    node.set_output("Output", current_output_value);
}

fn apply_colour_swatch_meta(container: &RawChunk, node: &mut Node) {
    let argb_str = container.item_argb("SwatchColor").unwrap_or("255;0;0;0");
    let parts: Vec<&str> = argb_str.split(';').collect();

    // We expect 4 parts: A, R, G, B. Take indices 1, 2, 3 for R, G, B.
    let rgb_values = if parts.len() == 4 {
        let r = parts[1].parse::<f64>().unwrap_or(0.0);
        let g = parts[2].parse::<f64>().unwrap_or(0.0);
        let b = parts[3].parse::<f64>().unwrap_or(0.0);
        vec![r, g, b]
    } else {
        vec![0.0, 0.0, 0.0]
    };

    let meta_list: Vec<MetaValue> = rgb_values.into_iter().map(MetaValue::Number).collect();
    node.insert_meta("SwatchColorRGB", MetaValue::List(meta_list));
    node.set_output("Output", Value::Null);
}

fn collect_param_chunks<'a>(root: &'a RawChunk, target_names: &[&str]) -> Vec<&'a RawChunk> {
    let mut collected = Vec::new();

    fn visit<'a>(chunk: &'a RawChunk, target_names: &[&str], output: &mut Vec<&'a RawChunk>) {
        for child in chunk.children() {
            if target_names
                .iter()
                .any(|name| child.name.eq_ignore_ascii_case(name))
            {
                output.push(child);
            }
            visit(child, target_names, output);
        }
    }

    visit(root, target_names, &mut collected);
    collected
}

fn parse_param_chunk(
    chunk: &RawChunk,
    fallback_prefix: &str,
    fallback_index: usize,
    component_guid: Option<&str>,
    is_output: bool,
) -> ParamInfo {
    let index = chunk.index.unwrap_or(fallback_index);

    let pin_name_raw = chunk
        .item_value("NickName")
        .or_else(|| chunk.item_value("Name"))
        .or_else(|| chunk.item_value("Description"))
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{fallback_prefix}{index}"));

    let pin_name = normalize_pin_name(pin_name_raw, component_guid, is_output);

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

fn normalize_pin_name(pin_name: String, component_guid: Option<&str>, is_output: bool) -> String {
    if !is_output {
        return pin_name;
    }

    let Some(guid) = component_guid else {
        return pin_name;
    };

    if guid == "3581f42a-9592-4549-bd6b-1c0fc39d067b" && pin_name.eq_ignore_ascii_case("pt") {
        return "P".to_owned();
    }

    pin_name
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
    #[serde(rename = "ARGB", default)]
    argb: Option<String>,
}

#[derive(Debug)]
struct ArchiveObjectParseResult {
    node: Node,
    instance_guid: Option<String>,
    outputs: Vec<OutputInfo>,
    default_output_pin: Option<String>,
    pending_inputs: Vec<PendingInput>,
}

#[derive(Debug)]
struct OutputInfo {
    guid: Option<String>,
    pin: String,
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

    fn item_argb(&self, name: &str) -> Option<&str> {
        self.items
            .items
            .iter()
            .find(|item| item.name.eq_ignore_ascii_case(name))
            .and_then(|item| item.argb.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::parse_str;
    use crate::graph::node::MetaValue;
    use crate::graph::value::Value;

    #[test]
    fn parses_point_with_default_value() {
        let xml = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tools/tools/ghx-samples/point_default.ghx"
        ));
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
        let xml = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tools/ghx-samples/minimal_line.ghx"
        ));
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
        let xml = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tools/ghx-samples/minimal_extrude.ghx"
        ));
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

    #[test]
    fn parses_colour_swatch_meta() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Archive name="Root">
  <chunks count="1">
    <chunk name="Definition">
      <chunks count="1">
        <chunk name="DefinitionObjects">
          <chunks count="1">
            <chunk name="Object" index="0">
              <items count="2">
                <item name="GUID" type_name="gh_guid" type_code="9">9c53bac0-ba66-40bd-8154-ce9829b9db1a</item>
              </items>
              <chunks count="1">
                <chunk name="Container">
                  <items count="1">
                     <item name="SwatchColor"><ARGB>255;100;50;25</ARGB></item>
                  </items>
                </chunk>
              </chunks>
            </chunk>
          </chunks>
        </chunk>
      </chunks>
    </chunk>
  </chunks>
</Archive>
"#;
        let graph = parse_str(xml).expect("parsed");
        let node = graph
            .nodes()
            .iter()
            .find(|n| n.guid.as_deref() == Some("{9c53bac0-ba66-40bd-8154-ce9829b9db1a}"))
            .unwrap();

        // Verify SwatchColorRGB meta
        let rgb = match node.meta("SwatchColorRGB").unwrap() {
            MetaValue::List(l) => l,
            _ => panic!("Expected list"),
        };
        assert_eq!(rgb.len(), 3);
        // Indices 1,2,3 of 255;100;50;25 -> 100, 50, 25
        assert_eq!(rgb[0], MetaValue::Number(100.0));
        assert_eq!(rgb[1], MetaValue::Number(50.0));
        assert_eq!(rgb[2], MetaValue::Number(25.0));

        // Verify Output pin exists
        assert!(node.outputs.contains_key("Output"));
    }

    #[test]
    fn parses_parameter_data_output_param_wires() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Archive name="Root">
  <items count="0" />
  <chunks count="1">
    <chunk name="Definition">
      <items count="0" />
      <chunks count="1">
        <chunk name="DefinitionObjects">
          <items count="1">
            <item name="ObjectCount" type_name="gh_int32" type_code="3">2</item>
          </items>
          <chunks count="2">
            <chunk name="Object" index="0">
              <items count="2">
                <item name="GUID" type_name="gh_guid" type_code="9">aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa</item>
                <item name="Name" type_name="gh_string" type_code="10">Producer</item>
              </items>
              <chunks count="1">
                <chunk name="Container">
                  <items count="4">
                    <item name="Description" type_name="gh_string" type_code="10"></item>
                    <item name="InstanceGuid" type_name="gh_guid" type_code="9">11111111-1111-1111-1111-111111111111</item>
                    <item name="Name" type_name="gh_string" type_code="10">Producer</item>
                    <item name="NickName" type_name="gh_string" type_code="10">Producer</item>
                  </items>
                  <chunks count="2">
                    <chunk name="ParameterData">
                      <items count="2">
                        <item name="InputCount" type_name="gh_int32" type_code="3">0</item>
                        <item name="OutputCount" type_name="gh_int32" type_code="3">2</item>
                      </items>
                      <chunks count="2">
                        <chunk name="OutputParam" index="0">
                          <items count="5">
                            <item name="Description" type_name="gh_string" type_code="10">Primary result</item>
                            <item name="Name" type_name="gh_string" type_code="10">Primary</item>
                            <item name="NickName" type_name="gh_string" type_code="10">P</item>
                            <item name="Optional" type_name="gh_bool" type_code="1">false</item>
                            <item name="SourceCount" type_name="gh_int32" type_code="3">0</item>
                          </items>
                        </chunk>
                        <chunk name="OutputParam" index="1">
                          <items count="6">
                            <item name="Description" type_name="gh_string" type_code="10">Secondary result</item>
                            <item name="InstanceGuid" type_name="gh_guid" type_code="9">22222222-2222-2222-2222-222222222222</item>
                            <item name="Name" type_name="gh_string" type_code="10">Secondary</item>
                            <item name="NickName" type_name="gh_string" type_code="10">S</item>
                            <item name="Optional" type_name="gh_bool" type_code="1">false</item>
                            <item name="SourceCount" type_name="gh_int32" type_code="3">0</item>
                          </items>
                        </chunk>
                      </chunks>
                    </chunk>
                    <chunk name="Attributes">
                      <items count="0" />
                    </chunk>
                  </chunks>
                </chunk>
              </chunks>
            </chunk>
            <chunk name="Object" index="1">
              <items count="2">
                <item name="GUID" type_name="gh_guid" type_code="9">bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb</item>
                <item name="Name" type_name="gh_string" type_code="10">Consumer</item>
              </items>
              <chunks count="1">
                <chunk name="Container">
                  <items count="4">
                    <item name="Description" type_name="gh_string" type_code="10"></item>
                    <item name="InstanceGuid" type_name="gh_guid" type_code="9">33333333-3333-3333-3333-333333333333</item>
                    <item name="Name" type_name="gh_string" type_code="10">Consumer</item>
                    <item name="NickName" type_name="gh_string" type_code="10">Consumer</item>
                  </items>
                  <chunks count="2">
                    <chunk name="ParameterData">
                      <items count="2">
                        <item name="InputCount" type_name="gh_int32" type_code="3">1</item>
                        <item name="OutputCount" type_name="gh_int32" type_code="3">1</item>
                      </items>
                      <chunks count="2">
                        <chunk name="InputParam" index="0">
                          <items count="7">
                            <item name="Description" type_name="gh_string" type_code="10">Input</item>
                            <item name="InstanceGuid" type_name="gh_guid" type_code="9">44444444-4444-4444-4444-444444444444</item>
                            <item name="Name" type_name="gh_string" type_code="10">In</item>
                            <item name="NickName" type_name="gh_string" type_code="10">I</item>
                            <item name="Optional" type_name="gh_bool" type_code="1">false</item>
                            <item name="Source" index="0" type_name="gh_guid" type_code="9">22222222-2222-2222-2222-222222222222</item>
                            <item name="SourceCount" type_name="gh_int32" type_code="3">1</item>
                          </items>
                        </chunk>
                        <chunk name="OutputParam" index="0">
                          <items count="5">
                            <item name="Description" type_name="gh_string" type_code="10">Out</item>
                            <item name="Name" type_name="gh_string" type_code="10">Out</item>
                            <item name="NickName" type_name="gh_string" type_code="10">Out</item>
                            <item name="Optional" type_name="gh_bool" type_code="1">false</item>
                            <item name="SourceCount" type_name="gh_int32" type_code="3">0</item>
                          </items>
                        </chunk>
                      </chunks>
                    </chunk>
                    <chunk name="Attributes">
                      <items count="0" />
                    </chunk>
                  </chunks>
                </chunk>
              </chunks>
            </chunk>
          </chunks>
        </chunk>
      </chunks>
    </chunk>
  </chunks>
</Archive>
"#;

        let graph = parse_str(xml).expect("graph parsed");
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.wire_count(), 1);

        let wire = &graph.wires()[0];
        let source_node = graph.node(wire.from_node).expect("source node exists");
        assert!(
            source_node.outputs.contains_key("S"),
            "secondary output pin should be registered"
        );
        assert_eq!(wire.from_pin.0, "S");
        assert_eq!(wire.to_pin.0, "I");
    }
}
