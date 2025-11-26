//! Helper voor het toepassen van interne expressies die op inputs worden ingesteld.

use std::collections::HashMap;
use std::fmt;

use crate::graph::value::{Value, ValueKind};
use meval::{Context, ContextProvider, Expr};
use rand::Rng;
use rand::rng;

/// Fouttype bij het evalueren van een interne expressie.
#[derive(Debug)]
pub enum InternalExpressionError {
    Parse(String),
    Evaluate(String),
    UnsupportedType(String),
}

impl fmt::Display for InternalExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(reason) => write!(f, "expressie kon niet geparst worden: {reason}"),
            Self::Evaluate(reason) => write!(f, "expressie kon niet geëvalueerd worden: {reason}"),
            Self::UnsupportedType(kind) => {
                write!(f, "expressie ondersteunt type `{kind}` niet")
            }
        }
    }
}

impl std::error::Error for InternalExpressionError {}

/// Past een interne expressie toe op de meegegeven waarde.
pub fn apply_internal_expression(
    value: &Value,
    expression: &str,
) -> Result<Value, InternalExpressionError> {
    let normalized = normalize_expression(expression);
    if normalized.is_empty() {
        return Ok(value.clone());
    }

    if normalized.eq_ignore_ascii_case("-x") {
        if let Some(negated) = unary_negate(value) {
            return Ok(negated);
        }
    }

    let expr: Expr = normalized
        .parse()
        .map_err(|error| InternalExpressionError::Parse(error.to_string()))?;

    let context = build_context();

    if let Value::List(entries) = value {
        let mut transformed = Vec::with_capacity(entries.len());
        for entry in entries {
            transformed.push(evaluate_numeric(entry, &expr, &context)?);
        }
        return Ok(Value::List(transformed));
    }

    evaluate_numeric(value, &expr, &context)
}

fn evaluate_numeric(
    value: &Value,
    expr: &Expr,
    context: &Context,
) -> Result<Value, InternalExpressionError> {
    let scalar = coerce_scalar(value)?;
    let variable_context = ValueVariableContext::from_scalar(scalar);

    let result = expr
        .eval_with_context((&variable_context, context))
        .map_err(|error| InternalExpressionError::Evaluate(error.to_string()))?;

    Ok(Value::Number(result))
}

fn coerce_scalar(value: &Value) -> Result<f64, InternalExpressionError> {
    match value {
        Value::Number(number) => Ok(*number),
        Value::Boolean(state) => Ok(if *state { 1.0 } else { 0.0 }),
        Value::Text(text) => text
            .trim()
            .parse::<f64>()
            .map_err(|_| InternalExpressionError::UnsupportedType(value.kind().to_string())),
        Value::List(items) if items.len() == 1 => coerce_scalar(&items[0]),
        other => Err(InternalExpressionError::UnsupportedType(
            other.kind().to_string(),
        )),
    }
}

fn unary_negate(value: &Value) -> Option<Value> {
    match value {
        Value::Number(n) => Some(Value::Number(-n)),
        Value::Boolean(state) => Some(Value::Number(if *state { -1.0 } else { 0.0 })),
        Value::Point([x, y, z]) => Some(Value::Point([-x, -y, -z])),
        Value::Vector([x, y, z]) => Some(Value::Vector([-x, -y, -z])),
        Value::List(items) => {
            let mut transformed = Vec::with_capacity(items.len());
            for item in items {
                if let Some(negated) = unary_negate(item) {
                    transformed.push(negated);
                } else {
                    return None;
                }
            }
            Some(Value::List(transformed))
        }
        Value::Complex(c) => Some(Value::Complex(-c)),
        _ => None,
    }
}

struct ValueVariableContext {
    mapping: HashMap<String, f64>,
}

impl ValueVariableContext {
    fn from_scalar(value: f64) -> Self {
        let mut mapping = HashMap::new();
        mapping.insert("x".to_owned(), value);
        mapping.insert("X".to_owned(), value);
        mapping.insert("y".to_owned(), value);
        mapping.insert("Y".to_owned(), value);
        mapping.insert("z".to_owned(), value);
        mapping.insert("Z".to_owned(), value);
        Self { mapping }
    }
}

impl ContextProvider for ValueVariableContext {
    fn get_var(&self, name: &str) -> Option<f64> {
        self.mapping.get(name).copied()
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
    } else if (divisor > 0.0 && remainder < 0.0) || (divisor < 0.0 && remainder > 0.0) {
        remainder + divisor
    } else {
        remainder
    }
}

fn conditional(args: &[f64]) -> f64 {
    let condition = to_boolean(args[0]);
    let truthy = args[1];
    let falsy = if args.len() == 3 { args[2] } else { truthy };
    if condition { truthy } else { falsy }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::value::Value;

    #[test]
    fn negates_vector_values() {
        let input = Value::Vector([1.0, -2.0, 3.0]);
        let output = apply_internal_expression(&input, "-x").expect("expression applied");
        assert_eq!(output, Value::Vector([-1.0, 2.0, -3.0]));
    }

    #[test]
    fn applies_numeric_expression_to_scalar() {
        let input = Value::Number(4.0);
        let output = apply_internal_expression(&input, "x * 2 + 1").expect("expression applied");
        match output {
            Value::Number(result) => assert!((result - 9.0).abs() < f64::EPSILON),
            other => panic!("unexpected value {other:?}"),
        }
    }

    #[test]
    fn transforms_list_elements() {
        let input = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let result = apply_internal_expression(&input, "x + 1").expect("expression applied");
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(items[0], Value::Number(n) if (n - 2.0).abs() < f64::EPSILON));
                assert!(matches!(items[1], Value::Number(n) if (n - 3.0).abs() < f64::EPSILON));
            }
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unsupported_value() {
        let input = Value::Point([0.0, 1.0, 2.0]);
        let error = apply_internal_expression(&input, "x + 1").unwrap_err();
        assert!(matches!(
            error,
            InternalExpressionError::UnsupportedType(ref kind) if kind == "Point"
        ));
    }
}
