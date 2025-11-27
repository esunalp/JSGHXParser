#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod components;
pub mod graph;
pub mod parse;

use std::collections::{BTreeMap, HashSet};
use std::fmt;

use components::{ComponentKind, ComponentRegistry};
use graph::Graph;
use graph::evaluator::{self, EvaluationPlan, EvaluationResult, GeometryEntry};
use graph::node::{MetaLookupExt, MetaMap, MetaValue, NodeId};
use graph::value::{ColorValue, MaterialValue, Value};
use serde::Serialize;
use wasm_bindgen::JsError;
use wasm_bindgen::prelude::*;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "console_error_panic_hook", target_arch = "wasm32"))] {
        #[wasm_bindgen(start)]
        pub fn initialize() {
            console_error_panic_hook::set_once();
            init_logger();
        }
    } else {
        #[wasm_bindgen(start)]
        pub fn initialize() {
            // no-op fallback when panic hook is disabled
            init_logger();
        }
    }
}

#[cfg(feature = "debug_logs")]
fn init_logger() {
    use log::LevelFilter;
    use wasm_bindgen_console_logger::DEFAULT_LOGGER;
    log::set_logger(&DEFAULT_LOGGER).expect("error initializing logger");
    log::set_max_level(LevelFilter::Debug);
}

#[cfg(not(feature = "debug_logs"))]
fn init_logger() {
    // no-op fallback when debug logs are disabled
}

#[cfg(all(feature = "parallel", target_arch = "wasm32"))]
#[wasm_bindgen]
pub async fn initialize_parallel(worker_count: Option<u32>) -> Result<(), JsError> {
    let threads = worker_count
        .map(|count| count.max(1) as usize)
        .or_else(|| {
            std::thread::available_parallelism()
                .map(|value| value.get())
                .ok()
        })
        .unwrap_or(1);

    wasm_bindgen_rayon::init_thread_pool(threads)
        .await
        .map_err(|err| JsError::new(&format!("kon rayon threadpool niet initialiseren: {err}")))
}

#[macro_export]
macro_rules! debug_log {
    ($($t:tt)*) => {{
        #[cfg(feature = "debug_logs")]
        {
            #[cfg(target_arch = "wasm32")]
            {
                ::web_sys::console::log_1(&::wasm_bindgen::JsValue::from_str(&format!($($t)*)));
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                println!("{}", format!($($t)*));
            }
        }
    }};
}

#[derive(Debug, Clone)]
struct InputBinding {
    id: String,
    node_id: NodeId,
    output_pin: String,
    search_keys: Vec<String>,
    kind: InputKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputKind {
    Slider,
    Toggle,
    ValueList,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum InputControl {
    #[serde(rename = "slider")]
    Slider {
        id: String,
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
        value: f64,
    },
    #[serde(rename = "toggle")]
    Toggle {
        id: String,
        name: String,
        value: bool,
    },
    #[serde(rename = "value-list")]
    ValueList {
        id: String,
        name: String,
        items: Vec<ValueListItem>,
        selected_index: usize,
        value: f64,
    },
}

#[derive(Debug, Serialize)]
struct ValueListItem {
    label: String,
}

#[derive(Debug, Default, Serialize)]
struct GeometryDiff<'a> {
    added: Vec<GeometryDiffItem<'a>>,
    updated: Vec<GeometryDiffItem<'a>>,
    removed: Vec<usize>,
}

#[derive(Debug, Serialize)]
struct GeometryDiffItem<'a> {
    id: usize,
    items: Vec<GeometryItem<'a>>,
}

#[derive(Debug, Serialize)]
struct NodeInfo {
    id: usize,
    name: String,
    outputs: BTreeMap<String, String>,
    connected_to: Vec<usize>,
}

#[derive(Debug, Serialize)]
struct NodeInfoResponse {
    nodes: Vec<NodeInfo>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
struct MaterialExport {
    diffuse: [f64; 3],
    specular: [f64; 3],
    emission: [f64; 3],
    transparency: f64,
    shine: f64,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(tag = "type")]
enum GeometryItem<'a> {
    Point {
        coordinates: [f64; 3],
    },
    Line {
        start: [f64; 3],
        end: [f64; 3],
    },
    Polyline {
        points: Vec<[f64; 3]>,
    },
    Mesh {
        vertices: &'a [[f64; 3]],
        faces: &'a [Vec<u32>],
        #[serde(skip_serializing_if = "Option::is_none")]
        material: Option<MaterialExport>,
    },
}

impl<'a> GeometryItem<'a> {
    fn deep_clone(&self) -> GeometryItem<'static> {
        match self {
            GeometryItem::Point { coordinates } => GeometryItem::Point {
                coordinates: *coordinates,
            },
            GeometryItem::Line { start, end } => GeometryItem::Line {
                start: *start,
                end: *end,
            },
            GeometryItem::Polyline { points } => GeometryItem::Polyline {
                points: points.clone(),
            },
            GeometryItem::Mesh {
                vertices,
                faces,
                material,
            } => GeometryItem::Mesh {
                vertices: Box::leak(vertices.to_vec().into_boxed_slice()),
                faces: Box::leak(faces.to_vec().into_boxed_slice()),
                material: material.clone(),
            },
        }
    }
}

impl From<MaterialValue> for MaterialExport {
    fn from(material: MaterialValue) -> Self {
        Self {
            diffuse: color_to_array(material.diffuse),
            specular: color_to_array(material.specular),
            emission: color_to_array(material.emission),
            transparency: material.transparency,
            shine: material.shine,
        }
    }
}

fn color_to_array(color: ColorValue) -> [f64; 3] {
    [color.r, color.g, color.b]
}

/// Public entry point for consumers.
#[wasm_bindgen]
pub struct Engine {
    initialized: bool,
    registry: ComponentRegistry,
    graph: Option<Graph>,
    last_result: Option<EvaluationResult>,
    input_bindings: Vec<InputBinding>,
    evaluation_plan: Option<EvaluationPlan>,
    dirty_nodes: HashSet<NodeId>,
    result_dirty: bool,
    geometry_map: BTreeMap<NodeId, Vec<GeometryItem<'static>>>,
    changed_nodes_since_geometry_update: HashSet<NodeId>,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Engine {
        Engine {
            initialized: true,
            registry: ComponentRegistry::default(),
            graph: None,
            last_result: None,
            input_bindings: Vec::new(),
            evaluation_plan: None,
            dirty_nodes: HashSet::new(),
            result_dirty: false,
            geometry_map: BTreeMap::new(),
            changed_nodes_since_geometry_update: HashSet::new(),
        }
    }

    /// Geeft terug of de engine de minimale initialisatie heeft doorlopen.
    #[wasm_bindgen]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Laad een GHX-bestand in de engine en prepareer slider-informatie.
    #[wasm_bindgen]
    pub fn load_ghx(&mut self, xml: &str) -> Result<(), JsValue> {
        let graph = parse::ghx_xml::parse_str(xml).map_err(to_js_error)?;
        let input_bindings = collect_input_bindings(&graph, &self.registry);
        let evaluation_plan = evaluator::EvaluationPlan::new(&graph).map_err(to_js_error)?;
        let node_ids: Vec<NodeId> = graph.nodes().iter().map(|node| node.id).collect();

        self.graph = Some(graph);
        self.input_bindings = input_bindings;
        self.last_result = None;
        self.evaluation_plan = Some(evaluation_plan);
        self.dirty_nodes.clear();
        self.dirty_nodes.extend(node_ids);
        self.result_dirty = true;
        self.geometry_map.clear();
        self.changed_nodes_since_geometry_update.clear();

        Ok(())
    }

    /// Haal input controls (sliders, toggles) op voor UI-generatie.
    #[wasm_bindgen]
    pub fn get_sliders(&self) -> Result<JsValue, JsValue> {
        let graph = match self.graph.as_ref() {
            Some(graph) => graph,
            None => return Err(js_error("er is geen GHX-bestand geladen")),
        };

        let mut controls = Vec::with_capacity(self.input_bindings.len());
        for binding in &self.input_bindings {
            let control = match input_control_state(graph, binding) {
                Ok(control) => control,
                Err(err) => return Err(js_error(&err)),
            };
            controls.push(control);
        }

        serde_wasm_bindgen::to_value(&controls).map_err(|err| JsError::new(&err.to_string()).into())
    }

    /// Stel een slider- of togglewaarde in op basis van id of naam.
    #[wasm_bindgen]
    pub fn set_slider_value(&mut self, id_or_name: &str, value: JsValue) -> Result<(), JsValue> {
        let val = if let Some(n) = value.as_f64() {
            Value::Number(n)
        } else if let Some(b) = value.as_bool() {
            Value::Boolean(b)
        } else {
            return Err(js_error("sliderwaarde moet een getal of boolean zijn"));
        };

        self.update_input_value(id_or_name, val)
            .map_err(|e| js_error(&e))
    }

    /// Evalueer de geladen graph.
    #[wasm_bindgen]
    pub fn evaluate(&mut self) -> Result<(), JsValue> {
        if !self.result_dirty && self.dirty_nodes.is_empty() {
            return Ok(());
        }

        let mut dirty_nodes = std::mem::take(&mut self.dirty_nodes);

        let graph = match self.graph.as_ref() {
            Some(graph) => graph,
            None => {
                self.dirty_nodes = dirty_nodes;
                return Err(js_error("er is geen GHX-bestand geladen"));
            }
        };

        let plan = match self.evaluation_plan.as_ref() {
            Some(plan) => plan,
            None => {
                self.dirty_nodes = dirty_nodes;
                return Err(js_error("graph is niet voorbereid voor evaluatie"));
            }
        };

        let previous = self.last_result.as_ref();
        let evaluation = evaluator::evaluate_with_plan_incremental(
            graph,
            &self.registry,
            plan,
            previous,
            &dirty_nodes,
        );

        match evaluation {
            Ok((result, changed)) => {
                self.last_result = Some(result);
                self.result_dirty = false;
                self.changed_nodes_since_geometry_update.extend(changed);
            }
            Err(error) => {
                self.dirty_nodes = dirty_nodes;
                return Err(to_js_error(error));
            }
        }

        dirty_nodes.clear();
        Ok(())
    }

    /// Haalt de geometrie op van de laatste evaluatie in een "diff" formaat.
    #[wasm_bindgen]
    pub fn get_geometry(&mut self) -> Result<JsValue, JsValue> {
        if self.result_dirty {
            return Err(js_error("graph is nog niet geëvalueerd"));
        }

        let result = match self.last_result.as_ref() {
            Some(result) => result,
            None => {
                let diff = GeometryDiff {
                    removed: self.geometry_map.keys().map(|id| id.0).collect(),
                    ..Default::default()
                };
                self.geometry_map.clear();
                self.changed_nodes_since_geometry_update.clear();
                return serde_wasm_bindgen::to_value(&diff)
                    .map_err(|err| JsError::new(&err.to_string()).into());
            }
        };

        let mut diff = GeometryDiff::default();
        let graph = self.graph.as_ref().unwrap();

        let geometry_by_node: BTreeMap<NodeId, Vec<GeometryEntry>> =
            result
                .geometry
                .iter()
                .fold(BTreeMap::new(), |mut acc, entry| {
                    acc.entry(entry.source_node)
                        .or_default()
                        .push(entry.clone());
                    acc
                });

        let changed_nodes = std::mem::take(&mut self.changed_nodes_since_geometry_update);

        for node_id in &changed_nodes {
            let is_hidden = graph
                .node(*node_id)
                .and_then(|node| node.meta("hidden"))
                .and_then(MetaValue::as_boolean)
                .unwrap_or(false);

            let new_items: Vec<GeometryItem<'static>> = if is_hidden {
                Vec::new()
            } else {
                geometry_by_node
                    .get(node_id)
                    .map(|entries| {
                        let mut items = Vec::new();
                        for entry in entries {
                            append_geometry_items(entry, &mut items);
                        }
                        items.into_iter().map(|item| item.deep_clone()).collect()
                    })
                    .unwrap_or_default()
            };

            if let Some(existing_items) = self.geometry_map.get(node_id) {
                if new_items.is_empty() {
                    diff.removed.push(node_id.0);
                    self.geometry_map.remove(node_id);
                } else if existing_items.iter().ne(new_items.iter()) {
                    diff.updated.push(GeometryDiffItem {
                        id: node_id.0,
                        items: new_items.clone(),
                    });
                    self.geometry_map.insert(*node_id, new_items);
                }
            } else if !new_items.is_empty() {
                diff.added.push(GeometryDiffItem {
                    id: node_id.0,
                    items: new_items.clone(),
                });
                self.geometry_map.insert(*node_id, new_items);
            }
        }

        serde_wasm_bindgen::to_value(&diff).map_err(|err| JsError::new(&err.to_string()).into())
    }

    /// Haalt een tekstuele weergave op van de topologisch gesorteerde graaf.
    #[wasm_bindgen]
    pub fn get_topology_map(&self) -> Result<JsValue, JsValue> {
        if self.graph.is_none() {
            return Err(js_error("er is geen GHX-bestand geladen"));
        }

        let plan = self
            .evaluation_plan
            .as_ref()
            .ok_or_else(|| js_error("graph is niet voorbereid voor evaluatie"))?;

        let map = plan
            .order()
            .iter()
            .map(|id| id.0.to_string())
            .collect::<Vec<_>>()
            .join(" -> ");

        Ok(JsValue::from_str(&map))
    }

    #[wasm_bindgen]
    pub fn get_node_info(&self) -> Result<JsValue, JsValue> {
        let graph = self
            .graph
            .as_ref()
            .ok_or_else(|| js_error("er is geen GHX-bestand geladen"))?;

        let result = if self.result_dirty {
            None
        } else {
            self.last_result.as_ref()
        };

        let mut nodes_info = Vec::new();

        for node in graph.nodes() {
            let resolved_outputs = result
                .and_then(|r| r.node_outputs.get(&node.id))
                .cloned()
                .unwrap_or_else(|| node.outputs.clone());

            let outputs = resolved_outputs
                .into_iter()
                .map(|(k, v)| (k, v.to_string()))
                .collect();

            let connected_to = graph
                .wires()
                .iter()
                .filter(|w| w.from_node == node.id)
                .map(|w| w.to_node.0)
                .collect();

            nodes_info.push(NodeInfo {
                id: node.id.0,
                name: node
                    .nickname
                    .clone()
                    .or(node.name.clone())
                    .unwrap_or_default(),
                outputs,
                connected_to,
            });
        }

        serde_wasm_bindgen::to_value(&NodeInfoResponse { nodes: nodes_info })
            .map_err(|err| JsError::new(&err.to_string()).into())
    }
}

fn collect_input_bindings(graph: &Graph, registry: &ComponentRegistry) -> Vec<InputBinding> {
    let mut bindings = Vec::new();

    for node in graph.nodes() {
        let component = registry.resolve(
            node.guid.as_deref(),
            node.name.as_deref(),
            node.nickname.as_deref(),
        );

        let kind = match component {
            Some(ComponentKind::ParamsInput(
                components::params_input::ComponentKind::NumberSlider,
            )) => Some(InputKind::Slider),
            Some(ComponentKind::ParamsInput(
                components::params_input::ComponentKind::BooleanToggle,
            )) => Some(InputKind::Toggle),
            Some(ComponentKind::ParamsInput(
                components::params_input::ComponentKind::ValueList,
            )) => Some(InputKind::ValueList),
            _ => None,
        };

        if let Some(kind) = kind {
            let output_pin = node
                .outputs
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "Output".to_string());

            let mut search_keys = Vec::new();
            if let Some(name) = node.name.as_deref() {
                search_keys.push(normalize_name(name));
            }
            if let Some(nickname) = node.nickname.as_deref() {
                search_keys.push(normalize_name(nickname));
            }

            bindings.push(InputBinding {
                id: node.id.0.to_string(),
                node_id: node.id,
                output_pin,
                search_keys,
                kind,
            });
        }
    }

    bindings
}

fn input_control_state(graph: &Graph, binding: &InputBinding) -> Result<InputControl, String> {
    let node = graph
        .node(binding.node_id)
        .ok_or_else(|| "interne inputreferentie is ongeldig".to_owned())?;

    let name = node
        .nickname
        .as_deref()
        .or(node.name.as_deref())
        .unwrap_or(&binding.id)
        .to_owned();

    match binding.kind {
        InputKind::Slider => {
            let value = required_meta_number(&node.meta, "value")?;
            let min = meta_number(&node.meta, "min")?;
            let max = meta_number(&node.meta, "max")?;
            let step = meta_number(&node.meta, "step")?;

            Ok(InputControl::Slider {
                id: binding.id.clone(),
                name,
                min,
                max,
                step,
                value,
            })
        }
        InputKind::ValueList => {
            let items = value_list_items(&node.meta);
            let selected_index = value_list_selected_index(&node.meta, items.len());

            Ok(InputControl::ValueList {
                id: binding.id.clone(),
                name,
                items,
                selected_index,
                value: selected_index as f64,
            })
        }
        InputKind::Toggle => {
            let value = node
                .meta("Value")
                .and_then(|v| match v {
                    MetaValue::Boolean(b) => Some(*b),
                    _ => None,
                })
                .unwrap_or(false);

            Ok(InputControl::Toggle {
                id: binding.id.clone(),
                name,
                value,
            })
        }
    }
}

fn value_list_items(meta: &MetaMap) -> Vec<ValueListItem> {
    let list_values = match meta
        .get_normalized("ListItems")
        .or_else(|| meta.get_normalized("Values"))
    {
        Some(MetaValue::List(entries)) => entries,
        _ => return Vec::new(),
    };

    list_values
        .iter()
        .filter_map(meta_value_to_value)
        .map(|value| ValueListItem {
            label: format_value_label(&value),
        })
        .collect()
}

fn value_list_selected_index(meta: &MetaMap, total_items: usize) -> usize {
    let mut index = meta
        .get_normalized("SelectedIndex")
        .or_else(|| meta.get_normalized("Value"))
        .and_then(meta_value_to_index)
        .unwrap_or(0);

    if total_items == 0 {
        return 0;
    }

    if index >= total_items {
        index = total_items - 1;
    }

    index
}

fn meta_value_to_value(value: &MetaValue) -> Option<Value> {
    match value {
        MetaValue::Number(n) => Some(Value::Number(*n)),
        MetaValue::Integer(i) => Some(Value::Number(*i as f64)),
        MetaValue::Boolean(b) => Some(Value::Boolean(*b)),
        MetaValue::Text(t) => Some(Value::Text(t.clone())),
        MetaValue::List(list) => {
            let values: Vec<Value> = list.iter().filter_map(meta_value_to_value).collect();

            if values.len() == list.len() {
                Some(Value::List(values))
            } else {
                None
            }
        }
    }
}

fn meta_value_to_index(value: &MetaValue) -> Option<usize> {
    if let MetaValue::Integer(i) = value {
        if *i >= 0 {
            return Some(*i as usize);
        }
        return None;
    }

    if let MetaValue::Number(n) = value {
        if n.is_finite() && *n >= 0.0 {
            return Some(*n as usize);
        }
    }

    None
}

fn format_value_label(value: &Value) -> String {
    match value {
        Value::Null => "Null".to_string(),
        Value::Text(text) => format!("Text: \"{text}\""),
        Value::Number(number) => format!("Number: {}", number),
        Value::Boolean(flag) => {
            if *flag {
                "True".to_string()
            } else {
                "False".to_string()
            }
        }
        Value::Point(point) => format!("Point: ({}, {}, {})", point[0], point[1], point[2]),
        Value::Vector(vector) => {
            format!("Vector: ({}, {}, {})", vector[0], vector[1], vector[2])
        }
        Value::List(list) => format!("List ({} items)", list.len()),
        _ => value.to_string(),
    }
}

fn append_geometry_items<'a>(entry: &'a GeometryEntry, items: &mut Vec<GeometryItem<'a>>) {
    append_geometry_value(&entry.value, entry.material, items);
}

fn append_geometry_value<'a>(
    value: &'a Value,
    material: Option<MaterialValue>,
    items: &mut Vec<GeometryItem<'a>>,
) {
    match value {
        Value::Point(point) => {
            items.push(GeometryItem::Point {
                coordinates: *point,
            });
        }
        Value::CurveLine { p1, p2 } => {
            items.push(GeometryItem::Line {
                start: *p1,
                end: *p2,
            });
        }
        Value::Surface { vertices, faces } => {
            items.push(GeometryItem::Mesh {
                vertices,
                faces,
                material: material.map(MaterialExport::from),
            });
        }
        Value::List(values) => {
            if let Some(polyline) = list_as_polyline(values) {
                items.push(GeometryItem::Polyline { points: polyline });
            } else {
                for entry in values {
                    append_geometry_value(entry, material, items);
                }
            }
        }
        Value::Null
        | Value::Number(_)
        | Value::Vector(_)
        | Value::Boolean(_)
        | Value::Domain(_)
        | Value::Matrix(_)
        | Value::Text(_)
        | Value::DateTime(_)
        | Value::Complex(_)
        | Value::Tag(_)
        | Value::Color(_)
        | Value::Material(_)
        | Value::Symbol(_) => {}
    }
}

fn list_as_polyline(values: &[Value]) -> Option<Vec<[f64; 3]>> {
    if values.len() < 2 {
        return None;
    }

    let mut points = Vec::with_capacity(values.len());
    for value in values {
        match value {
            Value::Point(point) => points.push(*point),
            _ => return None,
        }
    }

    Some(points)
}

#[cfg(test)]
mod tests {
    use super::{GeometryEntry, GeometryItem, append_geometry_items, list_as_polyline};
    use crate::graph::node::NodeId;
    use crate::graph::value::Value;

    #[test]
    fn detects_polyline_from_point_list() {
        let mut items = Vec::new();
        let list = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([1.0, 1.0, 0.0]),
        ]);

        let entry = GeometryEntry {
            source_node: NodeId::new(0),
            value: list,
            material: None,
        };

        append_geometry_items(&entry, &mut items);

        assert_eq!(items.len(), 1);
        match &items[0] {
            GeometryItem::Polyline { points } => {
                assert_eq!(points.len(), 3);
                assert_eq!(points[0], [0.0, 0.0, 0.0]);
                assert_eq!(points[2], [1.0, 1.0, 0.0]);
            }
            other => panic!("verwacht Polyline, kreeg {other:?}"),
        }
    }

    #[test]
    fn collects_nested_geometry_variants() {
        let mut items = Vec::new();
        let value = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::CurveLine {
                p1: [0.0, 0.0, 0.0],
                p2: [5.0, 0.0, 0.0],
            },
            Value::Surface {
                vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                faces: vec![vec![0, 1, 2]],
            },
        ]);

        let entry = GeometryEntry {
            source_node: NodeId::new(0),
            value,
            material: None,
        };

        append_geometry_items(&entry, &mut items);

        assert_eq!(items.len(), 3);
        assert!(matches!(items[0], GeometryItem::Point { .. }));
        assert!(matches!(items[1], GeometryItem::Line { .. }));
        assert!(matches!(items[2], GeometryItem::Mesh { .. }));
    }

    #[test]
    fn list_as_polyline_rejects_mixed_values() {
        let values = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Number(1.0),
            Value::Point([1.0, 1.0, 0.0]),
        ];

        assert!(list_as_polyline(&values).is_none());
    }
}

fn meta_number(meta: &MetaMap, key: &str) -> Result<Option<f64>, String> {
    match meta.get_normalized(key) {
        Some(MetaValue::Number(value)) => Ok(Some(*value)),
        Some(MetaValue::Integer(value)) => Ok(Some(*value as f64)),
        Some(MetaValue::List(list)) if list.len() == 1 => match &list[0] {
            MetaValue::Number(value) => Ok(Some(*value)),
            MetaValue::Integer(value) => Ok(Some(*value as f64)),
            _ => Err(format!("meta sleutel `{key}` bevat geen numerieke waarde")),
        },
        Some(MetaValue::Boolean(_)) | Some(MetaValue::Text(_)) | Some(MetaValue::List(_)) => {
            Err(format!("meta sleutel `{key}` bevat geen numerieke waarde"))
        }
        None => Ok(None),
    }
}

fn required_meta_number(meta: &MetaMap, key: &str) -> Result<f64, String> {
    meta_number(meta, key)?.ok_or_else(|| format!("meta sleutel `{key}` ontbreekt voor slider"))
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}

fn to_js_error<E: fmt::Display>(error: E) -> JsValue {
    js_error(&error.to_string())
}

fn js_error(message: &str) -> JsValue {
    #[cfg(target_arch = "wasm32")]
    {
        JsError::new(message).into()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = message;
        JsValue::NULL
    }
}

impl Engine {
    /// Interne methode om input waarden bij te werken (niet blootgesteld via WASM).
    pub fn update_input_value(&mut self, id_or_name: &str, value: Value) -> Result<(), String> {
        let index = match self.find_input_index(id_or_name) {
            Some(index) => index,
            None => return Err("onbekende inputreferentie".to_string()),
        };

        let graph = match self.graph.as_mut() {
            Some(graph) => graph,
            None => return Err("er is geen GHX-bestand geladen".to_string()),
        };

        let binding = self.input_bindings[index].clone();
        let node = match graph.node_mut(binding.node_id) {
            Some(node) => node,
            None => return Err("interne inputreferentie is ongeldig".to_string()),
        };

        match binding.kind {
            InputKind::Slider => {
                let value_f64 = match value {
                    Value::Number(n) => n,
                    _ => return Err("sliderwaarde moet een getal zijn".to_string()),
                };

                if !value_f64.is_finite() {
                    return Err("sliderwaarde moet een eindig getal zijn".to_string());
                }

                let min = meta_number(&node.meta, "min")?;
                let max = meta_number(&node.meta, "max")?;
                let step = meta_number(&node.meta, "step")?;

                let mut clamped = clamp(
                    value_f64,
                    min.unwrap_or(f64::NEG_INFINITY),
                    max.unwrap_or(f64::INFINITY),
                );

                if let Some(step) = step.filter(|s| *s > 0.0) {
                    if let Some(min) = min {
                        clamped = min + ((clamped - min) / step).round() * step;
                    }
                    clamped = clamp(
                        clamped,
                        min.unwrap_or(f64::NEG_INFINITY),
                        max.unwrap_or(f64::INFINITY),
                    );
                }

                node.insert_meta("value", clamped);
                node.set_output(binding.output_pin, Value::Number(clamped));
            }
            InputKind::Toggle => {
                let value_bool = match value {
                    Value::Boolean(b) => b,
                    _ => return Err("togglewaarde moet een boolean zijn".to_string()),
                };

                node.insert_meta("Value", value_bool);
                node.set_output(binding.output_pin, Value::Boolean(value_bool));
            }
        }

        self.dirty_nodes.insert(binding.node_id);
        self.result_dirty = true;
        Ok(())
    }

    fn find_input_index(&self, id_or_name: &str) -> Option<usize> {
        let trimmed = id_or_name.trim();
        if trimmed.is_empty() {
            return None;
        }

        let normalized = normalize_name(trimmed);

        self.input_bindings
            .iter()
            .enumerate()
            .find_map(|(idx, binding)| {
                if binding.id == trimmed {
                    Some(idx)
                } else if binding.search_keys.iter().any(|key| key == &normalized) {
                    Some(idx)
                } else {
                    None
                }
            })
    }
}
