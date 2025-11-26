//! Implements parameter components for primitive types.

use std::collections::BTreeMap;

use crate::components::{Component, ComponentError, ComponentResult};
use crate::graph::node::MetaMap;
use crate::graph::value::{Domain, Value, ValueKind};

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

// --- ComponentKind Enum ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentKind {
    Integer,
    Number,
    Text,
    Boolean,
    Domain,
    Domain2,
    Complex,
    Time,
    Color,
    Matrix,
    FilePath,
    DataPath,
    Guid,
    // Placeholders
    Shader,
    SymbolDisplay,
    Constant,
    Culture,
    Data,
    Receiver,
}

impl ComponentKind {
    pub fn evaluate(self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Integer => IntegerComponent.evaluate(inputs, meta),
            Self::Number => NumberComponent.evaluate(inputs, meta),
            Self::Text => TextComponent.evaluate(inputs, meta),
            Self::Boolean => BooleanComponent.evaluate(inputs, meta),
            Self::Domain => DomainComponent.evaluate(inputs, meta),
            Self::Domain2 => Domain2Component.evaluate(inputs, meta),
            Self::Complex => ComplexComponent.evaluate(inputs, meta),
            Self::Time => TimeComponent.evaluate(inputs, meta),
            Self::Color => ColorComponent.evaluate(inputs, meta),
            Self::Matrix => MatrixComponent.evaluate(inputs, meta),
            Self::FilePath => FilePathComponent.evaluate(inputs, meta),
            Self::DataPath => DataPathComponent.evaluate(inputs, meta),
            Self::Guid => GuidComponent.evaluate(inputs, meta),
            // Placeholders
            _ => Err(ComponentError::NotYetImplemented(self.name().to_string())),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Integer => "Integer",
            Self::Number => "Number",
            Self::Text => "Text",
            Self::Boolean => "Boolean",
            Self::Domain => "Domain",
            Self::Domain2 => "Domain²",
            Self::Complex => "Complex",
            Self::Time => "Time",
            Self::Color => "Colour",
            Self::Matrix => "Matrix",
            Self::FilePath => "File Path",
            Self::DataPath => "Data Path",
            Self::Guid => "Guid",
            Self::Shader => "Shader",
            Self::SymbolDisplay => "Symbol Display",
            Self::Constant => "Constant",
            Self::Culture => "Culture",
            Self::Data => "Data",
            Self::Receiver => "Receiver",
        }
    }
}

// A macro to define a parameter component that passes through a specific `Value` type.
macro_rules! define_param_component {
    (
        $struct_name:ident,
        $output_pin:expr,
        $expected_kind:path
    ) => {
        #[derive(Debug, Default, Clone, Copy)]
        struct $struct_name;

        impl Component for $struct_name {
            fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
                if inputs.is_empty() {
                    let mut outputs = BTreeMap::new();
                    outputs.insert($output_pin.to_owned(), Value::Null);
                    return Ok(outputs);
                }

                let input_value = &inputs[0];

                let is_valid = match input_value {
                    Value::List(items) => items
                        .iter()
                        .all(|item| item.kind() == $expected_kind || matches!(item, Value::Null)),
                    value => value.kind() == $expected_kind || matches!(value, Value::Null),
                };

                if !is_valid {
                    return Err(ComponentError::new(format!(
                        "Expected {} or a List of {}, but got {}.",
                        $expected_kind,
                        $expected_kind,
                        input_value.kind()
                    )));
                }

                let mut outputs = BTreeMap::new();
                outputs.insert($output_pin.to_owned(), input_value.clone());
                Ok(outputs)
            }
        }
    };
}

// --- Implemented Components ---
define_param_component!(IntegerComponent, "Int", ValueKind::Number);
define_param_component!(NumberComponent, "Num", ValueKind::Number);
define_param_component!(TextComponent, "Txt", ValueKind::Text);
define_param_component!(BooleanComponent, "Bool", ValueKind::Boolean);
define_param_component!(ComplexComponent, "C", ValueKind::Complex);
define_param_component!(TimeComponent, "Time", ValueKind::DateTime);
define_param_component!(ColorComponent, "Col", ValueKind::Color);
define_param_component!(MatrixComponent, "Matrix", ValueKind::Matrix);
define_param_component!(FilePathComponent, "Path", ValueKind::Text);
define_param_component!(DataPathComponent, "Path", ValueKind::Text);
define_param_component!(GuidComponent, "ID", ValueKind::Text);

#[derive(Debug, Default, Clone, Copy)]
struct DomainComponent;

impl Component for DomainComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            let mut outputs = BTreeMap::new();
            outputs.insert("Domain".to_owned(), Value::Null);
            return Ok(outputs);
        }

        let input_value = &inputs[0];

        let is_valid = match input_value {
            Value::List(items) => items.iter().all(|item| {
                matches!(item, Value::Domain(Domain::One(_))) || matches!(item, Value::Null)
            }),
            Value::Domain(Domain::One(_)) => true,
            Value::Null => true,
            _ => false,
        };

        if !is_valid {
            return Err(ComponentError::new(format!(
                "Expected Domain or a List of Domain, but got {}.",
                input_value.kind()
            )));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("Domain".to_owned(), input_value.clone());
        Ok(outputs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Domain2Component;

impl Component for Domain2Component {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            let mut outputs = BTreeMap::new();
            outputs.insert("Domain²".to_owned(), Value::Null);
            return Ok(outputs);
        }

        let input_value = &inputs[0];

        let is_valid = match input_value {
            Value::List(items) => items.iter().all(|item| {
                matches!(item, Value::Domain(Domain::Two(_))) || matches!(item, Value::Null)
            }),
            Value::Domain(Domain::Two(_)) => true,
            Value::Null => true,
            _ => false,
        };

        if !is_valid {
            return Err(ComponentError::new(format!(
                "Expected Domain² or a List of Domain², but got {}.",
                input_value.kind()
            )));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("Domain²".to_owned(), input_value.clone());
        Ok(outputs)
    }
}

// --- Registrations ---
pub const REGISTRATIONS: &[Registration<ComponentKind>] = &[
    // Implemented
    Registration::new(
        ComponentKind::Integer,
        &["2e3ab970-8545-46bb-836c-1c11e5610bce"],
        &["Integer", "Int"],
    ),
    Registration::new(
        ComponentKind::Number,
        &["3e8ca6be-fda8-4aaf-b5c0-3c54c8bb7312"],
        &["Number", "Num"],
    ),
    Registration::new(
        ComponentKind::Text,
        &["3ede854e-c753-40eb-84cb-b48008f14fd4"],
        &["Text", "Txt"],
    ),
    Registration::new(
        ComponentKind::Boolean,
        &["cb95db89-6165-43b6-9c41-5702bc5bf137"],
        &["Boolean", "Bool"],
    ),
    Registration::new(
        ComponentKind::Domain,
        &["15b7afe5-d0d0-43e1-b894-34fcfe3be384"],
        &["Domain"],
    ),
    Registration::new(
        ComponentKind::Domain2,
        &[
            "90744326-eb53-4a0e-b7ef-4b45f5473d6e",
            "fa36c19d-b108-440c-b33d-a0a4642b45cc",
        ],
        &["Domain²"],
    ),
    Registration::new(
        ComponentKind::Complex,
        &["476c0cf8-bc3c-4f1c-a61a-6e91e1f8b91e"],
        &["Complex", "C"],
    ),
    Registration::new(
        ComponentKind::Time,
        &["81dfff08-0c83-4f1b-a358-14791d642d9e"],
        &["Time"],
    ),
    Registration::new(
        ComponentKind::Color,
        &["203a91c3-287a-43b6-a9c5-ebb96240a650"],
        &["Colour", "Col"],
    ),
    Registration::new(
        ComponentKind::Matrix,
        &["bd4a8a18-a3cc-40ba-965b-3be91fee563b"],
        &["Matrix"],
    ),
    Registration::new(
        ComponentKind::FilePath,
        &["06953bda-1d37-4d58-9b38-4b3c74e54c8f"],
        &["File Path", "Path"],
    ),
    Registration::new(
        ComponentKind::DataPath,
        &["56c9c942-791f-4eeb-a4f0-82b93f1c0909"],
        &["Data Path", "Path"],
    ),
    Registration::new(
        ComponentKind::Guid,
        &["faf6e3bb-4c84-4cbf-bd88-6d6a0db5667a"],
        &["Guid", "ID"],
    ),
    // Placeholders
    Registration::new(
        ComponentKind::Shader,
        &["288cfe66-f3dc-4c9a-bb96-ef81f47fe724"],
        &["Shader"],
    ),
    Registration::new(
        ComponentKind::SymbolDisplay,
        &["2bcd153c-c964-4199-b8e4-4a19dfd34967"],
        &["Symbol Display", "SymDis"],
    ),
    Registration::new(
        ComponentKind::Constant,
        &["4ad6703b-84cd-4957-a1b3-f1c6ec270d9c"],
        &["Constant", "constant"],
    ),
    Registration::new(
        ComponentKind::Culture,
        &["7fa15783-70da-485c-98c0-a099e6988c3e"],
        &["Culture"],
    ),
    Registration::new(
        ComponentKind::Data,
        &[
            "8ec86459-bf01-4409-baee-174d0d2b13d0",
            "4018985c-f9e8-4a8f-8d4d-518aec276f60",
        ],
        &["Data"],
    ),
    Registration::new(
        ComponentKind::Receiver,
        &["f19b8c33-dff2-4cc2-b95b-b4005ff3c10c"],
        &["Receiver"],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::MetaMap;
    use crate::graph::value::{
        ColorValue, ComplexValue, DateTimeValue, Domain, Domain1D, Domain2D, Matrix,
    };
    use time::macros::datetime;

    #[test]
    fn test_integer_param() {
        let component = IntegerComponent;
        let value = Value::Number(42.0);
        let inputs = vec![value.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Int"), Some(&value));
    }

    #[test]
    fn test_number_param() {
        let component = NumberComponent;
        let value = Value::Number(42.0);
        let inputs = vec![value.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Num"), Some(&value));
    }

    #[test]
    fn test_text_param() {
        let component = TextComponent;
        let value = Value::Text("hello".to_string());
        let inputs = vec![value.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Txt"), Some(&value));
    }

    #[test]
    fn test_boolean_param() {
        let component = BooleanComponent;
        let value = Value::Boolean(true);
        let inputs = vec![value.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Bool"), Some(&value));
    }

    #[test]
    fn test_domain_param() {
        let component = DomainComponent;
        let value = Value::Domain(Domain::One(Domain1D {
            start: 0.0,
            end: 1.0,
            min: 0.0,
            max: 1.0,
            span: 1.0,
            length: 1.0,
            center: 0.5,
        }));
        let inputs = vec![value.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Domain"), Some(&value));

        let domain2 = Value::Domain(Domain::Two(Domain2D {
            u: Domain1D {
                start: 0.0,
                end: 1.0,
                min: 0.0,
                max: 1.0,
                span: 1.0,
                length: 1.0,
                center: 0.5,
            },
            v: Domain1D {
                start: 0.0,
                end: 1.0,
                min: 0.0,
                max: 1.0,
                span: 1.0,
                length: 1.0,
                center: 0.5,
            },
        }));
        let inputs2 = vec![domain2.clone()];
        assert!(component.evaluate(&inputs2, &MetaMap::new()).is_err());
    }

    #[test]
    fn test_domain2_param() {
        let component = Domain2Component;
        let value = Value::Domain(Domain::Two(Domain2D {
            u: Domain1D {
                start: 0.0,
                end: 1.0,
                min: 0.0,
                max: 1.0,
                span: 1.0,
                length: 1.0,
                center: 0.5,
            },
            v: Domain1D {
                start: 0.0,
                end: 1.0,
                min: 0.0,
                max: 1.0,
                span: 1.0,
                length: 1.0,
                center: 0.5,
            },
        }));
        let inputs = vec![value.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Domain²"), Some(&value));

        let domain1 = Value::Domain(Domain::One(Domain1D {
            start: 0.0,
            end: 1.0,
            min: 0.0,
            max: 1.0,
            span: 1.0,
            length: 1.0,
            center: 0.5,
        }));
        let inputs2 = vec![domain1.clone()];
        assert!(component.evaluate(&inputs2, &MetaMap::new()).is_err());
    }

    #[test]
    fn test_complex_param() {
        let component = ComplexComponent;
        let value = Value::Complex(ComplexValue::new(1.0, 2.0));
        let inputs = vec![value.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("C"), Some(&value));
    }

    #[test]
    fn test_time_param() {
        let component = TimeComponent;
        let value = Value::DateTime(DateTimeValue::from_primitive(datetime!(2024-01-01 0:00:00)));
        let inputs = vec![value.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Time"), Some(&value));
    }

    #[test]
    fn test_color_param() {
        let component = ColorComponent;
        let value = Value::Color(ColorValue::new(1.0, 0.5, 0.0));
        let inputs = vec![value.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Col"), Some(&value));
    }

    #[test]
    fn test_matrix_param() {
        let component = MatrixComponent;
        let value = Value::Matrix(Matrix::new(1, 1, vec![1.0]).unwrap());
        let inputs = vec![value.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Matrix"), Some(&value));
    }

    #[test]
    fn test_file_path_param() {
        let component = FilePathComponent;
        let value = Value::Text("C:\\file.txt".to_string());
        let inputs = vec![value.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Path"), Some(&value));
    }

    #[test]
    fn test_placeholder_component() {
        let kind = ComponentKind::Shader;
        let err = kind.evaluate(&[], &MetaMap::new()).unwrap_err();
        assert!(matches!(err, ComponentError::NotYetImplemented(_)));
    }
}
