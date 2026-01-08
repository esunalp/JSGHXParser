//! Implementaties van Grasshopper "Curve → Primitive" componenten.
//!
//! This module uses the `geom::curve` primitives for curve construction and
//! `geom::tessellation` for adaptive tessellation. Components remain thin
//! wrappers that coerce inputs, build geom curves, and return tessellated output.

use std::collections::BTreeMap;

use crate::components::coerce;
use crate::geom::{
    Arc3, Circle3, CurveTessellationOptions, Ellipse3, Line3, Point3 as GeomPoint3,
    Vec3 as GeomVec3, tessellate_curve_adaptive_points,
};
use crate::graph::node::MetaMap;
use crate::graph::value::{Domain, Value};

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_CIRCLE: &str = "C";
const PIN_OUTPUT_RECTANGLE: &str = "R";
const PIN_OUTPUT_LENGTH: &str = "L";
const PIN_OUTPUT_LINE: &str = "L";
const PIN_OUTPUT_POLYGON: &str = "P";
const PIN_OUTPUT_ARC: &str = "A";

/// Default maximum deviation for adaptive curve tessellation.
const DEFAULT_MAX_DEVIATION: f64 = 0.01;

/// Default maximum number of segments for adaptive curve tessellation.
const DEFAULT_MAX_SEGMENTS: usize = 64;

/// Legacy: Fixed segment count for backward compatibility fallback.
const CURVE_SEGMENTS: usize = 32;

// ============================================================================
// Helper functions for conversion between [f64; 3] and geom types
// ============================================================================

/// Converts an array [f64; 3] to a geom Point3.
#[inline]
fn to_geom_point(p: [f64; 3]) -> GeomPoint3 {
    GeomPoint3::new(p[0], p[1], p[2])
}

/// Converts a geom Point3 to an array [f64; 3].
#[inline]
fn from_geom_point(p: GeomPoint3) -> [f64; 3] {
    [p.x, p.y, p.z]
}

/// Converts an array [f64; 3] to a geom Vec3.
#[inline]
fn to_geom_vec(v: [f64; 3]) -> GeomVec3 {
    GeomVec3::new(v[0], v[1], v[2])
}

/// Tessellates a curve using the geom adaptive tessellator and returns points as arrays.
fn tessellate_curve_to_points<C: crate::geom::Curve3>(
    curve: &C,
    max_deviation: f64,
    max_segments: usize,
) -> Vec<[f64; 3]> {
    let options = CurveTessellationOptions::new(max_deviation, max_segments);
    let geom_points = tessellate_curve_adaptive_points(curve, options);
    geom_points.into_iter().map(from_geom_point).collect()
}

/// Creates default tessellation options for curve primitives.
fn default_curve_tessellation_options() -> (f64, usize) {
    (DEFAULT_MAX_DEVIATION, DEFAULT_MAX_SEGMENTS)
}

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
            Self::Rectangle3Pt => evaluate_rectangle_3pt(inputs),
            Self::Ellipse => evaluate_ellipse(inputs),
            Self::Circle3Pt => evaluate_circle_3pt(inputs),
            Self::Line => evaluate_line(inputs),
            Self::LineSDL => evaluate_line_sdl(inputs),
            Self::CircleTanTan => not_implemented(self.name()),
            Self::Line2Plane => not_implemented(self.name()),
            Self::Rectangle2Pt => evaluate_rectangle_2pt(inputs),
            Self::InEllipse => not_implemented(self.name()),
            Self::BiArc => not_implemented(self.name()),
            Self::Polygon => evaluate_polygon(inputs),
            Self::ArcSED => evaluate_arc_sed(inputs),
            Self::ModifiedArc => not_implemented(self.name()),
            Self::Line4Pt => not_implemented(self.name()),
            Self::Arc => evaluate_arc(inputs),
            Self::CircleFit => evaluate_circle_fit(inputs),
            Self::TwoByFourJam => not_implemented(self.name()),
            Self::CircleCNR => evaluate_circle_cnr(inputs),
            Self::TangentLinesEx => not_implemented(self.name()),
            Self::CircleTanTanTan => not_implemented(self.name()),
            Self::TangentLinesIn => not_implemented(self.name()),
            Self::TangentLines => not_implemented(self.name()),
            Self::TangentArcs => not_implemented(self.name()),
            Self::PolygonEdge => evaluate_polygon_edge(inputs),
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

    let p1_res = coerce::coerce_point_with_context(inputs.get(0).unwrap(), "Arc 3Pt");
    let p2_res = coerce::coerce_point_with_context(inputs.get(1).unwrap(), "Arc 3Pt");
    let p3_res = coerce::coerce_point_with_context(inputs.get(2).unwrap(), "Arc 3Pt");

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

/// Evaluates the Circle 3Pt component.
///
/// Creates a circle passing through three points. Uses geom::Circle3 for
/// construction and adaptive tessellation.
///
/// # Inputs
/// - `inputs[0]`: First point on the circle
/// - `inputs[1]`: Second point on the circle
/// - `inputs[2]`: Third point on the circle
///
/// # Outputs
/// - `C`: Circle as a list of points (tessellated polyline)
/// - `P`: Plane of the circle (origin, x-axis point, y-axis point)
/// - `R`: Radius of the circle
fn evaluate_circle_3pt(inputs: &[Value]) -> ComponentResult {
    const CONTEXT: &str = "Circle 3Pt";

    if inputs.len() < 3 {
        return Err(ComponentError::new(format!(
            "{} component vereist drie punten",
            CONTEXT
        )));
    }

    let p1_res = coerce::coerce_point_with_context(inputs.get(0).unwrap(), CONTEXT);
    let p2_res = coerce::coerce_point_with_context(inputs.get(1).unwrap(), CONTEXT);
    let p3_res = coerce::coerce_point_with_context(inputs.get(2).unwrap(), CONTEXT);

    // Handle invalid inputs gracefully
    if p1_res.is_err() || p2_res.is_err() || p3_res.is_err() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_CIRCLE.to_owned(), Value::Null);
        outputs.insert("P".to_owned(), Value::Null);
        outputs.insert("R".to_owned(), Value::Null);
        return Ok(outputs);
    }

    let p1 = p1_res.unwrap();
    let p2 = p2_res.unwrap();
    let p3 = p3_res.unwrap();

    // Find circle center, radius, and normal from three points
    let (center, radius, normal) = match circle_from_three_points(p1, p2, p3) {
        Some(result) => result,
        None => {
            // Collinear points - cannot form a circle
            let mut outputs = BTreeMap::new();
            outputs.insert(PIN_OUTPUT_CIRCLE.to_owned(), Value::Null);
            outputs.insert("P".to_owned(), Value::Null);
            outputs.insert("R".to_owned(), Value::Null);
            return Ok(outputs);
        }
    };

    // Compute x-axis from center to first point for stable orientation
    let x_axis = match safe_normalized(subtract(p1, center)) {
        Some((axis, _)) => axis,
        None => {
            // Degenerate case: center coincides with p1
            let mut outputs = BTreeMap::new();
            outputs.insert(PIN_OUTPUT_CIRCLE.to_owned(), Value::Null);
            outputs.insert("P".to_owned(), Value::Null);
            outputs.insert("R".to_owned(), Value::Null);
            return Ok(outputs);
        }
    };

    // Build geom::Circle3 with explicit x-axis for consistent orientation
    let circle = Circle3::from_center_xaxis_normal(
        to_geom_point(center),
        to_geom_vec(x_axis),
        to_geom_vec(normal),
        radius,
    );

    // Tessellate using adaptive algorithm
    let (max_deviation, max_segments) = default_curve_tessellation_options();
    let mut points = tessellate_curve_to_points(&circle, max_deviation, max_segments);

    // Add closing point for backward compatibility
    if let Some(first) = points.first().copied() {
        points.push(first);
    }

    // Build the output plane representation:
    // [origin, point_on_x_axis, point_on_y_axis]
    let y_axis = cross(normal, x_axis);
    let plane_points = vec![
        Value::Point(center),
        Value::Point(add(center, x_axis)),
        Value::Point(add(center, y_axis)),
    ];

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CIRCLE.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );
    outputs.insert("P".to_owned(), Value::List(plane_points));
    outputs.insert("R".to_owned(), Value::Number(radius));
    Ok(outputs)
}

fn evaluate_circle_cnr(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "CircleCNR component vereist een middelpunt, normaalvector en straal",
        ));
    }

    let center = coerce::coerce_point_with_context(inputs.get(0).unwrap(), "CircleCNR")?;
    let normal = coerce::coerce_point_with_context(inputs.get(1).unwrap(), "CircleCNR")?;
    let radius = require_number(inputs.get(2), "CircleCNR")?;

    if radius <= 0.0 {
        return Err(ComponentError::new(
            "CircleCNR component vereist een radius groter dan nul",
        ));
    }

    // Build Circle3 directly from center, normal, and radius using geom types
    let circle = Circle3::new(to_geom_point(center), to_geom_vec(normal), radius);

    // Tessellate using adaptive algorithm
    let (max_deviation, max_segments) = default_curve_tessellation_options();
    let mut points = tessellate_curve_to_points(&circle, max_deviation, max_segments);

    // Add closing point for backward compatibility
    if let Some(first) = points.first().copied() {
        points.push(first);
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CIRCLE.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );
    Ok(outputs)
}

/// Evaluates the Circle Fit component.
///
/// Fits a circle to a set of points using least squares approximation.
/// Uses geom::Circle3 for construction and adaptive tessellation.
///
/// # Algorithm
/// 1. Find the best-fit plane through the points (using PCA)
/// 2. Project points onto the plane
/// 3. Fit a 2D circle in the plane using algebraic least squares
/// 4. Build a 3D circle from the result
///
/// # Inputs
/// - `inputs[0]`: List of points to fit the circle to
///
/// # Outputs
/// - `C`: Circle as a list of points (tessellated polyline)
/// - `P`: Plane of the circle
/// - `R`: Radius of the fitted circle
fn evaluate_circle_fit(inputs: &[Value]) -> ComponentResult {
    const CONTEXT: &str = "Circle Fit";

    if inputs.is_empty() {
        return Err(ComponentError::new(format!(
            "{} component vereist een lijst van punten",
            CONTEXT
        )));
    }

    let points = coerce_points(inputs.get(0), CONTEXT)?;

    if points.len() < 3 {
        return Err(ComponentError::new(format!(
            "{} component vereist ten minste drie punten",
            CONTEXT
        )));
    }

    // Step 1: Compute centroid
    let n = points.len() as f64;
    let mut centroid = [0.0, 0.0, 0.0];
    for p in &points {
        centroid[0] += p[0];
        centroid[1] += p[1];
        centroid[2] += p[2];
    }
    centroid[0] /= n;
    centroid[1] /= n;
    centroid[2] /= n;

    // Step 2: Compute covariance matrix for PCA
    let mut cov = [[0.0; 3]; 3];
    for p in &points {
        let dx = p[0] - centroid[0];
        let dy = p[1] - centroid[1];
        let dz = p[2] - centroid[2];
        cov[0][0] += dx * dx;
        cov[0][1] += dx * dy;
        cov[0][2] += dx * dz;
        cov[1][1] += dy * dy;
        cov[1][2] += dy * dz;
        cov[2][2] += dz * dz;
    }
    cov[1][0] = cov[0][1];
    cov[2][0] = cov[0][2];
    cov[2][1] = cov[1][2];

    // Step 3: Find the normal (smallest eigenvector) using power iteration on (I - vv^T)
    // For a quick approximation, we use the cross product of two principal directions
    // This is a simplified approach - for more accuracy, use proper SVD
    let normal = fit_plane_normal(&cov);
    let normal = match safe_normalized(normal) {
        Some((n, _)) => n,
        None => [0.0, 0.0, 1.0], // Default to XY plane
    };

    // Create local coordinate system in the plane
    let x_axis = orthogonal_vector(normal);
    let y_axis = normalize(cross(normal, x_axis));

    // Step 4: Project points onto the plane and compute 2D coordinates
    let mut u_coords = Vec::with_capacity(points.len());
    let mut v_coords = Vec::with_capacity(points.len());
    for p in &points {
        let dp = subtract(*p, centroid);
        u_coords.push(dot(dp, x_axis));
        v_coords.push(dot(dp, y_axis));
    }

    // Step 5: Fit circle using Kåsa's algebraic method
    // Minimize sum of (u^2 + v^2 - 2*a*u - 2*b*v - c)^2
    // Which gives us a linear system in (a, b, c)
    let (center_u, center_v, radius) = fit_circle_2d(&u_coords, &v_coords)?;

    // Step 6: Convert back to 3D
    let center = add(
        centroid,
        add(scale(x_axis, center_u), scale(y_axis, center_v)),
    );

    // Build geom::Circle3
    let circle = Circle3::from_center_xaxis_normal(
        to_geom_point(center),
        to_geom_vec(x_axis),
        to_geom_vec(normal),
        radius,
    );

    // Tessellate using adaptive algorithm
    let (max_deviation, max_segments) = default_curve_tessellation_options();
    let mut circle_points = tessellate_curve_to_points(&circle, max_deviation, max_segments);

    // Add closing point for backward compatibility
    if let Some(first) = circle_points.first().copied() {
        circle_points.push(first);
    }

    // Build the output plane representation
    let plane_points = vec![
        Value::Point(center),
        Value::Point(add(center, x_axis)),
        Value::Point(add(center, y_axis)),
    ];

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_CIRCLE.to_owned(),
        Value::List(circle_points.into_iter().map(Value::Point).collect()),
    );
    outputs.insert("P".to_owned(), Value::List(plane_points));
    outputs.insert("R".to_owned(), Value::Number(radius));
    Ok(outputs)
}

/// Fits the normal vector of a plane from a 3x3 covariance matrix.
/// Returns the direction of smallest variance (the plane normal).
fn fit_plane_normal(cov: &[[f64; 3]; 3]) -> [f64; 3] {
    // Use power iteration to find the largest eigenvector, then compute smallest
    // For a 3x3 symmetric matrix, we can use a simpler approach:
    // The normal is perpendicular to the two directions of maximum variance

    // Start with a guess for the largest eigenvector
    let mut v = [1.0, 1.0, 1.0];
    let v_len = vector_length(v);
    if v_len > EPSILON {
        v = scale(v, 1.0 / v_len);
    }

    // Power iteration for largest eigenvector (10 iterations is usually enough)
    for _ in 0..10 {
        let new_v = [
            cov[0][0] * v[0] + cov[0][1] * v[1] + cov[0][2] * v[2],
            cov[1][0] * v[0] + cov[1][1] * v[1] + cov[1][2] * v[2],
            cov[2][0] * v[0] + cov[2][1] * v[1] + cov[2][2] * v[2],
        ];
        let len = vector_length(new_v);
        if len > EPSILON {
            v = scale(new_v, 1.0 / len);
        }
    }

    // Deflate and find second eigenvector
    let mut cov2 = *cov;
    let lambda1 = dot(
        [
            cov[0][0] * v[0] + cov[0][1] * v[1] + cov[0][2] * v[2],
            cov[1][0] * v[0] + cov[1][1] * v[1] + cov[1][2] * v[2],
            cov[2][0] * v[0] + cov[2][1] * v[1] + cov[2][2] * v[2],
        ],
        v,
    );
    for i in 0..3 {
        for j in 0..3 {
            cov2[i][j] -= lambda1 * v[i] * v[j];
        }
    }

    let mut w = orthogonal_vector(v);
    for _ in 0..10 {
        let new_w = [
            cov2[0][0] * w[0] + cov2[0][1] * w[1] + cov2[0][2] * w[2],
            cov2[1][0] * w[0] + cov2[1][1] * w[1] + cov2[1][2] * w[2],
            cov2[2][0] * w[0] + cov2[2][1] * w[1] + cov2[2][2] * w[2],
        ];
        let len = vector_length(new_w);
        if len > EPSILON {
            w = scale(new_w, 1.0 / len);
        }
    }

    // The normal is perpendicular to both v and w
    cross(v, w)
}

/// Fits a 2D circle to points using Kåsa's algebraic method.
/// Returns (center_u, center_v, radius).
fn fit_circle_2d(u: &[f64], v: &[f64]) -> Result<(f64, f64, f64), ComponentError> {
    let n = u.len();
    if n < 3 {
        return Err(ComponentError::new(
            "Circle fit vereist ten minste drie punten",
        ));
    }

    // Build the normal equations for the algebraic circle fit
    // We want to minimize sum((u^2 + v^2) - 2*a*u - 2*b*v - c)^2
    // This gives us the linear system:
    // [sum(u^2)    sum(u*v)    sum(u)]   [a]   [sum(u * (u^2+v^2))]
    // [sum(u*v)    sum(v^2)    sum(v)] * [b] = [sum(v * (u^2+v^2))]
    // [sum(u)      sum(v)      n    ]   [c]   [sum(u^2+v^2)]

    let mut su = 0.0;
    let mut sv = 0.0;
    let mut su2 = 0.0;
    let mut sv2 = 0.0;
    let mut suv = 0.0;
    let mut su3 = 0.0;
    let mut sv3 = 0.0;
    let mut su2v = 0.0;
    let mut suv2 = 0.0;

    for i in 0..n {
        let ui = u[i];
        let vi = v[i];
        let ui2 = ui * ui;
        let vi2 = vi * vi;
        su += ui;
        sv += vi;
        su2 += ui2;
        sv2 += vi2;
        suv += ui * vi;
        su3 += ui2 * ui;
        sv3 += vi2 * vi;
        su2v += ui2 * vi;
        suv2 += ui * vi2;
    }

    let nf = n as f64;

    // Build the matrix A and vector b
    let a = [[su2, suv, su], [suv, sv2, sv], [su, sv, nf]];
    let b = [
        su3 + suv2,
        su2v + sv3,
        su2 + sv2,
    ];

    // Solve the system
    let solution = solve_linear_3x3(a, b).ok_or_else(|| {
        ComponentError::new("Circle fit: singular matrix (punten zijn mogelijk collineair)")
    })?;

    let a_coef = solution[0] / 2.0;
    let b_coef = solution[1] / 2.0;
    let c_coef = solution[2];

    let center_u = a_coef;
    let center_v = b_coef;
    let radius_sq = c_coef + a_coef * a_coef + b_coef * b_coef;

    if radius_sq <= 0.0 {
        return Err(ComponentError::new(
            "Circle fit: geen geldige cirkel gevonden (negatieve radius)",
        ));
    }

    let radius = radius_sq.sqrt();
    Ok((center_u, center_v, radius))
}

fn evaluate_line_sdl(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "LineSDL component vereist een startpunt, richting en lengte",
        ));
    }

    let start = coerce::coerce_point_with_context(inputs.get(0).unwrap(), "LineSDL")?;
    let direction =
        coerce::coerce_point_with_context(inputs.get(1).unwrap(), "LineSDL")?;
    let length = require_number(inputs.get(2), "LineSDL")?;

    // Use geom::Line3 for construction - compute end point using normalized direction
    let dir_vec = to_geom_vec(direction);
    let end = match dir_vec.normalized() {
        Some(unit_dir) => {
            let end_point = to_geom_point(start).add_vec(unit_dir.mul_scalar(length));
            from_geom_point(end_point)
        }
        None => {
            // Zero-length direction vector - return degenerate line
            start
        }
    };

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

        // Use geom::Line3 for validation - check for degenerate lines
        let geom_line = Line3::new(to_geom_point(start), to_geom_point(end));
        let value = if geom_line.direction().length_squared() > 0.0 {
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

    let planes = collect_planes(inputs.get(0), "Arc")?;
    let radius = require_number(inputs.get(1), "Arc")?;
    let angle = require_number(inputs.get(2), "Arc")?;

    if radius <= 0.0 {
        return Err(ComponentError::new(
            "Arc component vereist een radius groter dan nul",
        ));
    }

    let (max_deviation, max_segments) = default_curve_tessellation_options();

    let mut arcs = Vec::new();
    let mut lengths = Vec::new();
    for plane in planes {
        let (points, length) = create_arc_points_geom(&plane, radius, 0.0, angle, max_deviation, max_segments);
        arcs.push(Value::List(points.into_iter().map(Value::Point).collect()));
        lengths.push(Value::Number(length));
    }

    let arc_output = if arcs.len() == 1 {
        arcs.into_iter().next().unwrap()
    } else {
        Value::List(arcs)
    };

    let length_output = if lengths.len() == 1 {
        lengths.into_iter().next().unwrap()
    } else {
        Value::List(lengths)
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_ARC.to_owned(), arc_output);
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), length_output);

    Ok(outputs)
}

/// Evaluates the Arc SED (Start/End/Direction) component.
///
/// Creates an arc from a start point, end point, and tangent direction at the start.
/// Uses geom::Arc3 for construction and adaptive tessellation.
///
/// # Algorithm
/// Given start point S, end point E, and tangent direction D at S:
/// 1. Compute the chord vector C = E - S
/// 2. The center lies on a line perpendicular to D through S
/// 3. The center also lies on the perpendicular bisector of the chord
/// 4. Solve for the intersection to find the center
///
/// # Inputs
/// - `inputs[0]`: Start point
/// - `inputs[1]`: End point
/// - `inputs[2]`: Tangent direction at start point
///
/// # Outputs
/// - `A`: Arc as a list of points (tessellated polyline)
/// - `L`: Length of the arc
/// - `P`: Plane of the arc
fn evaluate_arc_sed(inputs: &[Value]) -> ComponentResult {
    const CONTEXT: &str = "Arc SED";

    if inputs.len() < 3 {
        return Err(ComponentError::new(format!(
            "{} component vereist een startpunt, eindpunt en richting",
            CONTEXT
        )));
    }

    let start = coerce::coerce_point_with_context(inputs.get(0).unwrap(), CONTEXT)?;
    let end = coerce::coerce_point_with_context(inputs.get(1).unwrap(), CONTEXT)?;
    let direction = coerce::coerce_point_with_context(inputs.get(2).unwrap(), CONTEXT)?;

    // Convert to geom types for computation
    let start_pt = to_geom_point(start);
    let end_pt = to_geom_point(end);
    let dir_vec = to_geom_vec(direction);

    // Normalize the tangent direction
    let tangent = match dir_vec.normalized() {
        Some(t) => t,
        None => {
            return Err(ComponentError::new(format!(
                "{} vereist een niet-nul richtingvector",
                CONTEXT
            )));
        }
    };

    // Chord vector from start to end
    let chord = end_pt.sub_point(start_pt);
    let chord_length = chord.length();

    if chord_length < EPSILON {
        // Degenerate case: start and end are the same point
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_ARC.to_owned(), Value::List(vec![Value::Point(start)]));
        outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(0.0));
        outputs.insert("P".to_owned(), Value::Null);
        return Ok(outputs);
    }

    // Compute the normal to the arc plane: cross(tangent, chord)
    let normal = tangent.cross(chord);
    let normal_len = normal.length();

    if normal_len < EPSILON {
        // Tangent is parallel to chord - this is a straight line, not an arc
        // Return a line segment instead
        let mut outputs = BTreeMap::new();
        outputs.insert(
            PIN_OUTPUT_ARC.to_owned(),
            Value::List(vec![Value::Point(start), Value::Point(end)]),
        );
        outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(chord_length));
        outputs.insert("P".to_owned(), Value::Null);
        return Ok(outputs);
    }

    let normal = normal.mul_scalar(1.0 / normal_len);

    // The perpendicular to the tangent in the arc plane points toward the center
    let perp_to_tangent = normal.cross(tangent);

    // Midpoint of the chord
    let midpoint = GeomPoint3::new(
        (start_pt.x + end_pt.x) / 2.0,
        (start_pt.y + end_pt.y) / 2.0,
        (start_pt.z + end_pt.z) / 2.0,
    );

    // Direction perpendicular to the chord in the arc plane
    let chord_dir = chord.normalized().unwrap_or(GeomVec3::new(1.0, 0.0, 0.0));
    let perp_to_chord = normal.cross(chord_dir);

    // Find center: intersection of two lines:
    // Line 1: from start point in direction perp_to_tangent
    // Line 2: from midpoint in direction perp_to_chord
    //
    // Parametric form:
    // Line 1: S + t1 * perp_to_tangent
    // Line 2: M + t2 * perp_to_chord
    //
    // Solve: S + t1 * perp_to_tangent = M + t2 * perp_to_chord
    //
    // This gives us: t1 * perp_to_tangent - t2 * perp_to_chord = M - S
    let sm = midpoint.sub_point(start_pt);

    // Use the component perpendicular to the chord direction to find t1
    // Project onto perp_to_chord direction: dot(SM, perp_to_chord) = t1 * dot(perp_to_tangent, perp_to_chord)
    let denom = perp_to_tangent.dot(perp_to_chord);

    let t1 = if denom.abs() > EPSILON {
        sm.dot(perp_to_chord) / denom
    } else {
        // Lines are parallel - should not happen for a valid arc
        chord_length / 2.0
    };

    // Compute center
    let center = start_pt.add_vec(perp_to_tangent.mul_scalar(t1));
    let radius = center.sub_point(start_pt).length();

    if radius < EPSILON {
        return Err(ComponentError::new(format!(
            "{} kon geen geldige boog berekenen (radius te klein)",
            CONTEXT
        )));
    }

    // Compute the x-axis from center to start point
    let x_axis = start_pt.sub_point(center).mul_scalar(1.0 / radius);

    // Compute angle to end point from center
    let to_end = end_pt.sub_point(center);

    // Y-axis in the plane
    let y_axis = normal.cross(x_axis);

    // Start angle is 0 (we use start point as reference)
    let start_angle = 0.0;

    // End angle: use atan2 in the local coordinate system
    let end_x = to_end.dot(x_axis);
    let end_y = to_end.dot(y_axis);
    let mut end_angle = end_y.atan2(end_x);

    // Determine sweep direction based on the tangent
    // The tangent at start should point in the direction of positive sweep
    let expected_tangent = y_axis; // For counter-clockwise sweep starting from x_axis
    let tangent_dot = tangent.dot(expected_tangent);

    if tangent_dot < 0.0 {
        // Need to go the other way around
        if end_angle > 0.0 {
            end_angle -= std::f64::consts::TAU;
        }
    } else if end_angle < 0.0 {
        end_angle += std::f64::consts::TAU;
    }

    let sweep_angle = end_angle - start_angle;

    // Build the Arc3
    let arc = Arc3::from_center_xaxis_normal(
        center,
        x_axis,
        normal,
        radius,
        start_angle,
        sweep_angle,
    );

    // Tessellate using adaptive algorithm
    let (max_deviation, max_segments) = default_curve_tessellation_options();
    let points = tessellate_curve_to_points(&arc, max_deviation, max_segments);
    let length = radius * sweep_angle.abs();

    // Build the output plane representation
    let center_arr = from_geom_point(center);
    let plane_points = vec![
        Value::Point(center_arr),
        Value::Point(from_geom_point(center.add_vec(x_axis))),
        Value::Point(from_geom_point(center.add_vec(y_axis))),
    ];

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_ARC.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(length));
    outputs.insert("P".to_owned(), Value::List(plane_points));
    Ok(outputs)
}

/// Creates arc points using geom::Arc3 and adaptive tessellation.
///
/// Builds an arc from start_angle to end_angle (sweep = end_angle - start_angle)
/// in the given plane with the specified radius. The arc is tessellated adaptively
/// based on curvature/deviation thresholds.
fn create_arc_points_geom(
    plane: &Plane,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    max_deviation: f64,
    max_segments: usize,
) -> (Vec<[f64; 3]>, f64) {
    let sweep_angle = end_angle - start_angle;

    // Handle zero or near-zero sweep (degenerate arc)
    if sweep_angle.abs() < 1e-12 {
        let point = plane.apply(radius * start_angle.cos(), radius * start_angle.sin());
        return (vec![point], 0.0);
    }

    let center = to_geom_point(plane.origin);
    let normal = to_geom_vec(plane._z_axis);

    // Build Arc3 using the plane's x-axis for consistent orientation
    let arc = Arc3::from_center_xaxis_normal(
        center,
        to_geom_vec(plane.x_axis),
        normal,
        radius,
        start_angle,
        sweep_angle,
    );

    // Tessellate using adaptive algorithm
    let points = tessellate_curve_to_points(&arc, max_deviation, max_segments);
    let length = radius * sweep_angle.abs();

    (points, length)
}

/// Legacy create_arc_points wrapper for backward compatibility.
fn create_arc_points(plane: &Plane, radius: f64, angle: f64) -> (Vec<[f64; 3]>, f64) {
    let (max_deviation, max_segments) = default_curve_tessellation_options();
    create_arc_points_geom(plane, radius, 0.0, angle, max_deviation, max_segments)
}

/// Legacy segments_for_angle - kept for backward compatibility with rectangle fillet.
fn segments_for_angle(_angle: f64) -> usize {
    CURVE_SEGMENTS
}

/// Legacy create_arc_points_from_angles using fixed segment count.
#[allow(dead_code)]
fn create_arc_points_from_angles_legacy(
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

/// Create arc points using geom::Arc3, using default tessellation options.
/// This replaces the legacy create_arc_points_from_angles.
fn create_arc_points_from_angles(
    plane: &Plane,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
) -> (Vec<[f64; 3]>, f64) {
    let (max_deviation, max_segments) = default_curve_tessellation_options();
    create_arc_points_geom(plane, radius, start_angle, end_angle, max_deviation, max_segments)
}

fn evaluate_polygon(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Polygon component vereist een vlak, radius en segmenten",
        ));
    }

    let planes = collect_planes(inputs.get(0), "Polygon")?;
    let radius = require_number(inputs.get(1), "Polygon")?;
    let segments = require_number(inputs.get(2), "Polygon")? as usize;
    let fillet_radius = coerce::coerce_optional_number(inputs.get(3), "Polygon")?.unwrap_or(0.0);

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

    let mut polygons = Vec::new();
    let mut lengths = Vec::new();
    for plane in planes {
        let (points, length) = create_polygon_points(&plane, radius, segments, fillet_radius);
        polygons.push(Value::List(points.into_iter().map(Value::Point).collect()));
        lengths.push(Value::Number(length));
    }

    let polygon_output = if polygons.len() == 1 {
        polygons.into_iter().next().unwrap()
    } else {
        Value::List(polygons)
    };

    let length_output = if lengths.len() == 1 {
        lengths.into_iter().next().unwrap()
    } else {
        Value::List(lengths)
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POLYGON.to_owned(), polygon_output);
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), length_output);

    Ok(outputs)
}

/// Creates polygon points using geom types for point construction.
///
/// A polygon is a regular N-gon inscribed in a circle of the given radius.
/// When `fillet_radius > 0`, rounded corners are created using `geom::Arc3`
/// with adaptive tessellation for smooth arcs at each corner.
///
/// # Geometry
/// For a regular n-gon inscribed in a circle of radius `R`:
/// - Vertices are at angles `i * 2π/n` around the center
/// - The interior angle at each corner is `(n-2)π/n`
/// - The exterior (turning) angle is `2π/n`
///
/// For filleted corners with radius `r`:
/// - Each corner is replaced by a circular arc
/// - The arc subtends the exterior angle
/// - The arc center lies along the inward bisector at distance `r / sin(π/n)`
/// - The side length is reduced by `2 * r / tan(π/n)`
fn create_polygon_points(
    plane: &Plane,
    radius: f64,
    segments: usize,
    fillet_radius: f64,
) -> (Vec<[f64; 3]>, f64) {
    let center = to_geom_point(plane.origin);
    let x_axis = to_geom_vec(plane.x_axis);
    let y_axis = to_geom_vec(plane.y_axis);
    let normal = to_geom_vec(plane._z_axis);

    let angle_step = std::f64::consts::TAU / segments as f64;
    let half_corner = std::f64::consts::PI / segments as f64;
    
    // Original side length for a regular polygon inscribed in circle of given radius
    let side_length = 2.0 * radius * half_corner.sin();
    
    // Maximum fillet radius: occurs when the arc would extend to the midpoint of each side.
    // The trim distance from corner = fillet_radius / tan(half_corner)
    // This must be at most half the side length:
    // fillet_radius / tan(half_corner) <= side_length / 2
    // fillet_radius <= (side_length / 2) * tan(half_corner)
    // fillet_radius <= radius * sin(half_corner) * tan(half_corner)
    let max_fillet = 0.5 * side_length * half_corner.tan();
    let fillet_radius = fillet_radius.clamp(0.0, max_fillet.max(0.0));

    if fillet_radius < EPSILON {
        // Simple polygon without fillet
        let mut points = Vec::with_capacity(segments + 1);
        for i in 0..segments {
            let angle = i as f64 * angle_step;
            let point = center
                .add_vec(x_axis.mul_scalar(radius * angle.cos()))
                .add_vec(y_axis.mul_scalar(radius * angle.sin()));
            points.push(from_geom_point(point));
        }
        
        // Close the polygon
        if let Some(first) = points.first().copied() {
            points.push(first);
        }

        // Perimeter of a regular polygon: n * side_length
        let length = segments as f64 * side_length;
        (points, length)
    } else {
        // Polygon with rounded corners using geom::Arc3
        let (max_deviation, max_segments) = default_curve_tessellation_options();
        
        // The exterior angle (turning angle) at each vertex
        let exterior_angle = angle_step;
        
        // Distance from original vertex to fillet arc start/end along each edge
        let trim_distance = fillet_radius / half_corner.tan();
        
        // The arc center is offset from the original vertex toward the polygon center
        // by distance: d = fillet_radius / sin(half_corner)
        let arc_center_inward_dist = fillet_radius / half_corner.sin();
        
        // Reduced circumradius for arc centers
        let arc_center_radius = radius - arc_center_inward_dist;
        
        // Remaining straight segment length (per side)
        let straight_segment_length = side_length - 2.0 * trim_distance;
        
        // Build the polygon by iterating through corners and adding:
        // 1. Arc at corner i
        // 2. Straight segment from end of arc i to start of arc i+1
        
        let mut points = Vec::new();
        
        for i in 0..segments {
            let vertex_angle = i as f64 * angle_step;
            
            // Arc center position (on a smaller circle at the same angle as the vertex)
            let arc_center = center
                .add_vec(x_axis.mul_scalar(arc_center_radius * vertex_angle.cos()))
                .add_vec(y_axis.mul_scalar(arc_center_radius * vertex_angle.sin()));
            
            // Arc start angle (in the plane's coordinate system):
            // The direction from arc_center to arc start point is at angle:
            // vertex_angle - π/2 + half_corner
            let arc_start_angle = vertex_angle - std::f64::consts::FRAC_PI_2 + half_corner;
            
            // Arc sweeps through the exterior angle (counterclockwise for CCW polygon)
            let arc = Arc3::from_center_xaxis_normal(
                arc_center,
                x_axis,
                normal,
                fillet_radius,
                arc_start_angle,
                exterior_angle,
            );
            
            let arc_points = tessellate_curve_to_points(&arc, max_deviation, max_segments);
            
            // Add all arc points (first arc starts the polygon, subsequent ones may share endpoint)
            if i == 0 {
                points.extend(arc_points);
            } else {
                // The first point of this arc should match the last point we added
                // (connected by the previous straight segment), so skip it
                points.extend(arc_points.into_iter().skip(1));
            }
            
            // After this arc, add the straight segment to the next corner's arc start
            // (unless this is the last corner, in which case we'll close the loop)
            if straight_segment_length > EPSILON && i < segments - 1 {
                // Calculate the start of the next arc (end of this straight segment)
                let next_vertex_angle = (i + 1) as f64 * angle_step;
                let next_arc_center = center
                    .add_vec(x_axis.mul_scalar(arc_center_radius * next_vertex_angle.cos()))
                    .add_vec(y_axis.mul_scalar(arc_center_radius * next_vertex_angle.sin()));
                let next_arc_start_angle = next_vertex_angle - std::f64::consts::FRAC_PI_2 + half_corner;
                let next_arc_start = next_arc_center
                    .add_vec(x_axis.mul_scalar(fillet_radius * next_arc_start_angle.cos()))
                    .add_vec(y_axis.mul_scalar(fillet_radius * next_arc_start_angle.sin()));
                
                // Add the next arc start point (end of straight segment)
                points.push(from_geom_point(next_arc_start));
            }
        }
        
        // Handle the straight segment from the last arc to the first arc's start
        if straight_segment_length > EPSILON {
            // The last arc's end connects to the first arc's start
            let last_vertex_angle = (segments - 1) as f64 * angle_step;
            let last_arc_center = center
                .add_vec(x_axis.mul_scalar(arc_center_radius * last_vertex_angle.cos()))
                .add_vec(y_axis.mul_scalar(arc_center_radius * last_vertex_angle.sin()));
            let last_arc_start_angle = last_vertex_angle - std::f64::consts::FRAC_PI_2 + half_corner;
            let last_arc_end_angle = last_arc_start_angle + exterior_angle;
            let _last_arc_end = last_arc_center
                .add_vec(x_axis.mul_scalar(fillet_radius * last_arc_end_angle.cos()))
                .add_vec(y_axis.mul_scalar(fillet_radius * last_arc_end_angle.sin()));
            
            // First arc start point
            let first_vertex_angle: f64 = 0.0;
            let first_arc_center = center
                .add_vec(x_axis.mul_scalar(arc_center_radius * first_vertex_angle.cos()))
                .add_vec(y_axis.mul_scalar(arc_center_radius * first_vertex_angle.sin()));
            let first_arc_start_angle = first_vertex_angle - std::f64::consts::FRAC_PI_2 + half_corner;
            let first_arc_start = first_arc_center
                .add_vec(x_axis.mul_scalar(fillet_radius * first_arc_start_angle.cos()))
                .add_vec(y_axis.mul_scalar(fillet_radius * first_arc_start_angle.sin()));
            
            // Add the first arc start (connects from last arc end)
            points.push(from_geom_point(first_arc_start));
        }
        
        // Close the polygon by adding the first point again
        if !points.is_empty() {
            points.push(points[0]);
        }

        // Perimeter: n * (straight_segment_length + arc_length)
        // Arc length = fillet_radius * exterior_angle
        let arc_length = fillet_radius * exterior_angle;
        let total_length = segments as f64 * (straight_segment_length + arc_length);
        
        (points, total_length)
    }
}

/// Evaluates the Polygon Edge component.
///
/// Creates a regular polygon given the edge length instead of the circumradius.
/// Uses geom types for point construction.
///
/// # Inputs
/// - `inputs[0]`: Plane (optional, defaults to XY plane)
/// - `inputs[1]`: Edge length (length of each side)
/// - `inputs[2]`: Number of segments (sides)
///
/// # Outputs
/// - `P`: Polygon as a list of points
/// - `L`: Perimeter (total length)
fn evaluate_polygon_edge(inputs: &[Value]) -> ComponentResult {
    const CONTEXT: &str = "Polygon Edge";

    if inputs.len() < 3 {
        return Err(ComponentError::new(format!(
            "{} component vereist een vlak, randlengte en segmenten",
            CONTEXT
        )));
    }

    let planes = collect_planes(inputs.get(0), CONTEXT)?;
    let edge_length = require_number(inputs.get(1), CONTEXT)?;
    let segments = require_number(inputs.get(2), CONTEXT)? as usize;

    if edge_length <= 0.0 {
        return Err(ComponentError::new(format!(
            "{} component vereist een randlengte groter dan nul",
            CONTEXT
        )));
    }
    if segments < 3 {
        return Err(ComponentError::new(format!(
            "{} component vereist ten minste 3 segmenten",
            CONTEXT
        )));
    }

    // For a regular polygon with n sides:
    // edge_length = 2 * radius * sin(π/n)
    // Therefore: radius = edge_length / (2 * sin(π/n))
    let interior_angle = std::f64::consts::PI / segments as f64;
    let radius = edge_length / (2.0 * interior_angle.sin());

    let mut polygons = Vec::new();
    let mut lengths = Vec::new();
    for plane in planes {
        let (points, length) = create_polygon_points(&plane, radius, segments, 0.0);
        polygons.push(Value::List(points.into_iter().map(Value::Point).collect()));
        lengths.push(Value::Number(length));
    }

    let polygon_output = if polygons.len() == 1 {
        polygons.into_iter().next().unwrap()
    } else {
        Value::List(polygons)
    };

    let length_output = if lengths.len() == 1 {
        lengths.into_iter().next().unwrap()
    } else {
        Value::List(lengths)
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POLYGON.to_owned(), polygon_output);
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), length_output);

    Ok(outputs)
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
        Value::List(values) => values
            .iter()
            .map(|v| coerce::coerce_point_with_context(v, context))
            .collect(),
        Value::Point(p) => Ok(vec![*p]),
        other => Err(ComponentError::new(format!(
            "{} verwacht een lijst van punten, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn require_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    let value = value.ok_or_else(|| ComponentError::new(format!(
        "{} vereist een numerieke waarde",
        context
    )))?;
    coerce::coerce_number(value, Some(context))
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

fn coerce_size_from_domain_or_number(
    value: Option<&Value>,
    context: &str,
) -> Result<f64, ComponentError> {
    match value {
        None => Err(ComponentError::new(format!(
            "{} vereist een numerieke waarde of een domein",
            context
        ))),
        Some(value) => match value {
            Value::Domain(Domain::One(d)) => Ok(d.length),
            Value::List(values) if values.len() == 1 => {
                coerce_size_from_domain_or_number(values.get(0), context)
            }
            other => coerce::coerce_number(other, Some(context)),
        },
    }
}

fn evaluate_rectangle_3pt(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Rectangle 3Pt component vereist drie punten",
        ));
    }

    const CONTEXT: &str = "Rectangle 3Pt";
    let p1 = coerce::coerce_point_with_context(inputs.get(0).unwrap(), CONTEXT)?;
    let p2 = coerce::coerce_point_with_context(inputs.get(1).unwrap(), CONTEXT)?;
    let p3 = coerce::coerce_point_with_context(inputs.get(2).unwrap(), CONTEXT)?;

    let ab = subtract(p2, p1);
    let (ab_dir, ab_length) = match safe_normalized(ab) {
        Some((dir, len)) => (dir, len),
        None => {
            return Err(ComponentError::new(
                "Rectangle 3Pt component vereist twee verschillende punten voor AB",
            ));
        }
    };

    let ac = subtract(p3, p1);
    let projection = dot(ac, ab_dir);
    let perp = subtract(ac, scale(ab_dir, projection));
    let perp_length = vector_length(perp);

    if perp_length < EPSILON {
        return Err(ComponentError::new(
            "Rectangle 3Pt component vereist dat punt C niet op lijn AB ligt",
        ));
    }

    let corner_c = add(p2, perp);
    let corner_d = add(p1, perp);

    let perimeter = 2.0 * (ab_length + perp_length);

    if perp_length.is_nan() || ab_length.is_nan() || perimeter.is_nan() {
        return Err(ComponentError::new(
            "Rectangle 3Pt component kon geen geldige rechthoek berekenen",
        ));
    }

    let mut points = vec![p1, p2, corner_c, corner_d];
    if let Some(first) = points.first().copied() {
        points.push(first);
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_RECTANGLE.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(perimeter));
    Ok(outputs)
}

fn evaluate_rectangle_2pt(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Rectangle 2Pt component vereist twee punten",
        ));
    }

    const CONTEXT: &str = "Rectangle 2Pt";
    let plane_input = inputs.get(0).unwrap_or(&Value::Null);
    let base_plane = parse_plane(Some(plane_input), CONTEXT)?;
    let point_a = coerce::coerce_point_with_context(inputs.get(1).unwrap(), CONTEXT)?;
    let point_b = coerce::coerce_point_with_context(inputs.get(2).unwrap(), CONTEXT)?;
    let radius = coerce::coerce_optional_number(inputs.get(3), CONTEXT)?
        .unwrap_or(0.0);

    let (u_a, v_a) = base_plane.project(point_a);
    let (u_b, v_b) = base_plane.project(point_b);

    let u_min = u_a.min(u_b);
    let u_max = u_a.max(u_b);
    let v_min = v_a.min(v_b);
    let v_max = v_a.max(v_b);

    let x_size = u_max - u_min;
    let y_size = v_max - v_min;

    if x_size <= EPSILON || y_size <= EPSILON {
        return Err(ComponentError::new(format!(
            "{} vereist twee verschillende punten om een rechthoek te definiëren",
            CONTEXT
        )));
    }

    let center_u = (u_min + u_max) / 2.0;
    let center_v = (v_min + v_max) / 2.0;
    let center = base_plane.apply(center_u, center_v);
    let rectangle_plane =
        Plane::from_axes(center, base_plane.x_axis, base_plane.y_axis, base_plane._z_axis);

    let (points, length) =
        create_rectangle_points(&rectangle_plane, x_size, y_size, radius);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_RECTANGLE.to_owned(),
        Value::List(points.into_iter().map(Value::Point).collect()),
    );
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), Value::Number(length));
    Ok(outputs)
}

fn evaluate_rectangle(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Rectangle component vereist een x-grootte en y-grootte",
        ));
    }

    let planes = collect_planes(inputs.get(0), "Rectangle")?;
    let x_size = coerce_size_from_domain_or_number(inputs.get(1), "Rectangle X")?;
    let y_size = coerce_size_from_domain_or_number(inputs.get(2), "Rectangle Y")?;
    let radius = coerce::coerce_optional_number(inputs.get(3), "Rectangle")?
        .unwrap_or(0.0);

    if x_size <= 0.0 || y_size <= 0.0 {
        return Err(ComponentError::new(
            "Rectangle component vereist groottes groter dan nul",
        ));
    }

    let mut rectangle_values = Vec::new();
    let mut lengths = Vec::new();

    for plane in planes {
        let (points, length) = create_rectangle_points(&plane, x_size, y_size, radius);
        rectangle_values.push(Value::List(points.into_iter().map(Value::Point).collect()));
        lengths.push(Value::Number(length));
    }

    let rectangle_output = if rectangle_values.len() == 1 {
        rectangle_values.into_iter().next().unwrap()
    } else {
        Value::List(rectangle_values)
    };

    let length_output = if lengths.len() == 1 {
        lengths.into_iter().next().unwrap()
    } else {
        Value::List(lengths)
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_RECTANGLE.to_owned(), rectangle_output);
    outputs.insert(PIN_OUTPUT_LENGTH.to_owned(), length_output);

    Ok(outputs)
}

/// Creates rectangle points using geom types.
///
/// Constructs a rectangle centered at the plane origin with optional rounded corners.
/// When fillet_radius > 0, uses geom::Arc3 for adaptive tessellation of corner arcs.
fn create_rectangle_points(
    plane: &Plane,
    x_size: f64,
    y_size: f64,
    radius: f64,
) -> (Vec<[f64; 3]>, f64) {
    let half_x = x_size / 2.0;
    let half_y = y_size / 2.0;

    // Clamp fillet radius to valid range
    let max_radius = half_x.min(half_y);
    let radius = radius.clamp(0.0, max_radius);

    let center = to_geom_point(plane.origin);
    let x_axis = to_geom_vec(plane.x_axis);
    let y_axis = to_geom_vec(plane.y_axis);
    let normal = to_geom_vec(plane._z_axis);

    let length;
    let mut points = Vec::new();

    if radius < EPSILON {
        // Simple rectangle without fillet
        let corners = [
            (half_x, half_y),    // top-right
            (-half_x, half_y),   // top-left
            (-half_x, -half_y),  // bottom-left
            (half_x, -half_y),   // bottom-right
        ];

        for (u, v) in corners {
            let point = center
                .add_vec(x_axis.mul_scalar(u))
                .add_vec(y_axis.mul_scalar(v));
            points.push(from_geom_point(point));
        }

        length = 2.0 * x_size + 2.0 * y_size;
    } else {
        // Rectangle with rounded corners using geom::Arc3
        let (max_deviation, max_segments) = default_curve_tessellation_options();

        // Corner centers in UV coordinates
        let c_tr = (half_x - radius, half_y - radius);
        let c_tl = (-half_x + radius, half_y - radius);
        let c_bl = (-half_x + radius, -half_y + radius);
        let c_br = (half_x - radius, -half_y + radius);

        let quarter = std::f64::consts::FRAC_PI_2;

        // Helper to create arc at a corner and tessellate it
        let tessellate_corner_arc = |corner_uv: (f64, f64), start_angle: f64| -> Vec<[f64; 3]> {
            let corner_center = center
                .add_vec(x_axis.mul_scalar(corner_uv.0))
                .add_vec(y_axis.mul_scalar(corner_uv.1));

            let arc = Arc3::from_center_xaxis_normal(
                corner_center,
                x_axis,
                normal,
                radius,
                start_angle,
                quarter, // 90 degree arc
            );

            tessellate_curve_to_points(&arc, max_deviation, max_segments)
        };

        // Top-right corner (0 to PI/2)
        points.extend(tessellate_corner_arc(c_tr, 0.0));

        // Top-left corner (PI/2 to PI)
        let tl_points = tessellate_corner_arc(c_tl, quarter);
        points.extend(tl_points.into_iter().skip(1)); // Skip first point (already connected)

        // Bottom-left corner (PI to 3*PI/2)
        let bl_points = tessellate_corner_arc(c_bl, 2.0 * quarter);
        points.extend(bl_points.into_iter().skip(1));

        // Bottom-right corner (3*PI/2 to 2*PI)
        let br_points = tessellate_corner_arc(c_br, 3.0 * quarter);
        points.extend(br_points.into_iter().skip(1));

        // Perimeter: 4 straight edges + 4 quarter-circle arcs = 4 arcs + straight parts
        length = 2.0 * (x_size - 2.0 * radius)
            + 2.0 * (y_size - 2.0 * radius)
            + std::f64::consts::TAU * radius;
    }

    // Close the rectangle
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

    let planes = collect_planes(inputs.get(0), "Circle")?;
    let radius = require_number(inputs.get(1), "Circle")?;

    if radius <= 0.0 {
        return Err(ComponentError::new(
            "Circle component vereist een straal groter dan nul",
        ));
    }

    let (max_deviation, max_segments) = default_curve_tessellation_options();

    let mut circles = Vec::new();
    for plane in planes {
        let points = sample_circle_points_geom(&plane, radius, max_deviation, max_segments);
        circles.push(Value::List(points.into_iter().map(Value::Point).collect()));
    }

    let circle_output = if circles.len() == 1 {
        circles.into_iter().next().unwrap()
    } else {
        Value::List(circles)
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CIRCLE.to_owned(), circle_output);
    Ok(outputs)
}

/// Creates a circle using geom::Circle3 and tessellates it adaptively.
///
/// The circle is constructed in the given plane with the specified radius.
/// Points are tessellated adaptively based on curvature and deviation thresholds.
/// The output includes a closing point (first point repeated at the end).
fn sample_circle_points_geom(
    plane: &Plane,
    radius: f64,
    max_deviation: f64,
    max_segments: usize,
) -> Vec<[f64; 3]> {
    let center = to_geom_point(plane.origin);
    let normal = to_geom_vec(plane._z_axis);

    // Build the geom::Circle3 using the plane's x-axis for consistent orientation
    let circle = Circle3::from_center_xaxis_normal(
        center,
        to_geom_vec(plane.x_axis),
        normal,
        radius,
    );

    // Tessellate using adaptive algorithm
    let mut points = tessellate_curve_to_points(&circle, max_deviation, max_segments);

    // For closed curves, tessellate_curve_adaptive_points returns points without
    // duplicating the first point. We need to add the closing point for
    // backward compatibility with existing Grasshopper semantics.
    if let Some(first) = points.first().copied() {
        points.push(first);
    }

    points
}

/// Legacy sample_circle_points using fixed segment count (kept for backward compatibility).
#[allow(dead_code)]
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

/// Evaluates the Ellipse component.
///
/// Creates an ellipse in the given plane with the specified radii.
/// Uses geom::Ellipse3 for construction and adaptive tessellation.
///
/// # Inputs
/// - `inputs[0]`: Plane (optional, defaults to XY plane)
/// - `inputs[1]`: Radius 1 (semi-major axis along plane X)
/// - `inputs[2]`: Radius 2 (semi-minor axis along plane Y)
fn evaluate_ellipse(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Ellipse component vereist een vlak, radius 1 en radius 2",
        ));
    }

    let planes = collect_planes(inputs.get(0), "Ellipse")?;
    let radius_x = require_number(inputs.get(1), "Ellipse")?;
    let radius_y = require_number(inputs.get(2), "Ellipse")?;

    if radius_x <= 0.0 || radius_y <= 0.0 {
        return Err(ComponentError::new(
            "Ellipse component vereist radii groter dan nul",
        ));
    }

    let (max_deviation, max_segments) = default_curve_tessellation_options();

    let mut ellipses = Vec::new();
    for plane in planes {
        let points = sample_ellipse_points_geom(&plane, radius_x, radius_y, max_deviation, max_segments);
        ellipses.push(Value::List(points.into_iter().map(Value::Point).collect()));
    }

    let ellipse_output = if ellipses.len() == 1 {
        ellipses.into_iter().next().unwrap()
    } else {
        Value::List(ellipses)
    };

    let mut outputs = BTreeMap::new();
    // Use "E" as output pin for ellipse (common convention)
    outputs.insert("E".to_owned(), ellipse_output);
    Ok(outputs)
}

/// Creates ellipse points using geom::Ellipse3 and adaptive tessellation.
///
/// The ellipse is constructed in the given plane with radius_x along the X-axis
/// and radius_y along the Y-axis. Points are tessellated adaptively based on
/// curvature and deviation thresholds.
fn sample_ellipse_points_geom(
    plane: &Plane,
    radius_x: f64,
    radius_y: f64,
    max_deviation: f64,
    max_segments: usize,
) -> Vec<[f64; 3]> {
    let center = to_geom_point(plane.origin);
    let x_axis = to_geom_vec(plane.x_axis);
    let y_axis = to_geom_vec(plane.y_axis);

    // Build the geom::Ellipse3
    let ellipse = Ellipse3::new(center, x_axis, y_axis, radius_x, radius_y);

    // Tessellate using adaptive algorithm
    let mut points = tessellate_curve_to_points(&ellipse, max_deviation, max_segments);

    // Add closing point for backward compatibility
    if let Some(first) = points.first().copied() {
        points.push(first);
    }

    points
}

fn parse_plane(value: Option<&Value>, context: &str) -> Result<Plane, ComponentError> {
    match value {
        None => return Ok(Plane::default()),
        Some(Value::Null) => return Ok(Plane::default()),
        Some(Value::List(values)) if values.is_empty() => return Ok(Plane::default()),
        Some(Value::List(values)) => {
            if values.len() >= 3 {
                if let (Ok(origin), Ok(point_x), Ok(point_y)) = (
                    coerce::coerce_point_with_context(&values[0], context),
                    coerce::coerce_point_with_context(&values[1], context),
                    coerce::coerce_point_with_context(&values[2], context),
                ) {
                    return Ok(Plane::from_points(origin, point_x, point_y));
                }
            }

            if values.len() == 1 {
                return parse_plane(values.get(0), context);
            }

            if let Some(first) = values.first() {
                if let Ok(plane) = parse_plane(Some(first), context) {
                    return Ok(plane);
                }
            }

            Err(ComponentError::new(format!(
                "{} verwacht een vlak, kreeg lijst met {} items",
                context,
                values.len()
            )))
        }
        Some(Value::Point(point)) => Ok(Plane::from_origin(*point)),
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een vlak, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn collect_planes(value: Option<&Value>, context: &str) -> Result<Vec<Plane>, ComponentError> {
    let mut planes = Vec::new();
    if let Some(value) = value {
        collect_planes_into(value, context, &mut planes)?;
    }

    if planes.is_empty() {
        planes.push(Plane::default());
    }

    Ok(planes)
}

fn plane_from_point_list(
    values: &[Value],
    context: &str,
) -> Result<Option<Plane>, ComponentError> {
    if values.len() < 3 {
        return Ok(None);
    }

    let origin = coerce::coerce_point_with_context(&values[0], context);
    let point_x = coerce::coerce_point_with_context(&values[1], context);
    let point_y = coerce::coerce_point_with_context(&values[2], context);

    match (origin, point_x, point_y) {
        (Ok(o), Ok(px), Ok(py)) => Ok(Some(Plane::from_points(o, px, py))),
        _ => Ok(None),
    }
}

fn collect_planes_into(
    value: &Value,
    context: &str,
    output: &mut Vec<Plane>,
) -> Result<(), ComponentError> {
    match value {
        Value::Null => Ok(()),
        Value::List(values) => {
            if values.is_empty() {
                return Ok(());
            }

            if values.len() == 1 {
                return collect_planes_into(&values[0], context, output);
            }

            if values.len() == 3 {
                if let Some(plane) = plane_from_point_list(values, context)? {
                    output.push(plane);
                    return Ok(());
                }
            }

            for entry in values {
                collect_planes_into(entry, context, output)?;
            }
            Ok(())
        }
        _ => {
            output.push(parse_plane(Some(value), context)?);
            Ok(())
        }
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
            output.push(coerce::coerce_point_with_context(other, context)?);
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

    let x = match coerce::coerce_number(&values[0], Some(context)) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    let y = match coerce::coerce_number(&values[1], Some(context)) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    let z = match coerce::coerce_number(&values[2], Some(context)) {
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
    use super::*;

    fn default_plane() -> Plane {
        Plane::default()
    }

    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    fn point_distance(a: [f64; 3], b: [f64; 3]) -> f64 {
        let dx = a[0] - b[0];
        let dy = a[1] - b[1];
        let dz = a[2] - b[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    #[test]
    fn polygon_without_fillet_creates_correct_vertex_count() {
        let plane = default_plane();
        let radius = 1.0;
        let segments = 6; // hexagon

        let (points, length) = create_polygon_points(&plane, radius, segments, 0.0);

        // Should have n+1 points (n vertices + closing duplicate)
        assert_eq!(points.len(), segments + 1);

        // First and last point should be the same
        assert!(point_distance(points[0], points[segments]) < 1e-9);

        // Perimeter of a regular hexagon inscribed in circle of radius 1:
        // side_length = 2 * 1 * sin(π/6) = 2 * 0.5 = 1.0
        // perimeter = 6 * 1.0 = 6.0
        let expected_length = 6.0;
        assert!(
            approx_eq(length, expected_length, 1e-9),
            "Expected length {}, got {}",
            expected_length,
            length
        );
    }

    #[test]
    fn polygon_with_fillet_has_more_points() {
        let plane = default_plane();
        let radius = 1.0;
        let segments = 4; // square
        let fillet_radius = 0.1;

        let (no_fillet_points, _) = create_polygon_points(&plane, radius, segments, 0.0);
        let (fillet_points, _) = create_polygon_points(&plane, radius, segments, fillet_radius);

        // Filleted polygon should have more points due to arc tessellation
        assert!(
            fillet_points.len() > no_fillet_points.len(),
            "Filleted polygon should have more points: {} vs {}",
            fillet_points.len(),
            no_fillet_points.len()
        );
    }

    #[test]
    fn polygon_fillet_radius_is_clamped_to_max() {
        let plane = default_plane();
        let radius = 1.0;
        let segments = 4; // square
        
        // Very large fillet radius should be clamped
        let huge_fillet = 10.0;
        let (points1, length1) = create_polygon_points(&plane, radius, segments, huge_fillet);
        
        // Maximum fillet that makes sense
        let half_corner = std::f64::consts::PI / segments as f64;
        let side_length = 2.0 * radius * half_corner.sin();
        let max_fillet = 0.5 * side_length * half_corner.tan();
        let (points2, length2) = create_polygon_points(&plane, radius, segments, max_fillet);
        
        // Both should produce similar results (small tolerance for floating point)
        assert_eq!(points1.len(), points2.len());
        assert!(approx_eq(length1, length2, 1e-9));
    }

    #[test]
    fn polygon_fillet_perimeter_is_correct() {
        let plane = default_plane();
        let radius = 1.0;
        let segments = 6; // hexagon
        let fillet_radius = 0.1;

        let (_, length) = create_polygon_points(&plane, radius, segments, fillet_radius);

        // Calculate expected perimeter
        let half_corner = std::f64::consts::PI / segments as f64;
        let angle_step = std::f64::consts::TAU / segments as f64;
        let side_length = 2.0 * radius * half_corner.sin();
        let trim_distance = fillet_radius / half_corner.tan();
        let straight_segment = side_length - 2.0 * trim_distance;
        let arc_length = fillet_radius * angle_step;
        let expected_length = segments as f64 * (straight_segment + arc_length);

        assert!(
            approx_eq(length, expected_length, 1e-9),
            "Expected length {}, got {}",
            expected_length,
            length
        );
    }

    #[test]
    fn polygon_fillet_points_form_closed_loop() {
        let plane = default_plane();
        let radius = 1.0;
        let segments = 5; // pentagon
        let fillet_radius = 0.1;

        let (points, _) = create_polygon_points(&plane, radius, segments, fillet_radius);

        // First and last point should be the same (closed polygon)
        assert!(
            point_distance(points[0], *points.last().unwrap()) < 1e-9,
            "Polygon should be closed: first {:?}, last {:?}",
            points[0],
            points.last()
        );
    }

    #[test]
    fn polygon_fillet_points_are_continuous() {
        let plane = default_plane();
        let radius = 1.0;
        let segments = 4; // square
        let fillet_radius = 0.1;

        let (points, _) = create_polygon_points(&plane, radius, segments, fillet_radius);

        // Calculate the expected maximum gap (straight segment length)
        let half_corner = std::f64::consts::PI / segments as f64;
        let side_length = 2.0 * radius * half_corner.sin();
        let trim_distance = fillet_radius / half_corner.tan();
        let straight_segment_length = side_length - 2.0 * trim_distance;
        
        // Allow a small tolerance above the straight segment length for numerical errors
        let max_gap = straight_segment_length + 0.01;

        // No consecutive points should be too far apart
        for i in 0..points.len() - 1 {
            let dist = point_distance(points[i], points[i + 1]);
            assert!(
                dist < max_gap,
                "Gap between points {} and {} is too large: {} (max expected: {})",
                i,
                i + 1,
                dist,
                max_gap
            );
        }
    }

    #[test]
    fn polygon_triangle_with_fillet() {
        let plane = default_plane();
        let radius = 1.0;
        let segments = 3; // triangle
        let fillet_radius = 0.1;

        let (points, length) = create_polygon_points(&plane, radius, segments, fillet_radius);

        // Triangle should still work
        assert!(points.len() >= 4); // At least the 3 corners + closing point
        assert!(length > 0.0);

        // Should be closed
        assert!(point_distance(points[0], *points.last().unwrap()) < 1e-9);
    }
}
