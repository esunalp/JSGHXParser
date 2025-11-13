//! Implementaties van de standaard Grasshopper "Maths → Operators" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const EPSILON: f64 = 1e-9;

const PIN_RESULT: &str = "R";
const PIN_PARTIAL_RESULTS: &str = "Pr";
const PIN_SERIES: &str = "S";
const PIN_DIFFERENCES: &str = "D";
const PIN_OUTPUT_Y: &str = "y";
const PIN_FACTORIAL: &str = "F";
const PIN_GREATER_THAN: &str = ">";
const PIN_GREATER_OR_EQUAL: &str = ">=";
const PIN_LESS_THAN: &str = "<";
const PIN_LESS_OR_EQUAL: &str = "<=";
const PIN_EQUAL: &str = "=";
const PIN_NOT_EQUAL: &str = "≠";
const PIN_DIFFERENCE: &str = "dt";

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    GateAnd,
    GateOr,
    GateNot,
    GateNor,
    GateNand,
    GateXor,
    GateXnor,
    GateMajority,
    Absolute,
    Negative,
    Subtraction,
    Multiplication,
    Division,
    IntegerDivision,
    Modulus,
    Power,
    Factorial,
    MassAddition,
    MassMultiplication,
    SeriesAddition,
    RelativeDifferences,
    LargerThan,
    SmallerThan,
    Equality,
    Similarity,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de maths-operatoren.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{040f195d-0b4e-4fe0-901f-fedb2fd3db15}"],
        names: &["Gate And", "And"],
        kind: ComponentKind::GateAnd,
    },
    Registration {
        guids: &["{5cad70f9-5a53-4c5c-a782-54a479b4abe3}"],
        names: &["Gate Or", "Or"],
        kind: ComponentKind::GateOr,
    },
    Registration {
        guids: &["{cb2c7d3c-41b4-4c6d-a6bd-9235bd2851bb}"],
        names: &["Gate Not", "Not"],
        kind: ComponentKind::GateNot,
    },
    Registration {
        guids: &["{548177c2-d1db-4172-b667-bec979e2d38b}"],
        names: &["Gate Nor", "Nor"],
        kind: ComponentKind::GateNor,
    },
    Registration {
        guids: &["{5ca5de6b-bc71-46c4-a8f7-7f30d7040acb}"],
        names: &["Gate Nand", "Nand"],
        kind: ComponentKind::GateNand,
    },
    Registration {
        guids: &["{de4a0d86-2709-4564-935a-88bf4d40af89}"],
        names: &["Gate Xor", "Xor"],
        kind: ComponentKind::GateXor,
    },
    Registration {
        guids: &["{b6aedcac-bf43-42d4-899e-d763612f834d}"],
        names: &["Gate Xnor", "Xnor"],
        kind: ComponentKind::GateXnor,
    },
    Registration {
        guids: &["{78669f9c-4fea-44fd-ab12-2a69eeec58de}"],
        names: &["Gate Majority", "Vote"],
        kind: ComponentKind::GateMajority,
    },
    Registration {
        guids: &["{a3371040-e552-4bc8-b0ff-10a840258e88}"],
        names: &["Negative", "Neg"],
        kind: ComponentKind::Negative,
    },
    Registration {
        guids: &[
            "{2c56ab33-c7cc-4129-886c-d5856b714010}",
            "{9c007a04-d0d9-48e4-9da3-9ba142bc4d46}",
        ],
        names: &["Subtraction", "A-B"],
        kind: ComponentKind::Subtraction,
    },
    Registration {
        guids: &[
            "{b8963bb1-aa57-476e-a20e-ed6cf635a49c}",
            "{ce46b74e-00c9-43c4-805a-193b69ea4a11}",
        ],
        names: &["Multiplication", "A×B", "AxB"],
        kind: ComponentKind::Multiplication,
    },
    Registration {
        guids: &["{9c85271f-89fa-4e9f-9f4a-d75802120ccc}"],
        names: &["Division", "A/B"],
        kind: ComponentKind::Division,
    },
    Registration {
        guids: &["{54db2568-3441-4ae2-bcef-92c4cc608e11}"],
        names: &["Integer Division", "A\\B"],
        kind: ComponentKind::IntegerDivision,
    },
    Registration {
        guids: &["{431bc610-8ae1-4090-b217-1a9d9c519fe2}"],
        names: &["Modulus", "Mod"],
        kind: ComponentKind::Modulus,
    },
    Registration {
        guids: &["{78fed580-851b-46fe-af2f-6519a9d378e0}"],
        names: &["Power", "Pow"],
        kind: ComponentKind::Power,
    },
    Registration {
        guids: &[
            "{80da90e3-3ea9-4cfe-b7cc-2b6019f850e3}",
            "{a0a38131-c5fc-4984-b05d-34cf57f0c018}",
        ],
        names: &["Factorial", "Fac"],
        kind: ComponentKind::Factorial,
    },
    Registration {
        guids: &["{5b850221-b527-4bd6-8c62-e94168cd6efa}"],
        names: &["Mass Addition", "MA"],
        kind: ComponentKind::MassAddition,
    },
    Registration {
        guids: &[
            "{921775f7-bf22-4cfc-a4db-c415a56069c4}",
            "{e44c1bd7-72cc-4697-80c9-02787baf7bb4}",
        ],
        names: &["Mass Multiplication", "MM"],
        kind: ComponentKind::MassMultiplication,
    },
    Registration {
        guids: &["{586706a8-109b-43ec-b581-743e920c951a}"],
        names: &["Series Addition", "SA"],
        kind: ComponentKind::SeriesAddition,
    },
    Registration {
        guids: &["{dd17d442-3776-40b3-ad5b-5e188b56bd4c}"],
        names: &["Relative Differences", "RelDif"],
        kind: ComponentKind::RelativeDifferences,
    },
    Registration {
        guids: &["{30d58600-1aab-42db-80a3-f1ea6c4269a0}"],
        names: &["Larger Than", "Larger", ">"],
        kind: ComponentKind::LargerThan,
    },
    Registration {
        guids: &["{ae840986-cade-4e5a-96b0-570f007d4fc0}"],
        names: &["Smaller Than", "Smaller", "<"],
        kind: ComponentKind::SmallerThan,
    },
    Registration {
        guids: &["{5db0fb89-4f22-4f09-a777-fa5e55aed7ec}"],
        names: &["Equality", "Equals"],
        kind: ComponentKind::Equality,
    },
    Registration {
        guids: &["{40177d8a-a35c-4622-bca7-d150031fe427}"],
        names: &["Similarity", "Similar"],
        kind: ComponentKind::Similarity,
    },
    Registration {
        guids: &["{28124995-cf99-4298-b6f4-c75a8e379f18}"],
        names: &["Absolute", "Abs"],
        kind: ComponentKind::Absolute,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::GateAnd => evaluate_gate_and(inputs),
            Self::GateOr => evaluate_gate_or(inputs),
            Self::GateNot => evaluate_gate_not(inputs),
            Self::GateNor => evaluate_gate_nor(inputs),
            Self::GateNand => evaluate_gate_nand(inputs),
            Self::GateXor => evaluate_gate_xor(inputs),
            Self::GateXnor => evaluate_gate_xnor(inputs),
            Self::GateMajority => evaluate_gate_majority(inputs),
            Self::Absolute => evaluate_absolute(inputs),
            Self::Negative => evaluate_negative(inputs),
            Self::Subtraction => evaluate_subtraction(inputs),
            Self::Multiplication => evaluate_multiplication(inputs),
            Self::Division => evaluate_division(inputs),
            Self::IntegerDivision => evaluate_integer_division(inputs),
            Self::Modulus => evaluate_modulus(inputs),
            Self::Power => evaluate_power(inputs),
            Self::Factorial => evaluate_factorial(inputs),
            Self::MassAddition => evaluate_mass_addition(inputs),
            Self::MassMultiplication => evaluate_mass_multiplication(inputs),
            Self::SeriesAddition => evaluate_series_addition(inputs),
            Self::RelativeDifferences => evaluate_relative_differences(inputs),
            Self::LargerThan => evaluate_larger_than(inputs),
            Self::SmallerThan => evaluate_smaller_than(inputs),
            Self::Equality => evaluate_equality(inputs),
            Self::Similarity => evaluate_similarity(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::GateAnd => "Gate And",
            Self::GateOr => "Gate Or",
            Self::GateNot => "Gate Not",
            Self::GateNor => "Gate Nor",
            Self::GateNand => "Gate Nand",
            Self::GateXor => "Gate Xor",
            Self::GateXnor => "Gate Xnor",
            Self::GateMajority => "Gate Majority",
            Self::Absolute => "Absolute",
            Self::Negative => "Negative",
            Self::Subtraction => "Subtraction",
            Self::Multiplication => "Multiplication",
            Self::Division => "Division",
            Self::IntegerDivision => "Integer Division",
            Self::Modulus => "Modulus",
            Self::Power => "Power",
            Self::Factorial => "Factorial",
            Self::MassAddition => "Mass Addition",
            Self::MassMultiplication => "Mass Multiplication",
            Self::SeriesAddition => "Series Addition",
            Self::RelativeDifferences => "Relative Differences",
            Self::LargerThan => "Larger Than",
            Self::SmallerThan => "Smaller Than",
            Self::Equality => "Equality",
            Self::Similarity => "Similarity",
        }
    }
}

fn evaluate_gate_and(inputs: &[Value]) -> ComponentResult {
    evaluate_binary_gate(inputs, |a, b| a && b)
}

fn evaluate_gate_or(inputs: &[Value]) -> ComponentResult {
    evaluate_binary_gate(inputs, |a, b| a || b)
}

fn evaluate_gate_not(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 1 {
        return Err(ComponentError::new(
            "Gate Not component vereist een invoerwaarde",
        ));
    }
    let value = coerce_boolean(&inputs[0], "Gate Not")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_RESULT.to_owned(), Value::Boolean(!value));
    Ok(outputs)
}

fn evaluate_gate_nor(inputs: &[Value]) -> ComponentResult {
    evaluate_binary_gate(inputs, |a, b| !(a || b))
}

fn evaluate_gate_nand(inputs: &[Value]) -> ComponentResult {
    evaluate_binary_gate(inputs, |a, b| !(a && b))
}

fn evaluate_gate_xor(inputs: &[Value]) -> ComponentResult {
    evaluate_binary_gate(inputs, |a, b| a ^ b)
}

fn evaluate_gate_xnor(inputs: &[Value]) -> ComponentResult {
    evaluate_binary_gate(inputs, |a, b| !(a ^ b))
}

fn evaluate_gate_majority(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Gate Majority vereist drie booleaanse invoeren",
        ));
    }
    let a = coerce_boolean(&inputs[0], "Gate Majority")?;
    let b = coerce_boolean(&inputs[1], "Gate Majority")?;
    let c = coerce_boolean(&inputs[2], "Gate Majority")?;
    let majority = (a && b) || (a && c) || (b && c);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_RESULT.to_owned(), Value::Boolean(majority));
    Ok(outputs)
}

fn evaluate_absolute(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Absolute component vereist een invoer"));
    }
    let value = coerce_number(&inputs[0], "Absolute")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_Y.to_owned(), Value::Number(value.abs()));
    Ok(outputs)
}

fn evaluate_negative(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Negative component vereist een invoer"));
    }
    let value = coerce_number(&inputs[0], "Negative")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_Y.to_owned(), Value::Number(-value));
    Ok(outputs)
}

fn evaluate_subtraction(inputs: &[Value]) -> ComponentResult {
    evaluate_binary_arithmetic(inputs, "Subtraction", |a, b| Ok(a - b))
}

fn evaluate_multiplication(inputs: &[Value]) -> ComponentResult {
    evaluate_binary_arithmetic(inputs, "Multiplication", |a, b| Ok(a * b))
}

fn evaluate_division(inputs: &[Value]) -> ComponentResult {
    evaluate_binary_arithmetic(inputs, "Division", |a, b| {
        if b.abs() < EPSILON {
            return Err(ComponentError::new("Division door nul is niet toegestaan"));
        }
        Ok(a / b)
    })
}

fn evaluate_integer_division(inputs: &[Value]) -> ComponentResult {
    evaluate_binary_arithmetic(inputs, "Integer Division", |a, b| {
        if b.abs() < EPSILON {
            return Err(ComponentError::new(
                "Integer Division vereist een niet-nul deler",
            ));
        }
        Ok((a / b).trunc())
    })
}

fn evaluate_modulus(inputs: &[Value]) -> ComponentResult {
    evaluate_binary_arithmetic(inputs, "Modulus", |a, b| {
        if b.abs() < EPSILON {
            return Err(ComponentError::new("Modulus vereist een niet-nul deler"));
        }
        let remainder = ((a % b) + b) % b;
        Ok(remainder)
    })
}

fn evaluate_power(inputs: &[Value]) -> ComponentResult {
    evaluate_binary_arithmetic(inputs, "Power", |a, b| Ok(a.powf(b)))
}

fn evaluate_factorial(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Factorial component vereist een invoer",
        ));
    }
    let number = coerce_number(&inputs[0], "Factorial")?;
    if number < 0.0 {
        return Err(ComponentError::new(
            "Factorial verwacht een niet-negatief geheel getal",
        ));
    }
    let rounded = number.round();
    if (rounded - number).abs() > EPSILON {
        return Err(ComponentError::new("Factorial verwacht een geheel getal"));
    }
    let n = rounded as u64;
    let mut result = 1.0f64;
    for value in 2..=n {
        result *= value as f64;
        if !result.is_finite() {
            return Err(ComponentError::new(
                "Factorial resultaat is te groot voor een f64",
            ));
        }
    }
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_FACTORIAL.to_owned(), Value::Number(result));
    Ok(outputs)
}

fn evaluate_mass_addition(inputs: &[Value]) -> ComponentResult {
    let values = collect_math_values(inputs);
    if values.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_RESULT.to_owned(), Value::Number(0.0));
        outputs.insert(PIN_PARTIAL_RESULTS.to_owned(), Value::List(Vec::new()));
        return Ok(outputs);
    }
    let (result, partial) = sequential_combine(values, add_values)?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_RESULT.to_owned(), result.to_value());
    outputs.insert(
        PIN_PARTIAL_RESULTS.to_owned(),
        Value::List(partial.into_iter().map(|value| value.to_value()).collect()),
    );
    Ok(outputs)
}

fn evaluate_mass_multiplication(inputs: &[Value]) -> ComponentResult {
    let values = collect_math_values(inputs);
    if values.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_RESULT.to_owned(), Value::Number(1.0));
        outputs.insert(PIN_PARTIAL_RESULTS.to_owned(), Value::List(Vec::new()));
        return Ok(outputs);
    }
    let (result, partial) = sequential_combine(values, multiply_values)?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_RESULT.to_owned(), result.to_value());
    outputs.insert(
        PIN_PARTIAL_RESULTS.to_owned(),
        Value::List(partial.into_iter().map(|value| value.to_value()).collect()),
    );
    Ok(outputs)
}

fn evaluate_series_addition(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Series Addition vereist minstens een lijst met waarden",
        ));
    }
    let numbers = collect_number_list(&inputs[0]);
    let goal = optional_number(inputs.get(1), "Series Addition doel")?;
    let start = inputs
        .get(2)
        .map(|value| coerce_number(value, "Series Addition start"))
        .transpose()?
        .unwrap_or(0.0);

    let mut total = start;
    let mut series = Vec::new();
    if numbers.is_empty() {
        let remainder = goal.map_or(0.0, |g| total - g);
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_SERIES.to_owned(), Value::List(Vec::new()));
        outputs.insert(PIN_RESULT.to_owned(), Value::Number(remainder));
        return Ok(outputs);
    }

    let direction = goal.map_or(0.0, |goal_value| {
        let diff = goal_value - start;
        if diff > 0.0 {
            1.0
        } else if diff < 0.0 {
            -1.0
        } else {
            0.0
        }
    });

    for value in numbers {
        total += value;
        series.push(Value::Number(total));
        if let Some(goal_value) = goal {
            if (direction >= 0.0 && total >= goal_value) || (direction < 0.0 && total <= goal_value)
            {
                break;
            }
        }
    }

    let remainder = goal.map_or(0.0, |g| total - g);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_SERIES.to_owned(), Value::List(series));
    outputs.insert(PIN_RESULT.to_owned(), Value::Number(remainder));
    Ok(outputs)
}

fn evaluate_relative_differences(inputs: &[Value]) -> ComponentResult {
    let values = collect_math_values(inputs);
    if values.len() < 2 {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_DIFFERENCES.to_owned(), Value::List(Vec::new()));
        return Ok(outputs);
    }

    let mut differences = Vec::with_capacity(values.len() - 1);
    for window in values.windows(2) {
        let previous = &window[0];
        let current = &window[1];
        let diff = subtract_values(current.clone(), previous.clone())?;
        differences.push(diff.to_value());
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_DIFFERENCES.to_owned(), Value::List(differences));
    Ok(outputs)
}

fn evaluate_larger_than(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Larger Than component vereist twee invoeren",
        ));
    }
    let a = coerce_number(&inputs[0], "Larger Than")?;
    let b = coerce_number(&inputs[1], "Larger Than")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_GREATER_THAN.to_owned(), Value::Boolean(a > b));
    outputs.insert(PIN_GREATER_OR_EQUAL.to_owned(), Value::Boolean(a >= b));
    Ok(outputs)
}

fn evaluate_smaller_than(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Smaller Than component vereist twee invoeren",
        ));
    }
    let a = coerce_number(&inputs[0], "Smaller Than")?;
    let b = coerce_number(&inputs[1], "Smaller Than")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_LESS_THAN.to_owned(), Value::Boolean(a < b));
    outputs.insert(PIN_LESS_OR_EQUAL.to_owned(), Value::Boolean(a <= b));
    Ok(outputs)
}

fn evaluate_equality(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Equality component vereist twee invoeren",
        ));
    }

    let left = &inputs[0];
    let right = &inputs[1];
    let equal = match (to_vector(left), to_vector(right)) {
        (Some(a), Some(b)) => {
            let distance =
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
            distance <= EPSILON
        }
        _ => {
            let left_number = to_optional_number(Some(left))?;
            let right_number = to_optional_number(Some(right))?;
            if let (Some(a), Some(b)) = (left_number, right_number) {
                (a - b).abs() <= EPSILON
            } else {
                left == right
            }
        }
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_EQUAL.to_owned(), Value::Boolean(equal));
    outputs.insert(PIN_NOT_EQUAL.to_owned(), Value::Boolean(!equal));
    Ok(outputs)
}

fn evaluate_similarity(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Similarity component vereist drie invoeren",
        ));
    }

    let a = coerce_number(&inputs[0], "Similarity")?;
    let b = coerce_number(&inputs[1], "Similarity")?;
    let threshold = coerce_number(&inputs[2], "Similarity")?.abs();
    let difference = (a - b).abs();
    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_EQUAL.to_owned(),
        Value::Boolean(difference <= threshold),
    );
    outputs.insert(PIN_DIFFERENCE.to_owned(), Value::Number(difference));
    Ok(outputs)
}

fn evaluate_binary_gate(
    inputs: &[Value],
    operation: impl Fn(bool, bool) -> bool,
) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Booleaanse gate component vereist twee invoerwaarden",
        ));
    }
    let a = coerce_boolean(&inputs[0], "Boolean gate")?;
    let b = coerce_boolean(&inputs[1], "Boolean gate")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_RESULT.to_owned(), Value::Boolean(operation(a, b)));
    Ok(outputs)
}

fn evaluate_binary_arithmetic(
    inputs: &[Value],
    context: &str,
    operation: impl Fn(f64, f64) -> Result<f64, ComponentError>,
) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(format!(
            "{context} component vereist twee invoerwaarden"
        )));
    }
    let a = coerce_number(&inputs[0], context)?;
    let b = coerce_number(&inputs[1], context)?;
    let result = operation(a, b)?;
    let mut outputs = BTreeMap::new();
    outputs.insert("R".to_owned(), Value::Number(result));
    Ok(outputs)
}

fn coerce_boolean(value: &Value, context: &str) -> Result<bool, ComponentError> {
    match value {
        Value::Boolean(value) => Ok(*value),
        Value::Number(number) => {
            if number.is_nan() {
                Err(ComponentError::new(format!(
                    "{context} verwacht een booleaanse waarde, kreeg NaN"
                )))
            } else {
                Ok(*number != 0.0)
            }
        }
        Value::List(values) if values.len() == 1 => coerce_boolean(&values[0], context),
        other => Err(ComponentError::new(format!(
            "{context} verwacht een booleaanse waarde, kreeg {}",
            other.kind()
        ))),
    }
}

fn coerce_number(value: &Value, context: &str) -> Result<f64, ComponentError> {
    match value {
        Value::Number(number) => {
            if number.is_finite() {
                Ok(*number)
            } else {
                Err(ComponentError::new(format!(
                    "{context} verwacht een eindig getal"
                )))
            }
        }
        Value::Boolean(boolean) => Ok(if *boolean { 1.0 } else { 0.0 }),
        Value::List(values) if values.len() == 1 => coerce_number(&values[0], context),
        other => Err(ComponentError::new(format!(
            "{context} verwacht een numerieke waarde, kreeg {}",
            other.kind()
        ))),
    }
}

fn optional_number(value: Option<&Value>, context: &str) -> Result<Option<f64>, ComponentError> {
    match value {
        Some(value) => coerce_number(value, context).map(Some),
        None => Ok(None),
    }
}

fn to_optional_number(value: Option<&Value>) -> Result<Option<f64>, ComponentError> {
    match value {
        Some(Value::Number(number)) if number.is_finite() => Ok(Some(*number)),
        Some(Value::Boolean(boolean)) => Ok(Some(if *boolean { 1.0 } else { 0.0 })),
        Some(Value::List(values)) if values.len() == 1 => to_optional_number(values.first()),
        Some(_) => Ok(None),
        None => Ok(None),
    }
}

fn to_vector(value: &Value) -> Option<[f64; 3]> {
    match value {
        Value::Point(point) | Value::Vector(point) => Some(*point),
        Value::List(values) if values.len() == 1 => to_vector(&values[0]),
        _ => None,
    }
}

#[derive(Debug, Clone)]
struct MathValue(MathValueKind);

#[derive(Debug, Clone, Copy)]
enum MathValueKind {
    Scalar(f64),
    Vector([f64; 3]),
}

impl MathValue {
    fn scalar(value: f64) -> Self {
        Self(MathValueKind::Scalar(value))
    }

    fn vector(value: [f64; 3]) -> Self {
        Self(MathValueKind::Vector(value))
    }

    fn to_value(&self) -> Value {
        match self.0 {
            MathValueKind::Scalar(value) => Value::Number(value),
            MathValueKind::Vector(value) => Value::Vector(value),
        }
    }
}

fn collect_math_values(values: &[Value]) -> Vec<MathValue> {
    let mut result = Vec::new();
    for value in values {
        collect_math_values_recursive(value, &mut result);
    }
    result
}

fn collect_math_values_recursive(value: &Value, result: &mut Vec<MathValue>) {
    match value {
        Value::List(values) => {
            for entry in values {
                collect_math_values_recursive(entry, result);
            }
        }
        Value::Number(number) => {
            if number.is_finite() {
                result.push(MathValue::scalar(*number));
            }
        }
        Value::Boolean(boolean) => {
            result.push(MathValue::scalar(if *boolean { 1.0 } else { 0.0 }));
        }
        Value::Point(point) | Value::Vector(point) => {
            result.push(MathValue::vector(*point));
        }
        _ => {}
    }
}

fn collect_number_list(value: &Value) -> Vec<f64> {
    let mut result = Vec::new();
    collect_numbers_recursive(value, &mut result);
    result
}

fn collect_numbers_recursive(value: &Value, result: &mut Vec<f64>) {
    match value {
        Value::List(values) => {
            for entry in values {
                collect_numbers_recursive(entry, result);
            }
        }
        Value::Number(number) => {
            if number.is_finite() {
                result.push(*number);
            }
        }
        Value::Boolean(boolean) => result.push(if *boolean { 1.0 } else { 0.0 }),
        _ => {}
    }
}

fn sequential_combine(
    values: Vec<MathValue>,
    mut combine: impl FnMut(MathValue, MathValue) -> Result<MathValue, ComponentError>,
) -> Result<(MathValue, Vec<MathValue>), ComponentError> {
    let mut iter = values.into_iter();
    let mut accumulator = match iter.next() {
        Some(value) => value,
        None => {
            return Err(ComponentError::new(
                "sequential_combine vereist ten minste één waarde",
            ));
        }
    };
    let mut partial = vec![accumulator.clone()];
    for value in iter {
        accumulator = combine(accumulator, value)?;
        partial.push(accumulator.clone());
    }
    Ok((accumulator, partial))
}

fn add_values(left: MathValue, right: MathValue) -> Result<MathValue, ComponentError> {
    match (left.0, right.0) {
        (MathValueKind::Scalar(a), MathValueKind::Scalar(b)) => Ok(MathValue::scalar(a + b)),
        (MathValueKind::Vector(a), MathValueKind::Vector(b)) => {
            Ok(MathValue::vector([a[0] + b[0], a[1] + b[1], a[2] + b[2]]))
        }
        _ => Err(ComponentError::new(
            "Mass Addition verwacht invoer van hetzelfde type",
        )),
    }
}

fn multiply_values(left: MathValue, right: MathValue) -> Result<MathValue, ComponentError> {
    match (left.0, right.0) {
        (MathValueKind::Scalar(a), MathValueKind::Scalar(b)) => Ok(MathValue::scalar(a * b)),
        (MathValueKind::Vector(a), MathValueKind::Vector(b)) => {
            Ok(MathValue::vector([a[0] * b[0], a[1] * b[1], a[2] * b[2]]))
        }
        (MathValueKind::Vector(a), MathValueKind::Scalar(b))
        | (MathValueKind::Scalar(b), MathValueKind::Vector(a)) => {
            Ok(MathValue::vector([a[0] * b, a[1] * b, a[2] * b]))
        }
    }
}

fn subtract_values(left: MathValue, right: MathValue) -> Result<MathValue, ComponentError> {
    match (left.0, right.0) {
        (MathValueKind::Scalar(a), MathValueKind::Scalar(b)) => Ok(MathValue::scalar(a - b)),
        (MathValueKind::Vector(a), MathValueKind::Vector(b)) => {
            Ok(MathValue::vector([a[0] - b[0], a[1] - b[1], a[2] - b[2]]))
        }
        _ => Err(ComponentError::new(
            "Relatieve verschillen verwachten homogeen typematige invoer",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ComponentKind, PIN_DIFFERENCE, PIN_DIFFERENCES, PIN_EQUAL, PIN_NOT_EQUAL,
        PIN_PARTIAL_RESULTS, PIN_RESULT,
    };
    use crate::components::Component;
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn gate_and_evaluates_boolean_inputs() {
        let component = ComponentKind::GateAnd;
        let outputs = component
            .evaluate(
                &[Value::Boolean(true), Value::Boolean(false)],
                &MetaMap::new(),
            )
            .expect("gate and succeeds");
        assert!(matches!(
            outputs.get(PIN_RESULT),
            Some(Value::Boolean(false))
        ));
    }

    #[test]
    fn mass_addition_accumulates_numbers() {
        let component = ComponentKind::MassAddition;
        let inputs = [Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("mass addition succeeds");
        assert!(matches!(
            outputs.get(PIN_RESULT),
            Some(Value::Number(total)) if (*total - 6.0).abs() < 1e-9
        ));
        assert!(matches!(
            outputs.get(PIN_PARTIAL_RESULTS),
            Some(Value::List(partial)) if partial.len() == 3
        ));
    }

    #[test]
    fn relative_differences_compute_sequence() {
        let component = ComponentKind::RelativeDifferences;
        let inputs = [Value::List(vec![
            Value::Number(1.0),
            Value::Number(4.0),
            Value::Number(7.0),
        ])];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("relative differences succeed");
        assert!(matches!(
            outputs.get(PIN_DIFFERENCES),
            Some(Value::List(values)) if values.len() == 2
        ));
    }

    #[test]
    fn equality_handles_vectors() {
        let component = ComponentKind::Equality;
        let inputs = [
            Value::Vector([1.0, 2.0, 3.0]),
            Value::Vector([1.0 + 1e-10, 2.0, 3.0]),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("equality succeeds");
        assert!(matches!(outputs.get(PIN_EQUAL), Some(Value::Boolean(true))));
        assert!(matches!(
            outputs.get(PIN_NOT_EQUAL),
            Some(Value::Boolean(false))
        ));
    }

    #[test]
    fn similarity_returns_difference() {
        let component = ComponentKind::Similarity;
        let inputs = [Value::Number(10.0), Value::Number(12.0), Value::Number(3.0)];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("similarity succeeds");
        assert!(matches!(
            outputs.get(PIN_DIFFERENCE),
            Some(Value::Number(d)) if (*d - 2.0).abs() < 1e-9
        ));
        assert!(matches!(outputs.get(PIN_EQUAL), Some(Value::Boolean(true))));
    }
}
