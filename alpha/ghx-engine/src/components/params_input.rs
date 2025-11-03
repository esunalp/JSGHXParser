//! Grasshopper Input Parameter Components

use crate::graph::node::{MetaMap, MetaValue};
use crate::graph::value::{Value, ValueKind};
use super::{coerce, Component, ComponentError, ComponentResult};
use std::collections::BTreeMap;
use std::fmt;

/// Defines a component's registration information.
pub struct Registration<T> {
    /// The component's kind.
    pub kind: T,
    /// A list of GUIDs that identify the component.
    pub guids: &'static [&'static str],
    /// A list of names and nicknames for the component.
    pub names: &'static [&'static str],
}

impl<T: Copy> Registration<T> {
    /// Creates a new `Registration` instance.
    pub const fn new(
        kind: T,
        guids: &'static [&'static str],
        names: &'static [&'static str],
    ) -> Self {
        Self { kind, guids, names }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentKind {
    ValueList,
    BooleanToggle,
    MDSlider,
    ColourPicker,
    DigitScroller,
    ColourWheel,
    NumberSlider,
    Panel,
    Gradient,
    ColourSwatch,
    Button,
    Calendar,
    GraphMapper,
    ControlKnob,
    Clock,
    ImageResource,
    Import3DM,
    ImportPDB,
    ReadFile,
    AtomData,
    ImportSHP,
    ImportCoordinates,
    ImportImage,
    ObjectDetails,
    ImageSampler,
    FlagFields,
    RobotsLibrary,
}

impl ComponentKind {
    pub fn evaluate(self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::ValueList => ValueListComponent.evaluate(inputs, meta),
            Self::BooleanToggle => BooleanToggleComponent.evaluate(inputs, meta),
            Self::MDSlider => MDSliderComponent.evaluate(inputs, meta),
            Self::ColourPicker => ColourPickerComponent.evaluate(inputs, meta),
            Self::DigitScroller => DigitScrollerComponent.evaluate(inputs, meta),
            Self::ColourWheel => ColourWheelComponent.evaluate(inputs, meta),
            Self::NumberSlider => NumberSliderComponent.evaluate(inputs, meta),
            Self::Panel => PanelComponent.evaluate(inputs, meta),
            Self::Gradient => GradientComponent.evaluate(inputs, meta),
            Self::ColourSwatch => ColourSwatchComponent.evaluate(inputs, meta),
            Self::Button => ButtonComponent.evaluate(inputs, meta),
            Self::Calendar => CalendarComponent.evaluate(inputs, meta),
            Self::GraphMapper => GraphMapperComponent.evaluate(inputs, meta),
            Self::ControlKnob => ControlKnobComponent.evaluate(inputs, meta),
            Self::Clock => ClockComponent.evaluate(inputs, meta),
            Self::ImageResource => ImageResourceComponent.evaluate(inputs, meta),
            Self::Import3DM => Import3DMComponent.evaluate(inputs, meta),
            Self::ImportPDB => ImportPDBComponent.evaluate(inputs, meta),
            Self::ReadFile => ReadFileComponent.evaluate(inputs, meta),
            Self::AtomData => AtomDataComponent.evaluate(inputs, meta),
            Self::ImportSHP => ImportSHPComponent.evaluate(inputs, meta),
            Self::ImportCoordinates => ImportCoordinatesComponent.evaluate(inputs, meta),
            Self::ImportImage => ImportImageComponent.evaluate(inputs, meta),
            Self::ObjectDetails => ObjectDetailsComponent.evaluate(inputs, meta),
            Self::ImageSampler => ImageSamplerComponent.evaluate(inputs, meta),
            Self::FlagFields => FlagFieldsComponent.evaluate(inputs, meta),
            Self::RobotsLibrary => RobotsLibraryComponent.evaluate(inputs, meta),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::ValueList => "Value List",
            Self::BooleanToggle => "Boolean Toggle",
            Self::MDSlider => "MD Slider",
            Self::ColourPicker => "Colour Picker",
            Self::DigitScroller => "Digit Scroller",
            Self::ColourWheel => "Colour Wheel",
            Self::NumberSlider => "Number Slider",
            Self::Panel => "Panel",
            Self::Gradient => "Gradient",
            Self::ColourSwatch => "Colour Swatch",
            Self::Button => "Button",
            Self::Calendar => "Calendar",
            Self::GraphMapper => "Graph Mapper",
            Self::ControlKnob => "Control Knob",
            Self::Clock => "Clock",
            Self::ImageResource => "Image Resource",
            Self::Import3DM => "Import 3DM",
            Self::ImportPDB => "Import PDB",
            Self::ReadFile => "Read File",
            Self::AtomData => "Atom Data",
            Self::ImportSHP => "Import SHP",
            Self::ImportCoordinates => "Import Coordinates",
            Self::ImportImage => "Import Image",
            Self::ObjectDetails => "Object Details",
            Self::ImageSampler => "Image Sampler",
            Self::FlagFields => "Flag fields",
            Self::RobotsLibrary => "Robots library",
        }
    }
}

trait AsValue {
    fn as_value(&self) -> Option<Value>;
}

impl AsValue for MetaValue {
    fn as_value(&self) -> Option<Value> {
        match self {
            MetaValue::Number(n) => Some(Value::Number(*n)),
            MetaValue::Integer(i) => Some(Value::Number(*i as f64)),
            MetaValue::Boolean(b) => Some(Value::Boolean(*b)),
            MetaValue::Text(t) => Some(Value::Text(t.clone())),
            MetaValue::List(l) => {
                let values: Vec<Value> = l.iter().filter_map(|mv| mv.as_value()).collect();
                if values.len() == l.len() {
                    Some(Value::List(values))
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BooleanToggleComponent;

impl Component for BooleanToggleComponent {
    fn evaluate(&self, _inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        let val = meta.get("Value").and_then(|v| match v {
            MetaValue::Boolean(b) => Some(*b),
            _ => None
        }).unwrap_or(false);
        let mut outputs = BTreeMap::new();
        outputs.insert("Output".to_string(), Value::Boolean(val));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NumberSliderComponent;

impl Component for NumberSliderComponent {
    fn evaluate(&self, _inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        let val = meta.get("Value").and_then(|v| match v {
            MetaValue::Number(n) => Some(*n),
            MetaValue::Integer(i) => Some(*i as f64),
            _ => None
        }).unwrap_or(0.0);
        let mut outputs = BTreeMap::new();
        outputs.insert("Output".to_string(), Value::Number(val));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PanelComponent;

impl Component for PanelComponent {
    fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        let output_value = if !inputs.is_empty() && inputs.iter().any(|v| !matches!(v, Value::Null)) {
            inputs
                .iter()
                .filter(|v| !matches!(v, Value::Null))
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            meta.get("userText")
                .or_else(|| meta.get("Value"))
                .or_else(|| meta.get("text"))
                .and_then(|v| match v {
                    MetaValue::Text(t) => Some(t.clone()),
                    _ => None
                })
                .unwrap_or_default()
        };

        let mut outputs = BTreeMap::new();
        outputs.insert("Output".to_string(), Value::Text(output_value));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ValueListComponent;

impl Component for ValueListComponent {
    fn evaluate(&self, _inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        let list_items = match meta.get("ListItems").or_else(|| meta.get("Values")) {
            Some(MetaValue::List(items)) => items.iter().filter_map(|mv| mv.as_value()).collect::<Vec<Value>>(),
            _ => {
                let mut outputs = BTreeMap::new();
                outputs.insert("Output".to_string(), Value::Null);
                return Ok(outputs);
            }
        };

        let selected_index = meta.get("SelectedIndex")
            .or_else(|| meta.get("Value"))
            .and_then(|v| match v {
                MetaValue::Number(n) => Some(*n as usize),
                MetaValue::Integer(i) => Some(*i as usize),
                _ => None
            })
            .unwrap_or(0);

        let output_value = list_items
            .get(selected_index)
            .cloned()
            .unwrap_or(Value::Null);

        let mut outputs = BTreeMap::new();
        outputs.insert("Output".to_string(), output_value);
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MDSliderComponent;

impl Component for MDSliderComponent {
    fn evaluate(&self, _inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        let val = meta.get("Value")
            .and_then(|v| v.as_value())
            .unwrap_or(Value::List(vec![Value::Number(0.0), Value::Number(0.0)]));
        let mut outputs = BTreeMap::new();
        outputs.insert("Output".to_string(), val);
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ColourPickerComponent;

impl Component for ColourPickerComponent {
    fn evaluate(&self, _inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        let val = meta.get("Value")
            .and_then(|v| match v {
                MetaValue::Text(t) => Some(t.clone()),
                _ => None
            })
            .unwrap_or_else(|| "Color [A=255, R=128, G=128, B=128]".to_string());
        let mut outputs = BTreeMap::new();
        outputs.insert("Output".to_string(), Value::Text(val));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DigitScrollerComponent;

impl Component for DigitScrollerComponent {
    fn evaluate(&self, _inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        let val = meta.get("Value").and_then(|v| match v {
            MetaValue::Number(n) => Some(*n),
            MetaValue::Integer(i) => Some(*i as f64),
            _ => None
        }).unwrap_or(0.0);
        let mut outputs = BTreeMap::new();
        outputs.insert("Output".to_string(), Value::Number(val));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ColourSwatchComponent;

impl Component for ColourSwatchComponent {
     fn evaluate(&self, _inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        let val = meta.get("Value")
            .and_then(|v| match v {
                MetaValue::Text(t) => Some(t.clone()),
                _ => None
            })
            .unwrap_or_else(|| "Color [A=255, R=180, G=0, B=0]".to_string());
        let mut outputs = BTreeMap::new();
        outputs.insert("Output".to_string(), Value::Text(val));
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ButtonComponent;

impl Component for ButtonComponent {
    fn evaluate(&self, _inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        let pressed = meta.get("Pressed").and_then(|v| match v {
            MetaValue::Boolean(b) => Some(*b),
            _ => None
        }).unwrap_or(false);
        let mut outputs = BTreeMap::new();
        outputs.insert("Output".to_string(), Value::Boolean(pressed));
        Ok(outputs)
    }
}

macro_rules! define_placeholder_component {
    ($struct_name:ident, $output_pin:expr) => {
        #[derive(Debug, Default, Clone, Copy)]
        pub struct $struct_name;

        impl Component for $struct_name {
            fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
                let mut outputs = BTreeMap::new();
                outputs.insert($output_pin.to_string(), Value::Null);
                Ok(outputs)
            }
        }
    };
}

define_placeholder_component!(ColourWheelComponent, "Output");
#[derive(Debug, Default, Clone, Copy)]
pub struct GradientComponent;

impl Component for GradientComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 3 {
            return Err(ComponentError::new(
                "Not enough inputs for Gradient component.",
            ));
        }

        let l0 = coerce::coerce_number(&inputs[0]).unwrap_or(0.0);
        let l1 = coerce::coerce_number(&inputs[1]).unwrap_or(1.0);
        let t = coerce::coerce_number(&inputs[2]).unwrap_or(0.0);

        let factor = if (l1 - l0).abs() < 1e-9 {
            0.5
        } else {
            ((t - l0) / (l1 - l0)).clamp(0.0, 1.0)
        };

        // Placeholder for a real color gradient. This just interpolates between red and green.
        let r = (255.0 * (1.0 - factor)) as u8;
        let g = (255.0 * factor) as u8;
        let b = 0_u8;

        let color_string = format!("Color [A=255, R={}, G={}, B={}]", r, g, b);

        let mut outputs = BTreeMap::new();
        outputs.insert("Colour".to_string(), Value::Text(color_string));
        Ok(outputs)
    }
}
define_placeholder_component!(CalendarComponent, "Output");
define_placeholder_component!(GraphMapperComponent, "Output");
define_placeholder_component!(ControlKnobComponent, "Output");
define_placeholder_component!(ClockComponent, "Output");

macro_rules! define_not_implemented_component {
    ($struct_name:ident, $component_name:expr) => {
        #[derive(Debug, Default, Clone, Copy)]
        pub struct $struct_name;

        impl Component for $struct_name {
            fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
                Err(ComponentError::NotYetImplemented($component_name.to_string()))
            }
        }
    };
}

define_not_implemented_component!(ImageResourceComponent, "Image Resource");
define_not_implemented_component!(Import3DMComponent, "Import 3DM");
define_not_implemented_component!(ImportPDBComponent, "Import PDB");
define_not_implemented_component!(ReadFileComponent, "Read File");
define_not_implemented_component!(AtomDataComponent, "Atom Data");
define_not_implemented_component!(ImportSHPComponent, "Import SHP");
define_not_implemented_component!(ImportCoordinatesComponent, "Import Coordinates");
define_not_implemented_component!(ImportImageComponent, "Import Image");
define_not_implemented_component!(ObjectDetailsComponent, "Object Details");
define_not_implemented_component!(ImageSamplerComponent, "Image Sampler");
define_not_implemented_component!(FlagFieldsComponent, "Flag fields");
define_not_implemented_component!(RobotsLibraryComponent, "Robots library");

pub const REGISTRATIONS: &[Registration<ComponentKind>] = &[
    Registration::new(ComponentKind::ValueList, &["00027467-0d24-4fa7-b178-8dc0ac5f42ec"], &["Value List", "List"]),
    Registration::new(ComponentKind::FlagFields, &["0381b555-bf9c-4d68-8e5c-10b2fcb16f30"], &["Flag fields"]),
    Registration::new(ComponentKind::ImageResource, &["216bccd8-bf29-4d3c-b791-54c89a180db3"], &["Image Resource"]),
    Registration::new(ComponentKind::BooleanToggle, &["2e78987b-9dfb-42a2-8b76-3923ac8bd91a", "ad483f40-dc72-40dc-844d-c9e462c7d19f"], &["Boolean Toggle", "Toggle"]),
    Registration::new(ComponentKind::Import3DM, &["317f1cb2-820d-4a8f-b5c8-5de3594ddfba", "f055c5d7-5d97-4964-90c7-8e9eee9a8a39"], &["Import 3DM", "3DM"]),
    Registration::new(ComponentKind::MDSlider, &["318dacd7-9073-4ede-b043-a0c132eb77e0"], &["MD Slider"]),
    Registration::new(ComponentKind::ColourPicker, &["339c0ee1-cf11-444f-8e10-65c9150ea755"], &["Colour Picker", "Colour"]),
    Registration::new(ComponentKind::DigitScroller, &["33bcf975-a0b2-4b54-99fd-585c893b9e88"], &["Digit Scroller"]),
    Registration::new(ComponentKind::ImportPDB, &["383929c0-6515-4899-8b4b-3bd0d0b32471"], &["Import PDB", "PDB"]),
    Registration::new(ComponentKind::ColourWheel, &["51a2ede9-8f8c-4fdf-a375-999c2062eab7"], &["Colour Wheel", "Wheel"]),
    Registration::new(ComponentKind::NumberSlider, &["57da07bd-ecab-415d-9d86-af36d7073abc"], &["Number Slider"]),
    Registration::new(ComponentKind::Panel, &["59e0b89a-e487-49f8-bab8-b5bab16be14c"], &["Panel"]),
    Registration::new(ComponentKind::RobotsLibrary, &["5dd377ec-f6af-43f8-8e92-fc6669013e61"], &["Robots library"]),
    Registration::new(ComponentKind::ReadFile, &["6587fcbf-e3cf-480a-b2f5-641794474194"], &["Read File", "File"]),
    Registration::new(ComponentKind::Gradient, &["6da9f120-3ad0-4b6e-9fe0-f8cde3a649b7"], &["Gradient"]),
    Registration::new(ComponentKind::AtomData, &["7b371d04-53e3-47d8-b3dd-7b113c48bc59"], &["Atom Data", "Atom"]),
    Registration::new(ComponentKind::ColourSwatch, &["9c53bac0-ba66-40bd-8154-ce9829b9db1a"], &["Colour Swatch", "Swatch"]),
    Registration::new(ComponentKind::Button, &["a8b97322-2d53-47cd-905e-b932c3ccd74e"], &["Button"]),
    Registration::new(ComponentKind::ImportSHP, &["aa538b89-3df8-436f-9ae4-bc44525984de"], &["Import SHP", "SHP"]),
    Registration::new(ComponentKind::Calendar, &["ab898d46-b8b3-4ed5-b28f-4f8047920262"], &["Calendar"]),
    Registration::new(ComponentKind::ImportCoordinates, &["b8a66384-fc66-4574-a8a9-ad18e610d623"], &["Import Coordinates", "Coords"]),
    Registration::new(ComponentKind::GraphMapper, &["bc984576-7aa6-491f-a91d-e444c33675a7"], &["Graph Mapper", "Graph"]),
    Registration::new(ComponentKind::ControlKnob, &["bcac2747-348b-4edd-ae1f-77a782cebbdd"], &["Control Knob", "Knob"]),
    Registration::new(ComponentKind::ImportImage, &["c2c0c6cf-f362-4047-a159-21a72e7c272a"], &["Import Image", "IMG"]),
    Registration::new(ComponentKind::ObjectDetails, &["c7b5c66a-6360-4f5f-aa17-a918d0b1c314"], &["Object Details", "ObjDet"]),
    Registration::new(ComponentKind::ImageSampler, &["d69a3494-785b-4beb-969b-d2373f65abfd"], &["Image Sampler", "Image"]),
    Registration::new(ComponentKind::Clock, &["f8a94819-1e2b-4d67-9100-9e983ac493cc"], &["Clock"]),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::value::Value;

    #[test]
    fn test_boolean_toggle_component() {
        let component = BooleanToggleComponent;
        let mut meta = MetaMap::new();
        meta.insert("Value".to_string(), MetaValue::Boolean(true));
        let outputs = component.evaluate(&[], &meta).unwrap();
        assert_eq!(outputs.get("Output"), Some(&Value::Boolean(true)));
    }

    #[test]
    fn test_number_slider_component() {
        let component = NumberSliderComponent;
        let mut meta = MetaMap::new();
        meta.insert("Value".to_string(), MetaValue::Number(1.23));
        let outputs = component.evaluate(&[], &meta).unwrap();
        assert_eq!(outputs.get("Output"), Some(&Value::Number(1.23)));
    }

    #[test]
    fn test_panel_component_from_meta() {
        let component = PanelComponent;
        let mut meta = MetaMap::new();
        meta.insert("userText".to_string(), MetaValue::Text("hello".to_string()));
        let outputs = component.evaluate(&[], &meta).unwrap();
        assert_eq!(outputs.get("Output"), Some(&Value::Text("hello".to_string())));
    }

    #[test]
    fn test_panel_component_from_input() {
        let component = PanelComponent;
        let inputs = vec![Value::Text("world".to_string())];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Output"), Some(&Value::Text("world".to_string())));
    }

    #[test]
    fn test_value_list_component() {
        let component = ValueListComponent;
        let mut meta = MetaMap::new();
        let list_items = vec![MetaValue::Number(1.0), MetaValue::Text("two".to_string())];
        meta.insert("ListItems".to_string(), MetaValue::List(list_items));
        meta.insert("SelectedIndex".to_string(), MetaValue::Number(1.0));
        let outputs = component.evaluate(&[], &meta).unwrap();
        assert_eq!(outputs.get("Output"), Some(&Value::Text("two".to_string())));
    }

    #[test]
    fn test_md_slider_component() {
        let component = MDSliderComponent;
        let mut meta = MetaMap::new();
        let values = vec![MetaValue::Number(0.1), MetaValue::Number(0.9)];
        meta.insert("Value".to_string(), MetaValue::List(values.clone()));
        let outputs = component.evaluate(&[], &meta).unwrap();
        assert_eq!(outputs.get("Output"), Some(&Value::List(vec![Value::Number(0.1), Value::Number(0.9)])));
    }

    #[test]
    fn test_gradient_component() {
        let component = GradientComponent;
        let inputs = vec![Value::Number(0.0), Value::Number(100.0), Value::Number(50.0)];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let expected_color = format!("Color [A=255, R=127, G=127, B=0]");
        assert_eq!(outputs.get("Colour"), Some(&Value::Text(expected_color)));
    }
}
