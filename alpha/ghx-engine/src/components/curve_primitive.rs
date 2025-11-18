//! Implementaties van Grasshopper "Curve → Primitive" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::{Value, ValueKind};

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_CIRCLE: &str = "C";
const PIN_OUTPUT_RECTANGLE: &str = "R";
const PIN_OUTPUT_LENGTH: &str = "L";
const PIN_OUTPUT_LINE: &str = "L";
const PIN_OUTPUT_POLYGON: &str = "P";
const PIN_OUTPUT_ARC: &str = "A";

/// Vast aantal segmenten voor alle curves.
const CURVE_SEGMENTS: usize = 32;

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Circle,
    Rectangle,
    FitLine,
    InCircle,
    Arc3Pt,
    Rectangle3Pt,
    Ellipse,
    Circle3Pt,
    Line,
    LineSDL,
    CircleTanTan,
    Line2Plane,
    Rectangle2Pt,
    InEllipse,
    BiArc,
    Polygon,
    ArcSED,
    ModifiedArc,
    Line4Pt,
    Arc,
    CircleFit,
    TwoByFourJam,
    CircleCNR,
    TangentLinesEx,
    CircleTanTanTan,
    TangentLinesIn,
    TangentLines,
    TangentArcs,
    PolygonEdge,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration<T> {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: T,
}

/// Volledige lijst van componentregistraties voor de curve-primitive componenten.
pub const REGISTRATIONS: &[Registration<ComponentKind>] = &[
    Registration {
        guids: &["807b86e3-be8d-4970-92b5-f8cdcb45b06b"],
        names: &["Circle", "Cir"],
        kind: ComponentKind::Circle,
    },
    Registration {
        guids: &[
            "0ca0a214-396c-44ea-b22f-d3a1757c32d6",
            "d93100b6-d50b-40b2-831a-814659dc38e3",
        ],
        names: &["Rectangle"],
        kind: ComponentKind::Rectangle,
    },
    Registration {
        guids: &["1f798a28-9de6-47b5-8201-cac57256b777"],
        names: &["Fit Line", "FLine"],
        kind: ComponentKind::FitLine,
    },
    Registration {
        guids: &["28b1c4d4-ab1c-4309-accd-1b7a954ed948"],
        names: &["InCircle"],
        kind: ComponentKind::InCircle,
    },
    Registration {
        guids: &[
            "32c57b97-b653-47dd-b78f-121e89fdd01c",
            "9fa1b081-b1c7-4a12-a163-0aa8da9ff6c4",
        ],
        names: &["Arc 3Pt"],
        kind: ComponentKind::Arc3Pt,
    },
    Registration {
        guids: &[
            "34493ef6-3dfb-47c0-b149-691d02a93588",
            "9bc98a1d-2ecc-407e-948a-09a09ed3e69d",
        ],
        names: &["Rectangle 3Pt", "Rec 3Pt"],
        kind: ComponentKind::Rectangle3Pt,
    },
    Registration {
        guids: &["46b5564d-d3eb-4bf1-ae16-15ed132cfd88"],
        names: &["Ellipse"],
        kind: ComponentKind::Ellipse,
    },
    Registration {
        guids: &["47886835-e3ff-4516-a3ed-1b419f055464"],
        names: &["Circle 3Pt"],
        kind: ComponentKind::Circle3Pt,
    },
    Registration {
        guids: &["4c4e56eb-2f04-43f9-95a3-cc46a14f495a"],
        names: &["Line", "Ln"],
        kind: ComponentKind::Line,
    },
    Registration {
        guids: &["4c619bc9-39fd-4717-82a6-1e07ea237bbe"],
        names: &["Line SDL"],
        kind: ComponentKind::LineSDL,
    },
    Registration {
        guids: &["50b204ef-d3de-41bb-a006-02fba2d3f709"],
        names: &["Circle TanTan", "CircleTT"],
        kind: ComponentKind::CircleTanTan,
    },
    Registration {
        guids: &["510c4a63-b9bf-42e7-9d07-9d71290264da"],
        names: &["Line 2Plane", "Ln2Pl"],
        kind: ComponentKind::Line2Plane,
    },
    Registration {
        guids: &["575660b1-8c79-4b8d-9222-7ab4a6ddb359"],
        names: &["Rectangle 2Pt", "Rec 2Pt"],
        kind: ComponentKind::Rectangle2Pt,
    },
    Registration {
        guids: &["679a9c6a-ab97-4c20-b02c-680f9a9a1a44"],
        names: &["InEllipse"],
        kind: ComponentKind::InEllipse,
    },
    Registration {
        guids: &["75f4b0fd-9721-47b1-99e7-9c098b342e67"],
        names: &["BiArc"],
        kind: ComponentKind::BiArc,
    },
    Registration {
        guids: &["845527a6-5cea-4ae9-a667-96ae1667a4e8"],
        names: &["Polygon"],
        kind: ComponentKind::Polygon,
    },
    Registration {
        guids: &[
            "9d2583dd-6cf5-497c-8c40-c9a290598396",
            "f17c37ae-b44a-481a-bd65-b4398be55ec8",
        ],
        names: &["Arc SED"],
        kind: ComponentKind::ArcSED,
    },
    Registration {
        guids: &["9d8dec9c-3fd1-481c-9c3d-75ea5e15eb1a"],
        names: &["Modified Arc", "ModArc"],
        kind: ComponentKind::ModifiedArc,
    },
    Registration {
        guids: &["b9fde5fa-d654-4306-8ee1-6b69e6757604"],
        names: &["Line 4Pt", "Ln4Pt"],
        kind: ComponentKind::Line4Pt,
    },
    Registration {
        guids: &[
            "bb59bffc-f54c-4682-9778-f6c3fe74fce3",
            "fd9fe288-a188-4e9b-a464-1148876d18ed",
        ],
        names: &["Arc"],
        kind: ComponentKind::Arc,
    },
    Registration {
        guids: &["be52336f-a2e1-43b1-b5f5-178ba489508a"],
        names: &["Circle Fit", "FCircle"],
        kind: ComponentKind::CircleFit,
    },
    Registration {
        guids: &["c21e7bd5-b1f2-4448-ac56-206f98f90aa7"],
        names: &["TwoByFourJam", "2x4 Jam"],
        kind: ComponentKind::TwoByFourJam,
    },
    Registration {
        guids: &["d114323a-e6ee-4164-946b-e4ca0ce15efa"],
        names: &["Circle CNR"],
        kind: ComponentKind::CircleCNR,
    },
    Registration {
        guids: &["d6d68c93-d00f-4cd5-ba89-903c7f6be64c"],
        names: &["Tangent Lines (Ex)", "TanEx"],
        kind: ComponentKind::TangentLinesEx,
    },
    Registration {
        guids: &["dcaa922d-5491-4826-9a22-5adefa139f43"],
        names: &["Circle TanTanTan", "CircleTTT"],
        kind: ComponentKind::CircleTanTanTan,
    },
    Registration {
        guids: &["e0168047-c46a-48c6-8595-2fb3d8574f23"],
        names: &["Tangent Lines (In)", "TanIn"],
        kind: ComponentKind::TangentLinesIn,
    },
    Registration {
        guids: &["ea0f0996-af7a-481d-8099-09c041e6c2d5"],
        names: &["Tangent Lines", "Tan"],
        kind: ComponentKind::TangentLines,
    },
    Registration {
        guids: &["f1c0783b-60e9-42a7-8081-925bc755494c"],
        names: &["Tangent Arcs", "TArc"],
        kind: ComponentKind::TangentArcs,
    },
    Registration {
        guids: &["f4568ce6-aade-4511-8f32-f27d8a6bf9e9"],
        names: &["Polygon Edge", "PolEdge"],
        kind: ComponentKind::PolygonEdge,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Circle => evaluate_circle(inputs),
            Self::Rectangle => evaluate_rectangle(inputs),
            Self::FitLine => evaluate_fit_line(inputs),
            Self::InCircle => not_implemented(self.name()),
            Self::Arc3Pt => evaluate_arc_3pt(inputs),
            Self::Rectangle3Pt => not_implemented(self.name()),
            Self::Ellipse => not_implemented(self.name()),
            Self::Circle3Pt => not_implemented(self.name()),
            Self::Line => evaluate_line(inputs),
            Self::LineSDL => evaluate_line_sdl(inputs),
            Self::CircleTanTan => not_implemented(self.name()),
            Self::Line2Plane => not_implemented(self.name()),
            Self::Rectangle2Pt => not_implemented(self.name()),
            Self::InEllipse => not_implemented(self.name()),
            Self::BiArc => not_implemented(self.name()),
            Self::Polygon => evaluate_polygon(inputs),
            Self::ArcSED => not_implemented(self.name()),
            Self::ModifiedArc => not_implemented(self.name()),
            Self::Line4Pt => not_implemented(self.name()),
            Self::Arc => evaluate_arc(inputs),
            Self::CircleFit => not_implemented(self.name()),
            Self::TwoByFourJam => not_implemented(self.name()),
            Self::CircleCNR => evaluate_circle_cnr(inputs),
            Self::TangentLinesEx => not_implemented(self.name()),
            Self::CircleTanTanTan => not_implemented(self.name()),
            Self::TangentLinesIn => not_implemented(self.name()),
            Self::TangentLines => not_implemented(self.name()),
            Self::TangentArcs => not_implemented(self.name()),
            Self::PolygonEdge => not_implemented(self.name()),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Circle => "Circle",
            Self::Rectangle => "Rectangle",
            Self::FitLine => "Fit Line",
            Self::InCircle => "InCircle",
            Self::Arc3Pt => "Arc 3Pt",
            Self::Rectangle3Pt => "Rectangle 3Pt",
            Self::Ellipse => "Ellipse",
            Self::Circle3Pt => "Circle 3Pt",
            Self::Line => "Line",
            Self::LineSDL => "Line SDL",
            Self::CircleTanTan => "Circle TanTan",
            Self::Line2Plane => "Line 2Plane",
            Self::Rectangle2Pt => "Rectangle 2Pt",
            Self::InEllipse => "InEllipse",
            Self::BiArc => "BiArc",
            Self::Polygon => "Polygon",
            Self::ArcSED => "Arc SED",
            Self::ModifiedArc => "Modified Arc",
            Self::Line4Pt => "Line 4Pt",
            Self::Arc => "Arc",
            Self::CircleFit => "Circle Fit",
            Self::TwoByFourJam => "TwoByFourJam",
            Self::CircleCNR => "Circle CNR",
            Self::TangentLinesEx => "Tangent Lines (Ex)",
            Self::CircleTanTanTan => "Circle TanTanTan",
            Self::TangentLinesIn => "Tangent Lines (In)",
            Self::TangentLines => "Tangent Lines",
            Self::TangentArcs => "Tangent Arcs",
            Self::PolygonEdge => "Polygon Edge",
        }
    }
}

fn not_implemented(name: &str) -> ComponentResult {
    Err(ComponentError::NotYetImplemented(name.to_string()))
}

fn evaluate_arc_3pt(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new("Arc 3Pt component vereist drie punten"));
    }

    let p1_res = coerce_point(inputs.get(0).unwrap(), "Arc 3Pt");
    let p2_res = coerce_point(inputs.get(1).unwrap(), "Arc 3Pt");
    let p3_res = coerce_point(inputs.get(2).unwrap(), "Arc 3Pt");

    if p1_res.is_err() || p2_res.is_err() || p3_res.is_err() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_ARC.to_owned(), Value::Null);
        outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Null);
        return Ok(outputs);
    }

    let p1 = p1_res.unwrap();
    let p2 = p2_res.unwrap();
    let p3 = p3_res.unwrap();

    let (center, radius, normal) = match circle_from_three_points(p1, p2, p3) {
        Some(circle) => circle,
        None => {
            // Collineaire punten, maak een lijn
            let points = vec![Value::Point(p1), Value::Point(p3)];
            let length = vector_length(subtract(p3, p1));
            let mut outputs = BTreeMap::new();
            outputs.insert(PIN_OUTPUT_ARC.to_owned(), Value::List(points));
            outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(length));
            return Ok(outputs);
        }
    };

    let x_axis = match safe_normalized(subtract(p1, center)) {
        Some((axis, _)) => axis,
        None => {
            let mut outputs = BTreeMap::new();
            outputs.insert(PIN_OUTPUT_ARC.to_owned(), Value::Null);
            outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Null);
            return Ok(outputs);
        }
    };
    let y_axis = cross(normal, x_axis);
    if vector_length_squared(y_axis) < EPSILON {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_ARC.to_owned(), Value::Null);
        outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Null);
        return Ok(outputs);
    }

    let plane = Plane::from_axes(center, x_axis, y_axis, normal);

    let (u1, v1) = plane.project(p1);
    let (u2, v2) = plane.project(p2);
    let (u3, v3) = plane.project(p3);

    let mut angle1 = v1.atan2(u1);
    let mut angle2 = unwrap_angle(v2.atan2(u2), angle1);
    if angle2 < angle1 {
        angle2 += std::f64::consts::TAU;
    }

    let mut angle3 = unwrap_angle(v3.atan2(u3), angle2);
    if angle3 < angle2 {
        angle3 += std::f64::consts::TAU;
    }

    angle1 = unwrap_angle(angle1, 0.0);

    let (points, length) = create_arc_points_from_angles(&plane, radius, angle1, angle3);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_ARC.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(length));
    Ok(outputs)
}

fn circle_from_three_points(
    p1: [f64; 3],
    p2: [f64; 3],
    p3: [f64; 3],
) -> Option<([f64; 3], f64, [f64; 3])> {
    let v21 = subtract(p2, p1);
    let v31 = subtract(p3, p1);

    let normal = cross(v21, v31);
    if vector_length_squared(normal) < EPSILON * EPSILON {
        return None; // collineaire punten
    }
    let normal = normalize(normal);

    let row1 = scale(v21, 2.0);
    let row2 = scale(v31, 2.0);
    let row3 = normal;

    let matrix = [row1, row2, row3];
    let rhs = [
        dot(p2, p2) - dot(p1, p1),
        dot(p3, p3) - dot(p1, p1),
        dot(normal, p1),
    ];

    let center = solve_linear_3x3(matrix, rhs)?;
    let radius = vector_length(subtract(p1, center));

    Some((center, radius, normal))
}

fn evaluate_circle_cnr(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "CircleCNR component vereist een middelpunt, normaalvector en straal",
        ));
    }

    let center = coerce_point(inputs.get(0).unwrap(), "CircleCNR")?;
    let normal = coerce_point(inputs.get(1).unwrap(), "CircleCNR")?;
    let radius = coerce_number(inputs.get(2), "CircleCNR")?;

    if radius <= 0.0 {
        return Err(ComponentError::new(
            "CircleCNR component vereist een radius groter dan nul",
        ));
    }

    let plane = Plane::from_origin_and_normal(center, normal);
    let points = sample_circle_points(&plane, radius, CURVE_SEGMENTS);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CIRCLE.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );
    Ok(outputs)
}

fn evaluate_line_sdl(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "LineSDL component vereist een startpunt, richting en lengte",
        ));
    }

    let start = coerce_point(inputs.get(0).unwrap(), "LineSDL")?;
    let direction = coerce_point(inputs.get(1).unwrap(), "LineSDL")?;
    let length = coerce_number(inputs.get(2), "LineSDL")?;

    let end = add(start, scale(normalize(direction), length));

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_LINE.to_owned(),
        Value::CurveLine { p1: start, p2: end },
    );
    Ok(outputs)
}

fn evaluate_line(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Line component vereist twee punten"));
    }

    let starts = extract_points(inputs.get(0).unwrap(), "Line start")?;
    let ends = extract_points(inputs.get(1).unwrap(), "Line end")?;

    if starts.is_empty() {
        return Err(ComponentError::new(
            "Line component vereist minimaal één startpunt",
        ));
    }

    if ends.is_empty() {
        return Err(ComponentError::new(
            "Line component vereist minimaal één eindpunt",
        ));
    }

    let count = starts.len().max(ends.len());
    let mut values = Vec::with_capacity(count);

    for index in 0..count {
        let start = *starts
            .get(index)
            .or_else(|| starts.last())
            .expect("starts is niet leeg");
        let end = *ends
            .get(index)
            .or_else(|| ends.last())
            .expect("ends is niet leeg");

        let value = if start != end {
            Value::CurveLine { p1: start, p2: end }
        } else {
            Value::Null
        };
        values.push(value);
    }

    let output = if values.len() == 1 {
        values.into_iter().next().unwrap()
    } else {
        Value::List(values)
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_LINE.to_owned(), output);
    Ok(outputs)
}

fn evaluate_arc(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Arc component vereist een vlak, radius en hoek",
        ));
    }

    let plane = parse_plane(inputs.get(0), "Arc")?;
    let radius = coerce_number(inputs.get(1), "Arc")?;
    let angle = coerce_number(inputs.get(2), "Arc")?;

    if radius <= 0.0 {
        return Err(ComponentError::new(
            "Arc component vereist een radius groter dan nul",
        ));
    }

    let (points, length) = create_arc_points(&plane, radius, angle);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_ARC.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(length));

    Ok(outputs)
}

fn create_arc_points(plane: &Plane, radius: f64, angle: f64) -> (Vec<[f64; 3]>, f64) {
    create_arc_points_from_angles(plane, radius, 0.0, angle)
}

fn segments_for_angle(_angle: f64) -> usize {
    CURVE_SEGMENTS
}

fn create_arc_points_from_angles(
    plane: &Plane,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
) -> (Vec<[f64; 3]>, f64) {
    let total_angle = end_angle - start_angle;
    let mut points = Vec::new();
    let segments = segments_for_angle(total_angle);
    let angle_step = if segments == 0 {
        0.0
    } else {
        total_angle / segments as f64
    };

    for i in 0..=segments {
        let current_angle = start_angle + i as f64 * angle_step;
        points.push(plane.apply(radius * current_angle.cos(), radius * current_angle.sin()));
    }

    let length = radius * total_angle.abs();
    (points, length)
}

fn evaluate_polygon(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Polygon component vereist een vlak, radius en segmenten",
        ));
    }

    let plane = parse_plane(inputs.get(0), "Polygon")?;
    let radius = coerce_number(inputs.get(1), "Polygon")?;
    let segments = coerce_number(inputs.get(2), "Polygon")? as usize;
    let fillet_radius = coerce_number(inputs.get(3), "Polygon").unwrap_or(0.0);

    if radius <= 0.0 {
        return Err(ComponentError::new(
            "Polygon component vereist een radius groter dan nul",
        ));
    }
    if segments < 3 {
        return Err(ComponentError::new(
            "Polygon component vereist ten minste 3 segmenten",
        ));
    }

    let (points, length) = create_polygon_points(&plane, radius, segments, fillet_radius);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_POLYGON.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(length));

    Ok(outputs)
}

fn create_polygon_points(
    plane: &Plane,
    radius: f64,
    segments: usize,
    _fillet_radius: f64,
) -> (Vec<[f64; 3]>, f64) {
    let mut points = Vec::new();
    let angle_step = std::f64::consts::TAU / segments as f64;

    for i in 0..segments {
        let angle = i as f64 * angle_step;
        points.push(plane.apply(radius * angle.cos(), radius * angle.sin()));
    }

    if let Some(first) = points.first().copied() {
        points.push(first);
    }

    let length = segments as f64
        * 2.0
        * radius
        * (std::f64::consts::TAU / (2.0 * segments as f64)).sin();

    (points, length)
}

fn evaluate_fit_line(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Fit Line component vereist een lijst van punten",
        ));
    }

    let points = coerce_points(inputs.get(0), "Fit Line")?;

    if points.len() < 2 {
        return Err(ComponentError::new(
            "Fit Line component vereist ten minste twee punten",
        ));
    }

    let (p1, p2) = find_farthest_points(&points);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_LINE.to_owned(), Value::CurveLine { p1, p2 });
    Ok(outputs)
}

fn coerce_points(value: Option<&Value>, context: &str) -> Result<Vec<[f64; 3]>, ComponentError> {
    let value = value
        .ok_or_else(|| ComponentError::new(format!("{} vereist een lijst van punten", context)))?;

    match value {
        Value::List(values) => values.iter().map(|v| coerce_point(v, context)).collect(),
        Value::Point(p) => Ok(vec![*p]),
        other => Err(ComponentError::new(format!(
            "{} verwacht een lijst van punten, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn find_farthest_points(points: &[[f64; 3]]) -> ([f64; 3], [f64; 3]) {
    let mut max_dist_sq = -1.0;
    let mut p1 = [0.0; 3];
    let mut p2 = [0.0; 3];

    for (i, &pi) in points.iter().enumerate() {
        for &pj in points.iter().skip(i + 1) {
            let dist_sq = vector_length_squared(subtract(pi, pj));
            if dist_sq > max_dist_sq {
                max_dist_sq = dist_sq;
                p1 = pi;
                p2 = pj;
            }
        }
    }

    (p1, p2)
}

fn evaluate_rectangle(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Rectangle component vereist een x-grootte en y-grootte",
        ));
    }

    let plane = parse_plane(inputs.get(0), "Rectangle")?;
    let x_size = coerce_number(inputs.get(1), "Rectangle")?;
    let y_size = coerce_number(inputs.get(2), "Rectangle")?;
    let radius = coerce_number(inputs.get(3), "Rectangle").unwrap_or(0.0);

    if x_size <= 0.0 || y_size <= 0.0 {
        return Err(ComponentError::new(
            "Rectangle component vereist groottes groter dan nul",
        ));
    }

    let (points, length) = create_rectangle_points(&plane, x_size, y_size, radius);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_RECTANGLE.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(length));

    Ok(outputs)
}

fn create_rectangle_points(
    plane: &Plane,
    x_size: f64,
    y_size: f64,
    radius: f64,
) -> (Vec<[f64; 3]>, f64) {
    let mut points = Vec::new();
    let half_x = x_size / 2.0;
    let half_y = y_size / 2.0;

    let max_radius = half_x.min(half_y);
    let radius = if radius > max_radius {
        max_radius
    } else {
        radius
    };
    let radius = if radius < 0.0 { 0.0 } else { radius };

    let length;

    if radius < EPSILON {
        points.push(plane.apply(half_x, half_y));
        points.push(plane.apply(-half_x, half_y));
        points.push(plane.apply(-half_x, -half_y));
        points.push(plane.apply(half_x, -half_y));
        length = 2.0 * x_size + 2.0 * y_size;
    } else {
        let segments_per_corner = segments_for_angle(std::f64::consts::TAU / 4.0);

        // Corner centers in UV coordinates
        let c_tr_uv = (half_x - radius, half_y - radius);
        let c_tl_uv = (-half_x + radius, half_y - radius);
        let c_bl_uv = (-half_x + radius, -half_y + radius);
        let c_br_uv = (half_x - radius, -half_y + radius);

        // Arc for top-right corner (from 0 to PI/2)
        for i in 0..=segments_per_corner {
            let angle =
                0.0 + (std::f64::consts::TAU / 4.0) * (i as f64 / segments_per_corner as f64);
            points.push(plane.apply(
                c_tr_uv.0 + radius * angle.cos(),
                c_tr_uv.1 + radius * angle.sin(),
            ));
        }

        // Arc for top-left corner (from PI/2 to PI)
        for i in 1..=segments_per_corner {
            let angle = (std::f64::consts::TAU / 4.0)
                + (std::f64::consts::TAU / 4.0) * (i as f64 / segments_per_corner as f64);
            points.push(plane.apply(
                c_tl_uv.0 + radius * angle.cos(),
                c_tl_uv.1 + radius * angle.sin(),
            ));
        }

        // Arc for bottom-left corner (from PI to 3*PI/2)
        for i in 1..=segments_per_corner {
            let angle = (std::f64::consts::TAU / 2.0)
                + (std::f64::consts::TAU / 4.0) * (i as f64 / segments_per_corner as f64);
            points.push(plane.apply(
                c_bl_uv.0 + radius * angle.cos(),
                c_bl_uv.1 + radius * angle.sin(),
            ));
        }

        // Arc for bottom-right corner (from 3*PI/2 to 2*PI)
        for i in 1..=segments_per_corner {
            let angle = (std::f64::consts::TAU * 3.0 / 4.0)
                + (std::f64::consts::TAU / 4.0) * (i as f64 / segments_per_corner as f64);
            points.push(plane.apply(
                c_br_uv.0 + radius * angle.cos(),
                c_br_uv.1 + radius * angle.sin(),
            ));
        }

        length = 2.0 * (x_size - 2.0 * radius)
            + 2.0 * (y_size - 2.0 * radius)
            + std::f64::consts::TAU * radius;
    }

    if !points.is_empty() {
        points.push(points[0]);
    }

    (points, length)
}

fn evaluate_circle(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Circle component vereist een vlak en straal",
        ));
    }

    let plane = parse_plane(inputs.get(0), "Circle")?;
    let radius = coerce_number(inputs.get(1), "Circle")?;

    if radius <= 0.0 {
        return Err(ComponentError::new(
            "Circle component vereist een straal groter dan nul",
        ));
    }

    let points = sample_circle_points(&plane, radius, CURVE_SEGMENTS);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CIRCLE.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );
    Ok(outputs)
}

fn sample_circle_points(plane: &Plane, radius: f64, segments: usize) -> Vec<[f64; 3]> {
    let segments = segments.max(3);
    let mut points = Vec::with_capacity(segments + 1);
    let step = std::f64::consts::TAU / segments as f64;
    for i in 0..segments {
        let angle = i as f64 * step;
        let point = plane.apply(radius * angle.cos(), radius * angle.sin());
        points.push(point);
    }
    if let Some(first) = points.first().copied() {
        points.push(first);
    }
    points
}

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    match value {
        None => Err(ComponentError::new(format!(
            "{} vereist een numerieke waarde",
            context
        ))),
        Some(value) => match value {
            Value::Number(number) => Ok(*number),
            Value::Boolean(boolean) => Ok(if *boolean { 1.0 } else { 0.0 }),
            Value::List(values) if values.len() == 1 => coerce_number(values.get(0), context),
            Value::Text(text) => text.trim().parse::<f64>().map_err(|_| {
                ComponentError::new(format!(
                    "{} kon tekst '{}' niet als getal interpreteren",
                    context, text
                ))
            }),
            other => Err(ComponentError::new(format!(
                "{} verwacht een getal, kreeg {}",
                context,
                other.kind()
            ))),
        },
    }
}

fn parse_plane(value: Option<&Value>, context: &str) -> Result<Plane, ComponentError> {
    let Some(value) = value else {
        return Ok(Plane::default());
    };
    match value {
        Value::Null => return Ok(Plane::default()),
        Value::List(values) if values.len() >= 3 => {
            let origin = coerce_point(&values[0], context)?;
            let point_x = coerce_point(&values[1], context)?;
            let point_y = coerce_point(&values[2], context)?;
            Ok(Plane::from_points(origin, point_x, point_y))
        }
        Value::Point(point) => Ok(Plane::from_origin(*point)),
        Value::List(values) if values.len() == 1 => parse_plane(values.get(0), context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een vlak, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_point(value: &Value, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Point(point) | Value::Vector(point) => Ok(*point),
        Value::List(values) => {
            if values.len() == 1 {
                return coerce_point(&values[0], context);
            }

            if let Some(point) = try_point_from_list(values, context)? {
                return Ok(point);
            }

            if values.len() >= 3 {
                let x = coerce_number(Some(&values[0]), context)?;
                let y = coerce_number(Some(&values[1]), context)?;
                let z = coerce_number(Some(&values[2]), context)?;
                return Ok([x, y, z]);
            }

            Err(ComponentError::new(format!(
                "{} verwacht een punt, kreeg {}",
                context,
                ValueKind::List
            )))
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een punt, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn extract_points(value: &Value, context: &str) -> Result<Vec<[f64; 3]>, ComponentError> {
    let mut points = Vec::new();
    collect_points_recursive(value, context, &mut points)?;
    Ok(points)
}

fn collect_points_recursive(
    value: &Value,
    context: &str,
    output: &mut Vec<[f64; 3]>,
) -> Result<(), ComponentError> {
    match value {
        Value::Point(point) | Value::Vector(point) => {
            output.push(*point);
            Ok(())
        }
        Value::List(values) if values.is_empty() => Ok(()),
        Value::List(values) => {
            if let Some(point) = try_point_from_list(values, context)? {
                output.push(point);
                return Ok(());
            }

            for entry in values {
                collect_points_recursive(entry, context, output)?;
            }
            Ok(())
        }
        other => {
            output.push(coerce_point(other, context)?);
            Ok(())
        }
    }
}

fn try_point_from_list(
    values: &[Value],
    context: &str,
) -> Result<Option<[f64; 3]>, ComponentError> {
    if values.len() < 3 {
        return Ok(None);
    }

    let x = match coerce_number(Some(&values[0]), context) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    let y = match coerce_number(Some(&values[1]), context) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    let z = match coerce_number(Some(&values[2]), context) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };

    Ok(Some([x, y, z]))
}

#[derive(Debug, Clone, Copy)]
struct Plane {
    origin: [f64; 3],
    x_axis: [f64; 3],
    y_axis: [f64; 3],
    _z_axis: [f64; 3],
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            _z_axis: [0.0, 0.0, 1.0],
        }
    }
}

impl Plane {
    fn from_origin(origin: [f64; 3]) -> Self {
        Self {
            origin,
            ..Self::default()
        }
    }

    fn from_points(origin: [f64; 3], point_x: [f64; 3], point_y: [f64; 3]) -> Self {
        let x_axis = subtract(point_x, origin);
        let y_axis = subtract(point_y, origin);
        let z_axis = cross(x_axis, y_axis);
        Self::normalize_axes(origin, x_axis, y_axis, z_axis)
    }

    fn from_origin_and_normal(origin: [f64; 3], z_axis: [f64; 3]) -> Self {
        let x_axis = orthogonal_vector(z_axis);
        let y_axis = cross(z_axis, x_axis);
        Self::normalize_axes(origin, x_axis, y_axis, z_axis)
    }

    fn from_axes(origin: [f64; 3], x_axis: [f64; 3], y_axis: [f64; 3], z_axis: [f64; 3]) -> Self {
        Self::normalize_axes(origin, x_axis, y_axis, z_axis)
    }

    fn normalize_axes(
        origin: [f64; 3],
        x_axis: [f64; 3],
        y_axis: [f64; 3],
        z_axis: [f64; 3],
    ) -> Self {
        let z_axis = safe_normalized(z_axis)
            .map(|(vector, _)| vector)
            .unwrap_or([0.0, 0.0, 1.0]);

        let mut x_axis = safe_normalized(x_axis)
            .map(|(vector, _)| vector)
            .unwrap_or_else(|| orthogonal_vector(z_axis));

        let mut y_axis = safe_normalized(y_axis)
            .map(|(vector, _)| vector)
            .unwrap_or_else(|| normalize(cross(z_axis, x_axis)));

        let x_cross = cross(y_axis, z_axis);
        if vector_length_squared(x_cross) < EPSILON {
            x_axis = orthogonal_vector(z_axis);
        } else {
            x_axis = normalize(x_cross);
        }

        let y_cross = cross(z_axis, x_axis);
        if vector_length_squared(y_cross) < EPSILON {
            y_axis = orthogonal_vector(x_axis);
        } else {
            y_axis = normalize(y_cross);
        }

        Self {
            origin,
            x_axis,
            y_axis,
            _z_axis: z_axis,
        }
    }

    fn apply(&self, u: f64, v: f64) -> [f64; 3] {
        add(
            self.origin,
            add(scale(self.x_axis, u), scale(self.y_axis, v)),
        )
    }

    fn project(&self, point: [f64; 3]) -> (f64, f64) {
        let delta = subtract(point, self.origin);
        (dot(delta, self.x_axis), dot(delta, self.y_axis))
    }
}

const EPSILON: f64 = 1e-9;

fn unwrap_angle(angle: f64, reference: f64) -> f64 {
    let mut result = angle;
    while result < reference - std::f64::consts::PI {
        result += std::f64::consts::TAU;
    }
    while result > reference + std::f64::consts::PI {
        result -= std::f64::consts::TAU;
    }
    result
}

fn solve_linear_3x3(mut a: [[f64; 3]; 3], mut b: [f64; 3]) -> Option<[f64; 3]> {
    for i in 0..3 {
        let mut pivot_row = i;
        let mut pivot_value = a[i][i].abs();
        for row in (i + 1)..3 {
            let value = a[row][i].abs();
            if value > pivot_value {
                pivot_value = value;
                pivot_row = row;
            }
        }

        if pivot_value < EPSILON {
            return None;
        }

        if pivot_row != i {
            a.swap(i, pivot_row);
            b.swap(i, pivot_row);
        }

        let pivot = a[i][i];
        for col in 0..3 {
            a[i][col] /= pivot;
        }
        b[i] /= pivot;

        for row in 0..3 {
            if row == i {
                continue;
            }
            let factor = a[row][i];
            if factor.abs() < EPSILON {
                continue;
            }
            for col in 0..3 {
                a[row][col] -= factor * a[i][col];
            }
            b[row] -= factor * b[i];
        }
    }

    Some(b)
}

fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(v: [f64; 3], factor: f64) -> [f64; 3] {
    [v[0] * factor, v[1] * factor, v[2] * factor]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vector_length_squared(v: [f64; 3]) -> f64 {
    dot(v, v)
}

fn vector_length(v: [f64; 3]) -> f64 {
    vector_length_squared(v).sqrt()
}

fn safe_normalized(v: [f64; 3]) -> Option<([f64; 3], f64)> {
    let length = vector_length(v);
    if length < EPSILON {
        None
    } else {
        Some((scale(v, 1.0 / length), length))
    }
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    safe_normalized(v)
        .map(|(vector, _)| vector)
        .unwrap_or([0.0, 0.0, 0.0])
}

fn orthogonal_vector(reference: [f64; 3]) -> [f64; 3] {
    let mut candidate = if reference[0].abs() < reference[1].abs() {
        [0.0, -reference[2], reference[1]]
    } else {
        [-reference[2], 0.0, reference[0]]
    };
    if vector_length_squared(candidate) < EPSILON {
        candidate = [reference[1], -reference[0], 0.0];
    }
    let normalized = normalize(candidate);
    if vector_length_squared(normalized) < EPSILON {
        [1.0, 0.0, 0.0]
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Component, ComponentKind, CURVE_SEGMENTS, PIN_OUTPUT_ARC, PIN_OUTPUT_CIRCLE,
        PIN_OUTPUT_LENGTH, PIN_OUTPUT_LINE, PIN_OUTPUT_POLYGON, PIN_OUTPUT_RECTANGLE,
        segments_for_angle,
    };
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    #[test]
    fn rectangle_generates_points_and_length() {
        let component = ComponentKind::Rectangle;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Number(10.0),
                    Value::Number(20.0),
                    Value::Number(0.0),
                ],
                &MetaMap::new(),
            )
            .expect("rectangle generated");

        let Some(Value::List(points)) = outputs.get(PIN_OUTPUT_RECTANGLE) else {
            panic!("expected list of points");
        };
        assert_eq!(points.len(), 5);

        let Some(Value::Number(length)) = outputs.get(PIN_OUTPUT_LENGTH) else {
            panic!("expected length");
        };
        assert!((length - 60.0).abs() < 1e-9);
    }

    #[test]
    fn rectangle_with_radius_generates_points_and_length() {
        let component = ComponentKind::Rectangle;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Number(10.0),
                    Value::Number(20.0),
                    Value::Number(2.0),
                ],
                &MetaMap::new(),
            )
            .expect("rectangle generated");

        let Some(Value::List(points)) = outputs.get(PIN_OUTPUT_RECTANGLE) else {
            panic!("expected list of points");
        };
        let segments_per_corner = segments_for_angle(std::f64::consts::TAU / 4.0);
        assert_eq!(points.len(), 4 * segments_per_corner + 2);

        let Some(Value::Number(length)) = outputs.get(PIN_OUTPUT_LENGTH) else {
            panic!("expected length");
        };
        assert!(
            (length - (2.0 * (10.0 - 4.0) + 2.0 * (20.0 - 4.0) + std::f64::consts::TAU * 2.0))
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn rectangle_with_radius_on_rotated_plane() {
        let component = ComponentKind::Rectangle;
        let outputs = component
            .evaluate(
                &[
                    Value::List(vec![
                        Value::Point([1.0, 2.0, 3.0]),
                        Value::Point([2.0, 2.0, 3.0]),
                        Value::Point([1.0, 3.0, 3.0]),
                    ]),
                    Value::Number(10.0),
                    Value::Number(20.0),
                    Value::Number(2.0),
                ],
                &MetaMap::new(),
            )
            .expect("rectangle generated");

        let Some(Value::List(points)) = outputs.get(PIN_OUTPUT_RECTANGLE) else {
            panic!("expected list of points");
        };
        let segments_per_corner = segments_for_angle(std::f64::consts::TAU / 4.0);
        assert_eq!(points.len(), 4 * segments_per_corner + 2);
    }

    #[test]
    fn rectangle_without_plane_uses_default() {
        let component = ComponentKind::Rectangle;
        let outputs = component
            .evaluate(
                &[
                    Value::Null,
                    Value::Number(10.0),
                    Value::Number(20.0),
                    Value::Number(0.0),
                ],
                &MetaMap::new(),
            )
            .expect("rectangle generated");

        let Some(Value::List(points)) = outputs.get(PIN_OUTPUT_RECTANGLE) else {
            panic!("expected list of points");
        };
        assert_eq!(points.len(), 5); // 4 corners + closed loop point

        let Some(Value::Point(p)) = points.get(0) else {
            panic!("expected point");
        };
        // Expect a point on the XY plane
        assert!((p[2] - 0.0).abs() < 1e-9);

        let Some(Value::Number(length)) = outputs.get(PIN_OUTPUT_LENGTH) else {
            panic!("expected length");
        };
        assert!((length - 60.0).abs() < 1e-9);
    }

    #[test]
    fn fit_line_finds_farthest_points() {
        let component = ComponentKind::FitLine;
        let outputs = component
            .evaluate(
                &[Value::List(vec![
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Point([10.0, 0.0, 0.0]),
                    Value::Point([1.0, 1.0, 0.0]),
                    Value::Point([9.0, -1.0, 0.0]),
                ])],
                &MetaMap::new(),
            )
            .expect("fit line generated");

        let Some(Value::CurveLine { p1, p2 }) = outputs.get(PIN_OUTPUT_LINE) else {
            panic!("expected a line");
        };

        assert!(
            (*p1 == [0.0, 0.0, 0.0] && *p2 == [10.0, 0.0, 0.0])
                || (*p1 == [10.0, 0.0, 0.0] && *p2 == [0.0, 0.0, 0.0])
        );
    }

    #[test]
    fn polygon_generates_points_and_length() {
        let component = ComponentKind::Polygon;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Number(10.0),
                    Value::Number(6.0),
                    Value::Number(0.0),
                ],
                &MetaMap::new(),
            )
            .expect("polygon generated");

        let Some(Value::List(points)) = outputs.get(PIN_OUTPUT_POLYGON) else {
            panic!("expected list of points");
        };
        assert_eq!(points.len(), 7);

        let Some(Value::Number(length)) = outputs.get(PIN_OUTPUT_LENGTH) else {
            panic!("expected length");
        };
        assert!((length - 60.0).abs() < 1e-9);
    }

    #[test]
    fn arc_generates_points_and_length() {
        let component = ComponentKind::Arc;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Number(10.0),
                    Value::Number(std::f64::consts::PI),
                ],
                &MetaMap::new(),
            )
            .expect("arc generated");

        let Some(Value::List(points)) = outputs.get(PIN_OUTPUT_ARC) else {
            panic!("expected list of points");
        };
        let expected_points = segments_for_angle(std::f64::consts::PI) + 1;
        assert_eq!(points.len(), expected_points);

        let Some(Value::Number(length)) = outputs.get(PIN_OUTPUT_LENGTH) else {
            panic!("expected length");
        };
        assert!((length - 10.0 * std::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn circle_requires_plane_and_radius() {
        let component = ComponentKind::Circle;
        let err = component.evaluate(&[], &MetaMap::new()).unwrap_err();
        assert!(err.message().contains("vlak"));
    }

    #[test]
    fn circle_rejects_non_positive_radius() {
        let component = ComponentKind::Circle;
        let err = component
            .evaluate(
                &[
                    Value::List(vec![
                        Value::Point([0.0, 0.0, 0.0]),
                        Value::Point([1.0, 0.0, 0.0]),
                        Value::Point([0.0, 1.0, 0.0]),
                    ]),
                    Value::Number(-1.0),
                ],
                &MetaMap::new(),
            )
            .unwrap_err();
        assert!(err.message().contains("straal"));
    }

    #[test]
    fn circle_generates_points_on_plane() {
        let component = ComponentKind::Circle;
        let outputs = component
            .evaluate(
                &[
                    Value::List(vec![
                        Value::Point([1.0, 2.0, 3.0]),
                        Value::Point([2.0, 2.0, 3.0]),
                        Value::Point([1.0, 3.0, 3.0]),
                    ]),
                    Value::Number(2.0),
                ],
                &MetaMap::new(),
            )
            .expect("circle generated");
        let Some(Value::List(points)) = outputs.get(PIN_OUTPUT_CIRCLE) else {
            panic!("expected list of points");
        };
        assert_eq!(points.len(), CURVE_SEGMENTS + 1);
        assert!(matches!(points[0], Value::Point(_)));
        assert!(matches!(points.last(), Some(Value::Point(_))));
    }

    #[test]
    fn line_sdl_creates_line() {
        let component = ComponentKind::LineSDL;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([1.0, 2.0, 3.0]),
                    Value::Point([1.0, 0.0, 0.0]),
                    Value::Number(10.0),
                ],
                &MetaMap::new(),
            )
            .expect("line sdl generated");

        let Some(Value::CurveLine { p1, p2 }) = outputs.get(PIN_OUTPUT_LINE) else {
            panic!("expected a line");
        };

        assert_eq!(*p1, [1.0, 2.0, 3.0]);
        assert!((p2[0] - 11.0).abs() < 1e-9);
        assert!((p2[1] - 2.0).abs() < 1e-9);
        assert!((p2[2] - 3.0).abs() < 1e-9);
    }

    #[test]
    fn circle_cnr_creates_circle() {
        let component = ComponentKind::CircleCNR;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([1.0, 2.0, 3.0]),
                    Value::Point([0.0, 0.0, 1.0]),
                    Value::Number(10.0),
                ],
                &MetaMap::new(),
            )
            .expect("circle cnr generated");

        let Some(Value::List(points)) = outputs.get(PIN_OUTPUT_CIRCLE) else {
            panic!("expected list of points");
        };
        assert_eq!(points.len(), CURVE_SEGMENTS + 1);
    }

    #[test]
    fn arc_3pt_creates_arc() {
        let component = ComponentKind::Arc3Pt;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([0.0, 0.0, 0.0]),
                    Value::Point([10.0, 10.0, 0.0]),
                    Value::Point([20.0, 0.0, 0.0]),
                ],
                &MetaMap::new(),
            )
            .expect("arc 3pt generated");

        let Some(Value::List(points)) = outputs.get(PIN_OUTPUT_ARC) else {
            panic!("expected list of points");
        };
        assert!(points.len() > 1);

        let Value::Point(first) = points.first().expect("at least one point") else {
            panic!("expected first arc point");
        };
        let Value::Point(last) = points.last().expect("at least one point") else {
            panic!("expected last arc point");
        };

        assert!((first[0] - 0.0).abs() < 1e-9);
        assert!((first[1] - 0.0).abs() < 1e-9);
        assert!((last[0] - 20.0).abs() < 1e-9);
        assert!((last[1] - 0.0).abs() < 1e-9);
    }

    #[test]
    fn arc_3pt_respects_orientation_in_space() {
        let component = ComponentKind::Arc3Pt;
        let outputs = component
            .evaluate(
                &[
                    Value::Point([1.0, 0.0, 0.0]),
                    Value::Point([0.0, 1.0, 1.0]),
                    Value::Point([-1.0, 0.0, 0.0]),
                ],
                &MetaMap::new(),
            )
            .expect("arc 3pt generated");

        let Some(Value::List(points)) = outputs.get(PIN_OUTPUT_ARC) else {
            panic!("expected list of points");
        };

        let Value::Point(first) = points.first().expect("at least one point") else {
            panic!("expected first arc point");
        };
        let Value::Point(last) = points.last().expect("at least one point") else {
            panic!("expected last arc point");
        };

        assert!((first[0] - 1.0).abs() < 1e-9);
        assert!((first[1] - 0.0).abs() < 1e-9);
        assert!((first[2] - 0.0).abs() < 1e-9);
        assert!((last[0] + 1.0).abs() < 1e-9);
        assert!((last[1] - 0.0).abs() < 1e-9);
        assert!((last[2] - 0.0).abs() < 1e-9);
    }

    #[test]
    fn line_creates_curve_line_from_points() {
        let component = ComponentKind::Line;
        let outputs = component
            .evaluate(
                &[Value::Point([0.0, 0.0, 0.0]), Value::Point([1.0, 0.0, 0.0])],
                &MetaMap::new(),
            )
            .expect("line created");
        assert!(matches!(
            outputs.get(PIN_OUTPUT_LINE),
            Some(Value::CurveLine { .. })
        ));
    }

    #[test]
    fn line_outputs_multiple_segments_for_matching_lists() {
        let component = ComponentKind::Line;
        let inputs = [
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([2.0, 0.0, 0.0]),
            ]),
            Value::List(vec![
                Value::Point([0.0, 1.0, 0.0]),
                Value::Point([1.0, 1.0, 0.0]),
                Value::Point([2.0, 1.0, 0.0]),
            ]),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("parallel lists handled");
        let Some(Value::List(lines)) = outputs.get(PIN_OUTPUT_LINE) else {
            panic!("expected line list");
        };
        assert_eq!(lines.len(), 3);
        assert!(
            lines
                .iter()
                .all(|value| matches!(value, Value::CurveLine { .. }))
        );
    }

    #[test]
    fn line_repeats_single_point_to_match_list_length() {
        let component = ComponentKind::Line;
        let inputs = [
            Value::Point([0.0, 0.0, 0.0]),
            Value::List(vec![
                Value::Point([1.0, 0.0, 0.0]),
                Value::Point([2.0, 0.0, 0.0]),
            ]),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("single point repeated");
        let Some(Value::List(lines)) = outputs.get(PIN_OUTPUT_LINE) else {
            panic!("expected line list");
        };
        assert_eq!(lines.len(), 2);
        assert!(
            lines
                .iter()
                .all(|value| matches!(value, Value::CurveLine { .. }))
        );
    }

    #[test]
    fn line_collapses_single_item_lists() {
        let component = ComponentKind::Line;
        let inputs = [
            Value::List(vec![Value::Point([0.0, 0.0, 0.0])]),
            Value::List(vec![Value::Point([0.0, 1.0, 0.0])]),
        ];
        let outputs = component
            .evaluate(&inputs, &MetaMap::new())
            .expect("list inputs handled");
        assert!(matches!(
            outputs.get(PIN_OUTPUT_LINE),
            Some(Value::CurveLine { .. })
        ));
    }

    #[test]
    fn line_returns_null_for_identical_points() {
        let component = ComponentKind::Line;
        let outputs = component
            .evaluate(
                &[Value::Point([0.0, 0.0, 0.0]), Value::Point([0.0, 0.0, 0.0])],
                &MetaMap::new(),
            )
            .unwrap();
        assert!(matches!(outputs.get(PIN_OUTPUT_LINE), Some(Value::Null)));
    }

    #[test]
    fn line_returns_null_for_null_input() {
        let component = ComponentKind::Line;
        let err = component
            .evaluate(
                &[Value::Null, Value::Point([0.0, 0.0, 0.0])],
                &MetaMap::new(),
            )
            .unwrap_err();
        assert!(err.message().contains("kreeg Null"));
    }
}
