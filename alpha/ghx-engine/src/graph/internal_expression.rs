//! Helper voor het toepassen van interne expressies die op inputs worden ingesteld.

use std::collections::HashMap;
use std::fmt;

use crate::graph::value::Value;
use fasteval::Evaler;
use rand::Rng;
use rand::rng;

/// Fouttype bij het evalueren van een interne expressie.
#[derive(Debug, Clone)]
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

    if let Value::List(entries) = value {
        let mut transformed = Vec::with_capacity(entries.len());
        for entry in entries {
            transformed.push(evaluate_numeric(entry, &normalized)?);
        }
        return Ok(Value::List(transformed));
    }

    evaluate_numeric(value, &normalized)
}

fn evaluate_numeric(
    value: &Value,
    expression: &str,
) -> Result<Value, InternalExpressionError> {
    let scalar = coerce_scalar(value)?;
    
    // Build variable mapping for x, y, z (all map to the same scalar value)
    let mut variables = HashMap::new();
    variables.insert("x".to_owned(), scalar);
    variables.insert("X".to_owned(), scalar);
    variables.insert("y".to_owned(), scalar);
    variables.insert("Y".to_owned(), scalar);
    variables.insert("z".to_owned(), scalar);
    variables.insert("Z".to_owned(), scalar);

    let result = evaluate_with_fasteval(expression, &variables)
        .map_err(|e| InternalExpressionError::Evaluate(e))?;

    Ok(Value::Number(result))
}

/// Evaluate an expression using fasteval with variables and custom functions.
fn evaluate_with_fasteval(
    expression: &str,
    variables: &HashMap<String, f64>,
) -> Result<f64, String> {
    let parser = fasteval::Parser::new();
    let mut slab = fasteval::Slab::new();

    let expr_ref = parser
        .parse(expression, &mut slab.ps)
        .map_err(|e| format!("{e:?}"))?
        .from(&slab.ps);

    // Create namespace callback that handles both variables and custom functions
    let mut ns = |name: &str, args: Vec<f64>| -> Option<f64> {
        // First check if it's a variable
        if let Some(&value) = variables.get(name) {
            return Some(value);
        }

        // Handle custom functions
        match name {
            // Custom math functions not in fasteval builtins
            "clamp" => {
                if args.len() >= 3 {
                    Some(clamp(args[0], args[1], args[2]))
                } else {
                    None
                }
            }
            "lerp" => {
                if args.len() >= 3 {
                    Some(lerp(args[0], args[1], args[2]))
                } else {
                    None
                }
            }
            "deg" => args.first().map(|v| v.to_degrees()),
            "rad" => args.first().map(|v| v.to_radians()),
            "frac" => args.first().map(|v| v.fract()),
            "mod" | "modulo" => {
                if args.len() >= 2 {
                    Some(modulo(args[0], args[1]))
                } else {
                    None
                }
            }
            "sgn" => args.first().map(|v| v.signum()),
            "sec" => args.first().map(|v| 1.0 / v.cos()),
            "csc" => args.first().map(|v| 1.0 / v.sin()),
            "cot" => args.first().map(|v| 1.0 / v.tan()),
            // Boolean functions
            "not" => args.first().map(|v| if to_boolean(*v) { 0.0 } else { 1.0 }),
            "if" | "select" => Some(conditional(&args)),
            "random" | "rand" => Some(random_value(&args)),
            _ => None,
        }
    };

    expr_ref
        .eval(&slab, &mut ns)
        .map_err(|e| format!("{e:?}"))
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
