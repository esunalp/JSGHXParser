//! Implementaties van Grasshopper "Maths → Trig" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult, coerce};

const EPSILON: f64 = 1e-9;

const PIN_DEGREES: &str = "D";
const PIN_RADIANS: &str = "R";
const PIN_RESULT: &str = "y";
const PIN_ALPHA: &str = "α";
const PIN_BETA: &str = "β";
const PIN_GAMMA: &str = "γ";
const PIN_A_LENGTH: &str = "A";
const PIN_B_LENGTH: &str = "B";
const PIN_C_LENGTH: &str = "C";
const PIN_P_LENGTH: &str = "P";
const PIN_Q_LENGTH: &str = "Q";
const PIN_R_LENGTH: &str = "R";
const PIN_CENTRE: &str = "C";
const PIN_IN_CENTRE: &str = "I";
const PIN_LINE_AB: &str = "AB";
const PIN_LINE_BC: &str = "BC";
const PIN_LINE_CA: &str = "CA";

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Degrees,
    Radians,
    Sine,
    Cosine,
    Tangent,
    Cotangent,
    Secant,
    Cosecant,
    ArcSine,
    ArcCosine,
    ArcTangent,
    Sinc,
    TriangleTrigonometry,
    RightTrigonometry,
    Circumcentre,
    Orthocentre,
    Centroid,
    Incentre,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de maths-trig componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{0d77c51e-584f-44e8-aed2-c2ddf4803888}"],
        names: &["Degrees", "Deg"],
        kind: ComponentKind::Degrees,
    },
    Registration {
        guids: &["{a4cd2751-414d-42ec-8916-476ebf62d7fe}"],
        names: &["Radians", "Rad"],
        kind: ComponentKind::Radians,
    },
    Registration {
        guids: &["{7663efbb-d9b8-4c6a-a0da-c3750a7bbe77}"],
        names: &["Sine", "Sin"],
        kind: ComponentKind::Sine,
    },
    Registration {
        guids: &["{d2d2a900-780c-4d58-9a35-1f9d8d35df6f}"],
        names: &["Cosine", "Cos"],
        kind: ComponentKind::Cosine,
    },
    Registration {
        guids: &["{0f31784f-7177-4104-8500-1f4f4a306df4}"],
        names: &["Tangent", "Tan"],
        kind: ComponentKind::Tangent,
    },
    Registration {
        guids: &["{1f602c33-f38e-4f47-898b-359f0a4de3c2}"],
        names: &["CoTangent", "Cot"],
        kind: ComponentKind::Cotangent,
    },
    Registration {
        guids: &["{60103def-1bb7-4700-b294-3a89100525c4}"],
        names: &["Secant", "Sec"],
        kind: ComponentKind::Secant,
    },
    Registration {
        guids: &["{d222500b-dfd5-45e0-933e-eabefd07cbfa}"],
        names: &["CoSecant", "Csc"],
        kind: ComponentKind::Cosecant,
    },
    Registration {
        guids: &["{cc15ba56-fae7-4f05-b599-cb7c43b60e11}"],
        names: &["ArcSine", "ASin"],
        kind: ComponentKind::ArcSine,
    },
    Registration {
        guids: &["{49584390-d541-41f7-b5f6-1f9515ac0f73}"],
        names: &["ArcCosine", "ACos"],
        kind: ComponentKind::ArcCosine,
    },
    Registration {
        guids: &["{b4647919-d041-419e-99f5-fa0dc0ddb8b6}"],
        names: &["ArcTangent", "ATan"],
        kind: ComponentKind::ArcTangent,
    },
    Registration {
        guids: &["{a2d9503d-a83c-4d71-81e0-02af8d09cd0c}"],
        names: &["Sinc"],
        kind: ComponentKind::Sinc,
    },
    Registration {
        guids: &["{92af1a02-9b87-43a0-8c45-0ce1b81555ec}"],
        names: &["Triangle Trigonometry", "Trig"],
        kind: ComponentKind::TriangleTrigonometry,
    },
    Registration {
        guids: &["{e75d4624-8ee2-4067-ac8d-c56bdc901d83}"],
        names: &["Right Trigonometry", "RTrig"],
        kind: ComponentKind::RightTrigonometry,
    },
    Registration {
        guids: &["{21d0767c-5340-4087-aa09-398d0e706908}"],
        names: &["Circumcentre", "CCentre", "Circumcenter"],
        kind: ComponentKind::Circumcentre,
    },
    Registration {
        guids: &["{36dd5551-b6bd-4246-bd2f-1fd91eb2f02d}"],
        names: &["Orthocentre", "OCentre", "Orthocenter"],
        kind: ComponentKind::Orthocentre,
    },
    Registration {
        guids: &["{afbcbad4-2a2a-4954-8040-d999e316d2bd}"],
        names: &["Centroid"],
        kind: ComponentKind::Centroid,
    },
    Registration {
        guids: &["{c3342ea2-e181-46aa-a9b9-e438ccbfb831}"],
        names: &["Incentre", "ICentre", "Incenter"],
        kind: ComponentKind::Incentre,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Degrees => evaluate_degrees(inputs),
            Self::Radians => evaluate_radians(inputs),
            Self::Sine => evaluate_simple_trig(inputs, f64::sin),
            Self::Cosine => evaluate_simple_trig(inputs, f64::cos),
            Self::Tangent => evaluate_tangent(inputs),
            Self::Cotangent => evaluate_cotangent(inputs),
            Self::Secant => evaluate_secant(inputs),
            Self::Cosecant => evaluate_cosecant(inputs),
            Self::ArcSine => evaluate_arcsine(inputs),
            Self::ArcCosine => evaluate_arccosine(inputs),
            Self::ArcTangent => evaluate_simple_trig(inputs, f64::atan),
            Self::Sinc => evaluate_sinc(inputs),
            Self::TriangleTrigonometry => evaluate_triangle_trigonometry(inputs),
            Self::RightTrigonometry => evaluate_right_trigonometry(inputs),
            Self::Circumcentre => evaluate_circumcentre(inputs),
            Self::Orthocentre => evaluate_orthocentre(inputs),
            Self::Centroid => evaluate_centroid(inputs),
            Self::Incentre => evaluate_incentre(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Degrees => "Degrees",
            Self::Radians => "Radians",
            Self::Sine => "Sine",
            Self::Cosine => "Cosine",
            Self::Tangent => "Tangent",
            Self::Cotangent => "CoTangent",
            Self::Secant => "Secant",
            Self::Cosecant => "CoSecant",
            Self::ArcSine => "ArcSine",
            Self::ArcCosine => "ArcCosine",
            Self::ArcTangent => "ArcTangent",
            Self::Sinc => "Sinc",
            Self::TriangleTrigonometry => "Triangle Trigonometry",
            Self::RightTrigonometry => "Right Trigonometry",
            Self::Circumcentre => "Circumcentre",
            Self::Orthocentre => "Orthocentre",
            Self::Centroid => "Centroid",
            Self::Incentre => "Incentre",
        }
    }
}

fn evaluate_degrees(inputs: &[Value]) -> ComponentResult {
    let radians = coerce::coerce_number_with_default(inputs.get(0));
    let degrees = radians * 180.0 / std::f64::consts::PI;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_DEGREES.to_owned(), Value::Number(degrees));
    Ok(outputs)
}

fn evaluate_radians(inputs: &[Value]) -> ComponentResult {
    let degrees = coerce::coerce_number_with_default(inputs.get(0));
    let radians = degrees * std::f64::consts::PI / 180.0;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_RADIANS.to_owned(), Value::Number(radians));
    Ok(outputs)
}

fn evaluate_simple_trig(inputs: &[Value], compute: fn(f64) -> f64) -> ComponentResult {
    let numeric = coerce::coerce_number_with_default(inputs.get(0));
    let result = compute(numeric);

    Ok(single_result(result))
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

    Ok(single_result(result))
}

fn evaluate_tangent(inputs: &[Value]) -> ComponentResult {
    let numeric = coerce_number_any(inputs.get(0));
    let result = if let Some(value) = numeric.filter(|value| value.is_finite()) {
        let cos_value = value.cos();
        if cos_value.abs() < EPSILON {
            f64::NAN
        } else {
            value.tan()
        }
    } else {
        0.0
    };

    Ok(single_result(result))
}

fn evaluate_cotangent(inputs: &[Value]) -> ComponentResult {
    let numeric = coerce_number_any(inputs.get(0));
    let result = if let Some(value) = numeric.filter(|value| value.is_finite()) {
        let tan_value = value.tan();
        if tan_value.abs() < EPSILON {
            f64::NAN
        } else {
            1.0 / tan_value
        }
    } else {
        0.0
    };

    Ok(single_result(result))
}

fn evaluate_secant(inputs: &[Value]) -> ComponentResult {
    let numeric = coerce_number_any(inputs.get(0));
    let result = if let Some(value) = numeric.filter(|value| value.is_finite()) {
        let cos_value = value.cos();
        if cos_value.abs() < EPSILON {
            f64::NAN
        } else {
            1.0 / cos_value
        }
    } else {
        0.0
    };

    Ok(single_result(result))
}

fn evaluate_cosecant(inputs: &[Value]) -> ComponentResult {
    let numeric = coerce_number_any(inputs.get(0));
    let result = if let Some(value) = numeric.filter(|value| value.is_finite()) {
        let sin_value = value.sin();
        if sin_value.abs() < EPSILON {
            f64::NAN
        } else {
            1.0 / sin_value
        }
    } else {
        0.0
    };

    Ok(single_result(result))
}

fn evaluate_arcsine(inputs: &[Value]) -> ComponentResult {
    let numeric = coerce_number_any(inputs.get(0));
    let result = if let Some(value) = numeric.filter(|value| value.is_finite()) {
        if (-1.0..=1.0).contains(&value) {
            value.asin()
        } else {
            f64::NAN
        }
    } else {
        0.0
    };

    Ok(single_result(result))
}

fn evaluate_arccosine(inputs: &[Value]) -> ComponentResult {
    let numeric = coerce_number_any(inputs.get(0));
    let result = if let Some(value) = numeric.filter(|value| value.is_finite()) {
        if (-1.0..=1.0).contains(&value) {
            value.acos()
        } else {
            f64::NAN
        }
    } else {
        0.0
    };

    Ok(single_result(result))
}

fn evaluate_triangle_trigonometry(inputs: &[Value]) -> ComponentResult {
    let alpha_input = coerce_number_finite(inputs.get(0));
    let beta_input = coerce_number_finite(inputs.get(1));
    let gamma_input = coerce_number_finite(inputs.get(2));
    let a_input = coerce_positive_length(inputs.get(3));
    let b_input = coerce_positive_length(inputs.get(4));
    let c_input = coerce_positive_length(inputs.get(5));

    let unit = detect_angle_unit(
        &[alpha_input, beta_input, gamma_input],
        std::f64::consts::PI,
    );
    let to_radians = unit.to_radians_factor();
    let from_radians = unit.from_radians_factor();

    let mut state = TriangleState::default();
    state.alpha = alpha_input.map(|value| value * to_radians);
    state.beta = beta_input.map(|value| value * to_radians);
    state.gamma = gamma_input.map(|value| value * to_radians);
    state.a = a_input;
    state.b = b_input;
    state.c = c_input;

    let solution = solve_triangle(state);

    let mut outputs = BTreeMap::new();
    insert_angle_output(&mut outputs, PIN_ALPHA, solution.alpha, from_radians);
    insert_angle_output(&mut outputs, PIN_BETA, solution.beta, from_radians);
    insert_angle_output(&mut outputs, PIN_GAMMA, solution.gamma, from_radians);
    insert_length_output(&mut outputs, PIN_A_LENGTH, solution.a);
    insert_length_output(&mut outputs, PIN_B_LENGTH, solution.b);
    insert_length_output(&mut outputs, PIN_C_LENGTH, solution.c);

    Ok(outputs)
}

fn evaluate_right_trigonometry(inputs: &[Value]) -> ComponentResult {
    let alpha_input = coerce_number_finite(inputs.get(0));
    let beta_input = coerce_number_finite(inputs.get(1));
    let p_input = coerce_positive_length(inputs.get(2));
    let q_input = coerce_positive_length(inputs.get(3));
    let r_input = coerce_positive_length(inputs.get(4));

    let unit = detect_angle_unit(&[alpha_input, beta_input], std::f64::consts::PI / 2.0);
    let to_radians = unit.to_radians_factor();
    let from_radians = unit.from_radians_factor();

    let mut state = RightTriangleState::default();
    state.alpha = alpha_input.map(|value| value * to_radians);
    state.beta = beta_input.map(|value| value * to_radians);
    state.p = p_input;
    state.q = q_input;
    state.r = r_input;

    let solution = solve_right_triangle(state);

    let mut outputs = BTreeMap::new();
    insert_angle_output(&mut outputs, PIN_ALPHA, solution.alpha, from_radians);
    insert_angle_output(&mut outputs, PIN_BETA, solution.beta, from_radians);
    insert_length_output(&mut outputs, PIN_P_LENGTH, solution.p);
    insert_length_output(&mut outputs, PIN_Q_LENGTH, solution.q);
    insert_length_output(&mut outputs, PIN_R_LENGTH, solution.r);

    Ok(outputs)
}

fn evaluate_circumcentre(inputs: &[Value]) -> ComponentResult {
    ensure_input_len(inputs, 3, "Circumcentre")?;
    let a = coerce_point(&inputs[0], "Circumcentre A")?;
    let b = coerce_point(&inputs[1], "Circumcentre B")?;
    let c = coerce_point(&inputs[2], "Circumcentre C")?;

    if let Some(data) = create_triangle_data(a, b, c).and_then(compute_circumcentre_data) {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_CENTRE.to_owned(), Value::Point(data.centre));
        outputs.insert(PIN_LINE_AB.to_owned(), line_value(data.bisector_ab));
        outputs.insert(PIN_LINE_BC.to_owned(), line_value(data.bisector_bc));
        outputs.insert(PIN_LINE_CA.to_owned(), line_value(data.bisector_ca));
        Ok(outputs)
    } else {
        Ok(BTreeMap::new())
    }
}

fn evaluate_orthocentre(inputs: &[Value]) -> ComponentResult {
    ensure_input_len(inputs, 3, "Orthocentre")?;
    let a = coerce_point(&inputs[0], "Orthocentre A")?;
    let b = coerce_point(&inputs[1], "Orthocentre B")?;
    let c = coerce_point(&inputs[2], "Orthocentre C")?;

    if let Some(data) = create_triangle_data(a, b, c).and_then(compute_orthocentre_data) {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_CENTRE.to_owned(), Value::Point(data.orthocentre));
        outputs.insert(PIN_LINE_AB.to_owned(), line_value(data.altitude_ab));
        outputs.insert(PIN_LINE_BC.to_owned(), line_value(data.altitude_bc));
        outputs.insert(PIN_LINE_CA.to_owned(), line_value(data.altitude_ca));
        Ok(outputs)
    } else {
        Ok(BTreeMap::new())
    }
}

fn evaluate_centroid(inputs: &[Value]) -> ComponentResult {
    ensure_input_len(inputs, 3, "Centroid")?;
    let a = coerce_point(&inputs[0], "Centroid A")?;
    let b = coerce_point(&inputs[1], "Centroid B")?;
    let c = coerce_point(&inputs[2], "Centroid C")?;

    if let Some(data) = create_triangle_data(a, b, c).map(compute_centroid_data) {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_CENTRE.to_owned(), Value::Point(data.centroid));
        outputs.insert(PIN_LINE_AB.to_owned(), line_value(data.median_ab));
        outputs.insert(PIN_LINE_BC.to_owned(), line_value(data.median_bc));
        outputs.insert(PIN_LINE_CA.to_owned(), line_value(data.median_ca));
        Ok(outputs)
    } else {
        Ok(BTreeMap::new())
    }
}

fn evaluate_incentre(inputs: &[Value]) -> ComponentResult {
    ensure_input_len(inputs, 3, "Incentre")?;
    let a = coerce_point(&inputs[0], "Incentre A")?;
    let b = coerce_point(&inputs[1], "Incentre B")?;
    let c = coerce_point(&inputs[2], "Incentre C")?;

    if let Some(data) = create_triangle_data(a, b, c).and_then(compute_incentre_data) {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_IN_CENTRE.to_owned(), Value::Point(data.incentre));
        outputs.insert(PIN_A_LENGTH.to_owned(), line_value(data.bisector_a));
        outputs.insert(PIN_B_LENGTH.to_owned(), line_value(data.bisector_b));
        outputs.insert(PIN_C_LENGTH.to_owned(), line_value(data.bisector_c));
        Ok(outputs)
    } else {
        Ok(BTreeMap::new())
    }
}

fn ensure_input_len(
    inputs: &[Value],
    expected: usize,
    context: &str,
) -> Result<(), ComponentError> {
    if inputs.len() < expected {
        Err(ComponentError::new(format!(
            "{context} vereist {expected} invoerwaarden"
        )))
    } else {
        Ok(())
    }
}

fn single_result(value: f64) -> BTreeMap<String, Value> {
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_RESULT.to_owned(), Value::Number(value));
    outputs
}

type Point3 = [f64; 3];

type Line3 = (Point3, Point3);

#[derive(Default, Clone, Copy)]
struct TriangleState {
    alpha: Option<f64>,
    beta: Option<f64>,
    gamma: Option<f64>,
    a: Option<f64>,
    b: Option<f64>,
    c: Option<f64>,
}

#[derive(Default, Clone, Copy)]
struct RightTriangleState {
    alpha: Option<f64>,
    beta: Option<f64>,
    p: Option<f64>,
    q: Option<f64>,
    r: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
struct TriangleData {
    a: Point3,
    b: Point3,
    c: Point3,
    frame: Frame,
    coords: TriangleCoords,
}

#[derive(Debug, Clone, Copy)]
struct Frame {
    origin: Point3,
    x_axis: Point3,
    y_axis: Point3,
}

#[derive(Debug, Clone, Copy)]
struct TriangleCoords {
    a: Point2,
    b: Point2,
    c: Point2,
}

#[derive(Debug, Clone, Copy)]
struct Point2 {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Copy)]
struct CircumcentreData {
    centre: Point3,
    bisector_ab: Line3,
    bisector_bc: Line3,
    bisector_ca: Line3,
}

#[derive(Debug, Clone, Copy)]
struct OrthocentreData {
    orthocentre: Point3,
    altitude_ab: Line3,
    altitude_bc: Line3,
    altitude_ca: Line3,
}

#[derive(Debug, Clone, Copy)]
struct CentroidData {
    centroid: Point3,
    median_ab: Line3,
    median_bc: Line3,
    median_ca: Line3,
}

#[derive(Debug, Clone, Copy)]
struct IncentreData {
    incentre: Point3,
    bisector_a: Line3,
    bisector_b: Line3,
    bisector_c: Line3,
}

#[derive(Debug, Clone, Copy)]
enum AngleUnit {
    Radians,
    Degrees,
}

impl AngleUnit {
    fn to_radians_factor(self) -> f64 {
        match self {
            Self::Radians => 1.0,
            Self::Degrees => std::f64::consts::PI / 180.0,
        }
    }

    fn from_radians_factor(self) -> f64 {
        match self {
            Self::Radians => 1.0,
            Self::Degrees => 180.0 / std::f64::consts::PI,
        }
    }
}

fn detect_angle_unit(values: &[Option<f64>], sum_target: f64) -> AngleUnit {
    let valid: Vec<f64> = values
        .iter()
        .copied()
        .flatten()
        .filter(|value| value.is_finite())
        .collect();

    if valid.is_empty() {
        return AngleUnit::Radians;
    }

    let max_abs = valid
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max);
    if max_abs > sum_target + 0.1 {
        return AngleUnit::Degrees;
    }

    let sum: f64 = valid.iter().map(|value| value.abs()).sum();
    if sum > sum_target + 0.1 {
        AngleUnit::Degrees
    } else {
        AngleUnit::Radians
    }
}

fn solve_triangle(mut state: TriangleState) -> TriangleState {
    for _ in 0..32 {
        let mut changed = false;

        if let (Some(alpha), Some(beta)) = (state.alpha, state.beta) {
            if state.gamma.is_none() {
                changed |= assign_angle(&mut state.gamma, std::f64::consts::PI - alpha - beta);
            }
        }
        if let (Some(alpha), Some(gamma)) = (state.alpha, state.gamma) {
            if state.beta.is_none() {
                changed |= assign_angle(&mut state.beta, std::f64::consts::PI - alpha - gamma);
            }
        }
        if let (Some(beta), Some(gamma)) = (state.beta, state.gamma) {
            if state.alpha.is_none() {
                changed |= assign_angle(&mut state.alpha, std::f64::consts::PI - beta - gamma);
            }
        }

        if let Some(ratio) = compute_sine_ratio(&state) {
            if let Some(alpha) = state.alpha {
                if state.a.is_none() {
                    changed |= assign_length(&mut state.a, alpha.sin() * ratio);
                }
            }
            if let Some(beta) = state.beta {
                if state.b.is_none() {
                    changed |= assign_length(&mut state.b, beta.sin() * ratio);
                }
            }
            if let Some(gamma) = state.gamma {
                if state.c.is_none() {
                    changed |= assign_length(&mut state.c, gamma.sin() * ratio);
                }
            }

            if let Some(a) = state.a {
                if state.alpha.is_none() {
                    changed |= assign_angle(&mut state.alpha, clamp(a / ratio, -1.0, 1.0).asin());
                }
            }
            if let Some(b) = state.b {
                if state.beta.is_none() {
                    changed |= assign_angle(&mut state.beta, clamp(b / ratio, -1.0, 1.0).asin());
                }
            }
            if let Some(c) = state.c {
                if state.gamma.is_none() {
                    changed |= assign_angle(&mut state.gamma, clamp(c / ratio, -1.0, 1.0).asin());
                }
            }
        }

        if state.a.is_none() && is_length(state.b) && is_length(state.c) && is_angle(state.alpha) {
            let alpha = state.alpha.unwrap();
            let b = state.b.unwrap();
            let c = state.c.unwrap();
            let value = (b * b + c * c - 2.0 * b * c * alpha.cos()).max(0.0).sqrt();
            changed |= assign_length(&mut state.a, value);
        }
        if state.b.is_none() && is_length(state.a) && is_length(state.c) && is_angle(state.beta) {
            let beta = state.beta.unwrap();
            let a = state.a.unwrap();
            let c = state.c.unwrap();
            let value = (a * a + c * c - 2.0 * a * c * beta.cos()).max(0.0).sqrt();
            changed |= assign_length(&mut state.b, value);
        }
        if state.c.is_none() && is_length(state.a) && is_length(state.b) && is_angle(state.gamma) {
            let gamma = state.gamma.unwrap();
            let a = state.a.unwrap();
            let b = state.b.unwrap();
            let value = (a * a + b * b - 2.0 * a * b * gamma.cos()).max(0.0).sqrt();
            changed |= assign_length(&mut state.c, value);
        }

        if is_length(state.a) && is_length(state.b) && is_length(state.c) {
            let a = state.a.unwrap();
            let b = state.b.unwrap();
            let c = state.c.unwrap();
            if state.alpha.is_none() {
                let cos_alpha = clamp((b * b + c * c - a * a) / (2.0 * b * c), -1.0, 1.0);
                changed |= assign_angle(&mut state.alpha, cos_alpha.acos());
            }
            if state.beta.is_none() {
                let cos_beta = clamp((a * a + c * c - b * b) / (2.0 * a * c), -1.0, 1.0);
                changed |= assign_angle(&mut state.beta, cos_beta.acos());
            }
            if state.gamma.is_none() {
                let cos_gamma = clamp((a * a + b * b - c * c) / (2.0 * a * b), -1.0, 1.0);
                changed |= assign_angle(&mut state.gamma, cos_gamma.acos());
            }
        }

        if !changed {
            break;
        }
    }

    state
}

fn solve_right_triangle(mut state: RightTriangleState) -> RightTriangleState {
    for _ in 0..32 {
        let mut changed = false;

        if is_angle(state.alpha) && state.beta.is_none() {
            changed |= assign_angle(
                &mut state.beta,
                std::f64::consts::FRAC_PI_2 - state.alpha.unwrap(),
            );
        }
        if is_angle(state.beta) && state.alpha.is_none() {
            changed |= assign_angle(
                &mut state.alpha,
                std::f64::consts::FRAC_PI_2 - state.beta.unwrap(),
            );
        }

        if is_length(state.p) && is_length(state.q) && state.r.is_none() {
            changed |= assign_length(&mut state.r, state.p.unwrap().hypot(state.q.unwrap()));
        }
        if is_length(state.p)
            && is_length(state.r)
            && state.q.is_none()
            && state.r.unwrap() > state.p.unwrap()
        {
            let value = (state.r.unwrap() * state.r.unwrap() - state.p.unwrap() * state.p.unwrap())
                .max(0.0)
                .sqrt();
            changed |= assign_length(&mut state.q, value);
        }
        if is_length(state.q)
            && is_length(state.r)
            && state.p.is_none()
            && state.r.unwrap() > state.q.unwrap()
        {
            let value = (state.r.unwrap() * state.r.unwrap() - state.q.unwrap() * state.q.unwrap())
                .max(0.0)
                .sqrt();
            changed |= assign_length(&mut state.p, value);
        }

        if is_angle(state.alpha) && is_length(state.r) {
            let alpha = state.alpha.unwrap();
            let r = state.r.unwrap();
            if state.p.is_none() {
                changed |= assign_length(&mut state.p, r * alpha.sin());
            }
            if state.q.is_none() {
                changed |= assign_length(&mut state.q, r * alpha.cos());
            }
        }
        if is_angle(state.beta) && is_length(state.r) {
            let beta = state.beta.unwrap();
            let r = state.r.unwrap();
            if state.q.is_none() {
                changed |= assign_length(&mut state.q, r * beta.sin());
            }
            if state.p.is_none() {
                changed |= assign_length(&mut state.p, r * beta.cos());
            }
        }

        if is_angle(state.alpha) && is_length(state.p) && state.r.is_none() {
            changed |= assign_length(&mut state.r, state.p.unwrap() / state.alpha.unwrap().sin());
        }
        if is_angle(state.alpha) && is_length(state.q) && state.r.is_none() {
            changed |= assign_length(&mut state.r, state.q.unwrap() / state.alpha.unwrap().cos());
        }
        if is_angle(state.beta) && is_length(state.p) && state.r.is_none() {
            changed |= assign_length(&mut state.r, state.p.unwrap() / state.beta.unwrap().cos());
        }
        if is_angle(state.beta) && is_length(state.q) && state.r.is_none() {
            changed |= assign_length(&mut state.r, state.q.unwrap() / state.beta.unwrap().sin());
        }

        if is_length(state.p) && is_length(state.q) {
            if state.alpha.is_none() {
                changed |= assign_angle(&mut state.alpha, state.p.unwrap().atan2(state.q.unwrap()));
            }
            if state.beta.is_none() {
                changed |= assign_angle(&mut state.beta, state.q.unwrap().atan2(state.p.unwrap()));
            }
        }
        if is_length(state.p)
            && is_length(state.r)
            && state.alpha.is_none()
            && state.r.unwrap() > EPSILON
        {
            changed |= assign_angle(
                &mut state.alpha,
                clamp(state.p.unwrap() / state.r.unwrap(), -1.0, 1.0).asin(),
            );
        }
        if is_length(state.q)
            && is_length(state.r)
            && state.beta.is_none()
            && state.r.unwrap() > EPSILON
        {
            changed |= assign_angle(
                &mut state.beta,
                clamp(state.q.unwrap() / state.r.unwrap(), -1.0, 1.0).asin(),
            );
        }

        if !changed {
            break;
        }
    }

    if is_angle(state.alpha) && !is_angle(state.beta) {
        state.beta = state.alpha.map(|alpha| std::f64::consts::FRAC_PI_2 - alpha);
    }
    if is_angle(state.beta) && !is_angle(state.alpha) {
        state.alpha = state.beta.map(|beta| std::f64::consts::FRAC_PI_2 - beta);
    }

    state
}

fn compute_sine_ratio(state: &TriangleState) -> Option<f64> {
    let mut ratios = Vec::new();

    if is_angle(state.alpha) && is_length(state.a) {
        let alpha = state.alpha.unwrap();
        let a = state.a.unwrap();
        let sin_alpha = alpha.sin();
        if sin_alpha.abs() > EPSILON {
            ratios.push(a / sin_alpha);
        }
    }

    if is_angle(state.beta) && is_length(state.b) {
        let beta = state.beta.unwrap();
        let b = state.b.unwrap();
        let sin_beta = beta.sin();
        if sin_beta.abs() > EPSILON {
            ratios.push(b / sin_beta);
        }
    }

    if is_angle(state.gamma) && is_length(state.c) {
        let gamma = state.gamma.unwrap();
        let c = state.c.unwrap();
        let sin_gamma = gamma.sin();
        if sin_gamma.abs() > EPSILON {
            ratios.push(c / sin_gamma);
        }
    }

    if ratios.is_empty() {
        return None;
    }

    let valid: Vec<f64> = ratios
        .into_iter()
        .filter(|value| value.is_finite() && value.abs() > EPSILON)
        .collect();
    if valid.is_empty() {
        return None;
    }

    Some(valid.iter().copied().sum::<f64>() / valid.len() as f64)
}

fn assign_angle(slot: &mut Option<f64>, value: f64) -> bool {
    if !value.is_finite() || value <= EPSILON {
        return false;
    }
    if slot.is_none() {
        *slot = Some(value);
        true
    } else {
        false
    }
}

fn assign_length(slot: &mut Option<f64>, value: f64) -> bool {
    if !value.is_finite() || value <= EPSILON {
        return false;
    }
    if slot.is_none() {
        *slot = Some(value);
        true
    } else {
        false
    }
}

fn is_angle(value: Option<f64>) -> bool {
    matches!(value, Some(angle) if angle.is_finite() && angle > EPSILON)
}

fn is_length(value: Option<f64>) -> bool {
    matches!(value, Some(length) if length.is_finite() && length > EPSILON)
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

fn insert_angle_output(
    outputs: &mut BTreeMap<String, Value>,
    pin: &str,
    angle: Option<f64>,
    factor: f64,
) {
    if let Some(angle) = angle.filter(|value| value.is_finite() && *value > EPSILON) {
        outputs.insert(pin.to_owned(), Value::Number(angle * factor));
    }
}

fn insert_length_output(outputs: &mut BTreeMap<String, Value>, pin: &str, length: Option<f64>) {
    if let Some(length) = length.filter(|value| value.is_finite() && *value > EPSILON) {
        outputs.insert(pin.to_owned(), Value::Number(length));
    }
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

fn coerce_number_finite(value: Option<&Value>) -> Option<f64> {
    coerce_number_any(value).filter(|value| value.is_finite())
}

fn coerce_positive_length(value: Option<&Value>) -> Option<f64> {
    coerce_number_finite(value).filter(|value| *value > EPSILON)
}

fn coerce_point(value: &Value, context: &str) -> Result<Point3, ComponentError> {
    match value {
        Value::Point(point) => Ok(*point),
        Value::Vector(vector) => Ok(*vector),
        Value::List(values) if values.len() == 3 => {
            let x = coerce_list_number(values, 0, context)?;
            let y = coerce_list_number(values, 1, context)?;
            let z = coerce_list_number(values, 2, context)?;
            Ok([x, y, z])
        }
        Value::List(values) if values.len() == 1 => coerce_point(&values[0], context),
        other => Err(ComponentError::new(format!(
            "{context} verwacht een puntwaarde, kreeg {}",
            other.kind()
        ))),
    }
}

fn coerce_list_number(
    values: &[Value],
    index: usize,
    context: &str,
) -> Result<f64, ComponentError> {
    values
        .get(index)
        .and_then(|value| coerce_number_any(Some(value)))
        .ok_or_else(|| ComponentError::new(format!("{context} verwacht numerieke coördinaten")))
}

fn create_triangle_data(a: Point3, b: Point3, c: Point3) -> Option<TriangleData> {
    let ab = subtract(b, a);
    let ac = subtract(c, a);
    let normal = cross(ab, ac);
    let area_sq = dot(normal, normal);
    if area_sq < EPSILON {
        return None;
    }

    let ab_length = length(ab);
    if ab_length < EPSILON {
        return None;
    }

    let x_axis = scale(ab, 1.0 / ab_length);
    let z_axis = normalize(normal)?;
    let y_axis = normalize(cross(z_axis, x_axis))?;

    let c_relative = subtract(c, a);
    let c_x = dot(c_relative, x_axis);
    let c_y = dot(c_relative, y_axis);

    Some(TriangleData {
        a,
        b,
        c,
        frame: Frame {
            origin: a,
            x_axis,
            y_axis,
        },
        coords: TriangleCoords {
            a: Point2 { x: 0.0, y: 0.0 },
            b: Point2 {
                x: ab_length,
                y: 0.0,
            },
            c: Point2 { x: c_x, y: c_y },
        },
    })
}

fn compute_circumcentre_data(triangle: TriangleData) -> Option<CircumcentreData> {
    let TriangleData { frame, coords, .. } = triangle;
    let Point2 { x: ax, y: ay } = coords.a;
    let Point2 { x: bx, y: by } = coords.b;
    let Point2 { x: cx, y: cy } = coords.c;

    let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
    if d.abs() < EPSILON {
        return None;
    }

    let ax2ay2 = ax * ax + ay * ay;
    let bx2by2 = bx * bx + by * by;
    let cx2cy2 = cx * cx + cy * cy;

    let ux = (ax2ay2 * (by - cy) + bx2by2 * (cy - ay) + cx2cy2 * (ay - by)) / d;
    let uy = (ax2ay2 * (cx - bx) + bx2by2 * (ax - cx) + cx2cy2 * (bx - ax)) / d;
    let centre = from_2d(&frame, Point2 { x: ux, y: uy });

    let mid_ab = from_2d(&frame, midpoint(coords.a, coords.b));
    let mid_bc = from_2d(&frame, midpoint(coords.b, coords.c));
    let mid_ca = from_2d(&frame, midpoint(coords.c, coords.a));

    Some(CircumcentreData {
        centre,
        bisector_ab: (mid_ab, centre),
        bisector_bc: (mid_bc, centre),
        bisector_ca: (mid_ca, centre),
    })
}

fn compute_orthocentre_data(triangle: TriangleData) -> Option<OrthocentreData> {
    let TriangleData {
        frame,
        coords,
        a,
        b,
        c,
    } = triangle;
    let Point2 { x: bx, .. } = coords.b;
    let Point2 { x: cx, y: cy } = coords.c;
    if cy.abs() < EPSILON {
        return None;
    }

    let ortho_2d = Point2 {
        x: cx,
        y: (cx * (bx - cx)) / cy,
    };
    let orthocentre = from_2d(&frame, ortho_2d);
    let foot_ab = from_2d(&frame, Point2 { x: cx, y: 0.0 });
    let foot_bc = from_2d(&frame, project_point(coords.a, coords.b, coords.c));
    let foot_ca = from_2d(&frame, project_point(coords.b, coords.a, coords.c));

    Some(OrthocentreData {
        orthocentre,
        altitude_ab: (c, foot_ab),
        altitude_bc: (a, foot_bc),
        altitude_ca: (b, foot_ca),
    })
}

fn compute_centroid_data(triangle: TriangleData) -> CentroidData {
    let TriangleData {
        frame,
        coords,
        a,
        b,
        c,
    } = triangle;

    let centroid = add(add(a, b), c);
    let centroid = scale(centroid, 1.0 / 3.0);

    let mid_ab = from_2d(&frame, midpoint(coords.a, coords.b));
    let mid_bc = from_2d(&frame, midpoint(coords.b, coords.c));
    let mid_ca = from_2d(&frame, midpoint(coords.c, coords.a));

    CentroidData {
        centroid,
        median_ab: (c, mid_ab),
        median_bc: (a, mid_bc),
        median_ca: (b, mid_ca),
    }
}

fn compute_incentre_data(triangle: TriangleData) -> Option<IncentreData> {
    let TriangleData { a, b, c, .. } = triangle;

    let side_a = length(subtract(b, c));
    let side_b = length(subtract(a, c));
    let side_c = length(subtract(a, b));
    let perimeter = side_a + side_b + side_c;
    if !perimeter.is_finite() || perimeter < EPSILON {
        return None;
    }

    let incentre = scale(
        add(add(scale(a, side_a), scale(b, side_b)), scale(c, side_c)),
        1.0 / perimeter,
    );

    Some(IncentreData {
        incentre,
        bisector_a: (a, incentre),
        bisector_b: (b, incentre),
        bisector_c: (c, incentre),
    })
}

fn midpoint(a: Point2, b: Point2) -> Point2 {
    Point2 {
        x: (a.x + b.x) / 2.0,
        y: (a.y + b.y) / 2.0,
    }
}

fn project_point(point: Point2, start: Point2, end: Point2) -> Point2 {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let denom = dx * dx + dy * dy;
    if denom < EPSILON {
        return start;
    }
    let t = ((point.x - start.x) * dx + (point.y - start.y) * dy) / denom;
    Point2 {
        x: start.x + dx * t,
        y: start.y + dy * t,
    }
}

fn from_2d(frame: &Frame, point: Point2) -> Point3 {
    let mut result = frame.origin;
    result = add(result, scale(frame.x_axis, point.x));
    result = add(result, scale(frame.y_axis, point.y));
    result
}

fn line_value(line: Line3) -> Value {
    Value::CurveLine {
        p1: line.0,
        p2: line.1,
    }
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(vector: Point3) -> f64 {
    dot(vector, vector).sqrt()
}

fn normalize(vector: Point3) -> Option<Point3> {
    let len = length(vector);
    if len < EPSILON {
        None
    } else {
        Some(scale(vector, 1.0 / len))
    }
}

fn subtract(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: Point3, factor: f64) -> Point3 {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

#[cfg(test)]
mod tests {
    use super::{
        AngleUnit, Component, ComponentKind, coerce_number_any, detect_angle_unit, evaluate_degrees,
    };
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn degrees_converts_radians_to_degrees() {
        let outputs =
            evaluate_degrees(&[Value::Number(std::f64::consts::PI)]).expect("degrees succeeded");
        assert!(matches!(
            outputs.get(super::PIN_DEGREES),
            Some(Value::Number(value)) if (value - 180.0).abs() < 1e-9
        ));
    }

    #[test]
    fn triangle_trigonometry_solves_angles() {
        let component = ComponentKind::TriangleTrigonometry;
        let outputs = component
            .evaluate(
                &[
                    Value::Number(60.0),
                    Value::Number(60.0),
                    Value::List(vec![]),
                    Value::Number(2.0),
                    Value::List(vec![]),
                    Value::List(vec![]),
                ],
                &MetaMap::new(),
            )
            .expect("triangle trig succeeded");
        assert!(matches!(
            outputs.get(super::PIN_GAMMA),
            Some(Value::Number(value)) if (*value - 60.0).abs() < 1e-6
        ));
        assert!(matches!(
            outputs.get(super::PIN_B_LENGTH),
            Some(Value::Number(value)) if (*value - 2.0).abs() < 1e-6
        ));
        assert!(matches!(
            outputs.get(super::PIN_C_LENGTH),
            Some(Value::Number(value)) if (*value - 2.0).abs() < 1e-6
        ));
    }

    #[test]
    fn right_triangle_solves_missing_lengths() {
        let component = ComponentKind::RightTrigonometry;
        let outputs = component
            .evaluate(
                &[
                    Value::Number(30.0),
                    Value::List(vec![]),
                    Value::List(vec![]),
                    Value::List(vec![]),
                    Value::Number(6.0),
                ],
                &MetaMap::new(),
            )
            .expect("right trig succeeded");
        assert!(matches!(
            outputs.get(super::PIN_P_LENGTH),
            Some(Value::Number(value)) if (*value - 3.0).abs() < 1e-6
        ));
        assert!(matches!(
            outputs.get(super::PIN_Q_LENGTH),
            Some(Value::Number(value)) if (*value - 5.196152423).abs() < 1e-6
        ));
        assert!(matches!(
            outputs.get(super::PIN_R_LENGTH),
            Some(Value::Number(value)) if (*value - 6.0).abs() < 1e-6
        ));
    }

    #[test]
    fn circumcentre_returns_bisectors() {
        let component = ComponentKind::Circumcentre;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Point([1.0, 0.0, 0.0]),
                    Value::Point([0.0, 1.0, 0.0]),
                ],
                &MetaMap::new(),
            )
            .expect("circumcentre succeeded");
        assert!(matches!(
            outputs.get(super::PIN_CENTRE),
            Some(Value::Point(point)) if (point[0] - 0.5).abs() < 1e-6 && (point[1] - 0.5).abs() < 1e-6
        ));
        assert!(matches!(
            outputs.get(super::PIN_LINE_AB),
            Some(Value::CurveLine { .. })
        ));
    }

    #[test]
    fn angle_unit_detection_handles_degrees() {
        let unit = detect_angle_unit(&[Some(90.0), None, None], std::f64::consts::PI);
        match unit {
            AngleUnit::Degrees => {}
            _ => panic!("expected degrees"),
        }
    }

    #[test]
    fn coerce_number_any_handles_boolean() {
        assert_eq!(coerce_number_any(Some(&Value::Boolean(true))), Some(1.0));
        assert_eq!(coerce_number_any(Some(&Value::Boolean(false))), Some(0.0));
    }
}

#[test]
fn sine_defaults_to_zero() {
    let component = ComponentKind::Sine;
    let outputs = component
        .evaluate(&[], &MetaMap::new())
        .expect("sine with no inputs succeeds");
    assert!(matches!(outputs.get(PIN_RESULT), Some(Value::Number(r)) if r.abs() < 1e-9));
}

#[test]
fn degrees_defaults_to_zero() {
    let component = ComponentKind::Degrees;
    let outputs = component
        .evaluate(&[], &MetaMap::new())
        .expect("degrees with no inputs succeeds");
    assert!(matches!(outputs.get(PIN_DEGREES), Some(Value::Number(d)) if d.abs() < 1e-9));
}
