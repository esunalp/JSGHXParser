//! Implementaties van de Grasshopper "Maths → Script" expressiecomponenten.

use std::collections::{BTreeMap, HashMap, HashSet};

use meval::{Context, ContextProvider, Expr};
use rand::{Rng, rng};

use crate::graph::node::{MetaMap, MetaValue};
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_RESULT_DEFAULT: &str = "R";
const PIN_VALUE: &str = "V";

#[derive(Debug, Clone, Copy)]
pub struct ExpressionComponent {
    name: &'static str,
    variables: &'static [&'static str],
    output_pins: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Expression(&'static ExpressionComponent),
}

#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

const EXPRESSION_F1: ExpressionComponent = ExpressionComponent {
    name: "F1",
    variables: &["x"],
    output_pins: &[PIN_RESULT_DEFAULT, "r"],
};

const EXPRESSION_F2: ExpressionComponent = ExpressionComponent {
    name: "F2",
    variables: &["x", "y"],
    output_pins: &[PIN_RESULT_DEFAULT, "r"],
};

const EXPRESSION_F3: ExpressionComponent = ExpressionComponent {
    name: "F3",
    variables: &["x", "y", "z"],
    output_pins: &[PIN_RESULT_DEFAULT, "r"],
};

const EXPRESSION_F4: ExpressionComponent = ExpressionComponent {
    name: "F4",
    variables: &["a", "b", "c", "d"],
    output_pins: &[PIN_RESULT_DEFAULT, "r"],
};

const EXPRESSION_F5: ExpressionComponent = ExpressionComponent {
    name: "F5",
    variables: &["a", "b", "c", "d", "x"],
    output_pins: &[PIN_RESULT_DEFAULT, "r"],
};

const EXPRESSION_F6: ExpressionComponent = ExpressionComponent {
    name: "F6",
    variables: &["a", "b", "c", "d", "x", "y"],
    output_pins: &[PIN_RESULT_DEFAULT, "r"],
};

const EXPRESSION_F7: ExpressionComponent = ExpressionComponent {
    name: "F7",
    variables: &["a", "b", "c", "d", "x", "y", "z"],
    output_pins: &[PIN_RESULT_DEFAULT, "r"],
};

const EXPRESSION_F8: ExpressionComponent = ExpressionComponent {
    name: "F8",
    variables: &["a", "b", "c", "d", "w", "x", "y", "z"],
    output_pins: &[PIN_RESULT_DEFAULT, "r"],
};

const EXPRESSION_F1_OBSOLETE: ExpressionComponent = ExpressionComponent {
    name: "F(x) [OBSOLETE]",
    variables: &["x"],
    output_pins: &[PIN_RESULT_DEFAULT, "r", "y"],
};

const EXPRESSION_F2_OBSOLETE: ExpressionComponent = ExpressionComponent {
    name: "F(x,y) [OBSOLETE]",
    variables: &["x", "y"],
    output_pins: &[PIN_RESULT_DEFAULT, "r"],
};

const EXPRESSION_F3_OBSOLETE: ExpressionComponent = ExpressionComponent {
    name: "F(x,y,z) [OBSOLETE]",
    variables: &["x", "y", "z"],
    output_pins: &[PIN_RESULT_DEFAULT, "r"],
};

const EXPRESSION_F4_OBSOLETE: ExpressionComponent = ExpressionComponent {
    name: "F(a,b,c,d) [OBSOLETE]",
    variables: &["a", "b", "c", "d"],
    output_pins: &[PIN_RESULT_DEFAULT, "r"],
};

const EXPRESSION_EVAL: ExpressionComponent = ExpressionComponent {
    name: "Eval [OBSOLETE]",
    variables: &[],
    output_pins: &[PIN_VALUE, "Value"],
};

const EXPRESSION_EVALUATE_OBSOLETE: ExpressionComponent = ExpressionComponent {
    name: "Evaluate Expression [OBSOLETE]",
    variables: &["a", "b", "c", "x", "y", "z"],
    output_pins: &[PIN_VALUE, "Value"],
};

pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{0b7d1129-7b88-4322-aad3-56fd1036a8f6}"],
        names: &["F1", "F(x)"],
        kind: ComponentKind::Expression(&EXPRESSION_F1),
    },
    Registration {
        guids: &["{00ec9ecd-4e1d-45ba-a8fc-dff716dbd9e4}"],
        names: &["F2", "F(x,y)"],
        kind: ComponentKind::Expression(&EXPRESSION_F2),
    },
    Registration {
        guids: &["{2f77b45b-034d-4053-8872-f38d87cbc676}"],
        names: &["F3", "F(x,y,z)"],
        kind: ComponentKind::Expression(&EXPRESSION_F3),
    },
    Registration {
        guids: &["{07efd5e1-d7f4-4205-ab99-83e68175564e}"],
        names: &["F4", "F(a,b,c,d)"],
        kind: ComponentKind::Expression(&EXPRESSION_F4),
    },
    Registration {
        guids: &["{322f0e6e-d434-4d07-9f8d-f214bb248cb1}"],
        names: &["F5", "F(a,b,c,d,x)"],
        kind: ComponentKind::Expression(&EXPRESSION_F5),
    },
    Registration {
        guids: &["{4783b96f-6197-4058-a688-b4ba04c00962}"],
        names: &["F6", "F(a,b,c,d,x,y)"],
        kind: ComponentKind::Expression(&EXPRESSION_F6),
    },
    Registration {
        guids: &["{e9628b21-49d6-4e56-900e-49f4bd4adc85}"],
        names: &["F7", "F(a,b,c,d,x,y,z)"],
        kind: ComponentKind::Expression(&EXPRESSION_F7),
    },
    Registration {
        guids: &["{f2a97ac6-4f11-4c81-834d-50ecd782675c}"],
        names: &["F8", "F(a,b,c,d,w,x,y,z)"],
        kind: ComponentKind::Expression(&EXPRESSION_F8),
    },
    Registration {
        guids: &["{d3e721b4-f5ea-4e40-85fc-b68616939e47}"],
        names: &["F(x) [OBSOLETE]", "F(x) obsolete"],
        kind: ComponentKind::Expression(&EXPRESSION_F1_OBSOLETE),
    },
    Registration {
        guids: &["{d2b10b82-f612-4763-91ca-0cbdbe276171}"],
        names: &["F(x,y) [OBSOLETE]", "F(x,y) obsolete"],
        kind: ComponentKind::Expression(&EXPRESSION_F2_OBSOLETE),
    },
    Registration {
        guids: &["{e1c4bccc-4ecf-4f18-885d-dfd8983e572a}"],
        names: &["F(x,y,z) [OBSOLETE]", "F(x,y,z) obsolete"],
        kind: ComponentKind::Expression(&EXPRESSION_F3_OBSOLETE),
    },
    Registration {
        guids: &["{0f3a13d4-5bb7-499e-9b57-56bb6dce93fd}"],
        names: &["F(a,b,c,d) [OBSOLETE]", "F(a,b,c,d) obsolete"],
        kind: ComponentKind::Expression(&EXPRESSION_F4_OBSOLETE),
    },
    Registration {
        guids: &["{579c9f8c-6fb6-419b-8086-523a2dc99e8a}"],
        names: &["Eval [OBSOLETE]", "Eval"],
        kind: ComponentKind::Expression(&EXPRESSION_EVAL),
    },
    Registration {
        guids: &["{655c5f2f-1e40-42b8-a93a-f05032794449}"],
        names: &["Evaluate Expression [OBSOLETE]", "Evaluate Expression"],
        kind: ComponentKind::Expression(&EXPRESSION_EVALUATE_OBSOLETE),
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Expression(component) => component.evaluate(inputs, meta),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Expression(component) => component.name,
        }
    }
}

impl ExpressionComponent {
    fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new(format!(
                "Component `{}` verwacht ten minste één inputwaarde",
                self.name
            )));
        }

        if inputs.len() < self.variables.len() + 1 {
            return Err(ComponentError::new(format!(
                "Component `{}` verwacht {} variabelen, kreeg {}",
                self.name,
                self.variables.len(),
                inputs.len().saturating_sub(1)
            )));
        }

        let expression_source = expression_from_inputs(inputs)
            .or_else(|| expression_from_meta(meta))
            .ok_or_else(|| {
                ComponentError::new(format!("Component `{}` mist een expressiebron", self.name))
            })?;

        let expression = normalize_expression(&expression_source);
        if expression.is_empty() {
            return Err(ComponentError::new(format!(
                "Component `{}` ontving een lege expressie",
                self.name
            )));
        }

        let expr: Expr = expression.parse().map_err(|error| {
            ComponentError::new(format!(
                "Component `{}` kon expressie niet parsen: {error}",
                self.name
            ))
        })?;

        let mut values = Vec::with_capacity(self.variables.len());
        let mut mapping = HashMap::new();
        for (index, variable) in self.variables.iter().enumerate() {
            let value = coerce_number(&inputs[index + 1], variable)?;
            values.push(value);
            for variant in compute_name_variants(variable) {
                mapping.insert(variant, index);
            }
        }

        let variable_context = VariableContext::new(mapping, values);
        let context = build_context();

        let result = expr
            .eval_with_context((&variable_context, &context))
            .map_err(|error| {
                ComponentError::new(format!(
                    "Component `{}` kon expressie niet evalueren: {error}",
                    self.name
                ))
            })?;

        let mut outputs = BTreeMap::new();
        for pin in deduplicate_pins(self.output_pins) {
            outputs.insert(pin, Value::Number(result));
        }

        Ok(outputs)
    }
}

struct VariableContext {
    mapping: HashMap<String, usize>,
    values: Vec<f64>,
}

impl VariableContext {
    fn new(mapping: HashMap<String, usize>, values: Vec<f64>) -> Self {
        Self { mapping, values }
    }
}

impl ContextProvider for VariableContext {
    fn get_var(&self, name: &str) -> Option<f64> {
        self.mapping
            .get(name)
            .copied()
            .and_then(|index| self.values.get(index))
            .copied()
    }
}

fn expression_from_inputs(inputs: &[Value]) -> Option<String> {
    inputs.get(0).and_then(coerce_expression)
}

fn expression_from_meta(meta: &MetaMap) -> Option<String> {
    const CANDIDATES: &[&str] = &[
        "expression",
        "Expression",
        "expr",
        "Expr",
        "code",
        "Code",
        "formula",
        "Formula",
        "script",
        "Script",
    ];

    for key in CANDIDATES {
        if let Some(value) = meta.get(*key) {
            if let Some(text) = meta_text(value) {
                if !text.trim().is_empty() {
                    return Some(text);
                }
            }
        }
    }

    None
}

fn meta_text(value: &MetaValue) -> Option<String> {
    match value {
        MetaValue::Text(text) => Some(text.clone()),
        MetaValue::List(entries) => entries.iter().find_map(meta_text),
        MetaValue::Number(number) => Some(number.to_string()),
        MetaValue::Integer(integer) => Some(integer.to_string()),
        MetaValue::Boolean(boolean) => Some(boolean.to_string()),
    }
}

fn coerce_expression(value: &Value) -> Option<String> {
    match value {
        Value::Text(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Boolean(boolean) => Some(boolean.to_string()),
        Value::List(values) => values.iter().find_map(coerce_expression),
        Value::Matrix(_)
        | Value::Domain(_)
        | Value::Point(_)
        | Value::Vector(_)
        | Value::CurveLine { .. }
        | Value::Surface { .. }
        | Value::DateTime(_)
        | Value::Complex(_)
        | Value::Tag(_) => None,
    }
}

fn coerce_number(value: &Value, context: &str) -> Result<f64, ComponentError> {
    match value {
        Value::Number(number) => Ok(*number),
        Value::Boolean(boolean) => Ok(if *boolean { 1.0 } else { 0.0 }),
        Value::Text(text) => text.trim().parse::<f64>().map_err(|_| {
            ComponentError::new(format!(
                "{context} verwacht een numerieke waarde, kreeg tekst `{text}`"
            ))
        }),
        Value::List(values) if values.len() == 1 => coerce_number(&values[0], context),
        Value::List(_) => Err(ComponentError::new(format!(
            "{context} verwacht een enkelvoudige waarde"
        ))),
        Value::Complex(_) => Err(ComponentError::new(format!(
            "{context} ondersteunt geen complex getal"
        ))),
        other => Err(ComponentError::new(format!(
            "{context} verwacht een numerieke waarde, kreeg {}",
            other.kind()
        ))),
    }
}

fn compute_name_variants(name: &str) -> Vec<String> {
    let mut variants = HashSet::new();
    let trimmed = name.trim();
    if !trimmed.is_empty() && is_valid_identifier(trimmed) {
        variants.insert(trimmed.to_owned());
    }

    let lowercase = trimmed.to_lowercase();
    if is_valid_identifier(&lowercase) {
        variants.insert(lowercase);
    }

    let uppercase = trimmed.to_uppercase();
    if is_valid_identifier(&uppercase) {
        variants.insert(uppercase);
    }

    if let Some(first) = trimmed.chars().next() {
        let mut capitalized = String::new();
        capitalized.extend(first.to_uppercase());
        capitalized.push_str(&trimmed[first.len_utf8()..]);
        if is_valid_identifier(&capitalized) {
            variants.insert(capitalized);
        }
    }

    variants.into_iter().collect()
}

fn is_valid_identifier(candidate: &str) -> bool {
    let mut chars = candidate.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    }
}

fn normalize_expression(source: &str) -> String {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut normalized = trimmed.replace("<>", "!=");
    while normalized.ends_with(';') {
        normalized.pop();
        normalized = normalized.trim_end().to_owned();
    }

    normalized
}

fn deduplicate_pins(pins: &'static [&'static str]) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result = Vec::new();
    for pin in pins {
        let key = (*pin).to_owned();
        if seen.insert(key.clone()) {
            result.push(key);
        }
    }
    result
}

fn build_context() -> Context<'static> {
    let mut context = Context::new();
    context.func3("clamp", clamp);
    context.func3("lerp", lerp);
    context.func("deg", |value| value.to_degrees());
    context.func("rad", |value| value.to_radians());
    context.func("frac", |value| value.fract());
    context.func2("mod", modulo);
    context.func2("modulo", modulo);
    context.func("sign", f64::signum);
    context.func("sgn", f64::signum);
    context.func("sec", |value| 1.0 / value.cos());
    context.func("csc", |value| 1.0 / value.sin());
    context.func("cot", |value| 1.0 / value.tan());
    context.func2("and", |a, b| {
        if to_boolean(a) && to_boolean(b) {
            1.0
        } else {
            0.0
        }
    });
    context.func2("or", |a, b| {
        if to_boolean(a) || to_boolean(b) {
            1.0
        } else {
            0.0
        }
    });
    context.func2("xor", |a, b| {
        if to_boolean(a) ^ to_boolean(b) {
            1.0
        } else {
            0.0
        }
    });
    context.func("not", |value| if to_boolean(value) { 0.0 } else { 1.0 });
    context.funcn("if", conditional, 2..4);
    context.funcn("select", conditional, 2..4);
    context.funcn("random", random_value, 0..3);
    context.funcn("rand", random_value, 0..3);
    context
}

fn to_boolean(value: f64) -> bool {
    value != 0.0
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    let lower = min.min(max);
    let upper = min.max(max);
    if value <= lower {
        lower
    } else if value >= upper {
        upper
    } else {
        value
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn modulo(dividend: f64, divisor: f64) -> f64 {
    if divisor == 0.0 {
        return f64::NAN;
    }
    let remainder = dividend % divisor;
    if remainder == 0.0 {
        0.0
    } else if (remainder > 0.0) == (divisor > 0.0) {
        remainder
    } else {
        remainder + divisor
    }
}

fn random_value(values: &[f64]) -> f64 {
    let mut rng = rng();
    match values.len() {
        0 => rand::random::<f64>(),
        1 => {
            let end = values[0];
            if end == 0.0 {
                0.0
            } else {
                let (lower, upper) = if end >= 0.0 { (0.0, end) } else { (end, 0.0) };
                if (upper - lower).abs() < f64::EPSILON {
                    lower
                } else {
                    rng.random_range(lower..upper)
                }
            }
        }
        _ => {
            let min = values[0];
            let max = values[1];
            if min == max {
                min
            } else {
                let (lower, upper) = if min < max { (min, max) } else { (max, min) };
                if (upper - lower).abs() < f64::EPSILON {
                    lower
                } else {
                    rng.random_range(lower..upper)
                }
            }
        }
    }
}

fn conditional(values: &[f64]) -> f64 {
    match values.len() {
        2 => {
            if to_boolean(values[0]) {
                values[1]
            } else {
                0.0
            }
        }
        _ => {
            if to_boolean(values[0]) {
                values.get(1).copied().unwrap_or(0.0)
            } else {
                values.get(2).copied().unwrap_or(0.0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentKind, ExpressionComponent, PIN_RESULT_DEFAULT, REGISTRATIONS};
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    const TEST_COMPONENT: ExpressionComponent = ExpressionComponent {
        name: "F2",
        variables: &["x", "y"],
        output_pins: &[PIN_RESULT_DEFAULT, "r"],
    };

    #[test]
    fn evaluates_basic_expression() {
        let component = ComponentKind::Expression(&TEST_COMPONENT);
        let inputs = vec![
            Value::Text("x + y".to_string()),
            Value::Number(2.0),
            Value::Number(3.0),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("expression result");

        let value = outputs.get(PIN_RESULT_DEFAULT).expect("result pin");
        assert!(matches!(value, Value::Number(number) if (*number - 5.0).abs() < 1e-9));
    }

    #[test]
    fn registers_component_metadata() {
        assert!(REGISTRATIONS.iter().any(|registration| {
            matches!(registration.kind, ComponentKind::Expression(component) if component.name == "F1")
        }));
    }
}
