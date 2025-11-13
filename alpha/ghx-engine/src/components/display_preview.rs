//! Componenten voor weergave en preview in de GHX-engine.

use super::{Component, ComponentError, ComponentResult};
use crate::components::vector_point::parse_color_value;
use crate::graph::node::MetaMap;
use crate::graph::value::{ColorValue, MaterialValue, SymbolValue, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    CloudDisplay,
    CustomPreview,
    SymbolDisplay,
    DotDisplay,
    CreateMaterial,
    SymbolSimple,
    SymbolAdvanced,
}

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::CloudDisplay => cloud_display(inputs, meta),
            Self::CustomPreview => custom_preview(inputs, meta),
            Self::SymbolDisplay => symbol_display(inputs, meta),
            Self::DotDisplay => dot_display(inputs, meta),
            Self::CreateMaterial => create_material(inputs, meta),
            Self::SymbolSimple => symbol_simple(inputs, meta),
            Self::SymbolAdvanced => symbol_advanced(inputs, meta),
        }
    }
}

impl ComponentKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::CloudDisplay => "Cloud Display",
            Self::CustomPreview => "Custom Preview",
            Self::SymbolDisplay => "Symbol Display",
            Self::DotDisplay => "Dot Display",
            Self::CreateMaterial => "Create Material",
            Self::SymbolSimple => "Symbol (Simple)",
            Self::SymbolAdvanced => "Symbol (Advanced)",
        }
    }
}

fn cloud_display(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Expected 3 inputs: Points, Colours, Size",
        ));
    }
    let points = collect_points(&inputs[0])?;
    let colors = collect_colors(&inputs[1])?;
    let sizes = collect_numbers(&inputs[2])?;

    let mut tags = Vec::new();
    for i in 0..points.len() {
        let point = points[i];
        let color = colors
            .get(i)
            .cloned()
            .unwrap_or_else(|| ColorValue::from_rgb255(0.0, 0.0, 0.0));
        let size = sizes.get(i).cloned().unwrap_or(1.0);

        let tag = crate::graph::value::TextTagValue {
            plane: crate::graph::value::PlaneValue {
                origin: point,
                x_axis: [1.0, 0.0, 0.0],
                y_axis: [0.0, 1.0, 0.0],
                z_axis: [0.0, 0.0, 1.0],
            },
            text: "cloud".to_string(),
            size,
            color: Some(color),
        };
        tags.push(Value::Tag(tag));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert("Tags".to_string(), Value::List(tags));
    Ok(outputs)
}

fn custom_preview(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Expected 2 inputs: Geometry, Material"));
    }
    let geometry = inputs[0].clone();
    let material = coerce_material(&inputs[1])?;

    let mut outputs = BTreeMap::new();
    outputs.insert("Geometry".to_string(), geometry);
    outputs.insert("Material".to_string(), Value::Material(material));
    Ok(outputs)
}

fn symbol_display(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Expected 2 inputs: Location, Display"));
    }
    let location = inputs[0].clone();
    let symbol = coerce_symbol(&inputs[1])?;

    let mut outputs = BTreeMap::new();
    outputs.insert("Location".to_string(), location);
    outputs.insert("Symbol".to_string(), Value::Symbol(symbol));
    Ok(outputs)
}

fn dot_display(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Expected 3 inputs: Point, Colour, Size",
        ));
    }
    let points = collect_points(&inputs[0])?;
    let colors = collect_colors(&inputs[1])?;
    let sizes = collect_numbers(&inputs[2])?;

    let mut tags = Vec::new();
    for i in 0..points.len() {
        let point = points[i];
        let color = colors
            .get(i)
            .cloned()
            .unwrap_or_else(|| ColorValue::from_rgb255(0.0, 0.0, 0.0));
        let size = sizes.get(i).cloned().unwrap_or(1.0);

        let tag = crate::graph::value::TextTagValue {
            plane: crate::graph::value::PlaneValue {
                origin: point,
                x_axis: [1.0, 0.0, 0.0],
                y_axis: [0.0, 1.0, 0.0],
                z_axis: [0.0, 0.0, 1.0],
            },
            text: "".to_string(),
            size,
            color: Some(color),
        };
        tags.push(Value::Tag(tag));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert("Tags".to_string(), Value::List(tags));
    Ok(outputs)
}

fn create_material(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 5 {
        return Err(ComponentError::new(
            "Expected 5 inputs: Diffuse, Specular, Emission, Transparency, Shine",
        ));
    }
    let diffuse = coerce_color(&inputs[0])?;
    let specular = coerce_color(&inputs[1])?;
    let emission = coerce_color(&inputs[2])?;
    let transparency = coerce_number(&inputs[3])?;
    let shine = coerce_number(&inputs[4])?;

    let material = MaterialValue {
        diffuse,
        specular,
        emission,
        transparency,
        shine,
    };

    let mut outputs = BTreeMap::new();
    outputs.insert("M".to_string(), Value::Material(material));
    Ok(outputs)
}

fn symbol_simple(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Expected 4 inputs: Style, Size, Rotation, Colour",
        ));
    }
    let style = coerce_text(&inputs[0])?;
    let size = coerce_number(&inputs[1])?;
    let rotation = coerce_number(&inputs[2])?;
    let color = coerce_color(&inputs[3])?;

    let symbol = SymbolValue {
        style,
        size_primary: size,
        size_secondary: None,
        rotation,
        fill: color,
        edge: None,
        width: 1.0,
        adjust: false,
    };

    let mut outputs = BTreeMap::new();
    outputs.insert("D".to_string(), Value::Symbol(symbol));
    Ok(outputs)
}

fn symbol_advanced(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 8 {
        return Err(ComponentError::new(
            "Expected 8 inputs: Style, Size Primary, Size Secondary, Rotation, Fill, Edge, Width, Adjust",
        ));
    }
    let style = coerce_text(&inputs[0])?;
    let size_primary = coerce_number(&inputs[1])?;
    let size_secondary = Some(coerce_number(&inputs[2])?);
    let rotation = coerce_number(&inputs[3])?;
    let fill = coerce_color(&inputs[4])?;
    let edge = Some(coerce_color(&inputs[5])?);
    let width = coerce_number(&inputs[6])?;
    let adjust = coerce_boolean(&inputs[7])?;

    let symbol = SymbolValue {
        style,
        size_primary,
        size_secondary,
        rotation,
        fill,
        edge,
        width,
        adjust,
    };

    let mut outputs = BTreeMap::new();
    outputs.insert("D".to_string(), Value::Symbol(symbol));
    Ok(outputs)
}

fn coerce_number(value: &Value) -> Result<f64, ComponentError> {
    match value {
        Value::Number(n) => Ok(*n),
        other => Err(ComponentError::new(format!(
            "Expected a number, got {}",
            other.kind()
        ))),
    }
}

fn coerce_color(value: &Value) -> Result<ColorValue, ComponentError> {
    match value {
        Value::Color(c) => Ok(*c),
        other => parse_color_value(other)
            .ok_or_else(|| ComponentError::new(format!("Expected a color, got {}", other.kind()))),
    }
}

fn coerce_text(value: &Value) -> Result<String, ComponentError> {
    match value {
        Value::Text(t) => Ok(t.clone()),
        other => Err(ComponentError::new(format!(
            "Expected text, got {}",
            other.kind()
        ))),
    }
}

fn coerce_boolean(value: &Value) -> Result<bool, ComponentError> {
    match value {
        Value::Boolean(b) => Ok(*b),
        other => Err(ComponentError::new(format!(
            "Expected a boolean, got {}",
            other.kind()
        ))),
    }
}

pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["059b72b0-9bb3-4542-a805-2dcd27493164"],
        names: &["Cloud Display", "Cloud"],
        kind: ComponentKind::CloudDisplay,
    },
    Registration {
        guids: &["537b0419-bbc2-4ff4-bf08-afe526367b2c"],
        names: &["Custom Preview", "Preview"],
        kind: ComponentKind::CustomPreview,
    },
    Registration {
        guids: &["62d5ead4-53c4-4d0b-b5ce-6bd6e0850ab8"],
        names: &["Symbol Display", "Symbol"],
        kind: ComponentKind::SymbolDisplay,
    },
    Registration {
        guids: &["6b1bd8b2-47a4-4aa6-a471-3fd91c62a486"],
        names: &["Dot Display", "Dots"],
        kind: ComponentKind::DotDisplay,
    },
    Registration {
        guids: &["76975309-75a6-446a-afed-f8653720a9f2"],
        names: &["Create Material", "Material"],
        kind: ComponentKind::CreateMaterial,
    },
    Registration {
        guids: &["79747717-1874-4c34-b790-faef53b50569"],
        names: &["Symbol (Simple)", "SymSim"],
        kind: ComponentKind::SymbolSimple,
    },
    Registration {
        guids: &["e5c82975-8011-412c-b56d-bb7fc9e7f28d"],
        names: &["Symbol (Advanced)", "SymAdv"],
        kind: ComponentKind::SymbolAdvanced,
    },
];

#[cfg(test)]
mod tests {
    use super::{Component, ComponentKind};
    use crate::graph::{
        node::MetaMap,
        value::{ColorValue, Value},
    };

    #[test]
    fn test_dot_display_success() {
        let component = ComponentKind::DotDisplay;
        let inputs = &[
            Value::List(vec![Value::Point([1.0, 2.0, 3.0])]),
            Value::List(vec![Value::Color(ColorValue::from_rgb255(255.0, 0.0, 0.0))]),
            Value::List(vec![Value::Number(2.0)]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert!(matches!(outputs.get("Tags"), Some(Value::List(_))));
    }

    #[test]
    fn test_cloud_display_success() {
        let component = ComponentKind::CloudDisplay;
        let inputs = &[
            Value::List(vec![Value::Point([1.0, 2.0, 3.0])]),
            Value::List(vec![Value::Color(ColorValue::from_rgb255(255.0, 0.0, 0.0))]),
            Value::List(vec![Value::Number(2.0)]),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert!(matches!(outputs.get("Tags"), Some(Value::List(_))));
    }

    #[test]
    fn test_custom_preview_success() {
        let component = ComponentKind::CustomPreview;
        let inputs = &[
            Value::Point([1.0, 2.0, 3.0]),
            Value::Material(crate::graph::value::MaterialValue {
                diffuse: ColorValue::from_rgb255(255.0, 0.0, 0.0),
                specular: ColorValue::from_rgb255(255.0, 255.0, 255.0),
                emission: ColorValue::from_rgb255(0.0, 0.0, 0.0),
                transparency: 0.0,
                shine: 100.0,
            }),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert!(outputs.contains_key("Geometry"));
        assert!(outputs.contains_key("Material"));
    }

    #[test]
    fn test_symbol_display_success() {
        let component = ComponentKind::SymbolDisplay;
        let inputs = &[
            Value::Point([1.0, 2.0, 3.0]),
            Value::Symbol(crate::graph::value::SymbolValue {
                style: "circle".to_string(),
                size_primary: 1.0,
                size_secondary: None,
                rotation: 0.0,
                fill: ColorValue::from_rgb255(255.0, 0.0, 0.0),
                edge: None,
                width: 1.0,
                adjust: false,
            }),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert!(outputs.contains_key("Location"));
        assert!(outputs.contains_key("Symbol"));
    }

    #[test]
    fn test_create_material_success() {
        let component = ComponentKind::CreateMaterial;
        let inputs = &[
            Value::Color(ColorValue::from_rgb255(255.0, 0.0, 0.0)),
            Value::Color(ColorValue::from_rgb255(0.0, 255.0, 0.0)),
            Value::Color(ColorValue::from_rgb255(0.0, 0.0, 255.0)),
            Value::Number(0.5),
            Value::Number(50.0),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert!(matches!(outputs.get("M"), Some(Value::Material(_))));
    }

    #[test]
    fn test_create_material_accepts_text_colors() {
        let component = ComponentKind::CreateMaterial;
        let inputs = &[
            Value::Text("255;255;255".to_string()),
            Value::Text("128,128,128".to_string()),
            Value::Text("#0000ff".to_string()),
            Value::Number(0.25),
            Value::Number(25.0),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        match outputs.get("M") {
            Some(Value::Material(material)) => {
                assert!((material.transparency - 0.25).abs() < f64::EPSILON);
            }
            other => panic!("expected material output, got {other:?}"),
        }
    }

    #[test]
    fn test_create_material_wrong_input_type() {
        let component = ComponentKind::CreateMaterial;
        let inputs = &[
            Value::Null,
            Value::Color(ColorValue::from_rgb255(0.0, 255.0, 0.0)),
            Value::Color(ColorValue::from_rgb255(0.0, 0.0, 255.0)),
            Value::Number(0.5),
            Value::Number(50.0),
        ];
        let err = component.evaluate(inputs, &MetaMap::new()).unwrap_err();
        assert!(err.message().contains("Expected a color"));
    }

    #[test]
    fn test_symbol_simple_success() {
        let component = ComponentKind::SymbolSimple;
        let inputs = &[
            Value::Text("Circle".to_string()),
            Value::Number(10.0),
            Value::Number(45.0),
            Value::Color(ColorValue::from_rgb255(255.0, 0.0, 0.0)),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert!(matches!(outputs.get("D"), Some(Value::Symbol(_))));
    }

    #[test]
    fn test_symbol_advanced_success() {
        let component = ComponentKind::SymbolAdvanced;
        let inputs = &[
            Value::Text("Square".to_string()),
            Value::Number(10.0),
            Value::Number(5.0),
            Value::Number(45.0),
            Value::Color(ColorValue::from_rgb255(255.0, 0.0, 0.0)),
            Value::Color(ColorValue::from_rgb255(0.0, 255.0, 0.0)),
            Value::Number(1.0),
            Value::Boolean(true),
        ];
        let outputs = component.evaluate(inputs, &MetaMap::new()).unwrap();
        assert!(matches!(outputs.get("D"), Some(Value::Symbol(_))));
    }
}

fn coerce_symbol(value: &Value) -> Result<SymbolValue, ComponentError> {
    match value {
        Value::Symbol(s) => Ok(s.clone()),
        other => Err(ComponentError::new(format!(
            "Expected a symbol, got {}",
            other.kind()
        ))),
    }
}

fn coerce_material(value: &Value) -> Result<MaterialValue, ComponentError> {
    match value {
        Value::Material(m) => Ok(*m),
        other => Err(ComponentError::new(format!(
            "Expected a material, got {}",
            other.kind()
        ))),
    }
}

fn collect_points(value: &Value) -> Result<Vec<[f64; 3]>, ComponentError> {
    let mut points = Vec::new();
    collect_points_into(value, &mut points)?;
    Ok(points)
}

fn collect_points_into(value: &Value, output: &mut Vec<[f64; 3]>) -> Result<(), ComponentError> {
    match value {
        Value::Point(p) => {
            output.push(*p);
            Ok(())
        }
        Value::List(values) => {
            for value in values {
                collect_points_into(value, output)?;
            }
            Ok(())
        }
        _ => Err(ComponentError::new(format!(
            "Expected a point, got {}",
            value.kind()
        ))),
    }
}

fn collect_colors(value: &Value) -> Result<Vec<ColorValue>, ComponentError> {
    let mut colors = Vec::new();
    collect_colors_into(value, &mut colors)?;
    Ok(colors)
}

fn collect_colors_into(value: &Value, output: &mut Vec<ColorValue>) -> Result<(), ComponentError> {
    match value {
        Value::Color(c) => {
            output.push(*c);
            Ok(())
        }
        Value::List(values) => {
            for value in values {
                collect_colors_into(value, output)?;
            }
            Ok(())
        }
        _ => Err(ComponentError::new(format!(
            "Expected a color, got {}",
            value.kind()
        ))),
    }
}

fn collect_numbers(value: &Value) -> Result<Vec<f64>, ComponentError> {
    let mut numbers = Vec::new();
    collect_numbers_into(value, &mut numbers)?;
    Ok(numbers)
}

fn collect_numbers_into(value: &Value, output: &mut Vec<f64>) -> Result<(), ComponentError> {
    match value {
        Value::Number(n) => {
            output.push(*n);
            Ok(())
        }
        Value::List(values) => {
            for value in values {
                collect_numbers_into(value, output)?;
            }
            Ok(())
        }
        _ => Err(ComponentError::new(format!(
            "Expected a number, got {}",
            value.kind()
        ))),
    }
}
