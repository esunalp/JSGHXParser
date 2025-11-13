#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod components;
pub mod graph;
pub mod parse;

use std::collections::BTreeMap;
use std::fmt;

use components::{ComponentKind, ComponentRegistry};
use graph::Graph;
use graph::evaluator::{self, EvaluationPlan, EvaluationResult};
use graph::node::{MetaLookupExt, MetaMap, MetaValue, NodeId};
use graph::value::Value;
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
struct SliderBinding {
    id: String,
    node_id: NodeId,
    output_pin: String,
    search_keys: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SliderExport {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    min: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    step: Option<f32>,
    value: f32,
}

#[derive(Debug, Serialize)]
struct GeometryResponse {
    items: Vec<GeometryItem>,
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

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum GeometryItem {
    Point {
        coordinates: [f32; 3],
    },
    Line {
        start: [f32; 3],
        end: [f32; 3],
    },
    Polyline {
        points: Vec<[f32; 3]>,
    },
    Mesh {
        vertices: Vec<[f32; 3]>,
        faces: Vec<Vec<u32>>,
    },
}

/// Public entry point for consumers.
#[wasm_bindgen]
pub struct Engine {
    initialized: bool,
    registry: ComponentRegistry,
    graph: Option<Graph>,
    last_result: Option<EvaluationResult>,
    slider_bindings: Vec<SliderBinding>,
    evaluation_plan: Option<EvaluationPlan>,
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
            slider_bindings: Vec::new(),
            evaluation_plan: None,
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
        let slider_bindings = collect_slider_bindings(&graph, &self.registry);
        let evaluation_plan = evaluator::EvaluationPlan::new(&graph).map_err(to_js_error)?;

        self.graph = Some(graph);
        self.slider_bindings = slider_bindings;
        self.last_result = None;
        self.evaluation_plan = Some(evaluation_plan);

        Ok(())
    }

    /// Haal slider-specificaties op voor UI-generatie.
    #[wasm_bindgen]
    pub fn get_sliders(&self) -> Result<JsValue, JsValue> {
        let graph = match self.graph.as_ref() {
            Some(graph) => graph,
            None => return Err(js_error("er is geen GHX-bestand geladen")),
        };

        let mut sliders = Vec::with_capacity(self.slider_bindings.len());
        for binding in &self.slider_bindings {
            let slider = match slider_state(graph, binding) {
                Ok(slider) => slider,
                Err(err) => return Err(js_error(&err)),
            };
            sliders.push(slider);
        }

        serde_wasm_bindgen::to_value(&sliders).map_err(|err| JsError::new(&err.to_string()).into())
    }

    /// Stel een sliderwaarde in op basis van id of naam.
    #[wasm_bindgen]
    pub fn set_slider_value(&mut self, id_or_name: &str, value: f32) -> Result<(), JsValue> {
        if !value.is_finite() {
            return Err(JsError::new("sliderwaarde moet een eindig getal zijn").into());
        }

        let slider_index = match self.find_slider_index(id_or_name) {
            Some(index) => index,
            None => return Err(js_error("onbekende sliderreferentie")),
        };

        let graph = match self.graph.as_mut() {
            Some(graph) => graph,
            None => return Err(js_error("er is geen GHX-bestand geladen")),
        };

        let binding = self.slider_bindings[slider_index].clone();
        let node = match graph.node_mut(binding.node_id) {
            Some(node) => node,
            None => return Err(js_error("interne sliderreferentie is ongeldig")),
        };

        let min = match meta_number(&node.meta, "min") {
            Ok(value) => value,
            Err(err) => return Err(js_error(&err)),
        };
        let max = match meta_number(&node.meta, "max") {
            Ok(value) => value,
            Err(err) => return Err(js_error(&err)),
        };
        let step = match meta_number(&node.meta, "step") {
            Ok(value) => value,
            Err(err) => return Err(js_error(&err)),
        };

        let mut clamped = clamp(
            value,
            min.unwrap_or(f32::NEG_INFINITY),
            max.unwrap_or(f32::INFINITY),
        );

        if let Some(step) = step.filter(|s| *s > 0.0) {
            if let Some(min) = min {
                clamped = min + ((clamped - min) / step).round() * step;
            }
            clamped = clamp(
                clamped,
                min.unwrap_or(f32::NEG_INFINITY),
                max.unwrap_or(f32::INFINITY),
            );
        }

        node.insert_meta("value", clamped);
        node.set_output(binding.output_pin, Value::Number(clamped));

        self.last_result = None;
        Ok(())
    }

    /// Evalueer de geladen graph.
    #[wasm_bindgen]
    pub fn evaluate(&mut self) -> Result<(), JsValue> {
        let graph = match self.graph.as_ref() {
            Some(graph) => graph,
            None => return Err(js_error("er is geen GHX-bestand geladen")),
        };

        let plan = self
            .evaluation_plan
            .as_ref()
            .ok_or_else(|| js_error("graph is niet voorbereid voor evaluatie"))?;

        let result =
            evaluator::evaluate_with_plan(graph, &self.registry, plan).map_err(to_js_error)?;
        self.last_result = Some(result);
        Ok(())
    }

    /// Haal de geometrie van de laatst uitgevoerde evaluatie op.
    #[wasm_bindgen]
    pub fn get_geometry(&self) -> Result<JsValue, JsValue> {
        let result = match self.last_result.as_ref() {
            Some(result) => result,
            None => return Err(js_error("graph is nog niet geëvalueerd")),
        };

        let mut items = Vec::new();
        for value in &result.geometry {
            append_geometry_items(value, &mut items);
        }

        serde_wasm_bindgen::to_value(&GeometryResponse { items })
            .map_err(|err| JsError::new(&err.to_string()).into())
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

        let result = self.last_result.as_ref();

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

fn collect_slider_bindings(graph: &Graph, registry: &ComponentRegistry) -> Vec<SliderBinding> {
    let mut bindings = Vec::new();

    for node in graph.nodes() {
        if matches!(
            registry.resolve(
                node.guid.as_deref(),
                node.name.as_deref(),
                node.nickname.as_deref()
            ),
            Some(ComponentKind::ParamsInput(
                components::params_input::ComponentKind::NumberSlider,
            ))
        ) {
            let output_pin = node
                .outputs
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "OUT".to_string());

            let mut search_keys = Vec::new();
            if let Some(name) = node.name.as_deref() {
                search_keys.push(normalize_name(name));
            }
            if let Some(nickname) = node.nickname.as_deref() {
                search_keys.push(normalize_name(nickname));
            }

            bindings.push(SliderBinding {
                id: node.id.0.to_string(),
                node_id: node.id,
                output_pin,
                search_keys,
            });
        }
    }

    bindings
}

fn slider_state(graph: &Graph, binding: &SliderBinding) -> Result<SliderExport, String> {
    let node = graph
        .node(binding.node_id)
        .ok_or_else(|| "interne sliderreferentie is ongeldig".to_owned())?;

    let name = node
        .nickname
        .as_deref()
        .or(node.name.as_deref())
        .unwrap_or(&binding.id);

    let value = required_meta_number(&node.meta, "value")?;
    let min = meta_number(&node.meta, "min")?;
    let max = meta_number(&node.meta, "max")?;
    let step = meta_number(&node.meta, "step")?;

    Ok(SliderExport {
        id: binding.id.clone(),
        name: name.to_owned(),
        min,
        max,
        step,
        value,
    })
}

fn append_geometry_items(value: &Value, items: &mut Vec<GeometryItem>) {
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
                vertices: vertices.clone(),
                faces: faces.clone(),
            });
        }
        Value::List(values) => {
            if let Some(polyline) = list_as_polyline(values) {
                items.push(GeometryItem::Polyline { points: polyline });
            } else {
                for entry in values {
                    append_geometry_items(entry, items);
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

fn list_as_polyline(values: &[Value]) -> Option<Vec<[f32; 3]>> {
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
    use super::{append_geometry_items, list_as_polyline, GeometryItem};
    use crate::graph::value::Value;

    #[test]
    fn detects_polyline_from_point_list() {
        let mut items = Vec::new();
        let list = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([1.0, 1.0, 0.0]),
        ]);

        append_geometry_items(&list, &mut items);

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

        append_geometry_items(&value, &mut items);

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

fn meta_number(meta: &MetaMap, key: &str) -> Result<Option<f32>, String> {
    match meta.get_normalized(key) {
        Some(MetaValue::Number(value)) => Ok(Some(*value)),
        Some(MetaValue::Integer(value)) => Ok(Some(*value as f32)),
        Some(MetaValue::List(list)) if list.len() == 1 => match &list[0] {
            MetaValue::Number(value) => Ok(Some(*value)),
            MetaValue::Integer(value) => Ok(Some(*value as f32)),
            _ => Err(format!("meta sleutel `{key}` bevat geen numerieke waarde")),
        },
        Some(MetaValue::Boolean(_)) | Some(MetaValue::Text(_)) | Some(MetaValue::List(_)) => {
            Err(format!("meta sleutel `{key}` bevat geen numerieke waarde"))
        }
        None => Ok(None),
    }
}

fn required_meta_number(meta: &MetaMap, key: &str) -> Result<f32, String> {
    meta_number(meta, key)?.ok_or_else(|| format!("meta sleutel `{key}` ontbreekt voor slider"))
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
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
    fn find_slider_index(&self, id_or_name: &str) -> Option<usize> {
        let trimmed = id_or_name.trim();
        if trimmed.is_empty() {
            return None;
        }

        let normalized = normalize_name(trimmed);

        self.slider_bindings
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
