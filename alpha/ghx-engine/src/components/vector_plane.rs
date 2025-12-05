//! Implementaties van Grasshopper "Vector → Plane" componenten.

use std::collections::BTreeMap;

use crate::components::coerce::coerce_point_with_default;
use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_PLANE: &str = "P";
const PIN_OUTPUT_PLANES: &str = "P";
const PIN_OUTPUT_DEVIATION: &str = "dx";
const PIN_OUTPUT_ORIGIN: &str = "O";
const PIN_OUTPUT_X_AXIS: &str = "X";
const PIN_OUTPUT_Y_AXIS: &str = "Y";
const PIN_OUTPUT_Z_AXIS: &str = "Z";
const PIN_OUTPUT_U: &str = "X";
const PIN_OUTPUT_V: &str = "Y";
const PIN_OUTPUT_W: &str = "Z";
const PIN_OUTPUT_PROJECTED_POINT: &str = "P";
const PIN_OUTPUT_UV: &str = "uv";
const PIN_OUTPUT_DISTANCE: &str = "D";
const PIN_OUTPUT_ANGLE: &str = "A";

const EPSILON: f64 = 1e-9;

/// Beschikbare componentvarianten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    XYPlane,
    AlignPlanes,
    PlaneFit,
    PlaneOffset,
    Deconstruct,
    PlaneCoordinates,
    PlaneOrigin,
    XZPlane,
    AdjustPlane,
    PlaneClosestPoint,
    ConstructPlane,
    FlipPlane,
    PlaneThreePoint,
    LinePoint,
    PlaneNormal,
    LineLine,
    AlignPlane,
    RotatePlane,
    YZPlane,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de vector-plane componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{17b7152b-d30d-4d50-b9ef-c9fe25576fc2}"],
        names: &["XY Plane", "XY"],
        kind: ComponentKind::XYPlane,
    },
    Registration {
        guids: &["{2318aee8-01fe-4ea8-9524-6966023fc622}"],
        names: &["Align Planes", "Align"],
        kind: ComponentKind::AlignPlanes,
    },
    Registration {
        guids: &["{33bfc73c-19b2-480b-81e6-f3523a012ea6}"],
        names: &["Plane Fit", "PlFit"],
        kind: ComponentKind::PlaneFit,
    },
    Registration {
        guids: &["{3a0c7bda-3d22-4588-8bab-03f57a52a6ea}"],
        names: &["Plane Offset", "Pl Offset"],
        kind: ComponentKind::PlaneOffset,
    },
    Registration {
        guids: &["{3cd2949b-4ea8-4ffb-a70c-5c380f9f46ea}"],
        names: &["Deconstruct Plane", "DePlane"],
        kind: ComponentKind::Deconstruct,
    },
    Registration {
        guids: &["{5f127fa4-ca61-418e-bb2d-e3739d900f1f}"],
        names: &["Plane Coordinates", "PlCoord"],
        kind: ComponentKind::PlaneCoordinates,
    },
    Registration {
        guids: &["{75eec078-a905-47a1-b0d2-0934182b1e3d}"],
        names: &["Plane Origin", "Pl Origin"],
        kind: ComponentKind::PlaneOrigin,
    },
    Registration {
        guids: &["{8cc3a196-f6a0-49ea-9ed9-0cb343a3ae64}"],
        names: &["XZ Plane", "XZ"],
        kind: ComponentKind::XZPlane,
    },
    Registration {
        guids: &["{9ce34996-d8c6-40d3-b442-1a7c8c093614}"],
        names: &["Adjust Plane", "PAdjust"],
        kind: ComponentKind::AdjustPlane,
    },
    Registration {
        guids: &["{b075c065-efda-4c9f-9cc9-288362b1b4b9}"],
        names: &["Plane Closest Point", "CP"],
        kind: ComponentKind::PlaneClosestPoint,
    },
    Registration {
        guids: &["{bc3e379e-7206-4e7b-b63a-ff61f4b38a3e}"],
        names: &["Construct Plane", "Pl"],
        kind: ComponentKind::ConstructPlane,
    },
    Registration {
        guids: &["{c73e1ed0-82a2-40b0-b4df-8f10e445d60b}"],
        names: &["Flip Plane", "PFlip"],
        kind: ComponentKind::FlipPlane,
    },
    Registration {
        guids: &["{c98a6015-7a2f-423c-bc66-bdc505249b45}"],
        names: &["Plane 3Pt", "Pl 3Pt"],
        kind: ComponentKind::PlaneThreePoint,
    },
    Registration {
        guids: &["{ccc3f2ff-c9f6-45f8-aa30-8a924a9bda36}"],
        names: &["Line + Pt", "LnPt"],
        kind: ComponentKind::LinePoint,
    },
    Registration {
        guids: &["{cfb6b17f-ca82-4f5d-b604-d4f69f569de3}"],
        names: &["Plane Normal"],
        kind: ComponentKind::PlaneNormal,
    },
    Registration {
        guids: &["{d788ad7f-6d68-4106-8b2f-9e55e6e107c0}"],
        names: &["Line + Line", "LnLn"],
        kind: ComponentKind::LineLine,
    },
    Registration {
        guids: &["{e76040ec-3b91-41e1-8e00-c74c23b89391}"],
        names: &["Align Plane", "Align Plane"],
        kind: ComponentKind::AlignPlane,
    },
    Registration {
        guids: &["{f6f14b09-6497-4564-8403-09e4eb5a6b82}"],
        names: &["Rotate Plane", "PRot"],
        kind: ComponentKind::RotatePlane,
    },
    Registration {
        guids: &["{fad344bc-09b1-4855-a2e6-437ef5715fe3}"],
        names: &["YZ Plane", "YZ"],
        kind: ComponentKind::YZPlane,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::XYPlane => evaluate_xy_plane(inputs),
            Self::AlignPlanes => evaluate_align_planes(inputs),
            Self::PlaneFit => evaluate_plane_fit(inputs),
            Self::PlaneOffset => evaluate_plane_offset(inputs),
            Self::Deconstruct => evaluate_deconstruct(inputs),
            Self::PlaneCoordinates => evaluate_plane_coordinates(inputs),
            Self::PlaneOrigin => evaluate_plane_origin(inputs),
            Self::XZPlane => evaluate_xz_plane(inputs),
            Self::AdjustPlane => evaluate_adjust_plane(inputs),
            Self::PlaneClosestPoint => evaluate_plane_closest_point(inputs),
            Self::ConstructPlane => evaluate_construct_plane(inputs),
            Self::FlipPlane => evaluate_flip_plane(inputs),
            Self::PlaneThreePoint => evaluate_plane_three_point(inputs),
            Self::LinePoint => evaluate_line_point(inputs),
            Self::PlaneNormal => evaluate_plane_normal(inputs),
            Self::LineLine => evaluate_line_line(inputs),
            Self::AlignPlane => evaluate_align_plane(inputs),
            Self::RotatePlane => evaluate_rotate_plane(inputs),
            Self::YZPlane => evaluate_yz_plane(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::XYPlane => "XY Plane",
            Self::AlignPlanes => "Align Planes",
            Self::PlaneFit => "Plane Fit",
            Self::PlaneOffset => "Plane Offset",
            Self::Deconstruct => "Deconstruct Plane",
            Self::PlaneCoordinates => "Plane Coordinates",
            Self::PlaneOrigin => "Plane Origin",
            Self::XZPlane => "XZ Plane",
            Self::AdjustPlane => "Adjust Plane",
            Self::PlaneClosestPoint => "Plane Closest Point",
            Self::ConstructPlane => "Construct Plane",
            Self::FlipPlane => "Flip Plane",
            Self::PlaneThreePoint => "Plane 3Pt",
            Self::LinePoint => "Line + Pt",
            Self::PlaneNormal => "Plane Normal",
            Self::LineLine => "Line + Line",
            Self::AlignPlane => "Align Plane",
            Self::RotatePlane => "Rotate Plane",
            Self::YZPlane => "YZ Plane",
        }
    }
}

fn evaluate_xy_plane(inputs: &[Value]) -> ComponentResult {
    let origins = collect_points(inputs.get(0), "XY Plane")?;
    if origins.len() <= 1 {
        let origin = origins.first().copied().unwrap_or_else(|| coerce_point_with_default(None));
        let plane =
            Plane::normalize_axes(origin, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]);
        Ok(single_plane_output(plane))
    } else {
        let planes = origins
            .into_iter()
            .map(|origin| Plane::normalize_axes(origin, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]))
            .collect();
        Ok(single_planes_output(planes))
    }
}

fn evaluate_align_planes(inputs: &[Value]) -> ComponentResult {
    let planes = collect_planes(inputs.get(0), "Align Planes")?;
    if planes.is_empty() {
        return Ok(single_planes_output(Vec::new()));
    }
    let master = inputs
        .get(1)
        .map(|value| coerce_plane(value, "Align Planes"))
        .transpose()?;

    let mut result = Vec::new();
    let mut reference = master.unwrap_or(planes[0]);
    for (index, plane) in planes.iter().enumerate() {
        if index == 0 && master.is_none() {
            result.push(*plane);
            reference = *plane;
        } else {
            let aligned = align_plane_to_reference(reference, *plane);
            result.push(aligned);
            reference = aligned;
        }
    }

    Ok(single_planes_output(result))
}

fn evaluate_plane_fit(inputs: &[Value]) -> ComponentResult {
    let points = collect_points(inputs.get(0), "Plane Fit")?;
    let (plane, deviation) = fit_plane_to_points(&points);

    let mut outputs = single_plane_output(plane);
    outputs.insert(PIN_OUTPUT_DEVIATION.to_owned(), Value::Number(deviation));
    Ok(outputs)
}

fn evaluate_plane_offset(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Plane Offset vereist minimaal een vlak",
        ));
    }

    let plane = coerce_plane(&inputs[0], "Plane Offset")?;
    let offset = inputs
        .get(1)
        .map(|value| coerce_number(value, "Plane Offset"))
        .transpose()?
        .unwrap_or(0.0);

    let mut shifted = plane;
    shifted.origin = add(shifted.origin, scale(shifted.z_axis, offset));
    Ok(single_plane_output(shifted))
}

fn evaluate_deconstruct(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Deconstruct Plane vereist minimaal een vlak",
        ));
    }

    let plane = coerce_plane(&inputs[0], "Deconstruct Plane")?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_ORIGIN.to_owned(), Value::Point(plane.origin));
    outputs.insert(PIN_OUTPUT_X_AXIS.to_owned(), Value::Vector(plane.x_axis));
    outputs.insert(PIN_OUTPUT_Y_AXIS.to_owned(), Value::Vector(plane.y_axis));
    outputs.insert(PIN_OUTPUT_Z_AXIS.to_owned(), Value::Vector(plane.z_axis));
    Ok(outputs)
}

fn evaluate_plane_coordinates(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Plane Coordinates vereist minimaal een punt",
        ));
    }

    let point = coerce_point(&inputs[0], "Plane Coordinates")?;
    let plane = inputs
        .get(1)
        .map(|value| coerce_plane(value, "Plane Coordinates"))
        .transpose()?
        .unwrap_or_default();

    let coords = plane_coordinates(point, &plane);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_U.to_owned(), Value::Number(coords[0]));
    outputs.insert(PIN_OUTPUT_V.to_owned(), Value::Number(coords[1]));
    outputs.insert(PIN_OUTPUT_W.to_owned(), Value::Number(coords[2]));
    Ok(outputs)
}

fn evaluate_plane_origin(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Plane Origin vereist minimaal een vlak",
        ));
    }

    let plane = coerce_plane(&inputs[0], "Plane Origin")?;
    let origin = inputs
        .get(1)
        .map(|value| coerce_point(value, "Plane Origin"))
        .transpose()?
        .unwrap_or(plane.origin);

    let mut adjusted = plane;
    adjusted.origin = origin;
    Ok(single_plane_output(adjusted))
}

fn evaluate_xz_plane(inputs: &[Value]) -> ComponentResult {
    let origins = collect_points(inputs.get(0), "XZ Plane")?;
    if origins.len() <= 1 {
        let origin = origins.first().copied().unwrap_or_else(|| coerce_point_with_default(None));
        let plane =
            Plane::normalize_axes(origin, [1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]);
        Ok(single_plane_output(plane))
    } else {
        let planes = origins
            .into_iter()
            .map(|origin| Plane::normalize_axes(origin, [1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]))
            .collect();
        Ok(single_planes_output(planes))
    }
}

fn evaluate_adjust_plane(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Adjust Plane vereist minimaal een vlak",
        ));
    }

    let plane = coerce_plane(&inputs[0], "Adjust Plane")?;
    let mut normal = inputs
        .get(1)
        .map(|value| coerce_vector(value, "Adjust Plane"))
        .transpose()?
        .unwrap_or(plane.z_axis);

    if vector_length_squared(normal) < EPSILON {
        normal = plane.z_axis;
    } else {
        normal = normalize(normal);
    }

    let mut x_axis = subtract(plane.x_axis, scale(normal, dot(plane.x_axis, normal)));
    if vector_length_squared(x_axis) < EPSILON {
        x_axis = subtract(plane.y_axis, scale(normal, dot(plane.y_axis, normal)));
    }
    if vector_length_squared(x_axis) < EPSILON {
        x_axis = orthogonal_vector(normal);
    } else {
        x_axis = normalize(x_axis);
    }
    let y_axis = normalize(cross(normal, x_axis));
    let adjusted = Plane::normalize_axes(plane.origin, x_axis, y_axis, normal);
    Ok(single_plane_output(adjusted))
}

fn evaluate_plane_closest_point(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Plane Closest Point vereist een punt en vlak",
        ));
    }

    let point = coerce_point(&inputs[0], "Plane Closest Point")?;
    let plane = coerce_plane(&inputs[1], "Plane Closest Point")?;

    let coords = plane_coordinates(point, &plane);
    let projected = apply_plane(&plane, coords[0], coords[1], 0.0);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_PROJECTED_POINT.to_owned(),
        Value::Point(projected),
    );
    outputs.insert(
        PIN_OUTPUT_UV.to_owned(),
        Value::Point([coords[0], coords[1], 0.0]),
    );
    outputs.insert(PIN_OUTPUT_DISTANCE.to_owned(), Value::Number(coords[2]));
    Ok(outputs)
}

fn evaluate_construct_plane(inputs: &[Value]) -> ComponentResult {
    let origin = coerce_point_with_default(inputs.get(0));
    let mut x_axis = inputs
        .get(1)
        .map(|value| coerce_vector(value, "Construct Plane"))
        .transpose()?
        .unwrap_or([1.0, 0.0, 0.0]);
    if vector_length_squared(x_axis) < EPSILON {
        x_axis = [1.0, 0.0, 0.0];
    }
    let mut y_axis = inputs
        .get(2)
        .map(|value| coerce_vector(value, "Construct Plane"))
        .transpose()?
        .unwrap_or([0.0, 1.0, 0.0]);
    if vector_length_squared(y_axis) < EPSILON {
        y_axis = orthogonal_vector(x_axis);
    }
    let mut z_axis = cross(x_axis, y_axis);
    if vector_length_squared(z_axis) < EPSILON {
        y_axis = orthogonal_vector(x_axis);
        z_axis = cross(x_axis, y_axis);
    }
    let plane = Plane::normalize_axes(origin, x_axis, y_axis, z_axis);
    Ok(single_plane_output(plane))
}

fn evaluate_flip_plane(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Flip Plane vereist minimaal een vlak"));
    }

    let plane = coerce_plane(&inputs[0], "Flip Plane")?;
    let reverse_x = inputs
        .get(1)
        .map(|value| coerce_boolean(value, "Flip Plane"))
        .transpose()?
        .unwrap_or(false);
    let reverse_y = inputs
        .get(2)
        .map(|value| coerce_boolean(value, "Flip Plane"))
        .transpose()?
        .unwrap_or(false);
    let swap_axes = inputs
        .get(3)
        .map(|value| coerce_boolean(value, "Flip Plane"))
        .transpose()?
        .unwrap_or(false);

    let mut x_axis = plane.x_axis;
    let mut y_axis = plane.y_axis;
    if swap_axes {
        std::mem::swap(&mut x_axis, &mut y_axis);
    }
    if reverse_x {
        x_axis = scale(x_axis, -1.0);
    }
    if reverse_y {
        y_axis = scale(y_axis, -1.0);
    }
    let mut z_axis = cross(x_axis, y_axis);
    if vector_length_squared(z_axis) < EPSILON {
        z_axis = plane.z_axis;
    }
    let plane = Plane::normalize_axes(plane.origin, x_axis, y_axis, z_axis);
    Ok(single_plane_output(plane))
}

fn evaluate_plane_three_point(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new("Plane 3Pt vereist drie punten"));
    }
    let a = coerce_point(&inputs[0], "Plane 3Pt")?;
    let b = coerce_point(&inputs[1], "Plane 3Pt")?;
    let c = coerce_point(&inputs[2], "Plane 3Pt")?;
    Ok(single_plane_output(Plane::from_points(a, b, c)))
}

fn evaluate_line_point(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Line + Pt vereist een lijn en een punt",
        ));
    }

    let line = coerce_line(&inputs[0], "Line + Pt")?;
    let point = coerce_point(&inputs[1], "Line + Pt")?;

    Ok(single_plane_output(plane_from_line_and_point(line, point)))
}

fn evaluate_plane_normal(inputs: &[Value]) -> ComponentResult {
    let origins = collect_points(inputs.get(0), "Plane Normal")?;
    let mut normal = inputs
        .get(1)
        .map(|value| coerce_vector(value, "Plane Normal"))
        .transpose()?
        .unwrap_or([0.0, 0.0, 1.0]);
    if vector_length_squared(normal) < EPSILON {
        normal = [0.0, 0.0, 1.0];
    } else {
        normal = normalize(normal);
    }
    let make_plane = |origin: [f64; 3]| {
        let x_axis = orthogonal_vector(normal);
        let y_axis = normalize(cross(normal, x_axis));
        Plane::normalize_axes(origin, x_axis, y_axis, normal)
    };

    if origins.len() <= 1 {
        let origin = origins.first().copied().unwrap_or_else(|| coerce_point_with_default(None));
        Ok(single_plane_output(make_plane(origin)))
    } else {
        let planes = origins.into_iter().map(make_plane).collect();
        Ok(single_planes_output(planes))
    }
}

fn evaluate_line_line(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Line + Line vereist twee lijnen"));
    }

    let line_a = coerce_line(&inputs[0], "Line + Line")?;
    let line_b = coerce_line(&inputs[1], "Line + Line")?;
    Ok(single_plane_output(plane_from_lines(line_a, line_b)))
}

fn evaluate_align_plane(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Align Plane vereist een vlak en richting",
        ));
    }

    let plane = coerce_plane(&inputs[0], "Align Plane")?;
    let direction = coerce_vector(&inputs[1], "Align Plane")?;
    if vector_length_squared(direction) < EPSILON {
        let mut outputs = single_plane_output(plane);
        outputs.insert(PIN_OUTPUT_ANGLE.to_owned(), Value::Number(0.0));
        return Ok(outputs);
    }

    let projected = subtract(direction, scale(plane.z_axis, dot(direction, plane.z_axis)));
    if vector_length_squared(projected) < EPSILON {
        let mut outputs = single_plane_output(plane);
        outputs.insert(PIN_OUTPUT_ANGLE.to_owned(), Value::Number(0.0));
        return Ok(outputs);
    }
    let target = normalize(projected);

    let cos_theta = clamp_to_unit(dot(plane.x_axis, target));
    let sin_theta = dot(plane.y_axis, target);
    let angle = sin_theta.atan2(cos_theta);
    let rotation = quaternion_from_axis_angle(plane.z_axis, angle);
    let x_axis = apply_quaternion(plane.x_axis, rotation);
    let y_axis = apply_quaternion(plane.y_axis, rotation);

    let aligned = Plane::normalize_axes(plane.origin, x_axis, y_axis, plane.z_axis);
    let mut outputs = single_plane_output(aligned);
    outputs.insert(PIN_OUTPUT_ANGLE.to_owned(), Value::Number(angle));
    Ok(outputs)
}

fn evaluate_rotate_plane(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Rotate Plane vereist een vlak en hoek"));
    }

    let plane = coerce_plane(&inputs[0], "Rotate Plane")?;
    let angle = coerce_number(&inputs[1], "Rotate Plane")?;
    if angle.abs() < EPSILON {
        return Ok(single_plane_output(plane));
    }
    let rotation = quaternion_from_axis_angle(plane.z_axis, angle);
    let x_axis = apply_quaternion(plane.x_axis, rotation);
    let y_axis = apply_quaternion(plane.y_axis, rotation);
    Ok(single_plane_output(Plane::normalize_axes(
        plane.origin,
        x_axis,
        y_axis,
        plane.z_axis,
    )))
}

fn evaluate_yz_plane(inputs: &[Value]) -> ComponentResult {
    let origins = collect_points(inputs.get(0), "YZ Plane")?;
    if origins.len() <= 1 {
        let origin = origins.first().copied().unwrap_or_else(|| coerce_point_with_default(None));
        let plane =
            Plane::normalize_axes(origin, [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        Ok(single_plane_output(plane))
    } else {
        let planes = origins
            .into_iter()
            .map(|origin| Plane::normalize_axes(origin, [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]))
            .collect();
        Ok(single_planes_output(planes))
    }
}

fn single_plane_output(plane: Plane) -> BTreeMap<String, Value> {
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_PLANE.to_owned(), plane_to_value(plane));
    outputs
}

fn single_planes_output(planes: Vec<Plane>) -> BTreeMap<String, Value> {
    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_PLANES.to_owned(),
        Value::List(planes.into_iter().map(plane_to_value).collect()),
    );
    outputs
}

fn plane_to_value(plane: Plane) -> Value {
    let origin = Value::Point(plane.origin);
    let point_x = Value::Point(add(plane.origin, plane.x_axis));
    let point_y = Value::Point(add(plane.origin, plane.y_axis));
    Value::List(vec![origin, point_x, point_y])
}

#[derive(Debug, Clone, Copy)]
struct Plane {
    origin: [f64; 3],
    x_axis: [f64; 3],
    y_axis: [f64; 3],
    z_axis: [f64; 3],
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            z_axis: [0.0, 0.0, 1.0],
        }
    }
}

impl Plane {
    fn normalize_axes(
        origin: [f64; 3],
        x_axis: [f64; 3],
        y_axis: [f64; 3],
        z_axis: [f64; 3],
    ) -> Self {
        let z = safe_normalized(z_axis)
            .map(|(vector, _)| vector)
            .unwrap_or([0.0, 0.0, 1.0]);

        let mut x = safe_normalized(x_axis)
            .map(|(vector, _)| vector)
            .unwrap_or_else(|| orthogonal_vector(z));

        let mut y = safe_normalized(y_axis)
            .map(|(vector, _)| vector)
            .unwrap_or_else(|| normalize(cross(z, x)));

        x = normalize(cross(y, z));
        y = normalize(cross(z, x));

        Self {
            origin,
            x_axis: x,
            y_axis: y,
            z_axis: z,
        }
    }

    fn from_points(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Self {
        let ab = subtract(b, a);
        let ac = subtract(c, a);
        let normal = cross(ab, ac);
        if vector_length_squared(normal) < EPSILON {
            return Self::default();
        }
        let x_axis = if vector_length_squared(ab) < EPSILON {
            orthogonal_vector(normal)
        } else {
            normalize(ab)
        };
        let z_axis = normalize(normal);
        let y_axis = normalize(cross(z_axis, x_axis));
        Self::normalize_axes(a, x_axis, y_axis, z_axis)
    }
}

impl From<Line> for Plane {
    fn from(line: Line) -> Self {
        let direction = line.direction();
        if vector_length_squared(direction) < EPSILON {
            return Self::default();
        }
        let x_axis = normalize(direction);
        let y_axis = orthogonal_vector(x_axis);
        let z_axis = normalize(cross(x_axis, y_axis));
        Self::normalize_axes(line.start, x_axis, y_axis, z_axis)
    }
}

#[derive(Debug, Clone, Copy)]
struct Line {
    start: [f64; 3],
    end: [f64; 3],
}

impl Line {
    fn direction(self) -> [f64; 3] {
        subtract(self.end, self.start)
    }
}

fn fit_plane_to_points(points: &[[f64; 3]]) -> (Plane, f64) {
    match points.len() {
        0 => (Plane::default(), 0.0),
        1 => {
            let mut plane = Plane::default();
            plane.origin = points[0];
            (plane, 0.0)
        }
        2 => {
            let origin = points[0];
            let mut x_axis = subtract(points[1], points[0]);
            if vector_length_squared(x_axis) < EPSILON {
                x_axis = [1.0, 0.0, 0.0];
            }
            x_axis = normalize(x_axis);
            let normal = orthogonal_vector(x_axis);
            let y_axis = normalize(cross(normal, x_axis));
            (Plane::normalize_axes(origin, x_axis, y_axis, normal), 0.0)
        }
        _ => {
            let mut centroid = [0.0, 0.0, 0.0];
            for point in points {
                centroid = add(centroid, *point);
            }
            centroid = scale(centroid, 1.0 / points.len() as f64);

            let mut xx = 0.0;
            let mut xy = 0.0;
            let mut xz = 0.0;
            let mut yy = 0.0;
            let mut yz = 0.0;
            let mut zz = 0.0;

            for point in points {
                let dx = point[0] - centroid[0];
                let dy = point[1] - centroid[1];
                let dz = point[2] - centroid[2];
                xx += dx * dx;
                xy += dx * dy;
                xz += dx * dz;
                yy += dy * dy;
                yz += dy * dz;
                zz += dz * dz;
            }

            let (eigen_values, eigen_vectors) = jacobi_eigen_decomposition(xx, xy, xz, yy, yz, zz);
            let mut min_index = 0;
            if eigen_values[1] < eigen_values[min_index] {
                min_index = 1;
            }
            if eigen_values[2] < eigen_values[min_index] {
                min_index = 2;
            }

            let mut normal = [
                eigen_vectors[0][min_index],
                eigen_vectors[1][min_index],
                eigen_vectors[2][min_index],
            ];
            if vector_length_squared(normal) < EPSILON {
                normal = [0.0, 0.0, 1.0];
            } else {
                normal = normalize(normal);
            }

            let mut x_axis = subtract(points[0], centroid);
            x_axis = subtract(x_axis, scale(normal, dot(x_axis, normal)));
            if vector_length_squared(x_axis) < EPSILON {
                x_axis = orthogonal_vector(normal);
            } else {
                x_axis = normalize(x_axis);
            }
            let y_axis = normalize(cross(normal, x_axis));
            x_axis = normalize(cross(y_axis, normal));
            let plane = Plane::normalize_axes(centroid, x_axis, y_axis, normal);

            let mut deviation = 0.0_f64;
            for point in points {
                let coords = plane_coordinates(*point, &plane);
                deviation = deviation.max(coords[2].abs());
            }

            (plane, deviation)
        }
    }
}

fn plane_from_line_and_point(line: Line, point: [f64; 3]) -> Plane {
    let origin = line.start;
    let mut x_axis = line.direction();
    if vector_length_squared(x_axis) < EPSILON {
        x_axis = subtract(line.end, line.start);
    }
    if vector_length_squared(x_axis) < EPSILON {
        x_axis = [1.0, 0.0, 0.0];
    }
    x_axis = normalize(x_axis);
    let mut offset = subtract(point, origin);
    offset = subtract(offset, scale(x_axis, dot(offset, x_axis)));
    if vector_length_squared(offset) < EPSILON {
        offset = orthogonal_vector(x_axis);
    } else {
        offset = normalize(offset);
    }
    let mut normal = cross(x_axis, offset);
    if vector_length_squared(normal) < EPSILON {
        let fallback = orthogonal_vector(x_axis);
        offset = fallback;
        normal = cross(x_axis, offset);
    }
    normal = normalize(normal);
    let y_axis = normalize(cross(normal, x_axis));
    Plane::normalize_axes(origin, x_axis, y_axis, normal)
}

fn plane_from_lines(line_a: Line, line_b: Line) -> Plane {
    let origin = line_a.start;
    let mut x_axis = line_a.direction();
    if vector_length_squared(x_axis) < EPSILON {
        x_axis = subtract(line_a.end, line_a.start);
    }
    if vector_length_squared(x_axis) < EPSILON {
        x_axis = [1.0, 0.0, 0.0];
    }
    x_axis = normalize(x_axis);
    let mut reference = line_b.direction();
    if vector_length_squared(reference) < EPSILON {
        reference = subtract(line_b.end, line_b.start);
    }
    if vector_length_squared(reference) < EPSILON {
        reference = subtract(line_b.start, origin);
    }
    if vector_length_squared(reference) < EPSILON {
        reference = orthogonal_vector(x_axis);
    }
    let mut normal = cross(x_axis, reference);
    if vector_length_squared(normal) < EPSILON {
        normal = cross(x_axis, subtract(line_b.start, origin));
    }
    if vector_length_squared(normal) < EPSILON {
        normal = orthogonal_vector(x_axis);
    }
    normal = normalize(normal);
    let y_axis = normalize(cross(normal, x_axis));
    Plane::normalize_axes(origin, x_axis, y_axis, normal)
}

fn align_plane_to_reference(reference: Plane, plane: Plane) -> Plane {
    let mut target = plane;
    if dot(target.z_axis, reference.z_axis) < 0.0 {
        target.z_axis = scale(target.z_axis, -1.0);
        target.x_axis = scale(target.x_axis, -1.0);
        target.y_axis = scale(target.y_axis, -1.0);
    }

    let candidates = [
        target,
        Plane {
            origin: target.origin,
            x_axis: scale(target.x_axis, -1.0),
            y_axis: scale(target.y_axis, -1.0),
            z_axis: target.z_axis,
        },
    ];

    let mut best = candidates[0];
    let mut best_score = f64::NEG_INFINITY;
    for candidate in candidates {
        let score =
            dot(candidate.x_axis, reference.x_axis) + dot(candidate.y_axis, reference.y_axis);
        if score > best_score {
            best = candidate;
            best_score = score;
        }
    }
    Plane::normalize_axes(best.origin, best.x_axis, best.y_axis, best.z_axis)
}

fn plane_coordinates(point: [f64; 3], plane: &Plane) -> [f64; 3] {
    let relative = subtract(point, plane.origin);
    [
        dot(relative, plane.x_axis),
        dot(relative, plane.y_axis),
        dot(relative, plane.z_axis),
    ]
}

fn apply_plane(plane: &Plane, u: f64, v: f64, w: f64) -> [f64; 3] {
    add(
        add(
            add(plane.origin, scale(plane.x_axis, u)),
            scale(plane.y_axis, v),
        ),
        scale(plane.z_axis, w),
    )
}

fn collect_planes(value: Option<&Value>, context: &str) -> Result<Vec<Plane>, ComponentError> {
    let mut planes = Vec::new();
    if let Some(value) = value {
        collect_planes_into(value, context, &mut planes)?;
    }
    Ok(planes)
}

fn collect_planes_into(
    value: &Value,
    context: &str,
    output: &mut Vec<Plane>,
) -> Result<(), ComponentError> {
    match value {
        Value::List(values) => {
            if values.is_empty() {
                return Ok(());
            }
            match coerce_plane(value, context) {
                Ok(plane) => {
                    output.push(plane);
                    Ok(())
                }
                Err(_) => {
                    for entry in values {
                        collect_planes_into(entry, context, output)?;
                    }
                    Ok(())
                }
            }
        }
        _ => {
            output.push(coerce_plane(value, context)?);
            Ok(())
        }
    }
}

fn collect_points(value: Option<&Value>, context: &str) -> Result<Vec<[f64; 3]>, ComponentError> {
    let mut points = Vec::new();
    if let Some(value) = value {
        collect_points_into(value, context, &mut points)?;
    }
    Ok(points)
}

fn collect_points_into(
    value: &Value,
    context: &str,
    output: &mut Vec<[f64; 3]>,
) -> Result<(), ComponentError> {
    match value {
        Value::Point(point) | Value::Vector(point) => {
            output.push(*point);
            Ok(())
        }
        Value::List(values) => {
            if let Ok(point) = coerce_point(value, context) {
                output.push(point);
                return Ok(());
            }
            for entry in values {
                collect_points_into(entry, context, output)?;
            }
            Ok(())
        }
        Value::Number(number) => {
            output.push([*number, 0.0, 0.0]);
            Ok(())
        }
        Value::Boolean(boolean) => {
            output.push([if *boolean { 1.0 } else { 0.0 }, 0.0, 0.0]);
            Ok(())
        }
        Value::Text(text) => {
            if let Ok(parsed) = text.trim().parse::<f64>() {
                output.push([parsed, 0.0, 0.0]);
                Ok(())
            } else {
                Err(ComponentError::new(format!(
                    "{} kon tekst '{}' niet als punt interpreteren",
                    context, text
                )))
            }
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht puntwaarden, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_plane(value: &Value, context: &str) -> Result<Plane, ComponentError> {
    match value {
        Value::List(values) if values.len() >= 3 => {
            let a = coerce_point(&values[0], context)?;
            let b = coerce_point(&values[1], context)?;
            let c = coerce_point(&values[2], context)?;
            Ok(Plane::from_points(a, b, c))
        }
        Value::List(values) if values.len() == 2 => {
            let origin = coerce_point(&values[0], context)?;
            let direction = coerce_vector(&values[1], context)?;
            if vector_length_squared(direction) < EPSILON {
                Ok(Plane::default())
            } else {
                let x_axis = normalize(direction);
                let z_axis = orthogonal_vector(direction);
                let y_axis = normalize(cross(z_axis, x_axis));
                Ok(Plane::normalize_axes(origin, x_axis, y_axis, z_axis))
            }
        }
        Value::List(values) if values.len() == 1 => coerce_plane(&values[0], context),
        Value::Point(point) => {
            let mut plane = Plane::default();
            plane.origin = *point;
            Ok(plane)
        }
        Value::Vector(vector) => {
            let normal = if vector_length_squared(*vector) < EPSILON {
                [0.0, 0.0, 1.0]
            } else {
                normalize(*vector)
            };
            let x_axis = orthogonal_vector(normal);
            let y_axis = normalize(cross(normal, x_axis));
            Ok(Plane::normalize_axes(
                [0.0, 0.0, 0.0],
                x_axis,
                y_axis,
                normal,
            ))
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een vlak, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_line(value: &Value, context: &str) -> Result<Line, ComponentError> {
    match value {
        Value::CurveLine { p1, p2 } => Ok(Line {
            start: *p1,
            end: *p2,
        }),
        Value::List(values) if values.len() >= 2 => {
            let start = coerce_point(&values[0], context)?;
            let mut end = coerce_point(&values[1], context)?;
            if vector_length_squared(subtract(end, start)) < EPSILON && values.len() > 2 {
                end = add(start, coerce_vector(&values[2], context)?);
            }
            Ok(Line { start, end })
        }
        Value::List(values) if values.len() == 1 => coerce_line(&values[0], context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een lijn, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_number(value: &Value, context: &str) -> Result<f64, ComponentError> {
    match value {
        Value::Number(number) => Ok(*number),
        Value::Boolean(boolean) => Ok(if *boolean { 1.0 } else { 0.0 }),
        Value::List(values) if values.len() == 1 => coerce_number(&values[0], context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een numerieke waarde, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_boolean(value: &Value, context: &str) -> Result<bool, ComponentError> {
    match value {
        Value::Boolean(boolean) => Ok(*boolean),
        Value::Number(number) => Ok(*number != 0.0),
        Value::List(values) if values.len() == 1 => coerce_boolean(&values[0], context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een booleaanse waarde, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_vector(value: &Value, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Vector(vector) => Ok(*vector),
        Value::Point(point) => Ok(*point),
        Value::List(values) if values.len() == 1 => coerce_vector(&values[0], context),
        Value::List(values) if values.len() >= 3 => {
            let x = coerce_number(&values[0], context)?;
            let y = coerce_number(&values[1], context)?;
            let z = coerce_number(&values[2], context)?;
            Ok([x, y, z])
        }
        Value::List(values) if values.len() == 2 => {
            let x = coerce_number(&values[0], context)?;
            let y = coerce_number(&values[1], context)?;
            Ok([x, y, 0.0])
        }
        Value::Number(number) => Ok([0.0, 0.0, *number]),
        other => Err(ComponentError::new(format!(
            "{} verwacht een vector, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_point(value: &Value, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Point(point) => Ok(*point),
        Value::Vector(vector) => Ok(*vector),
        Value::List(values) if values.len() == 1 => coerce_point(&values[0], context),
        Value::List(values) if values.len() >= 3 => {
            let x = coerce_number(&values[0], context)?;
            let y = coerce_number(&values[1], context)?;
            let z = coerce_number(&values[2], context)?;
            Ok([x, y, z])
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een punt, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn jacobi_eigen_decomposition(
    xx: f64,
    xy: f64,
    xz: f64,
    yy: f64,
    yz: f64,
    zz: f64,
) -> ([f64; 3], [[f64; 3]; 3]) {
    let mut m = [[xx, xy, xz], [xy, yy, yz], [xz, yz, zz]];
    let mut eigen_vectors = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let tolerance = 1e-10;
    let max_iterations = 32;

    for _ in 0..max_iterations {
        let mut p = 0;
        let mut q = 1;
        if m[0][1].abs() < m[0][2].abs() {
            p = 0;
            q = 2;
        }
        if m[p][q].abs() < m[1][2].abs() {
            p = 1;
            q = 2;
        }
        if m[p][q].abs() < tolerance {
            break;
        }
        let app = m[p][p];
        let aqq = m[q][q];
        let apq = m[p][q];
        let angle = 0.5 * (2.0 * apq).atan2(aqq - app);
        let c = angle.cos();
        let s = angle.sin();
        for k in 0..3 {
            if k == p || k == q {
                continue;
            }
            let mkp = m[k][p];
            let mkq = m[k][q];
            m[k][p] = c * mkp - s * mkq;
            m[p][k] = m[k][p];
            m[k][q] = c * mkq + s * mkp;
            m[q][k] = m[k][q];
        }
        m[p][p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        m[q][q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        m[p][q] = 0.0;
        m[q][p] = 0.0;
        for k in 0..3 {
            let vip = eigen_vectors[k][p];
            let viq = eigen_vectors[k][q];
            eigen_vectors[k][p] = c * vip - s * viq;
            eigen_vectors[k][q] = s * vip + c * viq;
        }
    }

    ([m[0][0], m[1][1], m[2][2]], eigen_vectors)
}

fn quaternion_from_axis_angle(axis: [f64; 3], angle: f64) -> [f64; 4] {
    if vector_length_squared(axis) < EPSILON {
        return [1.0, 0.0, 0.0, 0.0];
    }
    let (unit_axis, _) = safe_normalized(axis).unwrap();
    let half = angle * 0.5;
    let sin_half = half.sin();
    [
        half.cos(),
        unit_axis[0] * sin_half,
        unit_axis[1] * sin_half,
        unit_axis[2] * sin_half,
    ]
}

fn apply_quaternion(vector: [f64; 3], quaternion: [f64; 4]) -> [f64; 3] {
    let [w, x, y, z] = quaternion;
    let q_vec = [x, y, z];
    let uv = cross(q_vec, vector);
    let uuv = cross(q_vec, uv);
    let uv = scale(uv, 2.0 * w);
    let uuv = scale(uuv, 2.0);
    add(vector, add(uv, uuv))
}

fn safe_normalized(vector: [f64; 3]) -> Option<([f64; 3], f64)> {
    let length = vector_length(vector);
    if length < EPSILON {
        None
    } else {
        Some((scale(vector, 1.0 / length), length))
    }
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    if let Some((normalized, _)) = safe_normalized(vector) {
        normalized
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn vector_length_squared(vector: [f64; 3]) -> f64 {
    dot(vector, vector)
}

fn vector_length(vector: [f64; 3]) -> f64 {
    vector_length_squared(vector).sqrt()
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

fn clamp_to_unit(value: f64) -> f64 {
    if value > 1.0 {
        1.0
    } else if value < -1.0 {
        -1.0
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentKind};
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn xy_plane_uses_custom_origin() {
        let component = ComponentKind::XYPlane;
        let outputs = component
            .evaluate(&[Value::Point([1.0, 2.0, 3.0])], &MetaMap::new())
            .expect("component slaagt");
        let plane = &outputs["P"];
        if let Value::List(entries) = plane {
            assert_eq!(entries.len(), 3);
            assert!(matches!(entries[0], Value::Point([1.0, 2.0, 3.0])));
        } else {
            panic!("verwacht lijstrepresentatie van vlak");
        }
    }

    #[test]
    fn xy_plane_accepts_point_list_for_origin() {
        let component = ComponentKind::XYPlane;
        let outputs = component
            .evaluate(
                &[Value::List(vec![
                    Value::Point([5.0, 6.0, 7.0]),
                    Value::Point([8.0, 9.0, 10.0]),
                ])],
                &MetaMap::new(),
            )
            .expect("component slaagt");

        let plane = &outputs["P"];
        let Some(Value::List(entries)) = plane else {
            panic!("verwacht lijstrepresentatie van vlak");
        };
        let Some(Value::Point(origin)) = entries.get(0) else {
            panic!("verwacht oorsprong als punt");
        };
        assert!((origin[0] - 5.0).abs() < 1e-9);
        assert!((origin[1] - 6.0).abs() < 1e-9);
        assert!((origin[2] - 7.0).abs() < 1e-9);
    }

    #[test]
    fn xy_plane_outputs_multiple_planes_for_point_list() {
        let component = ComponentKind::XYPlane;
        let outputs = component
            .evaluate(
                &[Value::List(vec![
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Point([10.0, 0.0, 0.0]),
                ])],
                &MetaMap::new(),
            )
            .expect("component slaagt");

        let Some(Value::List(planes)) = outputs.get("P") else {
            panic!("verwacht plane lijst");
        };
        assert_eq!(planes.len(), 2);
    }

    #[test]
    fn plane_fit_returns_deviation() {
        let component = ComponentKind::PlaneFit;
        let inputs = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.1]),
            Value::Point([0.0, 1.0, -0.1]),
            Value::Point([1.0, 1.0, 0.05]),
        ]);
        let outputs = component
            .evaluate(&[inputs], &MetaMap::new())
            .expect("component slaagt");
        let deviation = outputs
            .get("dx")
            .and_then(|value| match value {
                Value::Number(number) => Some(*number),
                _ => None,
            })
            .expect("deviation aanwezig");
        assert!(deviation >= 0.0);
    }

    #[test]
    fn align_plane_reports_rotation_angle() {
        let component = ComponentKind::AlignPlane;
        let plane = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
        ]);
        let direction = Value::Vector([0.0, 1.0, 0.0]);
        let outputs = component
            .evaluate(&[plane, direction], &MetaMap::new())
            .expect("component slaagt");
        let angle = outputs
            .get("A")
            .and_then(|value| match value {
                Value::Number(number) => Some(*number),
                _ => None,
            })
            .expect("hoek output aanwezig");
        assert!((angle - std::f64::consts::FRAC_PI_2).abs() < 1e-6);
    }
}
