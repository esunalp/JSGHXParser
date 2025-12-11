//! Implementaties van Grasshopper "Vector → Vector" componenten.

use std::collections::BTreeMap;
use std::f64::consts::PI;

use time::PrimitiveDateTime;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::coerce::Plane;
use super::{Component, ComponentError, ComponentResult, coerce};

const EPSILON: f64 = 1e-9;

const PIN_OUTPUT_ANGLE: &str = "A";
const PIN_OUTPUT_REFLEX: &str = "R";
const PIN_OUTPUT_VECTOR: &str = "V";
const PIN_OUTPUT_LENGTH: &str = "L";
const PIN_OUTPUT_DOT: &str = "D";
const PIN_OUTPUT_X: &str = "X";
const PIN_OUTPUT_Y: &str = "Y";
const PIN_OUTPUT_Z: &str = "Z";
const PIN_OUTPUT_DIRECTION: &str = "D";
const PIN_OUTPUT_ELEVATION: &str = "E";
const PIN_OUTPUT_HORIZON: &str = "H";
const PIN_OUTPUT_COLOUR: &str = "C";

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Angle,
    AnglePlane,
    CrossProduct,
    Divide,
    DotProduct,
    VectorXyz,
    SolarIncidence,
    MassAddition,
    MassAdditionTotal,
    Multiply,
    VectorLength,
    Amplitude,
    UnitX,
    UnitY,
    UnitZ,
    VectorTwoPoint,
    DeconstructVector,
    Rotate,
    UnitVector,
    Reverse,
    Addition,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de vector-vector componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{152a264e-fc74-40e5-88cc-d1a681cd09c3}"],
        names: &["Angle"],
        kind: ComponentKind::Angle,
    },
    Registration {
        guids: &["{b464fccb-50e7-41bd-9789-8438db9bea9f}"],
        names: &["Angle", "Angle Plane"],
        kind: ComponentKind::AnglePlane,
    },
    Registration {
        guids: &["{2a5cfb31-028a-4b34-b4e1-9b20ae15312e}"],
        names: &["Cross Product", "XProd"],
        kind: ComponentKind::CrossProduct,
    },
    Registration {
        guids: &["{310e1065-d03a-4858-bcd1-809d39c042af}"],
        names: &["Divide", "VDiv"],
        kind: ComponentKind::Divide,
    },
    Registration {
        guids: &["{43b9ea8f-f772-40f2-9880-011a9c3cbbb0}"],
        names: &["Dot Product", "DProd"],
        kind: ComponentKind::DotProduct,
    },
    Registration {
        guids: &["{56b92eab-d121-43f7-94d3-6cd8f0ddead8}"],
        names: &["Vector XYZ", "Vec"],
        kind: ComponentKind::VectorXyz,
    },
    Registration {
        guids: &["{59e1f848-38d4-4cbf-ad7f-40ffc52acdf5}"],
        names: &["Solar Incidence", "Solar"],
        kind: ComponentKind::SolarIncidence,
    },
    Registration {
        guids: &["{63f79e72-36c0-4489-a0c2-9ded0b9ca41f}"],
        names: &["Mass Addition", "MassAdd"],
        kind: ComponentKind::MassAddition,
    },
    Registration {
        guids: &["{b7f1178f-4222-47fd-9766-5d06e869362b}"],
        names: &["Mass Addition Total"],
        kind: ComponentKind::MassAdditionTotal,
    },
    Registration {
        guids: &["{63fff845-7c61-4dfb-ba12-44d481b4bf0f}"],
        names: &["Multiply", "VMul"],
        kind: ComponentKind::Multiply,
    },
    Registration {
        guids: &["{675e31bf-1775-48d7-bb8d-76b77786dd53}"],
        names: &["Vector Length", "VLen"],
        kind: ComponentKind::VectorLength,
    },
    Registration {
        guids: &["{6ec39468-dae7-4ffa-a766-f2ab22a2c62e}"],
        names: &["Amplitude", "Amp"],
        kind: ComponentKind::Amplitude,
    },
    Registration {
        guids: &["{79f9fbb3-8f1d-4d9a-88a9-f7961b1012cd}"],
        names: &["Unit X", "X"],
        kind: ComponentKind::UnitX,
    },
    Registration {
        guids: &["{d3d195ea-2d59-4ffa-90b1-8b7ff3369f69}"],
        names: &["Unit Y", "Y"],
        kind: ComponentKind::UnitY,
    },
    Registration {
        guids: &["{9103c240-a6a9-4223-9b42-dbd19bf38e2b}"],
        names: &["Unit Z", "Z"],
        kind: ComponentKind::UnitZ,
    },
    Registration {
        guids: &["{934ede4a-924a-4973-bb05-0dc4b36fae75}"],
        names: &["Vector 2Pt", "Vec2Pt"],
        kind: ComponentKind::VectorTwoPoint,
    },
    Registration {
        guids: &["{a50fcd4a-cf42-4c3f-8616-022761e6cc93}"],
        names: &["Deconstruct Vector", "DeVec"],
        kind: ComponentKind::DeconstructVector,
    },
    Registration {
        guids: &["{b6d7ba20-cf74-4191-a756-2216a36e30a7}"],
        names: &["Rotate", "VRot"],
        kind: ComponentKind::Rotate,
    },
    Registration {
        guids: &["{d2da1306-259a-4994-85a4-672d8a4c7805}"],
        names: &["Unit Vector", "Unit"],
        kind: ComponentKind::UnitVector,
    },
    Registration {
        guids: &["{d5788074-d75d-4021-b1a3-0bf992928584}"],
        names: &["Reverse", "Rev"],
        kind: ComponentKind::Reverse,
    },
    Registration {
        guids: &["{fb012ef9-4734-4049-84a0-b92b85bb09da}"],
        names: &["Vector Addition", "VAdd"],
        kind: ComponentKind::Addition,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Angle => evaluate_angle(inputs),
            Self::AnglePlane => evaluate_angle_plane(inputs),
            Self::CrossProduct => evaluate_cross_product(inputs),
            Self::Divide => evaluate_divide(inputs),
            Self::DotProduct => evaluate_dot_product(inputs),
            Self::VectorXyz => evaluate_vector_xyz(inputs),
            Self::SolarIncidence => evaluate_solar_incidence(inputs),
            Self::MassAddition => evaluate_mass_addition(inputs),
            Self::MassAdditionTotal => evaluate_mass_addition_total(inputs),
            Self::Multiply => evaluate_multiply(inputs),
            Self::VectorLength => evaluate_vector_length(inputs),
            Self::Amplitude => evaluate_amplitude(inputs),
            Self::UnitX => evaluate_unit_axis(inputs, [1.0, 0.0, 0.0], "Unit X"),
            Self::UnitY => evaluate_unit_axis(inputs, [0.0, 1.0, 0.0], "Unit Y"),
            Self::UnitZ => evaluate_unit_axis(inputs, [0.0, 0.0, 1.0], "Unit Z"),
            Self::VectorTwoPoint => evaluate_vector_two_point(inputs),
            Self::DeconstructVector => evaluate_deconstruct(inputs),
            Self::Rotate => evaluate_rotate(inputs),
            Self::UnitVector => evaluate_unit_vector(inputs),
            Self::Reverse => evaluate_reverse(inputs),
            Self::Addition => evaluate_addition(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Angle => "Vector Angle",
            Self::AnglePlane => "Vector Angle (Plane)",
            Self::CrossProduct => "Cross Product",
            Self::Divide => "Vector Divide",
            Self::DotProduct => "Dot Product",
            Self::VectorXyz => "Vector XYZ",
            Self::SolarIncidence => "Solar Incidence",
            Self::MassAddition => "Mass Addition",
            Self::MassAdditionTotal => "Mass Addition Total",
            Self::Multiply => "Vector Multiply",
            Self::VectorLength => "Vector Length",
            Self::Amplitude => "Amplitude",
            Self::UnitX => "Unit X",
            Self::UnitY => "Unit Y",
            Self::UnitZ => "Unit Z",
            Self::VectorTwoPoint => "Vector 2Pt",
            Self::DeconstructVector => "Deconstruct Vector",
            Self::Rotate => "Vector Rotate",
            Self::UnitVector => "Unit Vector",
            Self::Reverse => "Vector Reverse",
            Self::Addition => "Vector Addition",
        }
    }
}

fn evaluate_angle(inputs: &[Value]) -> ComponentResult {
    let a = coerce::coerce_vector_with_default(inputs.get(0));
    let b = coerce::coerce_vector_with_default(inputs.get(1));
    let (angle, reflex) = compute_angle_3d(a, b);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_ANGLE.to_owned(), Value::Number(angle));
    outputs.insert(PIN_OUTPUT_REFLEX.to_owned(), Value::Number(reflex));
    Ok(outputs)
}

fn evaluate_angle_plane(inputs: &[Value]) -> ComponentResult {
    let a = coerce::coerce_vector_with_default(inputs.get(0));
    let b = coerce::coerce_vector_with_default(inputs.get(1));
    let plane = match inputs.get(2) {
        Some(&Value::Null) | None => Plane::default(),
        Some(value) => coerce::coerce_plane(value, "Vector Angle (Plane)")?,
    };
    let (angle, reflex) = compute_angle_on_plane(a, b, &plane);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_ANGLE.to_owned(), Value::Number(angle));
    outputs.insert(PIN_OUTPUT_REFLEX.to_owned(), Value::Number(reflex));
    Ok(outputs)
}

fn evaluate_cross_product(inputs: &[Value]) -> ComponentResult {
    let a = coerce::coerce_vector_with_default(inputs.get(0));
    let b = coerce::coerce_vector_with_default(inputs.get(1));
    let unitize = coerce::coerce_boolean_with_default(inputs.get(2));

    let mut cross = cross(a, b);
    let length = vector_length(cross);
    if unitize {
        if length > EPSILON {
            cross = scale(cross, 1.0 / length);
        } else {
            cross = [0.0, 0.0, 0.0];
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_VECTOR.to_owned(), Value::Vector(cross));
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(length));
    Ok(outputs)
}

fn evaluate_divide(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Vector Divide vereist een vector en een factor",
        ));
    }

    let vector = coerce::coerce_vector(&inputs[0], "Vector Divide")?;
    let factor = coerce::coerce_number(&inputs[1], Some("Vector Divide"))?;
    let result = if factor.abs() < EPSILON {
        [0.0, 0.0, 0.0]
    } else {
        scale(vector, 1.0 / factor)
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_VECTOR.to_owned(), Value::Vector(result));
    outputs.insert(
        PIN_OUTPUT_LENGTH.to_owned(),
        Value::Number(vector_length(result)),
    );
    Ok(outputs)
}

fn evaluate_dot_product(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Dot Product vereist twee vectoren"));
    }

    let mut a = coerce::coerce_vector(&inputs[0], "Dot Product")?;
    let mut b = coerce::coerce_vector(&inputs[1], "Dot Product")?;
    let unitize = inputs
        .get(2)
        .map(|value| coerce::coerce_boolean(value))
        .transpose()?
        .unwrap_or(false);

    if unitize {
        if let Some((normalized, _)) = safe_normalized(a) {
            a = normalized;
        } else {
            return Ok(single_output(PIN_OUTPUT_DOT, Value::Number(0.0)));
        }

        if let Some((normalized, _)) = safe_normalized(b) {
            b = normalized;
        } else {
            return Ok(single_output(PIN_OUTPUT_DOT, Value::Number(0.0)));
        }
    }

    let dot = dot(a, b);
    Ok(single_output(PIN_OUTPUT_DOT, Value::Number(dot)))
}

fn evaluate_vector_xyz(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new("Vector XYZ vereist drie scalars"));
    }

    let x = coerce::coerce_number(&inputs[0], Some("Vector XYZ"))?;
    let y = coerce::coerce_number(&inputs[1], Some("Vector XYZ"))?;
    let z = coerce::coerce_number(&inputs[2], Some("Vector XYZ"))?;
    let vector = [x, y, z];

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_VECTOR.to_owned(), Value::Vector(vector));
    outputs.insert(
        PIN_OUTPUT_LENGTH.to_owned(),
        Value::Number(vector_length(vector)),
    );
    Ok(outputs)
}

fn evaluate_solar_incidence(inputs: &[Value]) -> ComponentResult {
    let location = inputs
        .get(0)
        .map(|value| coerce::coerce_geo_location(value, "Solar Incidence"))
        .transpose()?
        .unwrap_or((0.0, 0.0));

    let datetime = inputs
        .get(1)
        .map(coerce::coerce_date_time)
        .unwrap_or_else(coerce::default_datetime);

    let plane = inputs
        .get(2)
        .or_else(|| inputs.get(3))
        .map(|value| coerce::coerce_plane(value, "Solar Incidence"))
        .transpose()?
        .unwrap_or_default();

    let (direction, elevation, horizon) = compute_solar_data(datetime, location, &plane);
    let colour = color_for_elevation(elevation);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_DIRECTION.to_owned(), Value::Vector(direction));
    outputs.insert(PIN_OUTPUT_ELEVATION.to_owned(), Value::Number(elevation));
    outputs.insert(PIN_OUTPUT_HORIZON.to_owned(), Value::Boolean(horizon));
    outputs.insert(PIN_OUTPUT_COLOUR.to_owned(), Value::Vector(colour));
    Ok(outputs)
}

fn evaluate_mass_addition(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Mass Addition vereist minimaal een vectorlijst",
        ));
    }

    let vectors = coerce::coerce_vector_list(&inputs[0], "Mass Addition")?;
    let unitize = inputs
        .get(1)
        .map(|value| coerce::coerce_boolean(value))
        .transpose()?
        .unwrap_or(false);

    let vector = sum_vectors(&vectors, unitize);
    let length = vector_length(vector);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_VECTOR.to_owned(), Value::Vector(vector));
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(length));
    Ok(outputs)
}

fn evaluate_mass_addition_total(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Mass Addition vereist minimaal een vectorlijst",
        ));
    }

    let vectors = coerce::coerce_vector_list(&inputs[0], "Mass Addition Total")?;
    let unitize = inputs
        .get(1)
        .map(|value| coerce::coerce_boolean(value))
        .transpose()?
        .unwrap_or(false);

    let vector = sum_vectors(&vectors, unitize);
    Ok(single_output(PIN_OUTPUT_VECTOR, Value::Vector(vector)))
}

fn evaluate_multiply(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Vector Multiply vereist een vector en factor",
        ));
    }

    let vector = coerce::coerce_vector(&inputs[0], "Vector Multiply")?;
    let factor = coerce::coerce_number(&inputs[1], Some("Vector Multiply"))?;
    let result = scale(vector, factor);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_VECTOR.to_owned(), Value::Vector(result));
    outputs.insert(
        PIN_OUTPUT_LENGTH.to_owned(),
        Value::Number(vector_length(result)),
    );
    Ok(outputs)
}

fn evaluate_vector_length(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Vector Length vereist een vector"));
    }

    let vector = coerce::coerce_vector(&inputs[0], "Vector Length")?;
    Ok(single_output(
        PIN_OUTPUT_LENGTH,
        Value::Number(vector_length(vector)),
    ))
}

fn evaluate_amplitude(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Amplitude vereist een vector en amplitude",
        ));
    }

    let vector = coerce::coerce_vector(&inputs[0], "Amplitude")?;
    let amplitude = coerce::coerce_number(&inputs[1], Some("Amplitude"))?;
    if let Some((normalized, _)) = safe_normalized(vector) {
        Ok(single_output(
            PIN_OUTPUT_VECTOR,
            Value::Vector(scale(normalized, amplitude)),
        ))
    } else {
        Ok(single_output(
            PIN_OUTPUT_VECTOR,
            Value::Vector([0.0, 0.0, 0.0]),
        ))
    }
}

fn evaluate_unit_axis(inputs: &[Value], axis: [f64; 3], name: &str) -> ComponentResult {
    let Some(first) = inputs.get(0) else {
        return Ok(single_output(
            PIN_OUTPUT_VECTOR,
            Value::Vector(scale(axis, 1.0)),
        ));
    };

    match first {
        Value::Null => Ok(single_output(
            PIN_OUTPUT_VECTOR,
            Value::Vector(scale(axis, 1.0)),
        )),
        Value::List(items) => {
            if items.is_empty() {
                return Ok(single_output(PIN_OUTPUT_VECTOR, Value::List(Vec::new())));
            }

            let mut vectors = Vec::with_capacity(items.len());
            for item in items {
                let factor = coerce::coerce_number(item, Some(name))?;
                vectors.push(Value::Vector(scale(axis, factor)));
            }
            Ok(single_output(PIN_OUTPUT_VECTOR, Value::List(vectors)))
        }
        other => {
            let factor = coerce::coerce_number(other, Some(name))?;
            Ok(single_output(
                PIN_OUTPUT_VECTOR,
                Value::Vector(scale(axis, factor)),
            ))
        }
    }
}

fn evaluate_vector_two_point(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Vector 2Pt vereist twee punten"));
    }

    let a = coerce::coerce_point_with_context(&inputs[0], "Vector 2Pt")?;
    let b = coerce::coerce_point_with_context(&inputs[1], "Vector 2Pt")?;
    let unitize = inputs
        .get(2)
        .map(|value| coerce::coerce_boolean(value))
        .transpose()?
        .unwrap_or(false);

    let mut vector = subtract(b, a);
    let length = vector_length(vector);
    if unitize {
        if length > EPSILON {
            vector = scale(vector, 1.0 / length);
        } else {
            vector = [0.0, 0.0, 0.0];
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_VECTOR.to_owned(), Value::Vector(vector));
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(length));
    Ok(outputs)
}

fn evaluate_deconstruct(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Deconstruct Vector vereist een vector"));
    }

    let vector = coerce::coerce_vector(&inputs[0], "Deconstruct Vector")?;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_X.to_owned(), Value::Number(vector[0]));
    outputs.insert(PIN_OUTPUT_Y.to_owned(), Value::Number(vector[1]));
    outputs.insert(PIN_OUTPUT_Z.to_owned(), Value::Number(vector[2]));
    Ok(outputs)
}

fn evaluate_rotate(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Vector Rotate vereist minimaal een vector",
        ));
    }

    let vector = coerce::coerce_vector(&inputs[0], "Vector Rotate")?;
    let axis = inputs
        .get(1)
        .map(|value| coerce::coerce_vector(value, "Vector Rotate"))
        .transpose()?
        .unwrap_or([0.0, 0.0, 1.0]);
    let angle = inputs
        .get(2)
        .map(|value| coerce::coerce_number(value, Some("Vector Rotate")))
        .transpose()?
        .unwrap_or(0.0);

    Ok(single_output(
        PIN_OUTPUT_VECTOR,
        Value::Vector(rotate(vector, axis, angle)),
    ))
}

fn evaluate_unit_vector(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Unit Vector vereist een vector"));
    }

    let vector = coerce::coerce_vector(&inputs[0], "Unit Vector")?;
    if let Some((normalized, _)) = safe_normalized(vector) {
        Ok(single_output(PIN_OUTPUT_VECTOR, Value::Vector(normalized)))
    } else {
        Ok(single_output(
            PIN_OUTPUT_VECTOR,
            Value::Vector([0.0, 0.0, 0.0]),
        ))
    }
}

fn evaluate_reverse(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Vector Reverse vereist een vector"));
    }

    let vector = coerce::coerce_vector(&inputs[0], "Vector Reverse")?;
    Ok(single_output(
        PIN_OUTPUT_VECTOR,
        Value::Vector(scale(vector, -1.0)),
    ))
}

fn evaluate_addition(inputs: &[Value]) -> ComponentResult {
    let a = coerce::coerce_vector_with_default(inputs.get(0));
    let b = coerce::coerce_vector_with_default(inputs.get(1));
    let unitize = coerce::coerce_boolean_with_default(inputs.get(2));

    let mut vector = add(a, b);
    if unitize {
        if let Some((normalized, _)) = safe_normalized(vector) {
            vector = normalized;
        } else {
            vector = [0.0, 0.0, 0.0];
        }
    }
    let length = vector_length(vector);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_VECTOR.to_owned(), Value::Vector(vector));
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(length));
    Ok(outputs)
}
fn single_output(pin: &str, value: Value) -> BTreeMap<String, Value> {
    let mut outputs = BTreeMap::new();
    outputs.insert(pin.to_owned(), value);
    outputs
}

fn compute_angle_3d(a: [f64; 3], b: [f64; 3]) -> (f64, f64) {
    let length_a = vector_length(a);
    let length_b = vector_length(b);
    if length_a < EPSILON || length_b < EPSILON {
        return (0.0, 0.0);
    }
    let normalized_dot = clamp_to_unit(dot(a, b) / (length_a * length_b));
    let angle = normalized_dot.acos();
    (angle, 2.0 * PI - angle)
}

fn compute_angle_on_plane(a: [f64; 3], b: [f64; 3], plane: &Plane) -> (f64, f64) {
    let projected_a = [dot(a, plane.x_axis), dot(a, plane.y_axis)];
    let projected_b = [dot(b, plane.x_axis), dot(b, plane.y_axis)];
    let mag_a = projected_a[0].hypot(projected_a[1]);
    let mag_b = projected_b[0].hypot(projected_b[1]);
    if mag_a < EPSILON || mag_b < EPSILON {
        return compute_angle_3d(a, b);
    }

    let angle_a = projected_a[1].atan2(projected_a[0]);
    let angle_b = projected_b[1].atan2(projected_b[0]);
    let mut delta = angle_b - angle_a;
    while delta < 0.0 {
        delta += 2.0 * PI;
    }
    while delta >= 2.0 * PI {
        delta -= 2.0 * PI;
    }
    let reflex = if delta <= EPSILON {
        0.0
    } else {
        2.0 * PI - delta
    };
    (delta, reflex)
}

fn sum_vectors(vectors: &[[f64; 3]], unitize: bool) -> [f64; 3] {
    let mut sum = [0.0, 0.0, 0.0];
    for vector in vectors {
        if unitize {
            if let Some((normalized, _)) = safe_normalized(*vector) {
                sum = add(sum, normalized);
            }
        } else {
            sum = add(sum, *vector);
        }
    }
    sum
}

fn rotate(vector: [f64; 3], axis: [f64; 3], angle: f64) -> [f64; 3] {
    if let Some((unit_axis, _)) = safe_normalized(axis) {
        let cos = angle.cos();
        let sin = angle.sin();
        let term1 = scale(vector, cos);
        let term2 = scale(cross(unit_axis, vector), sin);
        let term3 = scale(unit_axis, dot(unit_axis, vector) * (1.0 - cos));
        add(add(term1, term2), term3)
    } else {
        vector
    }
}

fn compute_solar_data(
    datetime: PrimitiveDateTime,
    location: (f64, f64),
    plane: &Plane,
) -> ([f64; 3], f64, bool) {
    let (longitude_deg, latitude_deg) = location;
    let lat_rad = latitude_deg.to_radians();

    let date = datetime.date();
    let time = datetime.time();
    let day_of_year = date.ordinal() as f64;
    let minutes =
        f64::from(time.hour()) * 60.0 + f64::from(time.minute()) + f64::from(time.second()) / 60.0;

    let gamma = (2.0 * PI / 365.0) * (day_of_year - 1.0 + (minutes / 60.0 - 12.0) / 24.0);

    let equation_of_time = 229.18
        * (0.000075 + 0.001868 * gamma.cos()
            - 0.032077 * gamma.sin()
            - 0.014615 * (2.0 * gamma).cos()
            - 0.040849 * (2.0 * gamma).sin());

    let declination = 0.006918 - 0.399912 * gamma.cos() + 0.070257 * gamma.sin()
        - 0.006758 * (2.0 * gamma).cos()
        + 0.000907 * (2.0 * gamma).sin()
        - 0.002697 * (3.0 * gamma).cos()
        + 0.00148 * (3.0 * gamma).sin();

    let time_offset = equation_of_time + 4.0 * longitude_deg;
    let mut true_solar_time = minutes + time_offset;
    true_solar_time = ((true_solar_time % 1440.0) + 1440.0) % 1440.0;

    let mut hour_angle_deg = true_solar_time / 4.0 - 180.0;
    if hour_angle_deg < -180.0 {
        hour_angle_deg += 360.0;
    }
    let hour_angle = hour_angle_deg.to_radians();

    let cos_zenith = clamp_to_unit(
        lat_rad.sin() * declination.sin() + lat_rad.cos() * declination.cos() * hour_angle.cos(),
    );
    let zenith = cos_zenith.acos();
    let elevation = PI / 2.0 - zenith;

    let mut azimuth = 0.0;
    let sin_zenith = zenith.sin();
    if sin_zenith >= EPSILON {
        let azimuth_cos = clamp_to_unit(
            (lat_rad.sin() * zenith.cos() - declination.sin()) / (lat_rad.cos() * sin_zenith),
        );
        azimuth = azimuth_cos.acos();
        if true_solar_time > 720.0 {
            azimuth = 2.0 * PI - azimuth;
        }
    }

    let east = azimuth.sin() * elevation.cos();
    let north = azimuth.cos() * elevation.cos();
    let up = elevation.sin();

    let mut direction = add(
        add(scale(plane.x_axis, east), scale(plane.y_axis, north)),
        scale(plane.z_axis, up),
    );

    if let Some((normalized, _)) = safe_normalized(direction) {
        direction = normalized;
    }

    (direction, elevation, elevation > 0.0)
}

fn color_for_elevation(elevation: f64) -> [f64; 3] {
    if !(elevation > 0.0) {
        return [0.08, 0.09, 0.15];
    }

    let normalized = (elevation / (PI / 3.0)).clamp(0.0, 1.0);
    let hue = (0.12 - 0.05 * normalized).clamp(0.0, 1.0);
    let saturation = (0.75 + 0.15 * (1.0 - normalized)).clamp(0.0, 1.0);
    let lightness = (0.35 + 0.25 * normalized).clamp(0.0, 1.0);
    hsl_to_rgb(hue, saturation, lightness)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> [f64; 3] {
    if s <= 0.0 {
        return [l, l, l];
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    }

    [
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0),
    ]
}

fn clamp_to_unit(value: f64) -> f64 {
    value.max(-1.0).min(1.0)
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(vector: [f64; 3], factor: f64) -> [f64; 3] {
    [vector[0] * factor, vector[1] * factor, vector[2] * factor]
}

fn vector_length(vector: [f64; 3]) -> f64 {
    vector_length_squared(vector).sqrt()
}

fn vector_length_squared(vector: [f64; 3]) -> f64 {
    dot(vector, vector)
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    if let Some((normalized, _)) = safe_normalized(vector) {
        normalized
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn orthogonal_vector(vector: [f64; 3]) -> [f64; 3] {
    let abs_x = vector[0].abs();
    let abs_y = vector[1].abs();
    let abs_z = vector[2].abs();
    if abs_x <= abs_y && abs_x <= abs_z {
        normalize([0.0, -vector[2], vector[1]])
    } else if abs_y <= abs_x && abs_y <= abs_z {
        normalize([-vector[2], 0.0, vector[0]])
    } else {
        normalize([-vector[1], vector[0], 0.0])
    }
}

fn safe_normalized(vector: [f64; 3]) -> Option<([f64; 3], f64)> {
    let length = vector_length(vector);
    if length < EPSILON {
        None
    } else {
        Some((scale(vector, 1.0 / length), length))
    }
}