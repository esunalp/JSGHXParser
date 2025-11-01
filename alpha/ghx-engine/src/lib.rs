#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod components;
pub mod graph;
pub mod parse;

use std::fmt;

use components::{ComponentKind, ComponentRegistry};
use graph::Graph;
use graph::evaluator::{self, EvaluationResult};
use graph::node::{MetaMap, MetaValue, NodeId};
use graph::value::Value;
use serde::Serialize;
use wasm_bindgen::JsError;
use wasm_bindgen::prelude::*;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "console_error_panic_hook", target_arch = "wasm32"))] {
        #[wasm_bindgen(start)]
        pub fn initialize() {
            console_error_panic_hook::set_once();
        }
    } else {
        #[wasm_bindgen(start)]
        pub fn initialize() {
            // no-op fallback when panic hook is disabled
        }
    }
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
    min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    step: Option<f64>,
    value: f64,
}

#[derive(Debug, Serialize)]
struct GeometryResponse {
    items: Vec<GeometryItem>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum GeometryItem {
    Point {
        coordinates: [f64; 3],
    },
    CurveLine {
        points: [[f64; 3]; 2],
    },
    Surface {
        vertices: Vec<[f64; 3]>,
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

        self.graph = Some(graph);
        self.slider_bindings = slider_bindings;
        self.last_result = None;

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
    pub fn set_slider_value(&mut self, id_or_name: &str, value: f64) -> Result<(), JsValue> {
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

        node.meta
            .insert("value".to_string(), MetaValue::Number(clamped));
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

        let result = evaluator::evaluate(graph, &self.registry).map_err(to_js_error)?;
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

        let mut items = Vec::with_capacity(result.geometry.len());
        for value in &result.geometry {
            let item = geometry_item_from_value(value)
                .ok_or_else(|| JsError::new("niet-renderbaar geometrisch type aangetroffen"))?;
            items.push(item);
        }

        serde_wasm_bindgen::to_value(&GeometryResponse { items })
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
            Some(ComponentKind::NumberSlider(_))
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

fn geometry_item_from_value(value: &Value) -> Option<GeometryItem> {
    match value {
        Value::Point(point) => Some(GeometryItem::Point {
            coordinates: *point,
        }),
        Value::CurveLine { p1, p2 } => Some(GeometryItem::CurveLine { points: [*p1, *p2] }),
        Value::Surface { vertices, faces } => Some(GeometryItem::Surface {
            vertices: vertices.clone(),
            faces: faces.clone(),
        }),
        Value::List(_) | Value::Number(_) | Value::Vector(_) | Value::Boolean(_) => None,
        Value::Domain(_) => None,
        Value::Matrix(_) => None,
    }
}

fn meta_number(meta: &MetaMap, key: &str) -> Result<Option<f64>, String> {
    match meta.get(key) {
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
