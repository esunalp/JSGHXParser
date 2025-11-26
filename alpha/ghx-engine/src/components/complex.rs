//! Implementaties van Grasshopper "Complex" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::{ComplexValue, Value};

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_RESULT: &str = "R";
const PIN_OUTPUT_VALUE: &str = "y";

/// Beschikbare componenten binnen Complex.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Square,
    Tangent,
    Power,
    Multiplication,
    CoTangent,
    ArcTangent,
    Exponential,
    Addition,
    SquareRoot,
    Cosine,
    ArcCosine,
    Cosecant,
    Subtraction,
    Logarithm,
    Sine,
    Division,
    Secant,
    ArcSine,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Registraties van alle Complex componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["0b0f1203-2ea8-4250-a45a-cca7ad2e5b76"],
        names: &["Square", "Sqr"],
        kind: ComponentKind::Square,
    },
    Registration {
        guids: &["0bc93049-e1a7-44b5-8068-c7ddc85a9f46"],
        names: &["Tangent", "Tan"],
        kind: ComponentKind::Tangent,
    },
    Registration {
        guids: &["2d6cb24f-da89-4fab-be0f-e5d439e0217a"],
        names: &["Power", "Pow"],
        kind: ComponentKind::Power,
    },
    Registration {
        guids: &["2f643ab6-b9a4-4923-b3da-f9d52b0cba14"],
        names: &["Multiplication", "Multiply"],
        kind: ComponentKind::Multiplication,
    },
    Registration {
        guids: &["39461433-ac44-4298-94a9-988f983e347c"],
        names: &["CoTangent", "Cotan"],
        kind: ComponentKind::CoTangent,
    },
    Registration {
        guids: &["4e8aad42-9111-470c-9acd-7ae365d8bba4"],
        names: &["ArcTangent", "ATan"],
        kind: ComponentKind::ArcTangent,
    },
    Registration {
        guids: &["582f96c6-ed0c-4710-9b5e-a05addba9f42"],
        names: &["Exponential", "Exp"],
        kind: ComponentKind::Exponential,
    },
    Registration {
        guids: &["58669268-a825-4688-8072-7d3508fcf91c"],
        names: &["Addition", "CAdd"],
        kind: ComponentKind::Addition,
    },
    Registration {
        guids: &["5a22dc1a-907c-4e2f-b8da-0e496c4e25bb"],
        names: &["Square Root", "Sqrt"],
        kind: ComponentKind::SquareRoot,
    },
    Registration {
        guids: &["7874f26c-6f76-4da8-b527-2d567184b2bd"],
        names: &["Cosine", "Cos"],
        kind: ComponentKind::Cosine,
    },
    Registration {
        guids: &["8640c519-9bf6-4e9a-a108-75f9d89b2c58"],
        names: &["ArcCosine", "ACos"],
        kind: ComponentKind::ArcCosine,
    },
    Registration {
        guids: &["99197a17-d5c7-419b-acde-eca2737f3c58"],
        names: &["Cosecant", "Cosec"],
        kind: ComponentKind::Cosecant,
    },
    Registration {
        guids: &["babecca6-9813-4146-b150-cd72f743e47c"],
        names: &["Subtraction", "Minus"],
        kind: ComponentKind::Subtraction,
    },
    Registration {
        guids: &["bc4a27fc-cbb9-4802-bd4a-17ab33ad1826"],
        names: &["Logarithm", "Ln"],
        kind: ComponentKind::Logarithm,
    },
    Registration {
        guids: &["c53932eb-7c8c-4825-ae98-e36bba97232d"],
        names: &["Sine", "Sin"],
        kind: ComponentKind::Sine,
    },
    Registration {
        guids: &["cb4ec4a1-f48e-4685-b58c-72ed27b53681"],
        names: &["Division", "Divide"],
        kind: ComponentKind::Division,
    },
    Registration {
        guids: &["d879e74c-6fe3-4cbf-b3fa-60a7c48b73e7"],
        names: &["Secant", "Sec"],
        kind: ComponentKind::Secant,
    },
    Registration {
        guids: &["f18091e9-3264-4dd4-9ba6-32c77fca0ac0"],
        names: &["ArcSine", "ASin"],
        kind: ComponentKind::ArcSine,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Square => evaluate_unary(inputs, |c| c * c, PIN_OUTPUT_VALUE),
            Self::Tangent => evaluate_unary(inputs, |c| c.tan(), PIN_OUTPUT_VALUE),
            Self::Power => evaluate_binary(inputs, |a, b| a.powc(b), PIN_OUTPUT_RESULT),
            Self::Multiplication => evaluate_binary(inputs, |a, b| a * b, PIN_OUTPUT_RESULT),
            Self::CoTangent => evaluate_unary(inputs, |c| c.tan().inv(), PIN_OUTPUT_VALUE),
            Self::ArcTangent => evaluate_unary(inputs, |c| c.atan(), PIN_OUTPUT_VALUE),
            Self::Exponential => evaluate_unary(inputs, |c| c.exp(), PIN_OUTPUT_VALUE),
            Self::Addition => evaluate_binary(inputs, |a, b| a + b, PIN_OUTPUT_RESULT),
            Self::SquareRoot => evaluate_unary(inputs, |c| c.sqrt(), PIN_OUTPUT_VALUE),
            Self::Cosine => evaluate_unary(inputs, |c| c.cos(), PIN_OUTPUT_VALUE),
            Self::ArcCosine => evaluate_unary(inputs, |c| c.acos(), PIN_OUTPUT_VALUE),
            Self::Cosecant => evaluate_unary(inputs, |c| c.sin().inv(), PIN_OUTPUT_VALUE),
            Self::Subtraction => evaluate_binary(inputs, |a, b| a - b, PIN_OUTPUT_RESULT),
            Self::Logarithm => evaluate_unary(inputs, |c| c.ln(), PIN_OUTPUT_VALUE),
            Self::Sine => evaluate_unary(inputs, |c| c.sin(), PIN_OUTPUT_VALUE),
            Self::Division => evaluate_binary(inputs, |a, b| a / b, PIN_OUTPUT_RESULT),
            Self::Secant => evaluate_unary(inputs, |c| c.cos().inv(), PIN_OUTPUT_VALUE),
            Self::ArcSine => evaluate_unary(inputs, |c| c.asin(), PIN_OUTPUT_VALUE),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Square => "Square",
            Self::Tangent => "Tangent",
            Self::Power => "Power",
            Self::Multiplication => "Multiplication",
            Self::CoTangent => "CoTangent",
            Self::ArcTangent => "ArcTangent",
            Self::Exponential => "Exponential",
            Self::Addition => "Addition",
            Self::SquareRoot => "Square Root",
            Self::Cosine => "Cosine",
            Self::ArcCosine => "ArcCosine",
            Self::Cosecant => "Cosecant",
            Self::Subtraction => "Subtraction",
            Self::Logarithm => "Logarithm",
            Self::Sine => "Sine",
            Self::Division => "Division",
            Self::Secant => "Secant",
            Self::ArcSine => "ArcSine",
        }
    }
}

fn evaluate_unary<F>(inputs: &[Value], op: F, output_pin: &str) -> ComponentResult
where
    F: Fn(ComplexValue) -> ComplexValue,
{
    if inputs.is_empty() {
        return Err(ComponentError::new("Input ontbreekt"));
    }
    let complex = coerce_complex(&inputs[0])?;
    let result = op(complex);

    let mut outputs = BTreeMap::new();
    outputs.insert(output_pin.to_owned(), Value::Complex(result));
    Ok(outputs)
}

fn evaluate_binary<F>(inputs: &[Value], op: F, output_pin: &str) -> ComponentResult
where
    F: Fn(ComplexValue, ComplexValue) -> ComplexValue,
{
    if inputs.len() < 2 {
        return Err(ComponentError::new("Twee inputs vereist"));
    }
    let a = coerce_complex(&inputs[0])?;
    let b = coerce_complex(&inputs[1])?;
    let result = op(a, b);

    let mut outputs = BTreeMap::new();
    outputs.insert(output_pin.to_owned(), Value::Complex(result));
    Ok(outputs)
}

fn coerce_complex(value: &Value) -> Result<ComplexValue, ComponentError> {
    match value {
        Value::Complex(c) => Ok(*c),
        Value::Number(n) => Ok(ComplexValue::new(*n, 0.0)),
        Value::List(list) if list.len() == 1 => coerce_complex(&list[0]),
        other => Err(ComponentError::new(format!(
            "Verwacht een complex getal, kreeg {}",
            other.kind()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentKind};
    use crate::graph::node::MetaMap;
    use crate::graph::value::{ComplexValue, Value};

    #[test]
    fn test_addition() {
        let component = ComponentKind::Addition;
        let outputs = component
            .evaluate(
                &[
                    Value::Complex(ComplexValue::new(1.0, 2.0)),
                    Value::Complex(ComplexValue::new(3.0, 4.0)),
                ],
                &MetaMap::new(),
            )
            .expect("addition");

        let result = outputs.get(super::PIN_OUTPUT_RESULT).unwrap();
        assert_eq!(
            result.expect_complex().unwrap(),
            ComplexValue::new(4.0, 6.0)
        );
    }

    #[test]
    fn test_power() {
        let component = ComponentKind::Power;
        let outputs = component
            .evaluate(
                &[
                    Value::Complex(ComplexValue::new(2.0, 0.0)),
                    Value::Complex(ComplexValue::new(3.0, 0.0)),
                ],
                &MetaMap::new(),
            )
            .expect("power");

        let result = outputs.get(super::PIN_OUTPUT_RESULT).unwrap();
        assert!((result.expect_complex().unwrap().re - 8.0).abs() < 1e-9);
    }
}
