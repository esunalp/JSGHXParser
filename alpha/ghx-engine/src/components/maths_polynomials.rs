//! Implementaties van Grasshopper "Maths → Polynomials" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_RESULT_Y: &str = "y";
const PIN_RESULT_R: &str = "R";

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Square,
    NaturalLogarithm,
    Logarithm,
    PowerOfTen,
    CubeRoot,
    OneOverX,
    PowerOfTwo,
    LogN,
    Cube,
    SquareRoot,
    PowerOfE,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de maths-polynomial componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{2280dde4-9fa2-4b4a-ae2f-37d554861367}"],
        names: &["Square", "Sqr"],
        kind: ComponentKind::Square,
    },
    Registration {
        guids: &["{23afc7aa-2d2f-4ae7-b876-bf366246b826}"],
        names: &["Natural logarithm", "Ln"],
        kind: ComponentKind::NaturalLogarithm,
    },
    Registration {
        guids: &["{27d6f724-a701-4585-992f-3897488abf08}"],
        names: &["Logarithm", "Log"],
        kind: ComponentKind::Logarithm,
    },
    Registration {
        guids: &["{2ebb82ef-1f90-4ac9-9a71-1fe0f4ef7044}"],
        names: &["Power of 10", "10º"],
        kind: ComponentKind::PowerOfTen,
    },
    Registration {
        guids: &["{4ec88893-1573-4d47-8115-a215bca9b6dc}"],
        names: &["Cube Root", "Crt"],
        kind: ComponentKind::CubeRoot,
    },
    Registration {
        guids: &["{6eb1272b-2b1b-4e43-91d9-6b0623191fd7}"],
        names: &["One Over X", "1/x"],
        kind: ComponentKind::OneOverX,
    },
    Registration {
        guids: &["{6f54a10b-2e5b-4b67-aba5-f1fa89bbf908}"],
        names: &["Power of 2", "2º"],
        kind: ComponentKind::PowerOfTwo,
    },
    Registration {
        guids: &["{7ab8d289-26a2-4dd4-b4ad-df5b477999d8}"],
        names: &["Log N", "LogN"],
        kind: ComponentKind::LogN,
    },
    Registration {
        guids: &["{7e3185eb-a38c-4949-bcf2-0e80dee3a344}"],
        names: &["Cube", "Cube"],
        kind: ComponentKind::Cube,
    },
    Registration {
        guids: &["{ad476cb7-b6d1-41c8-986b-0df243a64146}"],
        names: &["Square Root", "Sqrt"],
        kind: ComponentKind::SquareRoot,
    },
    Registration {
        guids: &["{c717f26f-e4a0-475c-8e1c-b8f77af1bc99}"],
        names: &["Power of E", "Eº"],
        kind: ComponentKind::PowerOfE,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Square => evaluate_unary(inputs, "Square", PIN_RESULT_Y, |x| Ok(x * x)),
            Self::NaturalLogarithm => {
                evaluate_unary(inputs, "Natural logarithm", PIN_RESULT_Y, |x| {
                    if x > 0.0 {
                        Ok(x.ln())
                    } else {
                        Err(ComponentError::new(
                            "Natural logarithm vereist een positieve invoer",
                        ))
                    }
                })
            }
            Self::Logarithm => evaluate_unary(inputs, "Logarithm", PIN_RESULT_Y, |x| {
                if x > 0.0 {
                    Ok(x.log10())
                } else {
                    Err(ComponentError::new(
                        "Logarithm vereist een positieve invoer",
                    ))
                }
            }),
            Self::PowerOfTen => {
                evaluate_unary(inputs, "Power of 10", PIN_RESULT_Y, |x| Ok(10_f64.powf(x)))
            }
            Self::CubeRoot => evaluate_unary(inputs, "Cube Root", PIN_RESULT_Y, |x| Ok(x.cbrt())),
            Self::OneOverX => evaluate_unary(inputs, "One Over X", PIN_RESULT_Y, |x| {
                if x == 0.0 {
                    Err(ComponentError::new("One Over X kan niet delen door nul"))
                } else {
                    Ok(1.0 / x)
                }
            }),
            Self::PowerOfTwo => {
                evaluate_unary(inputs, "Power of 2", PIN_RESULT_Y, |x| Ok(2_f64.powf(x)))
            }
            Self::LogN => evaluate_log_n(inputs),
            Self::Cube => evaluate_unary(inputs, "Cube", PIN_RESULT_Y, |x| Ok(x * x * x)),
            Self::SquareRoot => evaluate_unary(inputs, "Square Root", PIN_RESULT_Y, |x| {
                if x >= 0.0 {
                    Ok(x.sqrt())
                } else {
                    Err(ComponentError::new(
                        "Square Root vereist een niet-negatieve invoer",
                    ))
                }
            }),
            Self::PowerOfE => evaluate_unary(inputs, "Power of E", PIN_RESULT_Y, |x| Ok(x.exp())),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Square => "Square",
            Self::NaturalLogarithm => "Natural logarithm",
            Self::Logarithm => "Logarithm",
            Self::PowerOfTen => "Power of 10",
            Self::CubeRoot => "Cube Root",
            Self::OneOverX => "One Over X",
            Self::PowerOfTwo => "Power of 2",
            Self::LogN => "Log N",
            Self::Cube => "Cube",
            Self::SquareRoot => "Square Root",
            Self::PowerOfE => "Power of E",
        }
    }
}

fn evaluate_unary(
    inputs: &[Value],
    context: &str,
    output_pin: &str,
    operation: impl Fn(f64) -> Result<f64, ComponentError>,
) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(format!(
            "{context} component vereist een invoerwaarde"
        )));
    }
    let value = coerce_number(&inputs[0], context)?;
    let result = operation(value)?;
    ensure_finite(result, context)?;
    let mut outputs = BTreeMap::new();
    outputs.insert(output_pin.to_owned(), Value::Number(result));
    Ok(outputs)
}

fn evaluate_log_n(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Log N component vereist twee invoerwaarden",
        ));
    }
    let value = coerce_number(&inputs[0], "Log N")?;
    let base = coerce_number(&inputs[1], "Log N")?;
    if value <= 0.0 {
        return Err(ComponentError::new("Log N vereist een positieve waarde"));
    }
    if base <= 0.0 || (base - 1.0).abs() < f64::EPSILON {
        return Err(ComponentError::new(
            "Log N vereist een positieve basis ongelijk aan 1",
        ));
    }
    let result = value.log(base);
    ensure_finite(result, "Log N")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_RESULT_R.to_owned(), Value::Number(result));
    Ok(outputs)
}

fn ensure_finite(value: f64, context: &str) -> Result<(), ComponentError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(ComponentError::new(format!(
            "{context} produceerde geen eindig resultaat"
        )))
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
        Value::List(_) => Err(ComponentError::new(format!(
            "{context} verwacht één numerieke waarde per pin"
        ))),
        other => Err(ComponentError::new(format!(
            "{context} verwacht een numerieke waarde, kreeg {}",
            other.kind()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Component, ComponentError, ComponentKind, PIN_RESULT_R, PIN_RESULT_Y, coerce_number,
    };
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn square_computes_power() {
        let component = ComponentKind::Square;
        let outputs = component
            .evaluate(&[Value::Number(3.0)], &MetaMap::new())
            .expect("square succeeded");
        assert!(matches!(
            outputs.get(PIN_RESULT_Y),
            Some(Value::Number(result)) if (*result - 9.0).abs() < 1e-9
        ));
    }

    #[test]
    fn natural_logarithm_rejects_non_positive() {
        let component = ComponentKind::NaturalLogarithm;
        let err = component
            .evaluate(&[Value::Number(-1.0)], &MetaMap::new())
            .unwrap_err();
        assert!(err.message().contains("positieve"));
    }

    #[test]
    fn log_n_works_for_custom_base() {
        let component = ComponentKind::LogN;
        let outputs = component
            .evaluate(&[Value::Number(8.0), Value::Number(2.0)], &MetaMap::new())
            .expect("log n succeeded");
        assert!(matches!(
            outputs.get(PIN_RESULT_R),
            Some(Value::Number(result)) if (*result - 3.0).abs() < 1e-9
        ));
    }

    #[test]
    fn log_n_rejects_invalid_base() {
        let component = ComponentKind::LogN;
        let err = component
            .evaluate(&[Value::Number(8.0), Value::Number(1.0)], &MetaMap::new())
            .unwrap_err();
        assert!(err.message().contains("basis"));
    }

    #[test]
    fn square_root_rejects_negative() {
        let component = ComponentKind::SquareRoot;
        let err = component
            .evaluate(&[Value::Number(-4.0)], &MetaMap::new())
            .unwrap_err();
        assert!(err.message().contains("niet-negatieve"));
    }

    #[test]
    fn one_over_x_rejects_zero() {
        let component = ComponentKind::OneOverX;
        let err = component
            .evaluate(&[Value::Number(0.0)], &MetaMap::new())
            .unwrap_err();
        assert!(err.message().contains("nul"));
    }

    #[test]
    fn coerce_number_rejects_multi_value_lists() {
        let err = coerce_number(
            &Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            "Square",
        )
        .unwrap_err();
        assert!(matches!(err, ComponentError { .. }));
    }
}
