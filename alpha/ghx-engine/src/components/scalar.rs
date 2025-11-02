//! Implementaties van Grasshopper "Scalar" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentResult};

const EPSILON: f64 = 1e-9;

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Tangent,
    Mean,
    Cosine,
    ArcSine,
    Multiplication,
    Modulus,
    Minimum,
    PowerOfE,
    MassAddition,
    ArcTangent,
    NaturalLogarithm,
    Power,
    PowerOf2,
    Truncate,
    Addition,
    ArcCosine,
    Logarithm,
    Sinc,
    Maximum,
    Division,
    Sine,
    PowerOf10,
    Subtraction,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de scalar componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["002b2feb-5d1b-41ea-913f-9f203c615792"],
        names: &[],
        kind: ComponentKind::Tangent,
    },
    Registration {
        guids: &["0bb7682f-333c-4bb7-b6fe-91ed2c886100"],
        names: &[],
        kind: ComponentKind::Mean,
    },
    Registration {
        guids: &["12278a4b-c131-4735-a3ee-bcb783083856"],
        names: &[],
        kind: ComponentKind::Cosine,
    },
    Registration {
        guids: &["22bba82d-32e8-448c-a59c-f054c8843ee3"],
        names: &[],
        kind: ComponentKind::ArcSine,
    },
    Registration {
        guids: &["3e6383e9-af39-427b-801a-19ca916160fa"],
        names: &[],
        kind: ComponentKind::Multiplication,
    },
    Registration {
        guids: &["481e1f0d-a945-4662-809d-f49d1a8f40bd", "9ebccbb4-f3e3-4ee1-af31-2f301f2516f0"],
        names: &[],
        kind: ComponentKind::Modulus,
    },
    Registration {
        guids: &["532b722d-9368-42ee-b99d-64a4732ee99a"],
        names: &[],
        kind: ComponentKind::Minimum,
    },
    Registration {
        guids: &["5f212b16-82a0-4699-be4c-11529a9810ae"],
        names: &[],
        kind: ComponentKind::PowerOfE,
    },
    Registration {
        guids: &["74d95062-0bec-4a4e-9026-5141fca954a6", "bb64b2fb-f87a-432f-86f8-393f4ee21310"],
        names: &[],
        kind: ComponentKind::MassAddition,
    },
    Registration {
        guids: &["7b312903-4782-438f-aa37-ba43f5083460"],
        names: &[],
        kind: ComponentKind::ArcTangent,
    },
    Registration {
        guids: &["8b62751f-6fb4-4d03-a238-11ad6db7483e"],
        names: &[],
        kind: ComponentKind::NaturalLogarithm,
    },
    Registration {
        guids: &["96c8c5f2-5f8e-4bb3-b19f-eb61d9cefa46"],
        names: &[],
        kind: ComponentKind::Power,
    },
    Registration {
        guids: &["a8bc9c24-1bce-4b92-b7ba-abced2457c22"],
        names: &[],
        kind: ComponentKind::PowerOf2,
    },
    Registration {
        guids: &["a8de2000-073d-412d-a0b2-3a4894ba71f8"],
        names: &[],
        kind: ComponentKind::Truncate,
    },
    Registration {
        guids: &["cae37d1c-8146-4e0b-9cf1-14cb3e337b94"],
        names: &[],
        kind: ComponentKind::Addition,
    },
    Registration {
        guids: &["cfc280bb-332a-4828-bb4e-aca6d88859aa"],
        names: &[],
        kind: ComponentKind::ArcCosine,
    },
    Registration {
        guids: &["d0787f37-d976-48c9-a4b0-29d6c4059cf3"],
        names: &[],
        kind: ComponentKind::Logarithm,
    },
    Registration {
        guids: &["da4be42b-ba75-4249-a685-69ce78b6ee44"],
        names: &[],
        kind: ComponentKind::Sinc,
    },
    Registration {
        guids: &["e9b807a3-dd48-4c2c-bada-e4f8e0edbbdb"],
        names: &[],
        kind: ComponentKind::Maximum,
    },
    Registration {
        guids: &["ec875825-61e4-4c1c-a343-0e0cee0b321b"],
        names: &[],
        kind: ComponentKind::Division,
    },
    Registration {
        guids: &["ecee923b-1b93-4cf2-acd6-680835503437"],
        names: &[],
        kind: ComponentKind::Sine,
    },
    Registration {
        guids: &["ed766861-662d-4462-90f6-29f87f8529cf"],
        names: &[],
        kind: ComponentKind::PowerOf10,
    },
    Registration {
        guids: &["f4a20a34-97e6-4ff5-9b26-7f7ed7a1e333"],
        names: &[],
        kind: ComponentKind::Subtraction,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Tangent => evaluate_simple_trig(inputs, f64::tan),
            Self::Mean => evaluate_mean(inputs),
            Self::Cosine => evaluate_simple_trig(inputs, f64::cos),
            Self::ArcSine => evaluate_simple_trig(inputs, f64::asin),
            Self::Multiplication => evaluate_binary_op(inputs, |a, b| a * b),
            Self::Modulus => evaluate_binary_op(inputs, |a, b| a % b),
            Self::Minimum => evaluate_binary_op(inputs, |a, b| a.min(b)),
            Self::PowerOfE => evaluate_simple_trig(inputs, f64::exp),
            Self::MassAddition => evaluate_mass_addition(inputs),
            Self::ArcTangent => evaluate_simple_trig(inputs, f64::atan),
            Self::NaturalLogarithm => evaluate_simple_trig(inputs, f64::ln),
            Self::Power => evaluate_binary_op(inputs, |a, b| a.powf(b)),
            Self::PowerOf2 => evaluate_simple_trig(inputs, |x| 2.0_f64.powf(x)),
            Self::Truncate => evaluate_truncate(inputs),
            Self::Addition => evaluate_binary_op(inputs, |a, b| a + b),
            Self::ArcCosine => evaluate_simple_trig(inputs, f64::acos),
            Self::Logarithm => evaluate_simple_trig(inputs, f64::log10),
            Self::Sinc => evaluate_sinc(inputs),
            Self::Maximum => evaluate_binary_op(inputs, |a, b| a.max(b)),
            Self::Division => evaluate_division(inputs),
            Self::Sine => evaluate_simple_trig(inputs, f64::sin),
            Self::PowerOf10 => evaluate_simple_trig(inputs, |x| 10.0_f64.powf(x)),
            Self::Subtraction => evaluate_binary_op(inputs, |a, b| a - b),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Tangent => "Tangent",
            Self::Mean => "Mean",
            Self::Cosine => "Cosine",
            Self::ArcSine => "ArcSine",
            Self::Multiplication => "Multiplication",
            Self::Modulus => "Modulus",
            Self::Minimum => "Minimum",
            Self::PowerOfE => "Power of E",
            Self::MassAddition => "Mass Addition",
            Self::ArcTangent => "ArcTangent",
            Self::NaturalLogarithm => "Natural logarithm",
            Self::Power => "Power",
            Self::PowerOf2 => "Power of 2",
            Self::Truncate => "Truncate",
            Self::Addition => "Addition",
            Self::ArcCosine => "ArcCosine",
            Self::Logarithm => "Logarithm",
            Self::Sinc => "Sinc",
            Self::Maximum => "Maximum",
            Self::Division => "Division",
            Self::Sine => "Sine",
            Self::PowerOf10 => "Power of 10",
            Self::Subtraction => "Subtraction",
        }
    }
}

fn evaluate_simple_trig(inputs: &[Value], compute: fn(f64) -> f64) -> ComponentResult {
    let numeric = coerce_number_any(inputs.get(0));
    let result = if let Some(value) = numeric.filter(|value| value.is_finite()) {
        compute(value)
    } else {
        0.0
    };
    let mut outputs = BTreeMap::new();
    outputs.insert("y".to_owned(), Value::Number(result));
    Ok(outputs)
}

fn evaluate_sinc(inputs: &[Value]) -> ComponentResult {
    let numeric = coerce_number_any(inputs.get(0));
    let result = if let Some(value) = numeric.filter(|value| value.is_finite()) {
        if value.abs() < EPSILON {
            1.0
        } else {
            value.sin() / value
        }
    } else {
        0.0
    };
    let mut outputs = BTreeMap::new();
    outputs.insert("y".to_owned(), Value::Number(result));
    Ok(outputs)
}

fn evaluate_binary_op(inputs: &[Value], op: fn(f64, f64) -> f64) -> ComponentResult {
    let a = coerce_number_any(inputs.get(0)).unwrap_or(0.0);
    let b = coerce_number_any(inputs.get(1)).unwrap_or(0.0);
    let result = op(a, b);
    let mut outputs = BTreeMap::new();
    outputs.insert("R".to_owned(), Value::Number(result));
    Ok(outputs)
}

fn evaluate_division(inputs: &[Value]) -> ComponentResult {
    let a = coerce_number_any(inputs.get(0)).unwrap_or(0.0);
    let b = coerce_number_any(inputs.get(1)).unwrap_or(0.0);
    let result = if b.abs() < EPSILON { f64::NAN } else { a / b };
    let mut outputs = BTreeMap::new();
    outputs.insert("R".to_owned(), Value::Number(result));
    Ok(outputs)
}

fn evaluate_mean(inputs: &[Value]) -> ComponentResult {
    let numbers = coerce_number_list(inputs.get(0));
    let mut outputs = BTreeMap::new();
    if numbers.is_empty() {
        outputs.insert("AM".to_owned(), Value::Number(0.0));
        outputs.insert("GM".to_owned(), Value::Number(0.0));
        outputs.insert("HM".to_owned(), Value::Number(0.0));
        return Ok(outputs);
    }

    let count = numbers.len() as f64;
    let sum: f64 = numbers.iter().sum();
    let am = sum / count;

    let prod: f64 = numbers.iter().product();
    let gm = prod.powf(1.0 / count);

    let hm = count / numbers.iter().map(|n| 1.0 / n).sum::<f64>();

    outputs.insert("AM".to_owned(), Value::Number(am));
    outputs.insert("GM".to_owned(), Value::Number(gm));
    outputs.insert("HM".to_owned(), Value::Number(hm));
    Ok(outputs)
}

fn evaluate_mass_addition(inputs: &[Value]) -> ComponentResult {
    let numbers = coerce_number_list(inputs.get(0));
    let mut partial_results = Vec::new();
    let mut current_sum = 0.0;
    for &number in &numbers {
        current_sum += number;
        partial_results.push(Value::Number(current_sum));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert("R".to_owned(), Value::Number(current_sum));
    outputs.insert("Pr".to_owned(), Value::List(partial_results));
    Ok(outputs)
}

fn evaluate_truncate(inputs: &[Value]) -> ComponentResult {
    let mut numbers = coerce_number_list(inputs.get(0));
    let factor = coerce_number_any(inputs.get(1)).unwrap_or(0.0).clamp(0.0, 1.0);

    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let len = numbers.len();
    let count = (len as f64 * factor / 2.0).floor() as usize;

    let truncated = numbers.into_iter().skip(count).take(len - 2 * count).map(Value::Number).collect();

    let mut outputs = BTreeMap::new();
    outputs.insert("T".to_owned(), Value::List(truncated));
    Ok(outputs)
}

fn coerce_number_any(value: Option<&Value>) -> Option<f64> {
    value.and_then(|value| match value {
        Value::Number(number) => Some(*number),
        Value::Boolean(boolean) => Some(if *boolean { 1.0 } else { 0.0 }),
        Value::Text(text) => text.trim().parse::<f64>().ok(),
        Value::List(values) if values.len() == 1 => coerce_number_any(values.get(0)),
        _ => None,
    })
}

fn coerce_number_list(value: Option<&Value>) -> Vec<f64> {
    if let Some(Value::List(values)) = value {
        values.iter().filter_map(|v| coerce_number_any(Some(v))).collect()
    } else if let Some(v) = coerce_number_any(value) {
        vec![v]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentKind};
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn test_addition() {
        let component = ComponentKind::Addition;
        let outputs = component
            .evaluate(&[Value::Number(2.0), Value::Number(3.0)], &MetaMap::new())
            .unwrap();
        assert!(matches!(outputs.get("R"), Some(Value::Number(value)) if (*value - 5.0).abs() < 1e-9));
    }

    #[test]
    fn test_mean() {
        let component = ComponentKind::Mean;
        let outputs = component
            .evaluate(&[Value::List(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)])], &MetaMap::new())
            .unwrap();
        assert!(matches!(outputs.get("AM"), Some(Value::Number(value)) if (*value - 2.0).abs() < 1e-9));
    }

    #[test]
    fn test_mass_addition() {
        let component = ComponentKind::MassAddition;
        let outputs = component
            .evaluate(&[Value::List(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)])], &MetaMap::new())
            .unwrap();
        assert!(matches!(outputs.get("R"), Some(Value::Number(value)) if (*value - 6.0).abs() < 1e-9));
        assert!(matches!(outputs.get("Pr"), Some(Value::List(values)) if values.len() == 3));
    }
}
