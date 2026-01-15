//! Implementaties van Grasshopper "Maths → Util" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::{ComplexValue, Value};

use super::{Component, ComponentError, ComponentResult, coerce};

const PIN_RESULT: &str = "R";
const PIN_OUTPUT_Y: &str = "y";
const PIN_OUTPUT_COMPLEX: &str = "C";
const PIN_OUTPUT_REAL: &str = "R";
const PIN_OUTPUT_IMAGINARY: &str = "i";
const PIN_OUTPUT_MINIMUM: &str = "V-";
const PIN_OUTPUT_MAXIMUM: &str = "V+";
const PIN_OUTPUT_MEAN: &str = "AM";
const PIN_OUTPUT_NUMBERS: &str = "N";
const PIN_OUTPUT_TRUNCATED: &str = "T";
const PIN_OUTPUT_ARGUMENT: &str = "A";
const PIN_OUTPUT_MODULUS: &str = "M";
const PIN_OUTPUT_NEAREST: &str = "N";
const PIN_OUTPUT_FLOOR: &str = "F";
const PIN_OUTPUT_CEILING: &str = "C";
const PIN_OUTPUT_VALUE: &str = "V";

const GOLDEN_RATIO: f64 = 1.618_033_988_749_895;

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Maximum,
    Pi,
    ComplexComponents,
    WeightedAverage,
    Extremes,
    Minimum,
    BlurNumbers,
    SmoothNumbers,
    CreateComplex,
    Average,
    ComplexConjugate,
    ComplexModulus,
    Round,
    NaturalNumber,
    Truncate,
    ComplexArgument,
    GoldenRatio,
    Epsilon,
    InterpolateData,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de maths-util componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{0d1e2027-f153-460d-84c0-f9af431b08cb}"],
        names: &["Maximum", "Max"],
        kind: ComponentKind::Maximum,
    },
    Registration {
        guids: &["{0d2ccfb3-9d41-4759-9452-da6a522c3eaa}"],
        names: &["Pi"],
        kind: ComponentKind::Pi,
    },
    Registration {
        guids: &["{1f384257-b26b-4160-a6d3-1dcd89b64acd}"],
        names: &["Complex Components", "Complex"],
        kind: ComponentKind::ComplexComponents,
    },
    Registration {
        guids: &["{338666eb-14c5-4d9b-82e2-2b5be60655df}"],
        names: &["Weighted Average", "Wav"],
        kind: ComponentKind::WeightedAverage,
    },
    Registration {
        guids: &["{37084b3f-2b66-4f3a-9737-80d0b0b7f0cb}"],
        names: &["Extremes", "Extrz"],
        kind: ComponentKind::Extremes,
    },
    Registration {
        guids: &["{57308b30-772d-4919-ac67-e86c18f3a996}"],
        names: &["Minimum", "Min"],
        kind: ComponentKind::Minimum,
    },
    Registration {
        guids: &["{57e1d392-e3fb-4de9-be98-982854a92351}"],
        names: &["Blur Numbers", "NBlur"],
        kind: ComponentKind::BlurNumbers,
    },
    Registration {
        guids: &["{5b424e1c-d061-43cd-8c20-db84564b0502}"],
        names: &["Smooth Numbers", "Smooth"],
        kind: ComponentKind::SmoothNumbers,
    },
    Registration {
        guids: &["{63d12974-2915-4ccf-ac26-5d566c3bac92}"],
        names: &["Create Complex"],
        kind: ComponentKind::CreateComplex,
    },
    Registration {
        guids: &["{7986486c-621a-48fb-8f27-a28a22c91cc9}"],
        names: &["Average", "Avr"],
        kind: ComponentKind::Average,
    },
    Registration {
        guids: &["{7d2a6064-51f0-45b2-adc4-f417b30dcd15}"],
        names: &["Complex Conjugate", "z*"],
        kind: ComponentKind::ComplexConjugate,
    },
    Registration {
        guids: &["{88fb33f9-f467-452b-a0e3-44bdb78a9b06}"],
        names: &["Complex Modulus", "CMod"],
        kind: ComponentKind::ComplexModulus,
    },
    Registration {
        guids: &["{a50c4a3b-0177-4c91-8556-db95de6c56c8}"],
        names: &["Round"],
        kind: ComponentKind::Round,
    },
    Registration {
        guids: &["{b6cac37c-21b9-46c6-bd0d-17ff67796578}"],
        names: &["Natural logarithm", "E"],
        kind: ComponentKind::NaturalNumber,
    },
    Registration {
        guids: &["{bd96f893-d57b-4f04-90d0-dca0d72ff2f9}"],
        names: &["Truncate", "Trunc"],
        kind: ComponentKind::Truncate,
    },
    Registration {
        guids: &["{be715e4c-d6d8-447b-a9c3-6fea700d0b83}"],
        names: &["Complex Argument", "Arg"],
        kind: ComponentKind::ComplexArgument,
    },
    Registration {
        guids: &["{cb22d3ed-93d8-4629-bdf2-c0c7c25afd2c}"],
        names: &["Golden Ratio", "Phi"],
        kind: ComponentKind::GoldenRatio,
    },
    Registration {
        guids: &["{deadf87d-99a6-4980-90c3-f98350aa6f0f}"],
        names: &["Epsilon", "Eps"],
        kind: ComponentKind::Epsilon,
    },
    Registration {
        guids: &["{e168ff6b-e5c0-48f1-b831-f6996bf3b459}"],
        names: &["Interpolate data", "Interp"],
        kind: ComponentKind::InterpolateData,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Maximum => evaluate_maximum(inputs),
            Self::Pi => evaluate_constant(inputs, std::f64::consts::PI, "Pi"),
            Self::ComplexComponents => evaluate_complex_components(inputs),
            Self::WeightedAverage => evaluate_weighted_average(inputs),
            Self::Extremes => evaluate_extremes(inputs),
            Self::Minimum => evaluate_minimum(inputs),
            Self::BlurNumbers => evaluate_blur_numbers(inputs),
            Self::SmoothNumbers => evaluate_smooth_numbers(inputs),
            Self::CreateComplex => evaluate_create_complex(inputs),
            Self::Average => evaluate_average(inputs),
            Self::ComplexConjugate => evaluate_complex_conjugate(inputs),
            Self::ComplexModulus => evaluate_complex_modulus(inputs),
            Self::Round => evaluate_round(inputs),
            Self::NaturalNumber => {
                evaluate_constant(inputs, std::f64::consts::E, "Natural logarithm")
            }
            Self::Truncate => evaluate_truncate(inputs),
            Self::ComplexArgument => evaluate_complex_argument(inputs),
            Self::GoldenRatio => evaluate_constant(inputs, GOLDEN_RATIO, "Golden Ratio"),
            Self::Epsilon => evaluate_constant(inputs, f64::EPSILON, "Epsilon"),
            Self::InterpolateData => evaluate_interpolate_data(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Maximum => "Maximum",
            Self::Pi => "Pi",
            Self::ComplexComponents => "Complex Components",
            Self::WeightedAverage => "Weighted Average",
            Self::Extremes => "Extremes",
            Self::Minimum => "Minimum",
            Self::BlurNumbers => "Blur Numbers",
            Self::SmoothNumbers => "Smooth Numbers",
            Self::CreateComplex => "Create Complex",
            Self::Average => "Average",
            Self::ComplexConjugate => "Complex Conjugate",
            Self::ComplexModulus => "Complex Modulus",
            Self::Round => "Round",
            Self::NaturalNumber => "Natural logarithm",
            Self::Truncate => "Truncate",
            Self::ComplexArgument => "Complex Argument",
            Self::GoldenRatio => "Golden Ratio",
            Self::Epsilon => "Epsilon",
            Self::InterpolateData => "Interpolate data",
        }
    }
}

fn evaluate_maximum(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 2, "Maximum")?;
    let a = coerce_number(&inputs[0], "Maximum")?;
    let b = coerce_number(&inputs[1], "Maximum")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_RESULT.to_owned(), Value::Number(a.max(b)));
    Ok(outputs)
}

fn evaluate_minimum(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 2, "Minimum")?;
    let a = coerce_number(&inputs[0], "Minimum")?;
    let b = coerce_number(&inputs[1], "Minimum")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_RESULT.to_owned(), Value::Number(a.min(b)));
    Ok(outputs)
}

fn evaluate_extremes(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 2, "Extremes")?;
    let a = coerce_number(&inputs[0], "Extremes")?;
    let b = coerce_number(&inputs[1], "Extremes")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_MINIMUM.to_owned(), Value::Number(a.min(b)));
    outputs.insert(PIN_OUTPUT_MAXIMUM.to_owned(), Value::Number(a.max(b)));
    Ok(outputs)
}

fn evaluate_constant(inputs: &[Value], constant: f64, context: &str) -> ComponentResult {
    let factor = coerce_optional_number(inputs.get(0), 1.0, context)?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_Y.to_owned(), Value::Number(constant * factor));
    Ok(outputs)
}

fn evaluate_weighted_average(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 2, "Weighted Average")?;
    let values = coerce_number_list(inputs.get(0), "Weighted Average values")?;
    let weights = coerce_number_list(inputs.get(1), "Weighted Average weights")?;

    if values.len() != weights.len() {
        return Err(ComponentError::new(
            "Aantal waarden en gewichten moet gelijk zijn voor Weighted Average",
        ));
    }

    if values.is_empty() {
        return Err(ComponentError::new(
            "Weighted Average vereist ten minste één waarde",
        ));
    }

    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;
    for (value, weight) in values.iter().zip(weights.iter()) {
        weighted_sum += value * weight;
        weight_total += weight;
    }

    if weight_total.abs() < f64::EPSILON {
        return Err(ComponentError::new(
            "Som van gewichten mag niet nul zijn voor Weighted Average",
        ));
    }

    let mean = weighted_sum / weight_total;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_MEAN.to_owned(), Value::Number(mean));
    Ok(outputs)
}

fn evaluate_average(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 1, "Average")?;
    let values = coerce_number_list(inputs.get(0), "Average")?;
    if values.is_empty() {
        return Err(ComponentError::new("Average vereist ten minste één waarde"));
    }
    let sum: f64 = values.iter().sum();
    let mean = sum / values.len() as f64;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_MEAN.to_owned(), Value::Number(mean));
    Ok(outputs)
}

fn evaluate_blur_numbers(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 1, "Blur Numbers")?;
    let numbers = coerce_number_list(inputs.get(0), "Blur Numbers")?;
    let strength = coerce_clamped_number(inputs.get(1), 0.5, 0.0, 1.0, "Blur Numbers strength")?;
    let iterations =
        coerce_integer_with_default(inputs.get(2), 1, 1, 10_000, "Blur Numbers iterations")?;
    let lock = coerce_boolean_with_default(inputs.get(3), false, "Blur Numbers lock")?;
    let wrap = coerce_boolean_with_default(inputs.get(4), false, "Blur Numbers wrap")?;

    if numbers.len() <= 1 || strength <= 0.0 || iterations == 0 {
        return map_with_list(
            PIN_OUTPUT_NUMBERS,
            numbers.into_iter().map(Value::Number).collect(),
        );
    }

    let mut current = numbers;
    let count = current.len();
    for _ in 0..iterations {
        let previous = current.clone();
        for index in 0..count {
            if lock && (index == 0 || index == count - 1) {
                continue;
            }

            let current_value = previous[index];
            let prev_value = if index == 0 {
                if wrap {
                    previous[count - 1]
                } else {
                    previous[0]
                }
            } else {
                previous[index - 1]
            };
            let next_value = if index + 1 >= count {
                if wrap {
                    previous[0]
                } else {
                    previous[count - 1]
                }
            } else {
                previous[index + 1]
            };
            let neighbour_mean = (prev_value + next_value) * 0.5;
            current[index] = current_value * (1.0 - strength) + neighbour_mean * strength;
        }
    }

    map_with_list(
        PIN_OUTPUT_NUMBERS,
        current.into_iter().map(Value::Number).collect(),
    )
}

fn evaluate_smooth_numbers(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 1, "Smooth Numbers")?;
    let tree = inputs
        .get(0)
        .ok_or_else(|| ComponentError::new("Smooth Numbers vereist invoer"))?;
    let branches = match tree {
        Value::List(values) => values,
        other => {
            return Err(ComponentError::new(format!(
                "Smooth Numbers verwacht een lijst van lijsten, kreeg {}",
                other.kind()
            )));
        }
    };

    let mut result_branches = Vec::with_capacity(branches.len());
    for branch in branches {
        match branch {
            Value::List(values) => {
                let mut smoothed = Vec::with_capacity(values.len());
                let mut last_value = None;
                for value in values {
                    let number = coerce_number(value, "Smooth Numbers tak")?;
                    let new_value = match last_value {
                        Some(previous) => 0.5 * previous + 0.5 * number,
                        None => number,
                    };
                    smoothed.push(Value::Number(new_value));
                    last_value = Some(new_value);
                }
                result_branches.push(Value::List(smoothed));
            }
            value => {
                let number = coerce_number(value, "Smooth Numbers element")?;
                result_branches.push(Value::List(vec![Value::Number(number)]));
            }
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_NUMBERS.to_owned(), Value::List(result_branches));
    Ok(outputs)
}

fn evaluate_create_complex(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 2, "Create Complex")?;
    let real = coerce_number(&inputs[0], "Create Complex real")?;
    let imag = coerce_number(&inputs[1], "Create Complex imaginary")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_COMPLEX.to_owned(),
        Value::Complex(ComplexValue::new(real, imag)),
    );
    Ok(outputs)
}

fn evaluate_complex_components(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 1, "Complex Components")?;
    let complex = coerce_complex(&inputs[0], "Complex Components")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_REAL.to_owned(), Value::Number(complex.re));
    outputs.insert(PIN_OUTPUT_IMAGINARY.to_owned(), Value::Number(complex.im));
    Ok(outputs)
}

fn evaluate_complex_conjugate(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 1, "Complex Conjugate")?;
    let complex = coerce_complex(&inputs[0], "Complex Conjugate")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_COMPLEX.to_owned(),
        Value::Complex(complex.conj()),
    );
    Ok(outputs)
}

fn evaluate_complex_modulus(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 1, "Complex Modulus")?;
    let complex = coerce_complex(&inputs[0], "Complex Modulus")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_MODULUS.to_owned(), Value::Number(complex.norm()));
    Ok(outputs)
}

fn evaluate_complex_argument(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 1, "Complex Argument")?;
    let complex = coerce_complex(&inputs[0], "Complex Argument")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_ARGUMENT.to_owned(), Value::Number(complex.arg()));
    Ok(outputs)
}

fn evaluate_round(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 1, "Round")?;
    let number = coerce_number(&inputs[0], "Round")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_NEAREST.to_owned(), Value::Number(number.round()));
    outputs.insert(PIN_OUTPUT_FLOOR.to_owned(), Value::Number(number.floor()));
    outputs.insert(PIN_OUTPUT_CEILING.to_owned(), Value::Number(number.ceil()));
    Ok(outputs)
}

fn evaluate_truncate(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 2, "Truncate")?;
    let values = coerce_number_list(inputs.get(0), "Truncate values")?;
    let factor = coerce_clamped_number(inputs.get(1), 0.5, 0.0, 1.0, "Truncate factor")?;

    if values.is_empty() {
        return Err(ComponentError::new(
            "Truncate vereist ten minste één waarde",
        ));
    }

    let mut sorted = values.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let remove_count = ((sorted.len() as f64) * factor).round() as usize;
    let retain = sorted
        .into_iter()
        .skip(remove_count)
        .take(values.len().saturating_sub(2 * remove_count))
        .collect::<Vec<_>>();

    map_with_list(
        PIN_OUTPUT_TRUNCATED,
        retain.into_iter().map(Value::Number).collect(),
    )
}

fn evaluate_interpolate_data(inputs: &[Value]) -> ComponentResult {
    ensure_input_count(inputs, 2, "Interpolate data")?;
    let values = coerce_number_list(inputs.get(0), "Interpolate data values")?;
    let t = coerce_number(
        inputs
            .get(1)
            .ok_or_else(|| ComponentError::new("Interpolate data vereist parameter"))?,
        "Interpolate data parameter",
    )?;

    if values.is_empty() {
        return Err(ComponentError::new(
            "Interpolate data vereist ten minste één waarde",
        ));
    }

    if values.len() == 1 {
        return map_with_number(PIN_OUTPUT_VALUE, values[0]);
    }

    let clamped_t = t.clamp(0.0, 1.0);
    let position = clamped_t * ((values.len() - 1) as f64);
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;

    if lower_index == upper_index {
        return map_with_number(PIN_OUTPUT_VALUE, values[lower_index]);
    }

    let ratio = position - lower_index as f64;
    let lower = values[lower_index];
    let upper = values[upper_index];
    let interpolated = lower + (upper - lower) * ratio;
    map_with_number(PIN_OUTPUT_VALUE, interpolated)
}

fn ensure_input_count(
    inputs: &[Value],
    expected: usize,
    context: &str,
) -> Result<(), ComponentError> {
    if inputs.len() < expected {
        Err(ComponentError::new(format!(
            "{context} verwacht ten minste {expected} invoerwaarden"
        )))
    } else {
        Ok(())
    }
}

fn coerce_number(value: &Value, context: &str) -> Result<f64, ComponentError> {
    match value {
        Value::Number(number) if number.is_finite() => Ok(*number),
        Value::Boolean(boolean) => Ok(if *boolean { 1.0 } else { 0.0 }),
        Value::List(values) if values.len() == 1 => coerce_number(&values[0], context),
        other => Err(ComponentError::new(format!(
            "{context} verwacht een numerieke waarde, kreeg {}",
            other.kind()
        ))),
    }
}

fn coerce_optional_number(
    value: Option<&Value>,
    default: f64,
    context: &str,
) -> Result<f64, ComponentError> {
    match value {
        Some(Value::Null) | None => Ok(default),
        Some(value) => coerce_number(value, context),
    }
}

fn coerce_clamped_number(
    value: Option<&Value>,
    default: f64,
    min: f64,
    max: f64,
    context: &str,
) -> Result<f64, ComponentError> {
    let number = coerce_optional_number(value, default, context)?;
    if number.is_nan() {
        return Err(ComponentError::new(format!("{context} resulteert in NaN")));
    }
    Ok(number.clamp(min, max))
}

fn coerce_integer_with_default(
    value: Option<&Value>,
    default: i64,
    min: i64,
    max: i64,
    context: &str,
) -> Result<usize, ComponentError> {
    let number = coerce_optional_number(value, default as f64, context)?;
    if !number.is_finite() {
        return Err(ComponentError::new(format!(
            "{context} verwacht een eindig geheel getal"
        )));
    }
    let rounded = number.round();
    let clamped = rounded.clamp(min as f64, max as f64);
    Ok(clamped as usize)
}

fn coerce_boolean_with_default(
    value: Option<&Value>,
    default: bool,
    context: &str,
) -> Result<bool, ComponentError> {
    match value {
        Some(Value::Boolean(boolean)) => Ok(*boolean),
        Some(Value::Number(number)) => {
            if number.is_nan() {
                Err(ComponentError::new(format!(
                    "{context} verwacht een booleaanse waarde, kreeg NaN"
                )))
            } else {
                Ok(*number != 0.0)
            }
        }
        Some(Value::List(values)) if values.len() == 1 => {
            coerce_boolean_with_default(values.first(), default, context)
        }
        Some(other) => Err(ComponentError::new(format!(
            "{context} verwacht een booleaanse waarde, kreeg {}",
            other.kind()
        ))),
        None => Ok(default),
    }
}

fn coerce_number_list(value: Option<&Value>, context: &str) -> Result<Vec<f64>, ComponentError> {
    let value = value
        .ok_or_else(|| ComponentError::new(format!("{context} vereist een lijst met waarden")))?;
    match value {
        Value::List(entries) => entries
            .iter()
            .map(|entry| coerce_number(entry, context))
            .collect(),
        Value::Number(_) | Value::Boolean(_) => Ok(vec![coerce_number(value, context)?]),
        other => Err(ComponentError::new(format!(
            "{context} verwacht een lijst met numerieke waarden, kreeg {}",
            other.kind()
        ))),
    }
}

fn coerce_complex(value: &Value, context: &str) -> Result<ComplexValue, ComponentError> {
    match value {
        Value::Complex(value) => Ok(*value),
        Value::Number(number) if number.is_finite() => Ok(ComplexValue::new(*number, 0.0)),
        Value::Boolean(boolean) => Ok(ComplexValue::new(if *boolean { 1.0 } else { 0.0 }, 0.0)),
        Value::List(values) if values.len() >= 2 => {
            let real = coerce_number(&values[0], context)?;
            let imag = coerce_number(&values[1], context)?;
            Ok(ComplexValue::new(real, imag))
        }
        Value::List(values) if values.len() == 1 => coerce_complex(&values[0], context),
        other => Err(ComponentError::new(format!(
            "{context} verwacht een complex getal, kreeg {}",
            other.kind()
        ))),
    }
}

fn map_with_list(pin: &str, values: Vec<Value>) -> ComponentResult {
    let mut outputs = BTreeMap::new();
    outputs.insert(pin.to_owned(), Value::List(values));
    Ok(outputs)
}

fn map_with_number(pin: &str, value: f64) -> ComponentResult {
    let mut outputs = BTreeMap::new();
    outputs.insert(pin.to_owned(), Value::Number(value));
    Ok(outputs)
}