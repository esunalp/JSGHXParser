//! Implementaties van Grasshopper "Surface → Freeform" componenten.
//!
//! This module uses `geom::extrusion`, `geom::loft`, `geom::sweep`, etc. for geometry
//! construction when the `mesh_engine_next` feature is enabled. Components remain thin
//! wrappers that coerce inputs, call geom functions, and return outputs.

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

use crate::graph::node::{MetaMap, MetaValue};
use crate::graph::value::{Domain, Value};

use super::{Component, ComponentError, ComponentResult, coerce};

// Import geom types for extrusion (always available, geom module is not feature-gated)
use crate::geom::{
    ExtrusionCaps, ExtrusionError, Point3 as GeomPoint3, Tolerance, Transform, Vec3 as GeomVec3,
    extrude_angled_polyline_with_tolerance, extrude_polyline_with_tolerance,
    extrude_to_point_with_tolerance,
};

// Import geom types for loft
use crate::geom::{
    LoftError, LoftOptions as GeomLoftOptions, LoftType as GeomLoftType,
    MeshQuality as GeomMeshQuality,
    loft_mesh_with_tolerance,
};

// Import geom types for sweep
use crate::geom::{
    FrenetFrame, MiterType, Point3, ProfilePlaneTransform, SweepCaps, SweepError,
    SweepOptions as GeomSweepOptions, Vec3,
    Sweep2MultiSectionOptions, Sweep2Section,
    align_sweep2_rails,
    sweep1_polyline_with_tolerance, sweep2_multi_section, sweep2_polyline_with_tolerance,
};

// Import geom types for pipe
use crate::geom::{
    PipeCaps, PipeError, PipeOptions as GeomPipeOptions,
    pipe_polyline_with_tolerance, pipe_variable_polyline_with_tolerance,
};

// Import geom types for patch
use crate::geom::{
    PatchError, PatchOptions,
    fragment_patch_meshes_with_tolerance,
    patch_mesh_with_options,
};

// Import geom types for revolve
use crate::geom::{
    RailRevolveAxis, RailRevolveOptions, RevolveCaps, RevolveError, RevolveOptions,
    rail_revolve_polyline_with_options, revolve_polyline_with_options,
};

// Import geom types for surface builders (FourPointSurface, RuledSurface, EdgeSurface, SumSurface, NetworkSurface)
use crate::geom::{
    SurfaceBuilderQuality,
    mesh_four_point_surface_from_points,
    mesh_ruled_surface,
    mesh_edge_surface_from_edges,
    mesh_sum_surface,
    mesh_network_surface,
};

// Import geom types for surface fitting (SurfaceFromPoints)
use crate::geom::{
    SurfaceFitError, SurfaceFitOptions,
    mesh_from_grid_with_options, mesh_from_scattered_points,
};

const PIN_OUTPUT_SURFACE: &str = "S";
const PIN_OUTPUT_MESH: &str = "M";
const PIN_OUTPUT_EXTRUSION: &str = "E";
const PIN_OUTPUT_OPTIONS: &str = "O";
const PIN_OUTPUT_PATCH: &str = "P";
const PIN_OUTPUT_PIPE: &str = "P";
const PIN_OUTPUT_LOFT: &str = "L";
const PIN_OUTPUT_SHAPE: &str = "S";

const EPSILON: f64 = 1e-9;

/// Beschikbare componentvarianten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    FitLoft,
    EdgeSurface,
    Extrude,
    ExtrudeAlong,
    LoftOptions,
    SurfaceFromPoints,
    Patch,
    ControlPointLoft,
    SumSurface,
    RuledSurface,
    NetworkSurface,
    Sweep2,
    PipeVariable,
    ExtrudeLinear,
    Loft,
    ExtrudeAngled,
    Sweep1,
    ExtrudePoint,
    Pipe,
    FourPointSurface,
    FragmentPatch,
    Revolution,
    BoundarySurfaces,
    RailRevolution,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de surface-freeform componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{342aa574-1327-4bc2-8daf-203da2a45676}"],
        names: &["Fit Loft", "FitLoft"],
        kind: ComponentKind::FitLoft,
    },
    Registration {
        guids: &["{36132830-e2ef-4476-8ea1-6a43922344f0}"],
        names: &["Edge Surface", "EdgeSrf"],
        kind: ComponentKind::EdgeSurface,
    },
    Registration {
        guids: &["{962034e9-cc27-4394-afc4-5c16e3447cf9}"],
        names: &["Extrude", "Extr"],
        kind: ComponentKind::Extrude,
    },
    Registration {
        guids: &["{38a5638b-6d01-4417-bf11-976d925f8a71}"],
        names: &["Extrude Along", "ExtrCrv"],
        kind: ComponentKind::ExtrudeAlong,
    },
    Registration {
        guids: &["{45f19d16-1c9f-4b0f-a9a6-45a77f3d206c}"],
        names: &["Loft Options", "Loft Opt"],
        kind: ComponentKind::LoftOptions,
    },
    Registration {
        guids: &["{4b04a1e1-cddf-405d-a7db-335aaa940541}"],
        names: &["Surface From Points", "SrfGrid"],
        kind: ComponentKind::SurfaceFromPoints,
    },
    Registration {
        guids: &["{57b2184c-8931-4e70-9220-612ec5b3809a}"],
        names: &["Patch"],
        kind: ComponentKind::Patch,
    },
    Registration {
        guids: &["{5c270622-ee80-45a4-b07a-bd8ffede92a2}"],
        names: &["Control Point Loft", "CPLoft"],
        kind: ComponentKind::ControlPointLoft,
    },
    Registration {
        guids: &["{5e33c760-adcd-4235-b1dd-05cf72eb7a38}"],
        names: &["Sum Surface", "SumSrf"],
        kind: ComponentKind::SumSurface,
    },
    Registration {
        guids: &["{6e5de495-ba76-42d0-9985-a5c265e9aeca}"],
        names: &["Ruled Surface", "RuleSrf"],
        kind: ComponentKind::RuledSurface,
    },
    Registration {
        guids: &["{71506fa8-9bf0-432d-b897-b2e0c5ac316c}"],
        names: &["Network Surface", "NetSurf"],
        kind: ComponentKind::NetworkSurface,
    },
    Registration {
        guids: &["{75164624-395a-4d24-b60b-6bf91cab0194}"],
        names: &["Sweep2", "Swp2"],
        kind: ComponentKind::Sweep2,
    },
    Registration {
        guids: &["{888f9c3c-f1e1-4344-94b0-5ee6a45aee11}"],
        names: &["Pipe Variable", "VPipe"],
        kind: ComponentKind::PipeVariable,
    },
    Registration {
        guids: &["{8efd5eb9-a896-486e-9f98-d8d1a07a49f3}"],
        names: &["Extrude Linear"],
        kind: ComponentKind::ExtrudeLinear,
    },
    Registration {
        guids: &["{a7a41d0a-2188-4f7a-82cc-1a2c4e4ec850}"],
        names: &["Loft"],
        kind: ComponentKind::Loft,
    },
    Registration {
        guids: &["{ae57e09b-a1e4-4d05-8491-abd232213bc9}"],
        names: &["Extrude Angled", "ExtrAng"],
        kind: ComponentKind::ExtrudeAngled,
    },
    Registration {
        guids: &["{bb6666e7-d0f4-41ec-a257-df2371619f13}"],
        names: &["Sweep1", "Swp1"],
        kind: ComponentKind::Sweep1,
    },
    Registration {
        guids: &["{be6636b2-2f1a-4d42-897b-fdef429b6f17}"],
        names: &["Extrude Point"],
        kind: ComponentKind::ExtrudePoint,
    },
    Registration {
        guids: &["{c277f778-6fdf-4890-8f78-347efb23c406}"],
        names: &["Pipe"],
        kind: ComponentKind::Pipe,
    },
    Registration {
        guids: &["{cdee962f-4202-456b-a1b4-f3ed9aa0dc29}"],
        names: &["Revolution", "RevSrf"],
        kind: ComponentKind::Revolution,
    },
    Registration {
        guids: &["{d51e9b65-aa4e-4fd6-976c-cef35d421d05}"],
        names: &["Boundary Surfaces", "Boundary"],
        kind: ComponentKind::BoundarySurfaces,
    },
    Registration {
        guids: &["{d8d68c35-f869-486d-adf3-69ee3cc2d501}"],
        names: &["Rail Revolution", "RailRev"],
        kind: ComponentKind::RailRevolution,
    },
    Registration {
        guids: &["{cb56b26c-2595-4d03-bdb2-eb2e6aeba82d}"],
        names: &["Fragment Patch"],
        kind: ComponentKind::FragmentPatch,
    },
    Registration {
        guids: &["{c77a8b3b-c569-4d81-9b59-1c27299a1c45}"],
        names: &["4Point Surface", "Srf4Pt"],
        kind: ComponentKind::FourPointSurface,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::FitLoft => evaluate_loft(inputs, meta, "Fit Loft", PIN_OUTPUT_SURFACE),
            Self::EdgeSurface => evaluate_edge_surface(inputs, meta),
            Self::Extrude => evaluate_extrude(inputs),
            Self::ExtrudeAlong => evaluate_extrude_along(inputs),
            Self::LoftOptions => evaluate_loft_options(inputs),
            Self::SurfaceFromPoints => evaluate_surface_from_points(inputs, "Surface From Points"),
            Self::Patch => evaluate_patch(inputs),
            Self::ControlPointLoft => {
                evaluate_loft(inputs, meta, "Control Point Loft", PIN_OUTPUT_SURFACE)
            }
            Self::SumSurface => evaluate_sum_surface(inputs, meta),
            Self::RuledSurface => evaluate_ruled_surface(inputs, meta),
            Self::NetworkSurface => evaluate_network_surface(inputs, meta),
            Self::Sweep2 => evaluate_sweep_two(inputs),
            Self::PipeVariable => evaluate_pipe_variable(inputs),
            Self::ExtrudeLinear => evaluate_extrude_linear(inputs),
            Self::Loft => evaluate_loft(inputs, meta, "Loft", PIN_OUTPUT_LOFT),
            Self::ExtrudeAngled => evaluate_extrude_angled(inputs),
            Self::Sweep1 => evaluate_sweep_one(inputs, meta),
            Self::ExtrudePoint => evaluate_extrude_point(inputs),
            Self::Pipe => evaluate_pipe(inputs),
            Self::FourPointSurface => evaluate_four_point_surface(inputs, meta),
            Self::FragmentPatch => evaluate_fragment_patch(inputs),
            Self::Revolution => evaluate_revolution(inputs),
            Self::BoundarySurfaces => evaluate_boundary_surfaces(inputs),
            Self::RailRevolution => evaluate_rail_revolution(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::FitLoft => "Fit Loft",
            Self::EdgeSurface => "Edge Surface",
            Self::Extrude => "Extrude",
            Self::ExtrudeAlong => "Extrude Along",
            Self::LoftOptions => "Loft Options",
            Self::SurfaceFromPoints => "Surface From Points",
            Self::Patch => "Patch",
            Self::ControlPointLoft => "Control Point Loft",
            Self::SumSurface => "Sum Surface",
            Self::RuledSurface => "Ruled Surface",
            Self::NetworkSurface => "Network Surface",
            Self::Sweep2 => "Sweep2",
            Self::PipeVariable => "Pipe Variable",
            Self::ExtrudeLinear => "Extrude Linear",
            Self::Loft => "Loft",
            Self::ExtrudeAngled => "Extrude Angled",
            Self::Sweep1 => "Sweep1",
            Self::ExtrudePoint => "Extrude Point",
            Self::Pipe => "Pipe",
            Self::FourPointSurface => "4Point Surface",
            Self::FragmentPatch => "Fragment Patch",
            Self::Revolution => "Revolution",
            Self::BoundarySurfaces => "Boundary Surfaces",
            Self::RailRevolution => "Rail Revolution",
        }
    }
}

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
#[allow(dead_code)]
fn from_geom_point(p: GeomPoint3) -> [f64; 3] {
    p.to_array()
}

/// Converts an array [f64; 3] to a geom Vec3.
#[inline]
fn to_geom_vec(v: [f64; 3]) -> GeomVec3 {
    GeomVec3::new(v[0], v[1], v[2])
}

/// Converts curve segments to a polyline of GeomPoint3.
///
/// Curve segments are pairs of points `(start, end)`. This function chains them
/// into a single polyline, assuming consecutive segments share endpoints.
fn segments_to_geom_polyline(segments: &[([f64; 3], [f64; 3])]) -> Vec<GeomPoint3> {
    if segments.is_empty() {
        return Vec::new();
    }

    let mut points = Vec::with_capacity(segments.len() + 1);
    points.push(to_geom_point(segments[0].0));

    for (_, end) in segments {
        points.push(to_geom_point(*end));
    }

    points
}

/// Converts a list of points `[f64; 3]` to a vector of `GeomPoint3`.
fn points_to_geom_polyline(points: &[[f64; 3]]) -> Vec<GeomPoint3> {
    points.iter().copied().map(to_geom_point).collect()
}

/// Converts a `geom::GeomMesh` to `Value::Mesh`.
fn geom_mesh_to_value_mesh(
    mesh: crate::geom::GeomMesh,
    diagnostics: Option<crate::geom::GeomMeshDiagnostics>,
) -> Value {
    Value::Mesh {
        vertices: mesh.positions,
        indices: mesh.indices,
        normals: mesh.normals,
        uvs: mesh.uvs,
        diagnostics: diagnostics.map(|d| crate::graph::value::MeshDiagnostics {
            vertex_count: d.vertex_count,
            triangle_count: d.triangle_count,
            welded_vertex_count: d.welded_vertex_count,
            flipped_triangle_count: d.flipped_triangle_count,
            degenerate_triangle_count: d.degenerate_triangle_count,
            open_edge_count: d.open_edge_count,
            non_manifold_edge_count: d.non_manifold_edge_count,
            self_intersection_count: d.self_intersection_count,
            boolean_fallback_used: d.boolean_fallback_used,
            warnings: d.warnings,
        }),
    }
}

/// Converts a `geom::GeomMesh` to `Value::Surface` (legacy format).
///
/// This is for backward compatibility with existing consumers expecting surfaces.
fn geom_mesh_to_value_surface(mesh: crate::geom::GeomMesh) -> Value {
    let faces: Vec<Vec<u32>> = mesh
        .indices
        .chunks(3)
        .filter(|chunk| chunk.len() == 3)
        .map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
        .collect();
    Value::Surface {
        vertices: mesh.positions,
        faces,
    }
}

/// Converts an `ExtrusionError` to a `ComponentError`.
fn extrusion_error_to_component_error(err: ExtrusionError, component: &str) -> ComponentError {
    ComponentError::new(format!("{component}: {err}"))
}

/// Converts a `LoftError` to a `ComponentError`.
fn loft_error_to_component_error(err: LoftError, component: &str) -> ComponentError {
    ComponentError::new(format!("{component}: {err}"))
}

/// Converts a `SweepError` to a `ComponentError`.
fn sweep_error_to_component_error(err: SweepError, component: &str) -> ComponentError {
    ComponentError::new(format!("{component}: {err}"))
}

/// Converts a `PipeError` to a `ComponentError`.
fn pipe_error_to_component_error(err: PipeError, component: &str) -> ComponentError {
    ComponentError::new(format!("{component}: {err}"))
}

/// Converts a `PatchError` to a `ComponentError`.
fn patch_error_to_component_error(err: PatchError, component: &str) -> ComponentError {
    ComponentError::new(format!("{component}: {err}"))
}

/// Converts a `RevolveError` to a `ComponentError`.
fn revolve_error_to_component_error(err: RevolveError, component: &str) -> ComponentError {
    ComponentError::new(format!("{component}: {err}"))
}

/// Converts a `SurfaceFitError` to a `ComponentError`.
fn surface_fit_error_to_component_error(err: SurfaceFitError, component: &str) -> ComponentError {
    ComponentError::new(format!("{component}: {err}"))
}

/// Parse revolve cap type from a boolean value.
/// true = caps on both ends (for closed profiles), false = no caps
#[allow(dead_code)] // Available for future use when explicit cap control is added
fn parse_revolve_caps_from_bool(caps: bool) -> RevolveCaps {
    if caps {
        RevolveCaps::BOTH
    } else {
        RevolveCaps::NONE
    }
}

/// Parse pipe cap type from a numeric value (Grasshopper convention).
/// 0 = None, 1 = Flat, 2 = Round (treated as Flat for now)
fn parse_pipe_caps_from_number(n: f64) -> PipeCaps {
    match n as i32 {
        0 => PipeCaps::NONE,
        1 | 2 => PipeCaps::BOTH, // Flat and Round both result in caps
        _ => PipeCaps::NONE, // Default fallback
    }
}

/// Parse the extrusion axis for ExtrudeLinear from inputs.
///
/// ExtrudeLinear supports several input patterns:
/// 1. Axis as a vector: `[dx, dy, dz]` - direction and distance combined
/// 2. Axis as a number: `distance` - extrudes along Z-axis (or orientation normal)
/// 3. Axis as a plane: extracts the normal vector with unit length
///
/// When the axis is a number, the orientation input (if present) provides
/// the extrusion direction. If no orientation is given, Z-axis is used.
///
/// # Arguments
/// * `axis_input` - The Axis (A) input, which can be a vector, number, or plane
/// * `orientation_input` - Optional Orientation (Po) input for direction when axis is a number
/// * `component` - Component name for error messages
fn parse_extrude_linear_axis(
    axis_input: Option<&Value>,
    orientation_input: Option<&Value>,
    component: &str,
) -> Result<[f64; 3], ComponentError> {
    let Some(axis_value) = axis_input else {
        // No axis provided - use default Z direction with unit length
        return Ok([0.0, 0.0, 1.0]);
    };

    // Unwrap single-element lists
    let axis_value = unwrap_single_element_list(axis_value);

    match axis_value {
        // Vector input: use directly as direction and distance
        Value::Vector(v) => Ok(*v),

        // Point treated as vector from origin
        Value::Point(p) => Ok(*p),

        // Number input: interpret as distance along a direction
        Value::Number(distance) => {
            let distance = *distance;
            if distance.abs() < EPSILON {
                return Err(ComponentError::new(format!(
                    "{component}: afstand mag niet nul zijn"
                )));
            }

            // Get direction from orientation input, or default to Z-axis
            let direction = extract_direction_from_orientation(orientation_input);

            // Normalize direction and scale by distance
            let len = (direction[0].powi(2) + direction[1].powi(2) + direction[2].powi(2)).sqrt();
            if len < EPSILON {
                // Orientation had zero-length direction, fall back to Z
                return Ok([0.0, 0.0, distance]);
            }

            Ok([
                direction[0] / len * distance,
                direction[1] / len * distance,
                direction[2] / len * distance,
            ])
        }

        // List of 3 numbers: treat as vector components
        Value::List(items) if items.len() >= 3 => {
            let x = coerce_number(&items[0], component, "Axis X")?;
            let y = coerce_number(&items[1], component, "Axis Y")?;
            let z = coerce_number(&items[2], component, "Axis Z")?;
            Ok([x, y, z])
        }

        // List of 2 numbers: treat as 2D vector in XY plane
        Value::List(items) if items.len() == 2 => {
            let x = coerce_number(&items[0], component, "Axis X")?;
            let y = coerce_number(&items[1], component, "Axis Y")?;
            Ok([x, y, 0.0])
        }

        // Null: use default
        Value::Null => Ok([0.0, 0.0, 1.0]),

        other => Err(ComponentError::new(format!(
            "{component}: verwacht een vector of getal voor de as, kreeg {}",
            other.kind()
        ))),
    }
}

/// Unwrap a single-element list to its contained value.
fn unwrap_single_element_list(value: &Value) -> &Value {
    if let Value::List(items) = value {
        if items.len() == 1 {
            return unwrap_single_element_list(&items[0]);
        }
    }
    value
}

/// Extract a direction vector from an orientation input.
///
/// The orientation can be:
/// - A vector: used directly (normalized)
/// - A plane: the Z-axis (normal) is extracted
/// - A point: interpreted as direction from origin
/// - A number: interpreted as Z component of direction
/// - None/other: returns default Z-axis direction
fn extract_direction_from_orientation(orientation: Option<&Value>) -> [f64; 3] {
    let Some(value) = orientation else {
        return [0.0, 0.0, 1.0];
    };

    let value = unwrap_single_element_list(value);

    match value {
        Value::Vector(v) => *v,
        Value::Point(p) => *p,
        Value::Number(n) => [0.0, 0.0, *n],
        // For plane values, try to extract the normal (Z-axis)
        Value::List(items) if items.len() >= 3 => {
            // Could be [origin, x_axis, y_axis] or similar
            // For now, if we have 3 points, compute the normal
            if items.iter().all(|i| matches!(i, Value::Point(_))) {
                if let (Some(Value::Point(p1)), Some(Value::Point(p2)), Some(Value::Point(p3))) =
                    (items.get(0), items.get(1), items.get(2))
                {
                    // Compute cross product of (p2-p1) x (p3-p1)
                    let ab = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
                    let ac = [p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]];
                    return [
                        ab[1] * ac[2] - ab[2] * ac[1],
                        ab[2] * ac[0] - ab[0] * ac[2],
                        ab[0] * ac[1] - ab[1] * ac[0],
                    ];
                }
            }
            // Try to interpret as numeric vector
            if let (Ok(x), Ok(y), Ok(z)) = (
                coerce_number_silent(&items[0]),
                coerce_number_silent(&items[1]),
                coerce_number_silent(&items[2]),
            ) {
                return [x, y, z];
            }
            [0.0, 0.0, 1.0]
        }
        _ => [0.0, 0.0, 1.0],
    }
}

/// Coerce a value to a number without generating errors.
fn coerce_number_silent(value: &Value) -> Result<f64, ()> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err(()),
    }
}

/// Parse loft type from a numeric value (Grasshopper convention).
/// 0 = Normal, 1 = Loose, 2 = Tight, 3 = Straight, 4 = Developable, 5 = Uniform
fn parse_loft_type_from_number(n: f64) -> GeomLoftType {
    match n as i32 {
        0 => GeomLoftType::Normal,
        1 => GeomLoftType::Loose,
        2 => GeomLoftType::Tight,
        3 => GeomLoftType::Straight,
        4 => GeomLoftType::Developable,
        5 => GeomLoftType::Uniform,
        _ => GeomLoftType::Normal, // Default fallback
    }
}

/// Parse `GeomLoftOptions` from a `Value` containing JSON options or component values.
///
/// # Supported Input Formats
///
/// - `Value::Text(json)` - JSON object: `{"closed":false,"adjust":true,"rebuild":0,"refit":0.0,"type":0}`
/// - `Value::Number(n)` - Loft type as number (0=Normal, 1=Loose, 2=Tight, 3=Straight, 4=Developable, 5=Uniform)
/// - `Value::Boolean(b)` - Closed flag
/// - `Value::List([...])` - First element parsed recursively, or structured as [type, closed, adjust, rebuild, refit]
///
/// The JSON parser is tolerant of whitespace and formatting variations.
///
/// # Arguments
///
/// * `value` - The value to parse options from
/// * `mesh_quality` - Optional mesh quality settings to inject into the options
fn parse_loft_options_from_value_with_quality(
    value: &Value,
    mesh_quality: Option<GeomMeshQuality>,
) -> GeomLoftOptions {
    let mut options = GeomLoftOptions::default();
    options.mesh_quality = mesh_quality;

    match value {
        Value::Text(text) => {
            // Normalize the text: trim whitespace, handle both compact and formatted JSON
            let json = text.trim();
            
            // Check if it looks like JSON (starts with '{' or '[')
            if json.starts_with('{') {
                // Parse JSON object format
                if let Some(closed) = extract_json_bool(json, "closed") {
                    options.closed = closed;
                }
                if let Some(adjust) = extract_json_bool(json, "adjust") {
                    options.adjust_seams = adjust;
                }
                if let Some(rebuild) = extract_json_number(json, "rebuild") {
                    // rebuild > 0 enables rebuild with that point count
                    if rebuild > 0.0 {
                        options.rebuild = true;
                        options.rebuild_point_count = rebuild as usize;
                    } else {
                        options.rebuild = false;
                    }
                }
                if let Some(refit) = extract_json_number(json, "refit") {
                    options.refit_tolerance = refit;
                }
                if let Some(loft_type) = extract_json_number(json, "type") {
                    options.loft_type = parse_loft_type_from_number(loft_type);
                }
            } else if let Ok(n) = json.parse::<f64>() {
                // Plain number string - treat as loft type
                options.loft_type = parse_loft_type_from_number(n);
            } else {
                // Try to parse as a loft type name
                options.loft_type = match json.to_ascii_lowercase().as_str() {
                    "normal" | "0" => GeomLoftType::Normal,
                    "loose" | "1" => GeomLoftType::Loose,
                    "tight" | "2" => GeomLoftType::Tight,
                    "straight" | "ruled" | "3" => GeomLoftType::Straight,
                    "developable" | "4" => GeomLoftType::Developable,
                    "uniform" | "5" => GeomLoftType::Uniform,
                    _ => GeomLoftType::Normal,
                };
            }
        }
        Value::Number(n) => {
            // If just a number is passed, treat it as the loft type
            options.loft_type = parse_loft_type_from_number(*n);
        }
        Value::Boolean(closed) => {
            // If just a boolean is passed, treat it as the closed flag
            options.closed = *closed;
        }
        Value::List(items) => {
            match items.len() {
                0 => {
                    // Empty list - use defaults
                }
                1 => {
                    // Single-element list - unwrap and parse recursively
                    return parse_loft_options_from_value_with_quality(&items[0], mesh_quality);
                }
                _ => {
                    // Multi-element list: try structured format [type, closed, adjust, rebuild, refit]
                    // Or the first element might be a JSON/options value
                    if let Value::Text(_) = &items[0] {
                        // First element is text, likely JSON - parse just that
                        return parse_loft_options_from_value_with_quality(&items[0], mesh_quality);
                    }
                    
                    // Try structured format: [type, closed?, adjust?, rebuild?, refit?]
                    if let Value::Number(n) = &items[0] {
                        options.loft_type = parse_loft_type_from_number(*n);
                    }
                    if let Some(Value::Boolean(closed)) = items.get(1) {
                        options.closed = *closed;
                    }
                    if let Some(Value::Boolean(adjust)) = items.get(2) {
                        options.adjust_seams = *adjust;
                    }
                    if let Some(Value::Number(rebuild)) = items.get(3) {
                        if *rebuild > 0.0 {
                            options.rebuild = true;
                            options.rebuild_point_count = *rebuild as usize;
                        }
                    }
                    if let Some(Value::Number(refit)) = items.get(4) {
                        options.refit_tolerance = *refit;
                    }
                }
            }
        }
        _ => {}
    }

    options
}

/// Parse `GeomLoftOptions` from a `Value` (legacy wrapper without mesh quality).
///
/// This function preserves backward compatibility for call sites that don't
/// pass mesh quality explicitly. Prefer `parse_loft_options_from_value_with_quality`
/// when mesh quality should be forwarded.
#[allow(dead_code)]
fn parse_loft_options_from_value(value: &Value) -> GeomLoftOptions {
    parse_loft_options_from_value_with_quality(value, None)
}

/// Simple JSON boolean extraction (no external dependency).
fn extract_json_bool(json: &str, key: &str) -> Option<bool> {
    let pattern = format!("\"{}\":", key);
    if let Some(idx) = json.find(&pattern) {
        let rest = &json[idx + pattern.len()..];
        let trimmed = rest.trim_start();
        if trimmed.starts_with("true") {
            return Some(true);
        } else if trimmed.starts_with("false") {
            return Some(false);
        }
    }
    None
}

/// Simple JSON number extraction (no external dependency).
fn extract_json_number(json: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\":", key);
    if let Some(idx) = json.find(&pattern) {
        let rest = &json[idx + pattern.len()..];
        let trimmed = rest.trim_start();
        // Find the end of the number (comma, }, or end of string)
        let end_idx = trimmed
            .find(|c: char| c == ',' || c == '}' || c == ' ' || c == '\n')
            .unwrap_or(trimmed.len());
        if let Ok(n) = trimmed[..end_idx].parse::<f64>() {
            return Some(n);
        }
    }
    None
}

fn unify_curve_directions(polylines: &mut [Vec<[f64; 3]>]) {
    if polylines.len() < 2 {
        return;
    }

    // Stap 1: Classificeer curves en vind gesloten curves
    let closed_indices: Vec<usize> = polylines
        .iter()
        .enumerate()
        .filter(|(_, p)| is_closed(p))
        .map(|(i, _)| i)
        .collect();

    // Stap 2: Als er gesloten curves zijn, standaardiseer hun richting
    if !closed_indices.is_empty() {
        // Neem de eerste gesloten curve als referentie.
        let first_closed_idx = closed_indices[0];
        let reference_normal = polyline_normal(&polylines[first_closed_idx]);
        let reference_winding =
            polyline_winding_direction(&polylines[first_closed_idx], reference_normal);

        // Streef naar een positieve winding (CCW). Als de referentie zelf CW is, keer de normaal om.
        let target_normal = if reference_winding < 0.0 {
            [
                -reference_normal[0],
                -reference_normal[1],
                -reference_normal[2],
            ]
        } else {
            reference_normal
        };

        // Keer elke gesloten curve om die niet overeenkomt met de doelrichting.
        for &i in &closed_indices {
            let winding = polyline_winding_direction(&polylines[i], target_normal);
            if winding < 0.0 {
                polylines[i].reverse();
            }
        }
    }

    // Stap 3: Oriënteer open curves ten opzichte van hun voorganger voor een vloeiende overgang
    // Open curves keep their original authoring direction; only closed curves are unified.
}

fn evaluate_loft(inputs: &[Value], meta: &MetaMap, component: &str, output: &str) -> ComponentResult {
    // Determine the loft variant based on component name
    let loft_variant = match component {
        "Fit Loft" => LoftVariant::Fit,
        "Control Point Loft" => LoftVariant::ControlPoint,
        _ => LoftVariant::Standard,
    };
    
    evaluate_loft_with_variant(inputs, meta, component, output, loft_variant)
}

/// Loft variant determines which geom loft function to use.
#[derive(Debug, Clone, Copy)]
enum LoftVariant {
    /// Standard loft with configurable options
    Standard,
    /// Fit loft that interpolates exactly through profiles
    Fit,
    /// Control-point loft where profiles act as B-spline control points
    ControlPoint,
}

/// Core loft evaluation function that calls the geom::loft module.
///
/// This function extracts `MeshQuality` from the `MetaMap` and forwards it
/// to the loft options, ensuring tessellation quality settings are respected.
fn evaluate_loft_with_variant(
    inputs: &[Value],
    meta: &MetaMap,
    component: &str,
    output: &str,
    variant: LoftVariant,
) -> ComponentResult {
    let curves_value = expect_input(inputs, 0, component, "curveverzameling")?;
    let multi_source = input_source_count(meta, 0) >= 2;
    let branch_values = collect_loft_branch_values(curves_value, multi_source);

    // Extract mesh quality from MetaMap (honors presets like "high", "low", etc.)
    // This allows components to specify tessellation quality via their metadata.
    let mesh_quality = coerce::geom_bridge::geom_quality_from_meta_optional(meta);

    // Parse loft options from optional second input (for standard Loft component)
    // Pass mesh_quality so it gets wired into the options
    let base_options = if let Some(opts_value) = inputs.get(1) {
        parse_loft_options_from_value_with_quality(opts_value, mesh_quality)
    } else {
        let mut opts = GeomLoftOptions::default();
        opts.mesh_quality = mesh_quality;
        opts
    };
    
    // Apply variant-specific overrides while preserving mesh_quality
    let loft_options = match variant {
        LoftVariant::Standard => base_options,
        LoftVariant::Fit => GeomLoftOptions {
            loft_type: GeomLoftType::Normal,
            rebuild: false, // Preserve original profile structure
            refit_tolerance: 0.0,
            mesh_quality: base_options.mesh_quality, // Preserve quality settings
            ..base_options
        },
        LoftVariant::ControlPoint => GeomLoftOptions {
            loft_type: GeomLoftType::Loose, // B-spline approximation
            rebuild: true,
            mesh_quality: base_options.mesh_quality, // Preserve quality settings
            ..base_options
        },
    };

    let tol = Tolerance::default();

    if branch_values.len() > 1 {
        let mut lofts = Vec::new();
        let mut mesh_outputs = Vec::new();
        let mut invalid_branch = false;

        for branch in branch_values {
            let polylines = collect_ruled_surface_curves(&branch)?;
            if polylines.is_empty() {
                continue;
            }
            if polylines.len() < 2 {
                invalid_branch = true;
                continue;
            }
            let (surface, mesh_val) = build_loft_surface_geom(polylines, loft_options.clone(), tol, component)?;
            lofts.push(surface);
            mesh_outputs.push(mesh_val);
        }

        if invalid_branch {
            return Err(ComponentError::new(format!(
                "{component} vereist minimaal twee sectiecurves per tak"
            )));
        }

        // Return both surface (for backward compatibility) and mesh outputs
        let mut out = BTreeMap::new();
        out.insert(output.to_string(), Value::List(lofts));
        out.insert(PIN_OUTPUT_MESH.to_string(), Value::List(mesh_outputs));
        return Ok(out);
    }

    let polylines = collect_ruled_surface_curves(curves_value)?;
    let (surface, mesh_val) = build_loft_surface_geom(polylines, loft_options, tol, component)?;
    
    let mut out = BTreeMap::new();
    out.insert(output.to_string(), surface);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_val);
    Ok(out)
}

/// Build a loft surface using the geom::loft module.
fn build_loft_surface_geom(
    mut polylines: Vec<Vec<[f64; 3]>>,
    options: GeomLoftOptions,
    tol: Tolerance,
    component: &str,
) -> Result<(Value, Value), ComponentError> {
    if polylines.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist minimaal twee sectiecurves"
        )));
    }

    // Unify curve directions for consistent loft orientation
    unify_curve_directions(&mut polylines);

    // Convert to GeomPoint3 format
    let geom_profiles: Vec<Vec<GeomPoint3>> = polylines
        .iter()
        .map(|polyline| polyline.iter().map(|p| to_geom_point(*p)).collect())
        .collect();
    
    // Create profile slices for the geom API
    let profile_slices: Vec<&[GeomPoint3]> = geom_profiles.iter().map(|v| v.as_slice()).collect();

    // Call the geom loft function
    let (mesh, mesh_diag, _loft_diag) = loft_mesh_with_tolerance(&profile_slices, options, tol)
        .map_err(|e| loft_error_to_component_error(e, component))?;

    // Convert to Value::Mesh (primary output)
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(mesh_diag));
    
    // Also provide legacy Value::Surface for backward compatibility
    let surface_value = geom_mesh_to_value_surface(mesh);

    Ok((surface_value, mesh_value))
}

/// Legacy build_loft_surface function - kept for reference but now uses geom internally.
#[allow(dead_code)]
fn build_loft_surface_legacy(
    mut polylines: Vec<Vec<[f64; 3]>>,
    component: &str,
) -> Result<Value, ComponentError> {
    if polylines.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist minimaal twee sectiecurves"
        )));
    }

    unify_curve_directions(&mut polylines);

    let target_count = polylines.iter().map(|p| p.len()).max().unwrap_or(0);
    if target_count < 2 {
        return Err(ComponentError::new(format!(
            "{component} kon geen curves met voldoende punten vinden"
        )));
    }

    let resampled_polylines: Vec<Vec<[f64; 3]>> = polylines
        .iter()
        .map(|p| {
            let dummy_target = vec![[0.0; 3]; target_count];
            super::curve_sampler::resample_polylines(p, &dummy_target).0
        })
        .collect();

    let mut vertices = Vec::new();
    let mut faces: Vec<Vec<u32>> = Vec::new();

    for polyline in &resampled_polylines {
        vertices.extend_from_slice(polyline);
    }

    let num_curves = resampled_polylines.len();
    let num_points_per_curve = target_count;

    for i in 0..(num_curves - 1) {
        for j in 0..(num_points_per_curve - 1) {
            let base_idx = (i * num_points_per_curve + j) as u32;
            let next_in_row_idx = base_idx + 1;
            let base_in_next_curve_idx = ((i + 1) * num_points_per_curve + j) as u32;
            let next_in_next_curve_idx = base_in_next_curve_idx + 1;

            faces.push(vec![base_idx, next_in_next_curve_idx, next_in_row_idx]);
            faces.push(vec![
                base_idx,
                base_in_next_curve_idx,
                next_in_next_curve_idx,
            ]);
        }
    }

    Ok(Value::Surface { vertices, faces })
}

fn collect_loft_branch_values(value: &Value, multi_source: bool) -> Vec<Value> {
    if multi_source {
        if let Value::List(items) = value {
            if let Some(merged) = merge_grafted_branch_sources(items) {
                return merged;
            }
        }
    }

    match value {
        Value::List(items) if should_expand_loft_branches(items) => items
            .iter()
            .filter_map(|entry| match entry {
                Value::List(list) if !list.is_empty() => Some(Value::List(list.clone())),
                _ => None,
            })
            .collect(),
        Value::List(_) => {
            if let Some(branches) = split_closed_curve_branches(value) {
                return branches;
            }
            vec![value.clone()]
        }
        _ => vec![value.clone()],
    }
}

fn merge_grafted_branch_sources(items: &[Value]) -> Option<Vec<Value>> {
    let mut sources: Vec<Vec<Vec<Value>>> = Vec::new();
    for entry in items {
        if matches!(entry, Value::Null) {
            continue;
        }

        let branches = collect_source_branches(entry);
        if branches.is_empty() {
            continue;
        }
        sources.push(branches);
    }

    if sources.len() < 2 {
        return None;
    }

    let max_branches = sources
        .iter()
        .map(|branches| branches.len())
        .max()
        .unwrap_or(0);

    if max_branches == 0 {
        return None;
    }

    let mut merged = Vec::with_capacity(max_branches);
    for branch_index in 0..max_branches {
        let mut combined_entries = Vec::new();
        for source in &sources {
            if let Some(branch_curves) = source.get(branch_index) {
                combined_entries.extend(branch_curves.clone());
            }
        }

        if !combined_entries.is_empty() {
            merged.push(Value::List(combined_entries));
        }
    }

    if merged.is_empty() {
        None
    } else {
        Some(merged)
    }
}

fn collect_source_branches(value: &Value) -> Vec<Vec<Value>> {
    if matches!(value, Value::Null) {
        return Vec::new();
    }

    if value_is_curve(value) {
        return vec![vec![value.clone()]];
    }

    if let Value::List(items) = value {
        if should_expand_loft_branches(items) {
            let mut branches = Vec::new();
            for entry in items {
                match entry {
                    Value::Null => continue,
                    Value::List(list) if !list.is_empty() => {
                        let curves: Vec<Value> = list
                            .iter()
                            .filter(|curve| !matches!(curve, Value::Null))
                            .cloned()
                            .collect();
                        if !curves.is_empty() {
                            branches.push(curves);
                        }
                    }
                    other => branches.push(vec![other.clone()]),
                }
            }
            return branches;
        }

        if items.iter().all(|entry| value_is_curve(entry)) {
            return items
                .iter()
                .filter(|entry| !matches!(entry, Value::Null))
                .map(|entry| vec![entry.clone()])
                .collect();
        }
    }

    vec![vec![value.clone()]]
}

fn should_expand_loft_branches(items: &[Value]) -> bool {
    let mut found_branch = false;
    for entry in items {
        if matches!(entry, Value::Null) {
            continue;
        }

        match entry {
            Value::List(list) if !list.is_empty() => {
                if value_is_curve(entry) {
                    return false;
                }
                found_branch = true;
            }
            _ => return false,
        }
    }
    found_branch
}

/// Detects grafted branches containing closed curve primitives.
/// This is needed because closed primitives are often represented as lists of curve segments,
/// which would otherwise be treated as section curves in a single branch.
fn split_closed_curve_branches(value: &Value) -> Option<Vec<Value>> {
    let Value::List(items) = value else {
        return None;
    };

    let mut branches = Vec::new();
    for entry in items {
        if matches!(entry, Value::Null) {
            continue;
        }
        let curves = collect_ruled_surface_curves(entry).ok()?;
        if curves.len() != 1 || !is_closed(&curves[0]) {
            return None;
        }
        branches.push(entry.clone());
    }

    if branches.len() > 1 {
        Some(branches)
    } else {
        None
    }
}

fn value_is_curve(value: &Value) -> bool {
    match value {
        Value::CurveLine { .. } => true,
        Value::List(items) => {
            if items.len() < 2 {
                false
            } else if items.iter().all(|item| matches!(item, Value::Point(_))) {
                true
            } else if items.iter().all(|item| matches!(item, Value::CurveLine { .. })) {
                true
            } else if items
                .iter()
                .all(|item| matches!(item, Value::List(_) | Value::Null))
            {
                false
            } else {
                false
            }
        }
        _ => false,
    }
}

fn evaluate_edge_surface(inputs: &[Value], meta: &MetaMap) -> ComponentResult {
    let component = "Edge Surface";
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Edge Surface vereist minimaal twee randcurves",
        ));
    }

    // Collect edge polylines from up to 4 input curves
    let mut edge_polylines: Vec<Vec<GeomPoint3>> = Vec::new();
    for value in inputs.iter().take(4) {
        let segments = coerce::coerce_curve_segments(value)?;
        if segments.is_empty() {
            continue;
        }
        // Convert segments to a continuous polyline
        let polylines = group_segments_into_polylines(segments);
        if let Some(poly) = pick_longest_polyline(polylines) {
            let geom_poly: Vec<GeomPoint3> = poly.iter().copied().map(to_geom_point).collect();
            if geom_poly.len() >= 2 {
                edge_polylines.push(geom_poly);
            }
        }
    }

    if edge_polylines.len() < 2 {
        return Err(ComponentError::new(
            "Edge Surface kon onvoldoende randcurves uit de input halen (minimaal 2 vereist)",
        ));
    }

    // Extract quality from meta, falling back to default if not specified
    let quality = coerce::geom_bridge::surface_builder_quality_from_meta_optional(meta)
        .unwrap_or_default();
    let (mesh, diagnostics) = mesh_edge_surface_from_edges(&edge_polylines, quality)
        .map_err(|e| ComponentError::new(format!("{component}: {e}")))?;

    // Convert to output values
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    // Return both mesh and surface outputs for compatibility
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_SURFACE.to_string(), surface_value);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_value);
    Ok(out)
}

fn evaluate_extrude(inputs: &[Value]) -> ComponentResult {
    let component = "Extrude";
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Extrude component vereist een curve en een hoogte",
        ));
    }

    let base_segments = coerce::coerce_curve_segments(&inputs[0])?;
    if base_segments.is_empty() {
        return Err(ComponentError::new(
            "Extrude component kon geen curve herkennen",
        ));
    }

    let direction = coerce::coerce_vector(&inputs[1], component)?;
    if is_zero_vector(direction) {
        return Err(ComponentError::new(
            "Extrude component vereist een niet-nul hoogte",
        ));
    }

    // Convert segments to a single polyline for the geom extrusion
    let profile = segments_to_geom_polyline(&base_segments);
    let geom_direction = to_geom_vec(direction);
    let tol = Tolerance::default();

    // Determine if the profile is closed (first and last points coincide)
    let is_profile_closed = profile.len() >= 3
        && tol.approx_eq_point3(profile[0], profile[profile.len() - 1]);

    // Use caps for closed profiles
    let caps = if is_profile_closed {
        ExtrusionCaps::BOTH
    } else {
        ExtrusionCaps::NONE
    };

    // Call the geom extrusion function
    let (mesh, diagnostics) = extrude_polyline_with_tolerance(&profile, geom_direction, caps, tol)
        .map_err(|e| extrusion_error_to_component_error(e, component))?;

    // Output as Value::Mesh (primary) and also provide legacy Value::Surface
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    // Return primary mesh on "S" pin for backward compatibility
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_SURFACE.to_string(), surface_value);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_value);
    Ok(out)
}

fn evaluate_extrude_along(inputs: &[Value]) -> ComponentResult {
    let component = "Extrude Along";
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Extrude Along vereist een basis en een railcurve",
        ));
    }
    let base_segments = coerce::coerce_curve_segments(&inputs[0])?;
    if base_segments.is_empty() {
        return Err(ComponentError::new(
            "Extrude Along kon geen basiscurve herkennen",
        ));
    }

    // Parse and process the rail curve properly:
    // 1. Coerce all rail segments
    // 2. Group into polylines (handles multi-segment rails correctly)
    // 3. Pick the longest polyline if multiple disconnected curves exist
    // 4. Compute direction from the overall rail displacement
    let rail_segments = coerce::coerce_curve_segments(&inputs[1])?;
    if rail_segments.is_empty() {
        return Err(ComponentError::new(
            "Extrude Along kon geen railcurve herkennen",
        ));
    }

    let rail_polylines = group_segments_into_polylines(rail_segments);
    let rail_polyline = pick_longest_polyline(rail_polylines).ok_or_else(|| {
        ComponentError::new("Extrude Along kon geen geldige rail-polyline vormen")
    })?;

    // Compute the extrusion direction from the full rail polyline.
    // This handles multi-segment rails correctly by using the overall displacement.
    let direction = compute_rail_direction(&rail_polyline)?;

    // Convert segments to a single polyline for the geom extrusion
    let profile = segments_to_geom_polyline(&base_segments);
    let geom_direction = to_geom_vec(direction);
    let tol = Tolerance::default();

    // Determine if the profile is closed (first and last points coincide)
    let is_profile_closed = profile.len() >= 3
        && tol.approx_eq_point3(profile[0], profile[profile.len() - 1]);

    // Use caps for closed profiles
    let caps = if is_profile_closed {
        ExtrusionCaps::BOTH
    } else {
        ExtrusionCaps::NONE
    };

    // Call the geom extrusion function
    let (mesh, diagnostics) = extrude_polyline_with_tolerance(&profile, geom_direction, caps, tol)
        .map_err(|e| extrusion_error_to_component_error(e, component))?;

    // Output as Value::Mesh (primary) and also provide legacy Value::Surface
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    // Return both mesh and surface outputs
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_EXTRUSION.to_string(), surface_value);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_value);
    Ok(out)
}

/// Computes the extrusion direction from a rail polyline.
///
/// The direction is the vector from the first non-degenerate point to the last point
/// of the rail. This handles:
/// - Multi-segment rails by computing the overall displacement
/// - Degenerate leading segments by finding the first point that differs from subsequent ones
/// - Closed rails by using the cumulative path direction
///
/// # Arguments
/// * `polyline` - The rail as a sequence of points
///
/// # Returns
/// The direction vector `[dx, dy, dz]` representing the extrusion path,
/// or an error if the rail is entirely degenerate (zero-length).
fn compute_rail_direction(polyline: &[[f64; 3]]) -> Result<[f64; 3], ComponentError> {
    if polyline.is_empty() {
        return Err(ComponentError::new(
            "Extrude Along: rail bevat geen punten",
        ));
    }

    if polyline.len() == 1 {
        return Err(ComponentError::new(
            "Extrude Along: rail bevat slechts één punt en heeft geen richting",
        ));
    }

    // Primary approach: compute direction from first point to last point.
    // This gives the overall displacement of the rail.
    let first = polyline[0];
    let last = polyline[polyline.len() - 1];
    let direction = subtract_points(last, first);

    if !is_zero_vector(direction) {
        return Ok(direction);
    }

    // If first == last (closed rail or degenerate), try to find any non-degenerate span.
    // Walk through the polyline to find the first point that differs from `first`.
    for point in polyline.iter().skip(1) {
        let candidate_dir = subtract_points(*point, first);
        if !is_zero_vector(candidate_dir) {
            // Found a valid direction; for closed rails, this represents
            // the initial tangent direction of the path.
            return Ok(candidate_dir);
        }
    }

    // All points are coincident - the rail is entirely degenerate.
    Err(ComponentError::new(
        "Extrude Along: rail heeft geen geldige richting (alle punten zijn samenvallend)",
    ))
}

fn evaluate_loft_options(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 5 {
        return Err(ComponentError::new(
            "Loft Options vereist geslotenheid, seam-aanpassing, rebuild, refit en type",
        ));
    }

    let closed = coerce_bool(&inputs[0], "Loft Options", "Closed")?;
    let adjust = coerce_bool(&inputs[1], "Loft Options", "Adjust")?;
    let rebuild = coerce_number(&inputs[2], "Loft Options", "Rebuild")?;
    let refit = coerce_number(&inputs[3], "Loft Options", "Refit")?;
    let loft_type = coerce_number(&inputs[4], "Loft Options", "Type")?;

    let summary = format!(
        "{{\"closed\":{closed},\"adjust\":{adjust},\"rebuild\":{rebuild},\"refit\":{refit},\"type\":{loft_type}}}"
    );

    into_output(PIN_OUTPUT_OPTIONS, Value::Text(summary))
}

fn evaluate_surface_from_points(inputs: &[Value], component: &str) -> ComponentResult {
    let points_value = expect_input(inputs, 0, component, "puntverzameling")?;
    let points = collect_points(points_value, component)?;
    if points.len() < 3 {
        return Err(ComponentError::new(format!(
            "{component} vereist minimaal drie punten"
        )));
    }

    // Parse optional U Count input (number of points in U direction for grid layout)
    // Uses coerce_optional_positive_integer to:
    // - Error on invalid types (not silently treat as None)
    // - Reject negative, NaN, and infinite values with clear messages
    // - Safely convert to usize after validation
    let u_count: Option<usize> = coerce_optional_positive_integer(
        inputs.get(1),
        component,
        "U Count",
    )?;

    // Parse optional Interpolate input (whether to interpolate through points)
    // When true (default): creates a smooth NURBS surface through all grid points,
    //   then tessellates it to produce a refined mesh with more vertices
    // When false: uses the grid points directly as mesh vertices (exact point positions)
    // Uses coerce_optional_bool_with_default to properly error on invalid types
    let interpolate: bool = coerce_optional_bool_with_default(
        inputs.get(2),
        true, // Default to interpolation
        component,
        "Interpolate",
    )?;

    // Convert points to geom Point3 format
    let geom_points: Vec<GeomPoint3> = points.iter().copied().map(to_geom_point).collect();

    // Determine if we have a valid grid layout or scattered points
    let (mesh, diagnostics) = if let Some(u) = u_count {
        // Grid layout: user specified U count, compute V count from point count
        let u = u.max(2); // Ensure at least 2 points in U direction
        let total = geom_points.len();

        // V count must be an integer divisor of total point count
        if total % u != 0 {
            return Err(ComponentError::new(format!(
                "{component}: aantal punten ({total}) is niet deelbaar door U count ({u})"
            )));
        }

        let v = total / u;
        if v < 2 {
            return Err(ComponentError::new(format!(
                "{component}: V count ({v}) moet minimaal 2 zijn voor een grid"
            )));
        }

        // Build fitting options based on the interpolate flag
        let options = SurfaceFitOptions {
            interpolate,
            degree_u: 3, // Cubic by default for smooth surfaces
            degree_v: 3,
            close_u: false,
            close_v: false,
            tolerance: Tolerance::default(),
        };

        // Use mesh_from_grid_with_options which respects the interpolate flag:
        // - interpolate=true: creates a NURBS surface, then tessellates it (smooth)
        // - interpolate=false: uses grid points directly as mesh vertices (exact)
        mesh_from_grid_with_options(&geom_points, u, v, options)
            .map_err(|e| surface_fit_error_to_component_error(e, component))?
    } else {
        // Scattered points: use best-fit plane triangulation
        // Note: interpolate flag doesn't affect scattered point handling
        let tol = Tolerance::default();
        mesh_from_scattered_points(&geom_points, tol)
            .map_err(|e| surface_fit_error_to_component_error(e, component))?
    };

    // Compute proper mesh diagnostics by analyzing the mesh topology.
    // This captures open edges, degenerate triangles, and other mesh quality metrics
    // that would otherwise be lost if we just copied the fitting diagnostics.
    let mesh_diagnostics = mesh.compute_diagnostics_with_warnings(
        Tolerance::default(),
        diagnostics.warnings.clone(),
    );
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(mesh_diagnostics));

    // Also create legacy Value::Surface for backward compatibility
    let surface_value = geom_mesh_to_value_surface(mesh);

    // Return both outputs: Surface on "S" pin and Mesh on "M" pin
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_SURFACE.to_string(), surface_value);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_value);
    Ok(out)
}

fn evaluate_patch(inputs: &[Value]) -> ComponentResult {
    let component = "Patch";
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Patch vereist minimaal een lijst met curves",
        ));
    }

    // =========================================================================
    // Parse input pins according to jsghxparser_nodelist_surface_freeform.json:
    //   inputs[0]: Curves (C) - Boundary curves to patch
    //   inputs[1]: Points (P) - Interior points to incorporate
    //   inputs[2]: Spans (S)  - Number of spans (subdivision control)
    //   inputs[3]: Flexibility (F) - Patch flexibility (0.0=stiff, higher=flexible)
    //   inputs[4]: Trim (T) - Whether to trim the result to the boundary
    // =========================================================================

    // Input 0: Boundary curves (required)
    let boundary_segments = coerce::coerce_curve_segments(&inputs[0])?;
    if boundary_segments.is_empty() {
        return Err(ComponentError::new(
            "Patch kon geen randcurves lezen",
        ));
    }

    // Input 1: Interior points (optional) - Points to incorporate into the patch mesh
    let interior_points: Vec<GeomPoint3> = if let Some(points_value) = inputs.get(1) {
        collect_optional_points(points_value, component)
    } else {
        Vec::new()
    };

    // Input 2: Spans (optional, default 10) - Controls boundary subdivision density
    let spans: u32 = if let Some(spans_value) = inputs.get(2) {
        coerce::coerce_optional_number_with_default(Some(spans_value), 10.0, "Spans")?
            .round()
            .clamp(1.0, 100.0) as u32
    } else {
        10
    };

    // Input 3: Flexibility (optional, default 1.0) - Controls internal subdivision
    let flexibility: f64 = if let Some(flex_value) = inputs.get(3) {
        coerce::coerce_optional_number_with_default(Some(flex_value), 1.0, "Flexibility")?
            .clamp(0.0, 10.0)
    } else {
        1.0
    };

    // Input 4: Trim (optional, default true) - Whether to trim to boundary
    let trim: bool = if let Some(trim_value) = inputs.get(4) {
        coerce::coerce_optional_boolean_with_default(Some(trim_value), true, "Trim")?
    } else {
        true
    };

    // Group segments into polylines (one per connected boundary loop)
    let boundary_polylines = group_segments_into_polylines(boundary_segments);
    if boundary_polylines.is_empty() {
        return Err(ComponentError::new(
            "Patch kon geen geldige randpolylines vormen",
        ));
    }

    // Prepare boundary loops: auto-close open curves with tracking for warnings.
    // Note: Patch requires closed boundaries, but we accept open curves for backward
    // compatibility and emit warnings when auto-closing occurs.
    let prepared = prepare_boundary_loops_for_patch(&boundary_polylines);
    // Call auto_close_warnings before moving closed_polylines to avoid borrow-after-move
    let auto_close_warnings = prepared.auto_close_warnings(component);
    let closed_polylines = prepared.closed_polylines;

    if closed_polylines.is_empty() {
        return Err(ComponentError::new(
            "Patch vereist minstens één gesloten randcurve met drie of meer punten",
        ));
    }

    // Convert to geom Point3 format
    let geom_polylines: Vec<Vec<GeomPoint3>> = closed_polylines
        .iter()
        .map(|poly| points_to_geom_polyline(poly))
        .collect();

    // Calculate signed area to find the outer boundary (largest absolute area)
    let tol = Tolerance::default();

    // Build PatchOptions with all parsed parameters
    let patch_options = PatchOptions::default()
        .with_spans(spans)
        .with_flexibility(flexibility)
        .with_trim(trim)
        .with_interior_points(interior_points);
    
    // If there's only one polyline, use patch_mesh_with_options for the outer boundary
    if geom_polylines.len() == 1 {
        let (mesh, mut diagnostics) = patch_mesh_with_options(
            &geom_polylines[0],
            &[],  // No holes
            patch_options,
            tol,
        ).map_err(|e| patch_error_to_component_error(e, component))?;

        // Add auto-close warnings to diagnostics
        for warning in &auto_close_warnings {
            diagnostics.add_warning(warning.clone());
        }

        let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
        let surface_value = geom_mesh_to_value_surface(mesh);

        let mut out = BTreeMap::new();
        out.insert(PIN_OUTPUT_PATCH.to_string(), surface_value);
        out.insert(PIN_OUTPUT_MESH.to_string(), mesh_value);
        return Ok(out);
    }

    // Multiple polylines: determine outer vs holes by signed area
    // The polyline with the largest absolute area is the outer boundary
    let mut areas: Vec<(usize, f64)> = geom_polylines
        .iter()
        .enumerate()
        .map(|(i, poly)| {
            // Use a simple normal estimate for the plane
            let normal = polyline_normal(&closed_polylines[i]);
            let area = signed_area_in_plane(&closed_polylines[i], normal).abs();
            (i, area)
        })
        .collect();
    
    areas.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

    let outer_idx = areas[0].0;
    let outer_boundary = &geom_polylines[outer_idx];
    
    // All other polylines are treated as holes
    let holes: Vec<Vec<GeomPoint3>> = areas
        .iter()
        .skip(1)
        .map(|(idx, _)| geom_polylines[*idx].clone())
        .collect();

    // Call geom::patch_mesh_with_options with outer boundary, holes, and options
    let (mesh, mut diagnostics) = patch_mesh_with_options(
        outer_boundary,
        &holes,
        patch_options,
        tol,
    ).map_err(|e| patch_error_to_component_error(e, component))?;

    // Add auto-close warnings to diagnostics
    for warning in &auto_close_warnings {
        diagnostics.add_warning(warning.clone());
    }

    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_PATCH.to_string(), surface_value);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_value);
    Ok(out)
}

/// Collects optional interior points from a Value, returning an empty Vec if the input
/// is null, empty, or cannot be parsed as points.
fn collect_optional_points(value: &Value, component: &str) -> Vec<GeomPoint3> {
    match value {
        Value::Null => Vec::new(),
        Value::Point(p) => vec![to_geom_point(*p)],
        Value::List(values) if values.is_empty() => Vec::new(),
        Value::List(values) => {
            let mut points = Vec::new();
            for entry in values {
                match entry {
                    Value::Point(p) => points.push(to_geom_point(*p)),
                    Value::List(nested) => {
                        for nested_entry in nested {
                            if let Value::Point(p) = nested_entry {
                                points.push(to_geom_point(*p));
                            }
                        }
                    }
                    _ => {
                        // Try to coerce as a point, silently skip if it fails
                        if let Ok(p) = coerce::coerce_point_with_context(entry, component) {
                            points.push(to_geom_point(p));
                        }
                    }
                }
            }
            points
        }
        // Try single point coercion as fallback
        _ => {
            if let Ok(p) = coerce::coerce_point_with_context(value, component) {
                vec![to_geom_point(p)]
            } else {
                Vec::new()
            }
        }
    }
}

fn evaluate_sum_surface(inputs: &[Value], meta: &MetaMap) -> ComponentResult {
    let component = "Sum Surface";
    if inputs.len() < 2 {
        return Err(ComponentError::new("Sum Surface vereist twee invoercurves"));
    }

    // Collect curves from inputs as lists for proper data-matching
    let curves_u = collect_sum_surface_curves(&inputs[0])?;
    let curves_v = collect_sum_surface_curves(&inputs[1])?;

    if curves_u.is_empty() || curves_v.is_empty() {
        return Err(ComponentError::new(
            "Sum Surface kon geen geldige curves herkennen uit de invoer",
        ));
    }

    // Determine target count using Grasshopper data-matching rules:
    // - If one list has length 1, repeat it for all items in the other list
    // - Otherwise, match by index up to the minimum length
    let target_count = match (curves_u.len(), curves_v.len()) {
        (1, v) => v,
        (u, 1) => u,
        (u, v) => u.min(v),
    };

    // Track successful results and failures separately for proper error reporting
    struct SumSurfaceResult {
        mesh: crate::geom::GeomMesh,
        diagnostics: crate::geom::GeomMeshDiagnostics,
    }
    struct SkippedPair {
        index: usize,
        reason: String,
    }

    let mut successes: Vec<SumSurfaceResult> = Vec::new();
    let mut skipped: Vec<SkippedPair> = Vec::new();

    // Extract quality from meta, falling back to default if not specified
    let quality = coerce::geom_bridge::surface_builder_quality_from_meta_optional(meta)
        .unwrap_or_default();

    for idx in 0..target_count {
        // Data-match: if a list has length 1, reuse that single curve for all pairs
        let polyline_u = if curves_u.len() == 1 {
            &curves_u[0]
        } else {
            &curves_u[idx]
        };
        let polyline_v = if curves_v.len() == 1 {
            &curves_v[0]
        } else {
            &curves_v[idx]
        };

        // Validate curve lengths and report specific issues
        let len_u = polyline_u.len();
        let len_v = polyline_v.len();
        if len_u < 2 || len_v < 2 {
            let reason = match (len_u < 2, len_v < 2) {
                (true, true) => format!(
                    "both curves have insufficient points (U-curve: {len_u}, V-curve: {len_v}, minimum: 2)"
                ),
                (true, false) => format!(
                    "U-curve has insufficient points ({len_u}, minimum: 2)"
                ),
                (false, true) => format!(
                    "V-curve has insufficient points ({len_v}, minimum: 2)"
                ),
                (false, false) => unreachable!(),
            };
            log::warn!("{component}: skipping curve pair {idx}: {reason}");
            skipped.push(SkippedPair { index: idx, reason });
            continue;
        }

        // Convert to geom Point3 format
        let geom_u: Vec<GeomPoint3> = polyline_u.iter().copied().map(to_geom_point).collect();
        let geom_v: Vec<GeomPoint3> = polyline_v.iter().copied().map(to_geom_point).collect();

        // Use the geom sum surface builder
        match mesh_sum_surface(&geom_u, &geom_v, quality) {
            Ok((mesh, diagnostics)) => {
                successes.push(SumSurfaceResult { mesh, diagnostics });
            }
            Err(e) => {
                let reason = e.to_string();
                log::warn!("{component}: skipping curve pair {idx}: {reason}");
                skipped.push(SkippedPair { index: idx, reason });
            }
        }
    }

    // Handle complete failure: return an error with details, not silent Null
    if successes.is_empty() {
        let error_details = if skipped.len() == 1 {
            format!("failed to create sum surface: {}", skipped[0].reason)
        } else {
            let reasons: Vec<String> = skipped
                .iter()
                .map(|s| format!("pair {}: {}", s.index, s.reason))
                .collect();
            format!(
                "failed to create sum surface for all {} curve pairs:\n  - {}",
                skipped.len(),
                reasons.join("\n  - ")
            )
        };
        return Err(ComponentError::new(format!("{component}: {error_details}")));
    }

    // Build output values with merged diagnostics that include warnings about skipped pairs
    let mut mesh_values = Vec::with_capacity(successes.len());
    let mut surface_values = Vec::with_capacity(successes.len());

    for (result_idx, result) in successes.into_iter().enumerate() {
        let mut diagnostics = result.diagnostics;

        // For the first result, add warnings about all skipped pairs (aggregate once)
        if result_idx == 0 && !skipped.is_empty() {
            for skip in &skipped {
                diagnostics.add_warning(format!(
                    "curve pair {} skipped: {}",
                    skip.index, skip.reason
                ));
            }
        }

        mesh_values.push(geom_mesh_to_value_mesh(result.mesh.clone(), Some(diagnostics)));
        surface_values.push(geom_mesh_to_value_surface(result.mesh));
    }

    // Return output matching the count pattern:
    // - Single result: return unwrapped values
    // - Multiple results: return as lists
    let (surface_output, mesh_output) = match (surface_values.len(), mesh_values.len()) {
        (1, 1) => (
            surface_values.into_iter().next().unwrap(),
            mesh_values.into_iter().next().unwrap(),
        ),
        _ => (Value::List(surface_values), Value::List(mesh_values)),
    };

    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_SURFACE.to_string(), surface_output);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_output);
    Ok(out)
}

/// Collects curves from a Value for Sum Surface, returning a list of polylines.
/// This function properly handles nested lists for data-matching support,
/// preserving individual curves so that list inputs produce multiple surfaces.
fn collect_sum_surface_curves(value: &Value) -> Result<Vec<Vec<[f64; 3]>>, ComponentError> {
    match value {
        Value::Null => Ok(Vec::new()),
        Value::CurveLine { p1, p2 } => Ok(vec![vec![*p1, *p2]]),
        Value::List(values) => {
            if values.is_empty() {
                return Ok(Vec::new());
            }

            // Case 1: List of points -> treat as a single polyline
            if values.iter().all(|entry| matches!(entry, Value::Point(_))) {
                let polyline = values
                    .iter()
                    .filter_map(|entry| match entry {
                        Value::Point(point) => Some(*point),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                if polyline.len() < 2 {
                    return Ok(Vec::new());
                }
                return Ok(vec![polyline]);
            }

            // Case 2: List of lists/nulls -> recurse to collect multiple curves
            if values
                .iter()
                .all(|entry| matches!(entry, Value::List(_) | Value::Null))
            {
                let mut curves = Vec::new();
                for entry in values {
                    curves.extend(collect_sum_surface_curves(entry)?);
                }
                return Ok(curves);
            }

            // Case 3: Mixed or curve types -> use segment grouping
            let segments = coerce::coerce_curve_segments(value)?;
            Ok(group_segments_into_polylines(segments))
        }
        Value::Surface { .. } => {
            let segments = coerce::coerce_curve_segments(value)?;
            Ok(group_segments_into_polylines(segments))
        }
        other => Err(ComponentError::new(format!(
            "Sum Surface kon invoer van type {} niet interpreteren als curve",
            other.kind()
        ))),
    }
}

fn collect_ruled_surface_curves(value: &Value) -> Result<Vec<Vec<[f64; 3]>>, ComponentError> {
    match value {
        Value::Null => Ok(Vec::new()),
        Value::CurveLine { p1, p2 } => Ok(vec![vec![*p1, *p2]]),
        Value::List(values) => {
            if values.is_empty() {
                return Ok(Vec::new());
            }

            if values.iter().all(|entry| matches!(entry, Value::Point(_))) {
                let polyline = values
                    .iter()
                    .filter_map(|entry| match entry {
                        Value::Point(point) => Some(*point),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                if polyline.len() < 2 {
                    return Ok(Vec::new());
                }
                return Ok(vec![polyline]);
            }

            if values
                .iter()
                .all(|entry| matches!(entry, Value::List(_) | Value::Null))
            {
                let mut curves = Vec::new();
                for entry in values {
                    curves.extend(collect_ruled_surface_curves(entry)?);
                }
                return Ok(curves);
            }

            let segments = coerce::coerce_curve_segments(value)?;
            Ok(group_segments_into_polylines(segments))
        }
        Value::Surface { .. } => {
            let segments = coerce::coerce_curve_segments(value)?;
            Ok(group_segments_into_polylines(segments))
        }
        other => Err(ComponentError::new(format!(
            "Ruled Surface kon invoer van type {} niet interpreteren als curve",
            other.kind()
        ))),
    }
}

fn group_segments_into_polylines(segments: Vec<([f64; 3], [f64; 3])>) -> Vec<Vec<[f64; 3]>> {
    if segments.is_empty() {
        return Vec::new();
    }

    // Maak een graaf van verbonden segmenten zodat we een consistente volgorde krijgen
    // ongeacht de volgorde van de inputsegmenten.
    let mut nodes: Vec<[f64; 3]> = Vec::new();
    let mut adjacency: Vec<Vec<usize>> = Vec::new(); // edge indices per node
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut edge_used: Vec<bool> = Vec::new();

    fn find_or_insert_node(
        nodes: &mut Vec<[f64; 3]>,
        adjacency: &mut Vec<Vec<usize>>,
        p: [f64; 3],
    ) -> usize {
        if let Some((idx, _)) = nodes
            .iter()
            .enumerate()
            .find(|(_, existing)| points_equal(**existing, p))
        {
            idx
        } else {
            let idx = nodes.len();
            nodes.push(p);
            adjacency.push(Vec::new());
            idx
        }
    }

    for (start, end) in segments {
        let a = find_or_insert_node(&mut nodes, &mut adjacency, start);
        let b = find_or_insert_node(&mut nodes, &mut adjacency, end);
        let edge_idx = edges.len();
        edges.push((a, b));
        edge_used.push(false);
        adjacency[a].push(edge_idx);
        adjacency[b].push(edge_idx);
    }

    let mut polylines = Vec::new();

    // Greedy traversal to build each polyline from unvisited edges.
    while let Some((edge_idx, &(start, end))) = edge_used
        .iter()
        .enumerate()
        .find(|(_, used)| !**used)
        .and_then(|(i, _)| edges.get(i).map(|edge| (i, edge)))
    {
        edge_used[edge_idx] = true;

        // Kies een startnode die een open einde heeft indien beschikbaar.
        let start_node = if adjacency[start].len() == 1 {
            start
        } else if adjacency[end].len() == 1 {
            end
        } else {
            start
        };
        let mut current_node = if start_node == start { end } else { start };
        let mut prev_node = start_node;

        let mut polyline = vec![nodes[start_node], nodes[current_node]];

        loop {
            // Zoek een onbenutte edge vanaf current_node.
            let next_edge_idx = adjacency[current_node]
                .iter()
                .copied()
                .find(|&idx| !edge_used[idx] && {
                    let (a, b) = edges[idx];
                    // Vermijd direct teruggaan over dezelfde edge; kies andere richting indien mogelijk.
                    let other = if a == current_node { b } else { a };
                    !points_equal(nodes[other], nodes[prev_node])
                })
                .or_else(|| {
                    adjacency[current_node]
                        .iter()
                        .copied()
                        .find(|&idx| !edge_used[idx])
                });

            let Some(next_idx) = next_edge_idx else {
                break;
            };

            edge_used[next_idx] = true;
            let (a, b) = edges[next_idx];
            let next_node = if a == current_node { b } else { a };
            prev_node = current_node;
            current_node = next_node;

            if !points_equal(*polyline.last().unwrap(), nodes[current_node]) {
                polyline.push(nodes[current_node]);
            }
        }

        // Sluit de polyline als het een echte gesloten lus is.
        if polyline.len() > 2 && !points_equal(polyline[0], *polyline.last().unwrap()) {
            let all_degree_two = polyline.iter().all(|point| {
                nodes
                    .iter()
                    .position(|p| points_equal(*p, *point))
                    .map(|idx| adjacency[idx].len() == 2)
                    .unwrap_or(false)
            });
            if all_degree_two {
                polyline.push(polyline[0]);
            }
        }

        if polyline.len() >= 2 {
            polylines.push(polyline);
        }
    }

    polylines
}

fn points_equal(a: [f64; 3], b: [f64; 3]) -> bool {
    a.iter()
        .zip(b.iter())
        .all(|(ax, bx)| (ax - bx).abs() <= EPSILON)
}

fn evaluate_ruled_surface(inputs: &[Value], meta: &MetaMap) -> ComponentResult {
    let component = "Ruled Surface";
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Ruled Surface vereist twee invoercurves",
        ));
    }
    let curves_a = collect_ruled_surface_curves(&inputs[0])?;
    let curves_b = collect_ruled_surface_curves(&inputs[1])?;

    if curves_a.is_empty() || curves_b.is_empty() {
        return Err(ComponentError::new(
            "Ruled Surface kon geen volledige curves interpreteren",
        ));
    }

    let target_count = match (curves_a.len(), curves_b.len()) {
        (1, b) => b,
        (a, 1) => a,
        (a, b) => a.min(b),
    };

    // Track successful results and failures separately for proper error reporting
    struct RuledSurfaceResult {
        mesh: crate::geom::GeomMesh,
        diagnostics: crate::geom::GeomMeshDiagnostics,
    }
    struct SkippedPair {
        index: usize,
        reason: String,
    }

    let mut successes: Vec<RuledSurfaceResult> = Vec::new();
    let mut skipped: Vec<SkippedPair> = Vec::new();

    // Extract quality from meta, falling back to default if not specified
    let quality = coerce::geom_bridge::surface_builder_quality_from_meta_optional(meta)
        .unwrap_or_default();

    for idx in 0..target_count {
        let polyline_a = if curves_a.len() == 1 {
            &curves_a[0]
        } else {
            &curves_a[idx]
        };
        let polyline_b = if curves_b.len() == 1 {
            &curves_b[0]
        } else {
            &curves_b[idx]
        };

        // Validate curve lengths and report specific issues
        let len_a = polyline_a.len();
        let len_b = polyline_b.len();
        if len_a < 2 || len_b < 2 {
            let reason = match (len_a < 2, len_b < 2) {
                (true, true) => format!(
                    "both curves have insufficient points (curve A: {len_a}, curve B: {len_b}, minimum: 2)"
                ),
                (true, false) => format!(
                    "curve A has insufficient points ({len_a}, minimum: 2)"
                ),
                (false, true) => format!(
                    "curve B has insufficient points ({len_b}, minimum: 2)"
                ),
                (false, false) => unreachable!(),
            };
            log::warn!("{component}: skipping curve pair {idx}: {reason}");
            skipped.push(SkippedPair { index: idx, reason });
            continue;
        }

        // Convert to geom Point3 format
        let geom_a: Vec<GeomPoint3> = polyline_a.iter().copied().map(to_geom_point).collect();
        let geom_b: Vec<GeomPoint3> = polyline_b.iter().copied().map(to_geom_point).collect();

        // Use the geom ruled surface builder (handles resampling internally)
        match mesh_ruled_surface(&geom_a, &geom_b, quality) {
            Ok((mesh, diagnostics)) => {
                successes.push(RuledSurfaceResult { mesh, diagnostics });
            }
            Err(e) => {
                let reason = e.to_string();
                log::warn!("{component}: skipping curve pair {idx}: {reason}");
                skipped.push(SkippedPair { index: idx, reason });
            }
        }
    }

    // Handle complete failure: return an error with details, not silent Null
    if successes.is_empty() {
        let error_details = if skipped.len() == 1 {
            format!(
                "failed to create ruled surface: {}",
                skipped[0].reason
            )
        } else {
            let reasons: Vec<String> = skipped
                .iter()
                .map(|s| format!("pair {}: {}", s.index, s.reason))
                .collect();
            format!(
                "failed to create ruled surface for all {} curve pairs:\n  - {}",
                skipped.len(),
                reasons.join("\n  - ")
            )
        };
        return Err(ComponentError::new(format!("{component}: {error_details}")));
    }

    // Build output values with merged diagnostics that include warnings about skipped pairs
    let mut mesh_values = Vec::with_capacity(successes.len());
    let mut surface_values = Vec::with_capacity(successes.len());

    for (result_idx, result) in successes.into_iter().enumerate() {
        let mut diagnostics = result.diagnostics;

        // For the first result, add warnings about all skipped pairs (aggregate once)
        if result_idx == 0 && !skipped.is_empty() {
            for skip in &skipped {
                diagnostics.add_warning(format!(
                    "curve pair {} skipped: {}",
                    skip.index, skip.reason
                ));
            }
        }

        mesh_values.push(geom_mesh_to_value_mesh(result.mesh.clone(), Some(diagnostics)));
        surface_values.push(geom_mesh_to_value_surface(result.mesh));
    }

    // Return output matching the count pattern
    let (surface_output, mesh_output) = match (surface_values.len(), mesh_values.len()) {
        (1, 1) => (
            surface_values.into_iter().next().unwrap(),
            mesh_values.into_iter().next().unwrap(),
        ),
        _ => (Value::List(surface_values), Value::List(mesh_values)),
    };

    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_SURFACE.to_string(), surface_output);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_output);
    Ok(out)
}

/// Evaluate Network Surface component using the geom::surface module.
///
/// Network Surface creates a surface that interpolates through a network of
/// intersecting U and V curves. This is commonly used for complex freeform surfaces
/// where control over both directions is needed.
///
/// # Inputs
/// - `inputs[0]`: U-curves (curves running in the U direction)
/// - `inputs[1]`: V-curves (curves running in the V direction)
/// - `inputs[2]` (optional): Continuity (0=Position/G0, 1=Tangent/G1, 2=Curvature/G2)
///
/// # Continuity Parameter
///
/// The Continuity parameter controls the surface blending at curve intersections:
/// - **0 (Position/G0)**: The surface passes through the intersection points. This is
///   the default and currently the only implemented mode.
/// - **1 (Tangent/G1)**: The surface would match tangent directions at intersections.
///   Not yet implemented - a warning is emitted if requested.
/// - **2 (Curvature/G2)**: The surface would match curvature at intersections.
///   Not yet implemented - a warning is emitted if requested.
///
/// When continuity > 0 is requested but not implemented, the surface is still
/// created with G0 continuity and a diagnostic warning is added.
///
/// # Outputs
/// - `S`: Surface output (legacy `Value::Surface` for backward compatibility)
/// - `M`: Mesh output (`Value::Mesh` for new consumers)
fn evaluate_network_surface(inputs: &[Value], meta: &MetaMap) -> ComponentResult {
    let component = "Network Surface";
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Network Surface vereist lijsten met U- en V-curves",
        ));
    }

    // Parse U-curves (first input)
    let u_segments = coerce::coerce_curve_segments(&inputs[0])?;
    let u_polylines_raw = group_segments_into_polylines(u_segments);
    let u_curves: Vec<Vec<GeomPoint3>> = u_polylines_raw
        .iter()
        .filter(|poly| poly.len() >= 2)
        .map(|poly| poly.iter().copied().map(to_geom_point).collect())
        .collect();

    // Parse V-curves (second input)
    let v_segments = coerce::coerce_curve_segments(&inputs[1])?;
    let v_polylines_raw = group_segments_into_polylines(v_segments);
    let v_curves: Vec<Vec<GeomPoint3>> = v_polylines_raw
        .iter()
        .filter(|poly| poly.len() >= 2)
        .map(|poly| poly.iter().copied().map(to_geom_point).collect())
        .collect();

    if u_curves.is_empty() || v_curves.is_empty() {
        return Err(ComponentError::new(
            "Network Surface vereist meerdere snijdende curves (minstens één U- en één V-curve)",
        ));
    }

    // Parse optional Continuity parameter (default: 0 = Position/G0)
    // Grasshopper convention: 0=Position, 1=Tangent, 2=Curvature
    let continuity = coerce_optional_number_with_default(
        inputs.get(2),
        0.0,
        component,
        "Continuity",
    )?;
    let continuity_level = continuity.round() as i32;

    // Track warnings for unimplemented continuity levels
    let mut extra_warnings: Vec<String> = Vec::new();
    if continuity_level > 0 {
        let continuity_name = match continuity_level {
            1 => "Tangent (G1)",
            2 => "Curvature (G2)",
            _ => "higher-order",
        };
        extra_warnings.push(format!(
            "{} continuity is not yet implemented; surface will be created with Position (G0) continuity instead",
            continuity_name
        ));
    }

    // Extract quality from meta, falling back to default if not specified
    let quality = coerce::geom_bridge::surface_builder_quality_from_meta_optional(meta)
        .unwrap_or_default();
    let (mesh, mut diagnostics) = mesh_network_surface(&u_curves, &v_curves, quality)
        .map_err(|e| ComponentError::new(format!("{component}: {e}")))?;

    // Add any extra warnings (e.g., continuity not implemented)
    diagnostics.warnings.extend(extra_warnings);

    // Convert to output values
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    // Return both mesh and surface outputs for compatibility
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_SURFACE.to_string(), surface_value);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_value);
    Ok(out)
}

/// Evaluate Sweep2 component using the geom::sweep module.
///
/// Sweep2 sweeps a profile curve along two rails. The rails define both the path
/// and the orientation of the swept profile.
///
/// # Inputs
/// - `inputs[0]`: Rail A (primary path curve)
/// - `inputs[1]`: Rail B (orientation/secondary rail)
/// - `inputs[2]`: Section/profile curve(s) - now supports multiple sections
/// - `inputs[3]` (optional): Same Height boolean
///
/// # Same Height Parameter
///
/// When enabled, Same Height would scale sections to maintain equal heights along the
/// sweep. Currently this parameter is parsed but not implemented - if enabled, a
/// diagnostic warning is emitted. This matches Grasshopper behavior where Same Height
/// is a specialized feature for architectural/structural workflows.
///
/// # Multi-Section Support
///
/// Unlike the original implementation that only used the first section, this version
/// supports multiple section curves. When multiple sections are provided:
/// - Sections are distributed evenly along the rails by default.
/// - The profile shape is interpolated between sections for a smooth transition.
///
/// # Rail Alignment
///
/// The implementation now properly aligns rails before sweeping:
/// - Rails are checked for compatible directions (start-to-start correspondence).
/// - If rails run in opposite directions, one is automatically reversed.
/// - Both rails are resampled using arc-length parameterization for consistent correspondence.
/// - Diagnostics warn when corrections are applied.
///
/// # Outputs
/// - `S`: Surface output (legacy `Value::Surface` for backward compatibility)
/// - `M`: Mesh output (`Value::Mesh` for new consumers)
fn evaluate_sweep_two(inputs: &[Value]) -> ComponentResult {
    let component = "Sweep2";
    if inputs.len() < 3 {
        return Err(ComponentError::new("Sweep2 vereist twee rails en secties"));
    }

    // Parse Rail A (primary rail)
    let rail_a_segments = coerce::coerce_curve_segments(&inputs[0])?;
    if rail_a_segments.is_empty() {
        return Err(ComponentError::new(
            "Sweep2 kon rail A niet lezen of rail A is leeg",
        ));
    }

    // Parse Rail B (secondary rail for orientation)
    let rail_b_segments = coerce::coerce_curve_segments(&inputs[1])?;
    if rail_b_segments.is_empty() {
        return Err(ComponentError::new(
            "Sweep2 kon rail B niet lezen of rail B is leeg",
        ));
    }

    // Parse optional Same Height parameter (Value::Null means unconnected pin -> use default)
    // Note: This is parsed but not fully implemented. If enabled, a warning is added
    // to diagnostics. Same Height would scale sections to maintain equal heights,
    // which is a specialized feature for architectural/structural workflows.
    let same_height = coerce_optional_bool_with_default(
        inputs.get(3),
        false, // Default: disabled
        component,
        "Same Height",
    )?;

    // Track warnings for the same_height parameter
    let mut extra_warnings: Vec<String> = Vec::new();
    if same_height {
        extra_warnings.push(
            "Same Height parameter is not yet implemented; section heights will vary naturally along the rails".to_string()
        );
    }

    // Convert rails to polylines
    let rail_a_polylines = group_segments_into_polylines(rail_a_segments);
    let rail_b_polylines = group_segments_into_polylines(rail_b_segments);

    // Use the longest polyline from each rail
    let rail_a = pick_longest_polyline(rail_a_polylines).ok_or_else(|| {
        ComponentError::new("Sweep2 kon geen geldige rail A polyline vormen")
    })?;
    let rail_b = pick_longest_polyline(rail_b_polylines).ok_or_else(|| {
        ComponentError::new("Sweep2 kon geen geldige rail B polyline vormen")
    })?;

    if rail_a.len() < 2 {
        return Err(ComponentError::new(
            "Sweep2 vereist een rail A met minstens twee punten",
        ));
    }
    if rail_b.len() < 2 {
        return Err(ComponentError::new(
            "Sweep2 vereist een rail B met minstens twee punten",
        ));
    }

    // Parse section curves (profiles) - now supports multiple sections
    let section_polylines = collect_ruled_surface_curves(&inputs[2])?;
    if section_polylines.is_empty() {
        return Err(ComponentError::new(
            "Sweep2 vereist minstens één sectiecurve",
        ));
    }

    // Validate all sections have at least 2 points
    for (i, section) in section_polylines.iter().enumerate() {
        if section.len() < 2 {
            return Err(ComponentError::new(format!(
                "Sweep2 sectie {} heeft minstens twee punten nodig (heeft {})",
                i + 1,
                section.len()
            )));
        }
    }

    let tol = Tolerance::default();

    // Convert rails to geom types
    let geom_rail_a = points_to_geom_polyline(&rail_a);
    let geom_rail_b = points_to_geom_polyline(&rail_b);

    // Align rails using the new alignment function
    // This handles:
    // - Direction detection and reversal if needed
    // - Arc-length resampling for consistent parameterization
    let target_count = rail_a.len().max(rail_b.len()).max(32); // Ensure smooth sampling
    let alignment = align_sweep2_rails(&geom_rail_a, &geom_rail_b, target_count, tol)
        .map_err(|e| sweep_error_to_component_error(e, component))?;

    // Determine if rail is closed (no caps for closed rails)
    let rail_closed = alignment.rail_a.len() >= 3
        && tol.approx_eq_point3(
            alignment.rail_a[0],
            *alignment.rail_a.last().unwrap(),
        );

    // Determine profile closure from first section
    let first_profile_closed = is_closed(&section_polylines[0]);

    // Set caps based on profile and rail closure
    let caps = if rail_closed {
        SweepCaps::NONE
    } else if first_profile_closed {
        SweepCaps::BOTH
    } else {
        SweepCaps::NONE
    };

    // Build sections for multi-section sweep
    // Each section is converted to geom format and positioned along the rails
    let sections: Vec<Sweep2Section> = if section_polylines.len() == 1 {
        // Single section: just use it directly
        let profile_points: Vec<Point3> = section_polylines[0]
            .iter()
            .map(|p| to_geom_point(*p))
            .collect();
        vec![Sweep2Section::auto(profile_points)]
    } else {
        // Multiple sections: distribute evenly along the rails
        // The auto_distribute flag in options will handle parameter assignment
        section_polylines
            .iter()
            .enumerate()
            .map(|(i, polyline)| {
                let profile_points: Vec<Point3> =
                    polyline.iter().map(|p| to_geom_point(*p)).collect();
                
                // Compute parameter position based on index
                let param = if section_polylines.len() > 1 {
                    i as f64 / (section_polylines.len() - 1) as f64
                } else {
                    0.5
                };
                
                Sweep2Section::at_parameter(profile_points, param)
            })
            .collect()
    };

    let options = Sweep2MultiSectionOptions {
        sweep: GeomSweepOptions::default(),
        arc_length_params: true,
        auto_distribute_sections: true,
    };

    // Call the multi-section sweep2
    let (mesh, mut diagnostics) = sweep2_multi_section(
        &sections,
        &alignment.rail_a,
        &alignment.rail_b,
        caps,
        options,
        tol,
    )
    .map_err(|e| sweep_error_to_component_error(e, component))?;

    // Add any extra warnings (e.g., Same Height not implemented)
    diagnostics.warnings.extend(extra_warnings);

    // Convert to output values
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    // Return both mesh and surface outputs for compatibility
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_SURFACE.to_string(), Value::List(vec![surface_value]));
    out.insert(PIN_OUTPUT_MESH.to_string(), Value::List(vec![mesh_value]));
    Ok(out)
}

/// Evaluates the Pipe Variable component using `geom::pipe::pipe_variable_polyline_with_tolerance`.
///
/// Pipe Variable creates a tube mesh with varying radius along a rail curve.
///
/// # Inputs
/// - `inputs[0]`: Rail curve (the path along which to create the pipe)
/// - `inputs[1]`: Parameters (list of parameter values along the rail, normalized 0-1 or arc-length based)
/// - `inputs[2]`: Radii (list of radius values corresponding to each parameter)
/// - `inputs[3]` (optional): Caps setting (0=None, 1=Flat, 2=Round)
///
/// # Outputs
/// - `P`: Pipe surface/mesh output
/// - `M`: Mesh output (`Value::Mesh` for new consumers)
fn evaluate_pipe_variable(inputs: &[Value]) -> ComponentResult {
    let component = "Pipe Variable";
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Pipe Variable vereist een curve, parameters en radii",
        ));
    }

    // Parse the rail curve
    let segments = coerce::coerce_curve_segments(&inputs[0])?;
    if segments.is_empty() {
        return Err(ComponentError::new(
            "Pipe Variable kon de railcurve niet lezen",
        ));
    }

    // Parse parameters and radii lists
    let raw_parameters = coerce_number_list(&inputs[1], component, "Parameters")?;
    let raw_radii = coerce_number_list(&inputs[2], component, "Radii")?;
    if raw_radii.is_empty() {
        return Err(ComponentError::new(
            "Pipe Variable vereist minstens één straal",
        ));
    }

    // Apply Grasshopper-style list matching: extend shorter list by repeating last element.
    // This preserves common Grasshopper usage patterns:
    // - Single radius value broadcasts to all parameter positions
    // - Shorter list's last value repeats for remaining positions
    let (parameters, radii) = match_list_lengths_grasshopper(raw_parameters, raw_radii);

    // Parse optional caps setting (Value::Null means unconnected pin -> use default)
    let caps = {
        let caps_value = coerce_optional_number_with_default(
            inputs.get(3),
            0.0, // Default: no caps
            component,
            "Caps",
        )?;
        parse_pipe_caps_from_number(caps_value)
    };

    // Convert segments to a continuous polyline
    let rail_polylines = group_segments_into_polylines(segments);
    let rail_polyline = pick_longest_polyline(rail_polylines).ok_or_else(|| {
        ComponentError::new("Pipe Variable kon de railcurve niet samenvoegen")
    })?;

    if rail_polyline.len() < 2 {
        return Err(ComponentError::new(
            "Pipe Variable vereist een rail met minstens twee punten",
        ));
    }

    // Convert to geom types
    let geom_rail = points_to_geom_polyline(&rail_polyline);
    let tol = Tolerance::default();
    let options = GeomPipeOptions::default();

    // Determine if the rail is closed (no caps allowed for closed rails)
    let rail_closed = is_closed(&rail_polyline);
    let effective_caps = if rail_closed {
        PipeCaps::NONE
    } else {
        caps
    };

    // Call the geom pipe variable function
    let (mesh, diagnostics) = pipe_variable_polyline_with_tolerance(
        &geom_rail,
        &parameters,
        &radii,
        effective_caps,
        options,
        tol,
    )
    .map_err(|e| pipe_error_to_component_error(e, component))?;

    // Convert to output values
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    // Return both mesh and surface outputs for compatibility
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_PIPE.to_string(), Value::List(vec![surface_value]));
    out.insert(PIN_OUTPUT_MESH.to_string(), Value::List(vec![mesh_value]));
    Ok(out)
}

fn evaluate_extrude_linear(inputs: &[Value]) -> ComponentResult {
    let component = "Extrude Linear";
    let profile_segments = coerce::coerce_curve_segments(inputs.get(0).unwrap_or(&Value::Null))?;
    if profile_segments.is_empty() {
        return Err(ComponentError::new(
            "Extrude Linear kon geen profielcurve herkennen",
        ));
    }

    // Inputs according to Grasshopper ExtrudeLinear:
    // inputs[0]: Profile (P) - the curve or surface to extrude
    // inputs[1]: Orientation (Po) - optional orientation plane for the profile
    // inputs[2]: Axis (A) - extrusion axis vector (defines direction AND distance)
    // inputs[3]: Orientation (Ao) - optional axis orientation plane
    //
    // The Axis input determines both the direction and the distance of extrusion.
    // When a number is provided instead of a vector, we interpret it as a distance
    // along the Z-axis (or the orientation direction if provided).

    let axis_direction = parse_extrude_linear_axis(
        inputs.get(2),
        inputs.get(1),
        component,
    )?;

    if is_zero_vector(axis_direction) {
        return Err(ComponentError::new(
            "Extrude Linear vereist een as met lengte",
        ));
    }

    // Convert segments to a single polyline for the geom extrusion
    let profile = segments_to_geom_polyline(&profile_segments);
    let geom_direction = to_geom_vec(axis_direction);
    let tol = Tolerance::default();

    // Determine if the profile is closed (first and last points coincide)
    let is_profile_closed = profile.len() >= 3
        && tol.approx_eq_point3(profile[0], profile[profile.len() - 1]);

    // Use caps for closed profiles
    let caps = if is_profile_closed {
        ExtrusionCaps::BOTH
    } else {
        ExtrusionCaps::NONE
    };

    // Call the geom extrusion function
    let (mesh, diagnostics) = extrude_polyline_with_tolerance(&profile, geom_direction, caps, tol)
        .map_err(|e| extrusion_error_to_component_error(e, component))?;

    // Output as Value::Mesh (primary) and also provide legacy Value::Surface
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    // Return both mesh and surface outputs
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_EXTRUSION.to_string(), surface_value);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_value);
    Ok(out)
}

fn evaluate_extrude_angled(inputs: &[Value]) -> ComponentResult {
    let component = "Extrude Angled";
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Extrude Angled vereist een polyline en twee hoogtes",
        ));
    }

    let base_points = collect_points(&inputs[0], component)?;
    if base_points.len() < 2 {
        return Err(ComponentError::new(
            "Extrude Angled verwacht minstens twee punten voor de polyline",
        ));
    }
    let base_height = coerce_number(&inputs[1], component, "Base height")?;
    let top_height = coerce_number(&inputs[2], component, "Top height")?;

    // Parse optional angles list
    let angles: Vec<f64> = if let Some(value) = inputs.get(3) {
        coerce_number_list(value, component, "Angles").unwrap_or_default()
    } else {
        Vec::new()
    };

    // Convert points to GeomPoint3 for the geom extrusion
    let profile = points_to_geom_polyline(&base_points);
    let tol = Tolerance::default();

    // Call the geom angled extrusion function
    let (mesh, diagnostics) = extrude_angled_polyline_with_tolerance(
        &profile,
        base_height,
        top_height,
        &angles,
        tol,
    )
    .map_err(|e| extrusion_error_to_component_error(e, component))?;

    // Output as Value::Mesh (primary) and also provide legacy Value::Surface
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    // Return both mesh and surface outputs
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_SHAPE.to_string(), surface_value);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_value);
    Ok(out)
}

/// Evaluate Sweep1 component using the geom::sweep module.
///
/// Sweep1 sweeps a profile curve along a single rail using rotation-minimizing frames.
/// This produces a smooth sweep without unexpected twisting.
///
/// # Inputs
/// - `inputs[0]`: Rail curve (the path to sweep along)
/// - `inputs[1]`: Section/profile curve(s) or surface(s)
/// - `inputs[2]` (optional): Miter parameter (0=None, 1=Trim, 2=Rotate)
///
/// # Miter Handling
/// Controls how sharp corners (kinks) in the rail are handled:
/// - **0 (None)**: Standard sweep through corners. May cause self-intersection at sharp bends.
/// - **1 (Trim)**: Uses bisector direction at corners for mitered joints.
/// - **2 (Rotate)**: Smooth rotation through corners using bisector orientation.
///
/// # Outputs
/// - `S`: Surface output (legacy `Value::Surface` for backward compatibility)
/// - `M`: Mesh output (`Value::Mesh` for new consumers)
///
/// # Section handling
/// - If the section is a surface, uses `sweep_surface_along_polyline` (legacy behavior)
/// - If the section is a single closed polyline, uses `geom::sweep1_polyline_with_tolerance`
/// - If the section is a single open polyline, uses `geom::sweep1_polyline_with_tolerance`
/// - If multiple section curves are provided, lofts them first then sweeps
fn evaluate_sweep_one(inputs: &[Value], meta: &MetaMap) -> ComponentResult {
    let component = "Sweep1";
    if inputs.len() < 2 {
        return Err(ComponentError::new("Sweep1 vereist een rail en secties"));
    }

    let rail_segments = coerce::coerce_curve_segments(&inputs[0])?;
    let rail_polyline = pick_longest_polyline(group_segments_into_polylines(rail_segments))
        .ok_or_else(|| ComponentError::new("Sweep1 kon de railcurve niet lezen"))?;
    if rail_polyline.len() < 2 {
        return Err(ComponentError::new("Sweep1 vereist een rail met lengte"));
    }

    // Parse Miter parameter: 0=None, 1=Trim, 2=Rotate (Value::Null means unconnected -> use default)
    let miter_type = {
        let miter_int = coerce_optional_number_with_default(
            inputs.get(2),
            0.0, // Default: MiterType::None
            component,
            "Miter",
        )? as i32;
        MiterType::from_int(miter_int)
    };

    // Check for surface inputs (legacy path)
    let mut section_surfaces = Vec::new();
    collect_surfaces_recursive(&inputs[1], &mut section_surfaces)?;

    if !section_surfaces.is_empty() {
        // Legacy surface sweep path (miter not applicable to surface sweeps)
        let mut sweeps = Vec::new();
        let mut mesh_outputs = Vec::new();
        for surface in section_surfaces {
            let solid = sweep_surface_along_polyline(surface, &rail_polyline, component, true)?;
            // For legacy surface sweeps, also create a mesh output
            if let Value::Surface { ref vertices, ref faces } = solid {
                let mesh_val = legacy_surface_to_mesh_value(vertices, faces);
                mesh_outputs.push(mesh_val);
            }
            sweeps.push(solid);
        }
        let mut out = BTreeMap::new();
        out.insert(PIN_OUTPUT_SURFACE.to_string(), Value::List(sweeps));
        out.insert(PIN_OUTPUT_MESH.to_string(), Value::List(mesh_outputs));
        return Ok(out);
    }

    let multi_source = input_source_count(meta, 1) >= 2;
    let branch_values = collect_loft_branch_values(&inputs[1], multi_source);
    let mut sweep_surfaces = Vec::new();
    let mut sweep_meshes = Vec::new();
    let mut found_sections = false;

    let tol = Tolerance::default();

    for branch in branch_values {
        let mut sections = collect_ruled_surface_curves(&branch)?;
        if sections.is_empty() {
            continue;
        }
        found_sections = true;
        unify_curve_directions(&mut sections);

        // Determine sweep result based on section configuration
        let (surface_value, mesh_value) = if sections.len() == 1 {
            // Single profile case - use geom::sweep1 with miter handling
            sweep_single_profile_geom(&sections[0], &rail_polyline, miter_type, tol, component)?
        } else {
            // Multiple sections - orient each section along the rail, then loft
            let oriented_sections = orient_sections_along_rail(&sections, &rail_polyline, component)?;
            let (surface, mesh_val) = build_loft_surface_geom(
                oriented_sections,
                GeomLoftOptions::default(),
                tol,
                component,
            )?;
            (surface, mesh_val)
        };

        sweep_surfaces.push(surface_value);
        sweep_meshes.push(mesh_value);
    }

    if !found_sections {
        return Err(ComponentError::new(
            "Sweep1 verwacht minstens één sectiepolyline",
        ));
    }

    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_SURFACE.to_string(), Value::List(sweep_surfaces));
    out.insert(PIN_OUTPUT_MESH.to_string(), Value::List(sweep_meshes));
    Ok(out)
}

/// Sweeps a single profile polyline along a rail using `geom::sweep1_polyline_with_tolerance`.
///
/// This helper handles the conversion between component types and geom types,
/// and properly sets up caps based on profile closure.
///
/// # Arguments
/// * `profile` - The profile polyline points to sweep
/// * `rail` - The rail polyline points to sweep along
/// * `miter` - How to handle sharp corners in the rail
/// * `tol` - Geometric tolerance for comparisons
/// * `component` - Component name for error messages
fn sweep_single_profile_geom(
    profile: &[[f64; 3]],
    rail: &[[f64; 3]],
    miter: MiterType,
    tol: Tolerance,
    component: &str,
) -> Result<(Value, Value), ComponentError> {
    // Prepare profile - remove duplicate closing point if exists
    let mut profile_points = profile.to_vec();
    let profile_closed = is_polyline_closed_with_tolerance(&profile_points, tol);
    
    if profile_closed && profile_points.len() > 2 {
        profile_points.pop(); // Remove duplicate closing point for clean processing
    }

    // The profile needs to be centered at the rail start for proper sweep behavior.
    // Calculate profile centroid and translate profile to origin.
    let profile_centroid = if !profile_points.is_empty() {
        let n = profile_points.len() as f64;
        let sum = profile_points.iter().fold([0.0; 3], |acc, p| {
            [acc[0] + p[0], acc[1] + p[1], acc[2] + p[2]]
        });
        [sum[0] / n, sum[1] / n, sum[2] / n]
    } else {
        [0.0, 0.0, 0.0]
    };

    // Translate profile to be centered at origin (geom sweep expects this)
    let centered_profile: Vec<[f64; 3]> = profile_points
        .iter()
        .map(|p| [
            p[0] - profile_centroid[0],
            p[1] - profile_centroid[1],
            p[2] - profile_centroid[2],
        ])
        .collect();

    // Translate rail so it starts at the profile's original centroid
    let rail_start = rail.first().copied().unwrap_or([0.0; 3]);
    let translation = [
        profile_centroid[0] - rail_start[0],
        profile_centroid[1] - rail_start[1],
        profile_centroid[2] - rail_start[2],
    ];
    let translated_rail: Vec<[f64; 3]> = rail
        .iter()
        .map(|p| [p[0] + translation[0], p[1] + translation[1], p[2] + translation[2]])
        .collect();

    // Convert to geom types
    let geom_profile = points_to_geom_polyline(&centered_profile);
    let geom_rail = points_to_geom_polyline(&translated_rail);

    // Determine if rail is closed using tolerance-aware comparison
    let rail_closed = is_polyline_closed_with_tolerance(rail, tol);

    // Set caps based on closure
    let caps = if rail_closed {
        SweepCaps::NONE
    } else if profile_closed {
        SweepCaps::BOTH
    } else {
        SweepCaps::NONE
    };

    // Build sweep options with miter type
    let options = GeomSweepOptions {
        twist_radians_total: 0.0,
        miter,
    };

    // Call geom sweep1
    let (mesh, diagnostics) = sweep1_polyline_with_tolerance(
        &geom_profile,
        &geom_rail,
        caps,
        options,
        tol,
    )
    .map_err(|e| sweep_error_to_component_error(e, component))?;

    // Convert to output values
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    Ok((surface_value, mesh_value))
}

/// Checks if a polyline is closed using tolerance-aware point comparison.
///
/// A polyline is considered closed if it has at least 3 points and the first
/// and last points are within tolerance of each other.
fn is_polyline_closed_with_tolerance(polyline: &[[f64; 3]], tol: Tolerance) -> bool {
    if polyline.len() < 3 {
        return false;
    }
    let first = polyline.first().unwrap();
    let last = polyline.last().unwrap();
    
    let first_pt = Point3::new(first[0], first[1], first[2]);
    let last_pt = Point3::new(last[0], last[1], last[2]);
    
    tol.approx_eq_point3(first_pt, last_pt)
}

/// Orients multiple sections along a rail curve for multi-section sweep operations.
///
/// This function properly handles multi-section sweeps by:
/// 1. Finding each section's parameter along the rail based on its original position
/// 2. Computing rotation-minimizing (parallel transport) frames at those parameters
/// 3. Transforming sections while respecting the first section's plane orientation
///
/// This approach ensures consistent twist behavior that matches single-section sweeps
/// and respects the original placement of sections along the rail.
fn orient_sections_along_rail(
    sections: &[Vec<[f64; 3]>],
    rail_polyline: &[[f64; 3]],
    component: &str,
) -> Result<Vec<Vec<[f64; 3]>>, ComponentError> {
    if sections.is_empty() {
        return Ok(Vec::new());
    }

    if rail_polyline.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist een rail met minstens twee unieke punten",
        )));
    }

    let tol = Tolerance::default();

    // Convert rail to geom types for frame computation
    let geom_rail: Vec<Point3> = rail_polyline
        .iter()
        .map(|p| Point3::new(p[0], p[1], p[2]))
        .collect();

    // Compute rotation-minimizing frames along the entire rail.
    // We'll sample these at the parameters where sections are located.
    let rail_frames = compute_sweep_frames_for_rail(&geom_rail, tol);
    if rail_frames.is_empty() {
        return Err(ComponentError::new(format!(
            "{component} kon geen frames langs de rail berekenen",
        )));
    }

    // Find the parameter along the rail for each section based on its centroid.
    // If sections are placed at specific positions, respect those positions.
    // Otherwise, distribute them evenly (fallback for imported/legacy data).
    let section_params = compute_section_parameters_along_rail(sections, rail_polyline);

    // Compute the first section's plane transform for consistent orientation.
    let first_section_points: Vec<Point3> = sections[0]
        .iter()
        .map(|p| Point3::new(p[0], p[1], p[2]))
        .collect();
    let first_section_plane = ProfilePlaneTransform::from_points(&first_section_points, tol);

    // Sample frames at the section parameters and apply alignment
    let mut oriented_sections = Vec::with_capacity(sections.len());
    
    for (idx, (section, &param)) in sections.iter().zip(section_params.iter()).enumerate() {
        // Get the frame at this parameter by interpolating rail frames
        let frame = interpolate_rail_frame(&rail_frames, &geom_rail, param, tol);
        
        // For the first section, compute the rotation needed to align the sweep frame
        // with the section's original plane. Apply this same rotation to all subsequent frames.
        let aligned_frame = if idx == 0 {
            if let Some(ref plane) = first_section_plane {
                align_frame_to_section_plane(&frame, plane, &geom_rail, tol)
            } else {
                frame
            }
        } else {
            // Apply the same frame alignment computed for the first section
            // by using the aligned rail frames directly
            frame
        };

        // Transform the section to the rail position
        oriented_sections.push(apply_sweep_frame_to_section(section, &aligned_frame));
    }

    Ok(oriented_sections)
}

/// Computes rotation-minimizing frames along the rail for sweep operations.
/// Uses parallel transport to minimize twist, matching the behavior of sweep1.
fn compute_sweep_frames_for_rail(rail: &[Point3], tol: Tolerance) -> Vec<FrenetFrame> {
    if rail.len() < 2 {
        return Vec::new();
    }

    let mut frames = Vec::with_capacity(rail.len());

    // Compute initial tangent and frame
    let initial_tangent = rail[1].sub_point(rail[0]);
    let first_frame = match FrenetFrame::from_tangent(initial_tangent) {
        Some(f) => f,
        None => {
            // Fallback for degenerate tangent
            FrenetFrame::from_tangent(Vec3::new(0.0, 0.0, 1.0)).unwrap()
        }
    };
    frames.push(first_frame);

    // Use parallel transport for subsequent frames
    for i in 1..rail.len() {
        let prev_idx = i - 1;
        let next_idx = (i + 1).min(rail.len() - 1);

        // Compute tangent using central difference for interior points
        let tangent = if i < rail.len() - 1 {
            let forward = rail[next_idx].sub_point(rail[i]);
            let backward = rail[i].sub_point(rail[prev_idx]);
            forward.add(backward)
        } else {
            rail[i].sub_point(rail[prev_idx])
        };

        let tangent = tangent.normalized().unwrap_or(frames[prev_idx].tangent);

        // Parallel transport: rotate the previous frame's normal/binormal to the new tangent
        let prev_frame = &frames[prev_idx];
        let new_frame = parallel_transport_sweep_frame(prev_frame, tangent, tol);
        frames.push(new_frame);
    }

    frames
}

/// Parallel transport a frame to a new tangent direction.
fn parallel_transport_sweep_frame(prev_frame: &FrenetFrame, new_tangent: Vec3, tol: Tolerance) -> FrenetFrame {
    let old_tangent = prev_frame.tangent;
    let cross = old_tangent.cross(new_tangent);
    let cross_len_sq = cross.length_squared();

    if cross_len_sq < tol.eps_squared() {
        // Tangents are nearly parallel
        let dot = old_tangent.dot(new_tangent);
        if dot < 0.0 {
            // Tangent reversed - flip normal and binormal
            FrenetFrame {
                tangent: new_tangent,
                normal: prev_frame.normal.mul_scalar(-1.0),
                binormal: prev_frame.binormal.mul_scalar(-1.0),
            }
        } else {
            FrenetFrame {
                tangent: new_tangent,
                normal: prev_frame.normal,
                binormal: prev_frame.binormal,
            }
        }
    } else {
        // Rotate the frame around the axis perpendicular to both tangents
        let rotation_axis = cross.normalized().unwrap_or(Vec3::Z);
        let dot = old_tangent.dot(new_tangent).clamp(-1.0, 1.0);
        let angle = dot.acos();

        let new_normal = rotate_sweep_vector(prev_frame.normal, rotation_axis, angle)
            .normalized()
            .unwrap_or(prev_frame.normal);
        let new_binormal = new_tangent.cross(new_normal)
            .normalized()
            .unwrap_or(prev_frame.binormal);

        FrenetFrame {
            tangent: new_tangent,
            normal: new_normal,
            binormal: new_binormal,
        }
    }
}

/// Rotates a vector around an axis by the given angle (Rodrigues' rotation formula).
fn rotate_sweep_vector(v: Vec3, axis: Vec3, angle: f64) -> Vec3 {
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();

    let k_cross_v = axis.cross(v);
    let k_dot_v = axis.dot(v);

    v.mul_scalar(cos_angle)
        .add(k_cross_v.mul_scalar(sin_angle))
        .add(axis.mul_scalar(k_dot_v * (1.0 - cos_angle)))
}

/// Computes the parameter (0-1) along the rail for each section based on its position.
fn compute_section_parameters_along_rail(
    sections: &[Vec<[f64; 3]>],
    rail_polyline: &[[f64; 3]],
) -> Vec<f64> {
    if sections.is_empty() {
        return Vec::new();
    }

    // Compute rail arc lengths
    let mut arc_lengths = vec![0.0];
    let mut cumulative = 0.0;
    for i in 1..rail_polyline.len() {
        let dx = rail_polyline[i][0] - rail_polyline[i-1][0];
        let dy = rail_polyline[i][1] - rail_polyline[i-1][1];
        let dz = rail_polyline[i][2] - rail_polyline[i-1][2];
        cumulative += (dx*dx + dy*dy + dz*dz).sqrt();
        arc_lengths.push(cumulative);
    }
    let total_length = arc_lengths.last().copied().unwrap_or(1.0).max(1e-10);

    // For each section, find its closest point on the rail and compute the parameter
    let mut params = Vec::with_capacity(sections.len());
    for section in sections {
        let centroid = polyline_centroid(section);
        let closest_param = find_closest_point_on_polyline(&centroid, rail_polyline, &arc_lengths, total_length);
        params.push(closest_param);
    }

    // If sections are nearly coincident (all at the same parameter), distribute evenly
    let param_variance = if params.len() > 1 {
        let mean = params.iter().sum::<f64>() / params.len() as f64;
        params.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / params.len() as f64
    } else {
        0.0
    };

    if param_variance < 1e-6 {
        // Sections are clustered at the same location - distribute evenly
        let n = sections.len();
        (0..n).map(|i| i as f64 / (n - 1).max(1) as f64).collect()
    } else {
        // Sort and normalize parameters to ensure monotonic ordering
        let mut indexed_params: Vec<(usize, f64)> = params.iter().copied().enumerate().collect();
        indexed_params.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Rebuild in original order
        let mut result = vec![0.0; sections.len()];
        for (new_idx, (orig_idx, _param)) in indexed_params.iter().enumerate() {
            // Renormalize to ensure proper spacing
            result[*orig_idx] = new_idx as f64 / (sections.len() - 1).max(1) as f64;
        }
        result
    }
}

/// Finds the closest point parameter on a polyline to a given point.
fn find_closest_point_on_polyline(
    point: &[f64; 3],
    polyline: &[[f64; 3]],
    arc_lengths: &[f64],
    total_length: f64,
) -> f64 {
    let mut best_param = 0.0;
    let mut best_dist_sq = f64::MAX;

    for i in 0..polyline.len().saturating_sub(1) {
        let a = polyline[i];
        let b = polyline[i + 1];
        
        // Project point onto segment
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ap = [point[0] - a[0], point[1] - a[1], point[2] - a[2]];
        let ab_len_sq = ab[0]*ab[0] + ab[1]*ab[1] + ab[2]*ab[2];
        
        let t = if ab_len_sq > 1e-12 {
            let dot = ap[0]*ab[0] + ap[1]*ab[1] + ap[2]*ab[2];
            (dot / ab_len_sq).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Closest point on segment
        let closest = [
            a[0] + t * ab[0],
            a[1] + t * ab[1],
            a[2] + t * ab[2],
        ];
        let dx = point[0] - closest[0];
        let dy = point[1] - closest[1];
        let dz = point[2] - closest[2];
        let dist_sq = dx*dx + dy*dy + dz*dz;

        if dist_sq < best_dist_sq {
            best_dist_sq = dist_sq;
            // Convert to global parameter
            let segment_length = arc_lengths[i + 1] - arc_lengths[i];
            best_param = (arc_lengths[i] + t * segment_length) / total_length;
        }
    }

    best_param
}

/// Interpolates a frame at a given parameter along the rail.
fn interpolate_rail_frame(
    frames: &[FrenetFrame],
    rail: &[Point3],
    param: f64,
    _tol: Tolerance,
) -> FrenetFrame {
    if frames.is_empty() {
        return FrenetFrame::from_tangent(Vec3::Z).unwrap();
    }
    if frames.len() == 1 {
        return frames[0];
    }

    // Find the segment containing this parameter
    let n = frames.len();
    let scaled = param * (n - 1) as f64;
    let idx = (scaled.floor() as usize).min(n - 2);
    let t = scaled - idx as f64;

    let f0 = &frames[idx];
    let f1 = &frames[idx + 1];

    // Interpolate position from rail points (computed but may be used for origin in future)
    let _rail_n = rail.len();

    // Interpolate frame vectors using spherical linear interpolation (simplified)
    let tangent = f0.tangent.lerp(f1.tangent, t).normalized().unwrap_or(f0.tangent);
    let normal = f0.normal.lerp(f1.normal, t).normalized().unwrap_or(f0.normal);
    let binormal = tangent.cross(normal).normalized().unwrap_or(f0.binormal);
    let normal = binormal.cross(tangent).normalized().unwrap_or(normal);

    FrenetFrame {
        tangent,
        normal,
        binormal,
    }
}

/// Aligns a sweep frame to match a section's original plane orientation.
fn align_frame_to_section_plane(
    frame: &FrenetFrame,
    section_plane: &ProfilePlaneTransform,
    _rail: &[Point3],
    _tol: Tolerance,
) -> FrenetFrame {
    // Project the section's local_x axis onto the plane perpendicular to the frame's tangent
    let proj_local_x = section_plane.local_x
        .sub(frame.tangent.mul_scalar(section_plane.local_x.dot(frame.tangent)));
    
    let target_normal = match proj_local_x.normalized() {
        Some(n) => n,
        None => {
            // Section plane is parallel to rail - use local_y or fall back
            let proj_local_y = section_plane.local_y
                .sub(frame.tangent.mul_scalar(section_plane.local_y.dot(frame.tangent)));
            match proj_local_y.normalized() {
                Some(b) => frame.tangent.cross(b).normalized().unwrap_or(frame.normal),
                None => frame.normal,
            }
        }
    };

    let binormal = frame.tangent.cross(target_normal).normalized().unwrap_or(frame.binormal);

    FrenetFrame {
        tangent: frame.tangent,
        normal: target_normal,
        binormal,
    }
}

/// Applies a sweep frame to transform a section to its position along the rail.
fn apply_sweep_frame_to_section(section: &[[f64; 3]], frame: &FrenetFrame) -> Vec<[f64; 3]> {
    let centroid = polyline_centroid(section);

    // Sample rail point from the frame's origin (we need to interpolate this)
    // For now, use the centroid projected onto the rail plane defined by the frame
    
    section
        .iter()
        .map(|point| {
            // Transform point from local coordinates to world coordinates
            // local.x -> normal direction, local.y -> binormal direction
            let local = subtract_points(*point, centroid);
            [
                centroid[0] + frame.normal.x * local[0] + frame.binormal.x * local[1] + frame.tangent.x * local[2],
                centroid[1] + frame.normal.y * local[0] + frame.binormal.y * local[1] + frame.tangent.y * local[2],
                centroid[2] + frame.normal.z * local[0] + frame.binormal.z * local[1] + frame.tangent.z * local[2],
            ]
        })
        .collect()
}

/// Convert a legacy `Value::Surface` representation to `Value::Mesh`.
fn legacy_surface_to_mesh_value(vertices: &[[f64; 3]], faces: &[Vec<u32>]) -> Value {
    // Flatten faces into triangle indices
    let mut indices = Vec::new();
    for face in faces {
        if face.len() >= 3 {
            // Triangulate the face using fan triangulation
            for i in 1..(face.len() - 1) {
                indices.push(face[0]);
                indices.push(face[i] as u32);
                indices.push(face[i + 1] as u32);
            }
        }
    }

    Value::Mesh {
        vertices: vertices.to_vec(),
        indices,
        normals: None,
        uvs: None,
        diagnostics: None,
    }
}

fn evaluate_extrude_point(inputs: &[Value]) -> ComponentResult {
    let component = "Extrude Point";
    let base_segments = coerce::coerce_curve_segments(inputs.get(0).unwrap_or(&Value::Null))?;
    if base_segments.is_empty() {
        return Err(ComponentError::new(
            "Extrude Point kon de basiscurve niet lezen",
        ));
    }
    let tip = coerce::coerce_point_with_default(inputs.get(1));
    let tol = Tolerance::default();

    // Convert segments to a single polyline for the geom extrusion
    let profile = segments_to_geom_polyline(&base_segments);
    let geom_tip = to_geom_point(tip);

    // Determine if the profile is closed (for capping the base)
    let is_profile_closed = profile.len() >= 3
        && tol.approx_eq_point3(profile[0], profile[profile.len() - 1]);

    // Call the geom extrude_to_point function
    let (mesh, diagnostics) = extrude_to_point_with_tolerance(&profile, geom_tip, is_profile_closed, tol)
        .map_err(|e| extrusion_error_to_component_error(e, component))?;

    // Output as Value::Mesh (primary) and also provide legacy Value::Surface
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    // Return both mesh and surface outputs
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_EXTRUSION.to_string(), surface_value);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_value);
    Ok(out)
}

/// Evaluates the Pipe component using `geom::pipe::pipe_polyline_with_tolerance`.
///
/// Pipe creates a tube mesh with constant radius along a rail curve.
///
/// # Inputs
/// - `inputs[0]`: Rail curve (the path along which to create the pipe)
/// - `inputs[1]`: Radius (the constant radius of the pipe)
/// - `inputs[2]` (optional): Caps setting (0=None, 1=Flat, 2=Round)
///
/// # Outputs
/// - `P`: Pipe surface/mesh output
/// - `M`: Mesh output (`Value::Mesh` for new consumers)
fn evaluate_pipe(inputs: &[Value]) -> ComponentResult {
    let component = "Pipe";

    // Parse the rail curve
    let segments = coerce::coerce_curve_segments(inputs.get(0).unwrap_or(&Value::Null))?;
    if segments.is_empty() {
        return Err(ComponentError::new("Pipe kon de railcurve niet lezen"));
    }

    // Parse radius: default to 1.0 when unconnected, then take absolute value.
    // Uses coerce_optional_number_with_default so that Null/None -> 1.0 (not 0.0).
    let radius = coerce_optional_number_with_default(
        inputs.get(1),
        1.0, // Default radius when pin is unconnected
        component,
        "Radius",
    )?
    .abs();
    if radius <= 0.0 {
        // This can only happen if the user explicitly provides 0 (or negative which becomes 0 after abs)
        return Err(ComponentError::new(
            "Pipe vereist een positieve straal (waarde 0 is niet toegestaan)",
        ));
    }

    // Parse optional caps setting (Value::Null means unconnected pin -> use default)
    let caps = {
        let caps_value = coerce_optional_number_with_default(
            inputs.get(2),
            0.0, // Default: no caps
            component,
            "Caps",
        )?;
        parse_pipe_caps_from_number(caps_value)
    };

    // Convert segments to a continuous polyline
    let rail_polylines = group_segments_into_polylines(segments);
    let rail_polyline = pick_longest_polyline(rail_polylines).ok_or_else(|| {
        ComponentError::new("Pipe kon de railcurve niet samenvoegen")
    })?;

    if rail_polyline.len() < 2 {
        return Err(ComponentError::new(
            "Pipe vereist een rail met minstens twee punten",
        ));
    }

    // Convert to geom types
    let geom_rail = points_to_geom_polyline(&rail_polyline);
    let tol = Tolerance::default();
    let options = GeomPipeOptions::default();

    // Determine if the rail is closed (no caps allowed for closed rails)
    let rail_closed = is_closed(&rail_polyline);
    let effective_caps = if rail_closed {
        PipeCaps::NONE
    } else {
        caps
    };

    // Call the geom pipe function
    let (mesh, diagnostics) = pipe_polyline_with_tolerance(
        &geom_rail,
        radius,
        effective_caps,
        options,
        tol,
    )
    .map_err(|e| pipe_error_to_component_error(e, component))?;

    // Convert to output values
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    // Return both mesh and surface outputs for compatibility
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_PIPE.to_string(), Value::List(vec![surface_value]));
    out.insert(PIN_OUTPUT_MESH.to_string(), Value::List(vec![mesh_value]));
    Ok(out)
}

fn evaluate_four_point_surface(inputs: &[Value], meta: &MetaMap) -> ComponentResult {
    let component = "4Point Surface";

    // Collect only *actually provided* corner points (up to 4).
    // Using a helper that returns None for missing/null inputs instead of a default.
    let mut points: Vec<GeomPoint3> = Vec::with_capacity(4);
    for index in 0..4 {
        if let Some(pt) = try_coerce_optional_point(inputs.get(index)) {
            points.push(to_geom_point(pt));
        }
    }

    // Validate that we have at least 3 *actually provided* points.
    if points.len() < 3 {
        return Err(ComponentError::new(format!(
            "{component} vereist minimaal 3 hoekpunten, maar slechts {} ontvangen",
            points.len()
        )));
    }

    // Validate points are not all coincident (would produce a degenerate surface).
    if are_points_coincident(&points, Tolerance::default_geom()) {
        return Err(ComponentError::new(format!(
            "{component}: alle punten zijn samenvallend (coincident) — \
             kan geen geldig oppervlak maken"
        )));
    }

    // For 3 points, the fourth is inferred as a parallelogram completion.
    // Check that the 3 provided points are not collinear (would produce a degenerate patch).
    if points.len() == 3 && are_points_collinear(&points, Tolerance::default_geom()) {
        return Err(ComponentError::new(format!(
            "{component}: de 3 hoekpunten liggen op één lijn (collineair) — \
             kan geen geldig oppervlak maken; geef een niet-collineair vierde punt"
        )));
    }

    // For 4 points, check if any 3 of them are collinear (would make surface degenerate).
    if points.len() == 4 && is_quad_degenerate(&points, Tolerance::default_geom()) {
        return Err(ComponentError::new(format!(
            "{component}: de 4 hoekpunten vormen een gedegenereerde vierhoek — \
             controleer of de punten correct zijn geplaatst"
        )));
    }

    // Extract quality from meta, falling back to default if not specified
    let quality = coerce::geom_bridge::surface_builder_quality_from_meta_optional(meta)
        .unwrap_or_default();
    let (mesh, diagnostics) = mesh_four_point_surface_from_points(&points, quality)
        .map_err(|e| ComponentError::new(format!("{component}: {e}")))?;

    // Convert to output values
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    // Return both mesh and surface outputs for compatibility
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_SURFACE.to_string(), surface_value);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_value);
    Ok(out)
}

// ────────────────────────────────────────────────────────────────────────────
// Helper: try_coerce_optional_point
// ────────────────────────────────────────────────────────────────────────────

/// Attempts to coerce a value to a point, returning `None` for missing/null inputs.
///
/// Unlike `coerce_point_with_default`, this does NOT substitute `[0,0,0]` for
/// missing inputs, allowing callers to distinguish between "no input" and an
/// explicit origin point.
fn try_coerce_optional_point(value: Option<&Value>) -> Option<[f64; 3]> {
    match value {
        None | Some(Value::Null) => None,
        Some(Value::Point(p)) => Some(*p),
        Some(Value::List(values)) if !values.is_empty() => {
            // Recursively try the first element of a single-element list
            if values.len() == 1 {
                try_coerce_optional_point(Some(&values[0]))
            } else {
                // For multi-element lists, try to interpret as [x, y, z] numbers
                if values.len() >= 3 {
                    let x = try_coerce_number(&values[0])?;
                    let y = try_coerce_number(&values[1])?;
                    let z = try_coerce_number(&values[2])?;
                    Some([x, y, z])
                } else {
                    None
                }
            }
        }
        Some(Value::Vector(v)) => Some(*v), // Accept vectors as points
        _ => None,
    }
}

/// Tries to coerce a value to a number, returning `None` on failure.
fn try_coerce_number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => Some(*n),
        _ => None,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Helper: degeneracy checks for FourPointSurface
// ────────────────────────────────────────────────────────────────────────────

/// Returns `true` if all points in the slice are coincident (within tolerance).
fn are_points_coincident(points: &[GeomPoint3], tol: Tolerance) -> bool {
    if points.len() < 2 {
        return false; // Single point cannot be "coincident" in a meaningful sense
    }
    let first = points[0];
    points.iter().skip(1).all(|p| tol.approx_eq_point3(first, *p))
}

/// Returns `true` if all points in the slice are collinear (lie on a single line).
///
/// Uses the cross-product magnitude: if all cross products are near zero, points
/// are collinear.
fn are_points_collinear(points: &[GeomPoint3], tol: Tolerance) -> bool {
    if points.len() < 3 {
        return true; // 0, 1, or 2 points are trivially collinear
    }
    let p0 = points[0];
    let v01 = points[1] - p0;

    // Check if the baseline vector is zero-length (first two points coincident)
    if tol.is_zero_vec3(v01) {
        // Need to find a non-zero baseline; check subsequent points
        for i in 2..points.len() {
            let vi = points[i] - p0;
            if !tol.is_zero_vec3(vi) {
                // Found a non-zero baseline; now check remaining points against it
                return check_collinearity_with_baseline(points, p0, vi, tol);
            }
        }
        // All points are coincident with p0
        return true;
    }

    check_collinearity_with_baseline(points, p0, v01, tol)
}

/// Helper for collinearity check: given a baseline vector, verify all points lie on it.
fn check_collinearity_with_baseline(
    points: &[GeomPoint3],
    base: GeomPoint3,
    baseline: GeomVec3,
    tol: Tolerance,
) -> bool {
    let baseline_len = baseline.length();
    if baseline_len < tol.eps {
        return true; // Degenerate baseline
    }

    for p in points {
        let v = *p - base;
        let cross = baseline.cross(v);
        // For collinearity, cross product magnitude should be near zero
        // Normalize by baseline length for scale-invariance
        let cross_mag = cross.length();
        let v_len = v.length();
        // Use a relative tolerance: cross / (|baseline| * |v|) should be small
        if v_len > tol.eps && cross_mag > tol.eps * baseline_len.max(v_len) * 1e3 {
            return false;
        }
    }
    true
}

/// Checks if a 4-point quad is degenerate (cannot form a valid bilinear surface).
///
/// A quad is degenerate if:
/// - Any three corners are collinear (the surface would collapse to a line/triangle)
/// - All four points are coincident
fn is_quad_degenerate(points: &[GeomPoint3], tol: Tolerance) -> bool {
    if points.len() != 4 {
        return false;
    }

    // Check if all 4 are coincident
    if are_points_coincident(points, tol) {
        return true;
    }

    // Check if any subset of 3 points is collinear
    let subsets: [[usize; 3]; 4] = [
        [0, 1, 2],
        [0, 1, 3],
        [0, 2, 3],
        [1, 2, 3],
    ];

    for subset in &subsets {
        let triple = [points[subset[0]], points[subset[1]], points[subset[2]]];
        if are_points_collinear(&triple, tol) {
            return true;
        }
    }

    false
}

fn evaluate_fragment_patch(inputs: &[Value]) -> ComponentResult {
    let component = "Fragment Patch";
    let boundary = expect_input(inputs, 0, component, "boundary")?;
    
    // Collect boundary curves
    let boundary_segments = coerce::coerce_curve_segments(boundary)?;
    if boundary_segments.is_empty() {
        return Err(ComponentError::new(
            "Fragment Patch kon geen randcurves lezen",
        ));
    }

    // Group segments into polylines (one per connected boundary loop)
    let boundary_polylines = group_segments_into_polylines(boundary_segments);
    if boundary_polylines.is_empty() {
        return Err(ComponentError::new(
            "Fragment Patch kon geen geldige randpolylines vormen",
        ));
    }

    // Prepare boundary loops: auto-close open curves with tracking for warnings.
    // Note: Fragment Patch requires closed boundaries, but we accept open curves for
    // backward compatibility and emit warnings when auto-closing occurs.
    let prepared = prepare_boundary_loops_for_patch(&boundary_polylines);
    // Call auto_close_warnings before moving closed_polylines to avoid borrow-after-move
    let auto_close_warnings = prepared.auto_close_warnings(component);
    let closed_polylines = prepared.closed_polylines;

    if closed_polylines.is_empty() {
        return Err(ComponentError::new(
            "Fragment Patch vereist minstens één gesloten randcurve met drie of meer punten",
        ));
    }

    // Convert to geom Point3 format
    let geom_polylines: Vec<Vec<GeomPoint3>> = closed_polylines
        .iter()
        .map(|poly| points_to_geom_polyline(poly))
        .collect();

    let tol = Tolerance::default();

    // Call geom::fragment_patch_meshes to handle multiple boundary loops with automatic nesting
    let patch_results = fragment_patch_meshes_with_tolerance(&geom_polylines, tol)
        .map_err(|e| patch_error_to_component_error(e, component))?;

    if patch_results.is_empty() {
        return Err(ComponentError::new(
            "Fragment Patch kon geen patches genereren uit de gegeven grenzen",
        ));
    }

    // Collect all mesh outputs, adding auto-close warnings to the first diagnostics
    let mut mesh_values: Vec<Value> = Vec::with_capacity(patch_results.len());
    let mut surface_values: Vec<Value> = Vec::with_capacity(patch_results.len());
    let mut first_mesh = true;

    for (mesh, mut diagnostics) in patch_results {
        // Add auto-close warnings only to the first mesh's diagnostics to avoid duplication
        if first_mesh && !auto_close_warnings.is_empty() {
            for warning in &auto_close_warnings {
                diagnostics.add_warning(warning.clone());
            }
            first_mesh = false;
        }
        mesh_values.push(geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics)));
        surface_values.push(geom_mesh_to_value_surface(mesh));
    }

    // Return the primary patch output (first surface) and list of all meshes
    let mut out = BTreeMap::new();
    if surface_values.len() == 1 {
        out.insert(PIN_OUTPUT_PATCH.to_string(), surface_values.into_iter().next().unwrap());
        out.insert(PIN_OUTPUT_MESH.to_string(), mesh_values.into_iter().next().unwrap());
    } else {
        out.insert(PIN_OUTPUT_PATCH.to_string(), Value::List(surface_values));
        out.insert(PIN_OUTPUT_MESH.to_string(), Value::List(mesh_values));
    }
    Ok(out)
}

fn evaluate_revolution(inputs: &[Value]) -> ComponentResult {
    let component = "Revolution";

    // Parse profile curve (input 0: Curve/P)
    let profile_segments = coerce::coerce_curve_segments(inputs.get(0).unwrap_or(&Value::Null))?;
    if profile_segments.is_empty() {
        return Err(ComponentError::new(
            "Revolution kon profiel niet lezen",
        ));
    }

    // Parse axis (input 1: Axis/A) - expected as a line segment defining the axis
    let axis_segments = coerce::coerce_curve_segments(inputs.get(1).unwrap_or(&Value::Null))?;
    if axis_segments.is_empty() {
        return Err(ComponentError::new(
            "Revolution kon as niet lezen - verwacht een lijnsegment",
        ));
    }
    let (axis_start_raw, axis_end_raw) = axis_segments[0];
    let axis_start = to_geom_point(axis_start_raw);
    let axis_end = to_geom_point(axis_end_raw);

    // Validate axis has length
    let axis_vec = axis_end.sub_point(axis_start);
    if axis_vec.length_squared() < EPSILON * EPSILON {
        return Err(ComponentError::new(
            "Revolution as heeft geen lengte - start en eindpunt zijn gelijk",
        ));
    }
    let axis_dir = axis_vec.normalized().unwrap_or(Vec3::new(0.0, 0.0, 1.0));

    // Parse angle domain (input 2: Domain/D) - defaults to full revolution (2π)
    // This extracts both start angle (seam location) and signed sweep (direction).
    let angle_params = coerce_angle_domain_params(
        inputs.get(2).unwrap_or(&Value::Null),
        component,
    )?;

    // Extract the sweep magnitude (clamped to valid range)
    let sweep_magnitude = angle_params.clamped_sweep_magnitude();

    // Determine effective axis direction based on sweep sign:
    // - Positive sweep: use original axis direction (counter-clockwise by right-hand rule)
    // - Negative sweep: flip axis direction to reverse rotation sense
    let (effective_axis_start, effective_axis_end) = if angle_params.is_clockwise() {
        // Flip axis to reverse rotation direction
        (axis_end, axis_start)
    } else {
        (axis_start, axis_end)
    };

    // Convert profile segments to a continuous polyline
    let profile_polylines = group_segments_into_polylines(profile_segments);
    let profile_polyline = pick_longest_polyline(profile_polylines).ok_or_else(|| {
        ComponentError::new("Revolution kon profiel niet samenvoegen tot een polyline")
    })?;

    if profile_polyline.len() < 2 {
        return Err(ComponentError::new(
            "Revolution vereist een profiel met minstens twee punten",
        ));
    }

    // Convert to geom types
    let mut geom_profile = points_to_geom_polyline(&profile_polyline);
    let tol = Tolerance::default();

    // Apply start angle offset to profile if non-zero.
    // This pre-rotates the profile around the axis, effectively shifting the seam location.
    // The rotation is applied in the same direction as the sweep (using effective axis).
    if angle_params.start_angle.abs() > EPSILON {
        let effective_axis_dir = effective_axis_end.sub_point(effective_axis_start)
            .normalized()
            .unwrap_or(axis_dir);

        // Build a rotation transform: translate to axis origin, rotate, translate back
        let to_origin = Transform::translate(Vec3::new(
            -effective_axis_start.x,
            -effective_axis_start.y,
            -effective_axis_start.z,
        ));
        let rotation = Transform::rotate_axis(effective_axis_dir, angle_params.start_angle)
            .unwrap_or(Transform::identity());
        let from_origin = Transform::translate(Vec3::new(
            effective_axis_start.x,
            effective_axis_start.y,
            effective_axis_start.z,
        ));
        let full_transform = from_origin.compose(rotation.compose(to_origin));

        // Apply the rotation to each profile point
        for point in &mut geom_profile {
            *point = full_transform.apply_point(*point);
        }
    }

    // Determine if the profile is closed (for caps)
    let is_profile_closed = profile_polyline.len() >= 3 && is_closed(&profile_polyline);

    // Determine if this is a full 360° revolution.
    // For full revolutions, the resulting surface is inherently closed, so caps would
    // create redundant/overlapping geometry (similar to Grasshopper's surface-only output).
    let is_full_rev = angle_params.is_full_revolution();

    // Use caps only for closed profiles with partial (non-full) revolutions.
    // Full revolutions already close the surface, making caps unnecessary.
    let caps = if is_profile_closed && !is_full_rev {
        RevolveCaps::BOTH
    } else {
        RevolveCaps::NONE
    };

    // Use default revolve options with adaptive subdivision
    let options = RevolveOptions::default();

    // Call the geom revolve function with effective axis and sweep magnitude
    let (mesh, diagnostics) = revolve_polyline_with_options(
        &geom_profile,
        effective_axis_start,
        effective_axis_end,
        sweep_magnitude,
        caps,
        options,
        tol,
    )
    .map_err(|e| revolve_error_to_component_error(e, component))?;

    // Output as Value::Mesh (primary) and also provide legacy Value::Surface
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    // Return primary surface on "S" pin for backward compatibility
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_SURFACE.to_string(), surface_value);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_value);
    Ok(out)
}

fn evaluate_boundary_surfaces(inputs: &[Value]) -> ComponentResult {
    let component = "Boundary Surfaces";
    let edges = expect_input(inputs, 0, component, "edges")?;
    let segments = coerce::coerce_curve_segments(edges)?;
    if segments.is_empty() {
        return Err(ComponentError::new(
            "Boundary Surfaces vereist minstens één gesloten rand",
        ));
    }

    // Group segments into polylines (one per connected boundary loop)
    let boundary_polylines = group_segments_into_polylines(segments);
    if boundary_polylines.is_empty() {
        return Err(ComponentError::new(
            "Boundary Surfaces kon geen geldige randpolylines vormen",
        ));
    }

    // Prepare boundary loops: auto-close open curves with tracking for warnings.
    // Note: Boundary Surfaces requires closed boundaries, but we accept open curves
    // for backward compatibility and emit warnings when auto-closing occurs.
    let prepared = prepare_boundary_loops_for_patch(&boundary_polylines);
    let auto_close_warnings = prepared.auto_close_warnings(component);

    // Convert closed polylines to geom Point3 format
    let closed_polylines: Vec<Vec<Point3>> = prepared
        .closed_polylines
        .iter()
        .map(|poly| points_to_geom_polyline(poly))
        .collect();

    if closed_polylines.is_empty() {
        return Err(ComponentError::new(
            "Boundary Surfaces vereist minstens één gesloten randcurve met drie of meer punten",
        ));
    }

    let tol = Tolerance::default();

    // Use fragment_patch_meshes which properly handles nested loops:
    // - Projects all loops onto a shared plane
    // - Detects containment relationships (nesting) between loops
    // - Assigns inner loops as holes to their containing outer loops
    // - Returns one patch per outer region (with holes properly cut out)
    let results = fragment_patch_meshes_with_tolerance(&closed_polylines, tol)
        .map_err(|e| patch_error_to_component_error(e, component))?;

    if results.is_empty() {
        return Err(ComponentError::new(
            "Boundary Surfaces kon geen geldige oppervlakken genereren",
        ));
    }

    // Collect all mesh outputs, adding auto-close warnings to the first diagnostics
    let mut mesh_values: Vec<Value> = Vec::with_capacity(results.len());
    let mut surface_values: Vec<Value> = Vec::with_capacity(results.len());
    let mut first_mesh = true;

    for (mesh, mut diagnostics) in results {
        // Add auto-close warnings only to the first mesh's diagnostics to avoid duplication
        if first_mesh && !auto_close_warnings.is_empty() {
            for warning in &auto_close_warnings {
                diagnostics.add_warning(warning.clone());
            }
            first_mesh = false;
        }
        mesh_values.push(geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics)));
        surface_values.push(geom_mesh_to_value_surface(mesh));
    }

    // Return the list of surfaces (Boundary Surfaces always returns a list)
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_SURFACE.to_string(), Value::List(surface_values));
    out.insert(PIN_OUTPUT_MESH.to_string(), Value::List(mesh_values));
    Ok(out)
}

fn evaluate_rail_revolution(inputs: &[Value]) -> ComponentResult {
    let component = "Rail Revolution";

    // Parse profile curve (input 0: Curve/P)
    let profile_segments = coerce::coerce_curve_segments(inputs.get(0).unwrap_or(&Value::Null))?;
    if profile_segments.is_empty() {
        return Err(ComponentError::new(
            "Rail Revolution vereist een profielcurve",
        ));
    }

    // Parse rail curve (input 1: Rail/R)
    let rail_segments = coerce::coerce_curve_segments(inputs.get(1).unwrap_or(&Value::Null))?;
    if rail_segments.is_empty() {
        return Err(ComponentError::new(
            "Rail Revolution vereist een railcurve",
        ));
    }

    // Parse axis (input 2: Axis/A) - defines reference orientation for the profile
    // The axis in Grasshopper RailRevolution serves two purposes:
    // 1. Its origin defines where the profile should be positioned (translated to rail start)
    // 2. Its direction defines the reference "up" direction for frame orientation
    let axis_segments = coerce::coerce_curve_segments(inputs.get(2).unwrap_or(&Value::Null))?;

    // Parse scale (input 3: Scale/S) - scale factor for the profile (Value::Null means unconnected -> use default)
    let scale = coerce_optional_number_with_default(
        inputs.get(3),
        1.0, // Default: no scaling
        component,
        "Scale",
    )?.abs().max(EPSILON);

    // Convert profile segments to a continuous polyline
    let profile_polylines = group_segments_into_polylines(profile_segments);
    let profile_polyline = pick_longest_polyline(profile_polylines).ok_or_else(|| {
        ComponentError::new("Rail Revolution kon profiel niet samenvoegen tot een polyline")
    })?;

    if profile_polyline.len() < 2 {
        return Err(ComponentError::new(
            "Rail Revolution vereist een profiel met minstens twee punten",
        ));
    }

    // Convert rail segments to a continuous polyline
    let rail_polylines = group_segments_into_polylines(rail_segments);
    let rail_polyline = pick_longest_polyline(rail_polylines).ok_or_else(|| {
        ComponentError::new("Rail Revolution kon rail niet samenvoegen tot een polyline")
    })?;

    if rail_polyline.len() < 2 {
        return Err(ComponentError::new(
            "Rail Revolution vereist een rail met minstens twee punten",
        ));
    }

    // Apply scale to the profile (scale around profile centroid)
    let scaled_profile = if (scale - 1.0).abs() > EPSILON {
        scale_profile_around_centroid(&profile_polyline, scale)
    } else {
        profile_polyline.clone()
    };

    // Prepare the profile and reference axis for the geom function.
    // 
    // The profile coordinate system in geom::rail_revolve:
    // - Profile Z -> rail tangent
    // - Profile X -> frame normal (influenced by axis direction)
    // - Profile Y -> frame binormal
    //
    // If an axis is provided:
    // - Translate profile so the axis origin becomes the profile origin
    // - Use axis direction for frame orientation
    // If no axis is provided:
    // - Center profile at its centroid (legacy behavior for backward compatibility)
    // - Use default frame orientation (arbitrary but consistent)
    let (prepared_profile, reference_axis) = if !axis_segments.is_empty() {
        let (axis_start, axis_end) = axis_segments[0];
        let axis_origin = Point3::new(axis_start[0], axis_start[1], axis_start[2]);
        let axis_end_pt = Point3::new(axis_end[0], axis_end[1], axis_end[2]);
        let axis_direction = axis_end_pt.sub_point(axis_origin);
        
        // Validate axis has non-zero length
        if axis_direction.length_squared() < EPSILON * EPSILON {
            return Err(ComponentError::new(
                "Rail Revolution as heeft geen lengte - start en eindpunt zijn gelijk",
            ));
        }
        
        // Transform profile: translate so axis origin becomes the profile origin
        // This preserves the profile's position/orientation relative to the axis
        let translated_profile: Vec<[f64; 3]> = scaled_profile
            .iter()
            .map(|p| [
                p[0] - axis_origin.x,
                p[1] - axis_origin.y,
                p[2] - axis_origin.z,
            ])
            .collect();
        
        let axis = Some(RailRevolveAxis {
            origin: axis_origin,
            direction: axis_direction,
        });
        
        (translated_profile, axis)
    } else {
        // No axis provided: use legacy behavior (center at centroid, no reference direction)
        let centered_profile = center_profile_at_origin(&scaled_profile);
        (centered_profile, None)
    };

    // Convert to geom types
    let geom_profile = points_to_geom_polyline(&prepared_profile);
    let geom_rail = points_to_geom_polyline(&rail_polyline);
    let tol = Tolerance::default();

    // Determine if the profile is closed (for caps)
    let is_profile_closed = scaled_profile.len() >= 3 && is_closed(&scaled_profile);

    // Determine if the rail is closed (forms a full loop).
    // For closed rails, the profile sweeps all the way around and the surface is
    // inherently closed, so caps would create redundant/overlapping geometry.
    let is_rail_closed = rail_polyline.len() >= 3 && is_closed(&rail_polyline);

    // Use caps only for closed profiles with open rails.
    // Closed rails already close the surface, making caps unnecessary.
    let caps = if is_profile_closed && !is_rail_closed {
        RevolveCaps::BOTH
    } else {
        RevolveCaps::NONE
    };

    // Build options with reference axis
    let options = RailRevolveOptions {
        reference_axis,
        caps,
    };

    // Call the geom rail revolve function with full options
    let (mesh, diagnostics) = rail_revolve_polyline_with_options(
        &geom_profile,
        &geom_rail,
        options,
        tol,
    )
    .map_err(|e| revolve_error_to_component_error(e, component))?;

    // Output as Value::Mesh (primary) and also provide legacy Value::Surface
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));
    let surface_value = geom_mesh_to_value_surface(mesh);

    // Return primary surface on "S" pin for backward compatibility
    let mut out = BTreeMap::new();
    out.insert(PIN_OUTPUT_SURFACE.to_string(), surface_value);
    out.insert(PIN_OUTPUT_MESH.to_string(), mesh_value);
    Ok(out)
}

/// Compute the centroid of a profile, excluding the duplicate closing point for closed profiles.
///
/// For closed profiles where the first and last points coincide (within tolerance),
/// the closing point is excluded from the centroid calculation to avoid biasing
/// the result towards the start/end location.
fn compute_profile_centroid(profile: &[[f64; 3]]) -> Option<[f64; 3]> {
    if profile.is_empty() {
        return None;
    }

    // Determine which points to include in centroid calculation.
    // For closed profiles, exclude the duplicate last point.
    let points_for_centroid: &[[f64; 3]] = if profile.len() >= 3
        && points_equal(*profile.first().unwrap(), *profile.last().unwrap())
    {
        // Closed profile: exclude the last (duplicate) point
        &profile[..profile.len() - 1]
    } else {
        profile
    };

    if points_for_centroid.is_empty() {
        return None;
    }

    let n = points_for_centroid.len() as f64;
    let cx = points_for_centroid.iter().map(|p| p[0]).sum::<f64>() / n;
    let cy = points_for_centroid.iter().map(|p| p[1]).sum::<f64>() / n;
    let cz = points_for_centroid.iter().map(|p| p[2]).sum::<f64>() / n;

    Some([cx, cy, cz])
}

/// Scale a profile polyline around its centroid.
///
/// For closed profiles, the centroid calculation excludes the duplicate closing point
/// to ensure the scaling center is geometrically accurate.
fn scale_profile_around_centroid(profile: &[[f64; 3]], scale: f64) -> Vec<[f64; 3]> {
    if profile.is_empty() {
        return Vec::new();
    }

    // Calculate centroid (properly handling closed profiles)
    let [cx, cy, cz] = match compute_profile_centroid(profile) {
        Some(c) => c,
        None => return Vec::new(),
    };

    // Scale each point relative to centroid
    profile
        .iter()
        .map(|p| {
            [
                cx + (p[0] - cx) * scale,
                cy + (p[1] - cy) * scale,
                cz + (p[2] - cz) * scale,
            ]
        })
        .collect()
}

/// Center a profile polyline at the origin (translate so centroid is at origin).
/// This is required for geom::rail_revolve which expects the profile to be centered.
///
/// For closed profiles, the centroid calculation excludes the duplicate closing point
/// to ensure the centering is geometrically accurate.
fn center_profile_at_origin(profile: &[[f64; 3]]) -> Vec<[f64; 3]> {
    if profile.is_empty() {
        return Vec::new();
    }

    // Calculate centroid (properly handling closed profiles)
    let [cx, cy, cz] = match compute_profile_centroid(profile) {
        Some(c) => c,
        None => return Vec::new(),
    };

    // Translate each point so centroid is at origin
    profile
        .iter()
        .map(|p| [p[0] - cx, p[1] - cy, p[2] - cz])
        .collect()
}

fn collect_points(value: &Value, component: &str) -> Result<Vec<[f64; 3]>, ComponentError> {
    match value {
        Value::Point(point) => Ok(vec![*point]),
        Value::Vector(vector) => Ok(vec![*vector]),
        Value::CurveLine { p1, p2 } => Ok(vec![*p1, *p2]),
        Value::Surface { vertices, .. } => Ok(vertices.clone()),
        Value::List(values) => {
            let mut points = Vec::new();
            for entry in values {
                points.extend(collect_points(entry, component)?);
            }
            Ok(points)
        }
        other => Err(ComponentError::new(format!(
            "{component} verwacht punt-achtige invoer, kreeg {}",
            other.kind()
        ))),
    }
}

fn coerce_direction(
    value: &Value,
    component: &str,
    name: &str,
) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Vector(vector) => Ok(*vector),
        Value::CurveLine { p1, p2 } => Ok(subtract_points(*p2, *p1)),
        Value::Number(height) => Ok([0.0, 0.0, *height]),
        Value::List(values) if values.len() == 1 => coerce_direction(&values[0], component, name),
        other => Err(ComponentError::new(format!(
            "{component} verwacht een richting voor {name}, kreeg {}",
            other.kind()
        ))),
    }
}

fn coerce_point(value: &Value, component: &str, name: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Value::Point(point) => Ok(*point),
        Value::List(values) if values.len() == 1 => coerce_point(&values[0], component, name),
        other => Err(ComponentError::new(format!(
            "{component} verwacht een punt voor {name}, kreeg {}",
            other.kind()
        ))),
    }
}

fn coerce_number(value: &Value, component: &str, name: &str) -> Result<f64, ComponentError> {
    match value {
        Value::Number(number) => Ok(*number),
        Value::List(values) if values.len() == 1 => coerce_number(&values[0], component, name),
        other => Err(ComponentError::new(format!(
            "{component} verwacht een getal voor {name}, kreeg {}",
            other.kind()
        ))),
    }
}

/// Coerces an optional value to a number, treating `Value::Null` and `None` as unset.
/// Returns the default value when the input is unset, and errors only if the value
/// is present but cannot be coerced to a number.
///
/// This is the preferred function for optional numeric component inputs where
/// unconnected pins should use a sensible default without generating errors.
fn coerce_optional_number_with_default(
    value: Option<&Value>,
    default: f64,
    component: &str,
    name: &str,
) -> Result<f64, ComponentError> {
    match value {
        Some(Value::Null) | None => Ok(default),
        Some(v) => coerce_number(v, component, name),
    }
}

fn coerce_bool(value: &Value, component: &str, name: &str) -> Result<bool, ComponentError> {
    match value {
        Value::Boolean(flag) => Ok(*flag),
        Value::Number(number) => Ok(*number != 0.0),
        Value::List(values) if values.len() == 1 => coerce_bool(&values[0], component, name),
        other => Err(ComponentError::new(format!(
            "{component} verwacht een booleaanse waarde voor {name}, kreeg {}",
            other.kind()
        ))),
    }
}

/// Coerces an optional value to a boolean, treating `Value::Null` and `None` as unset.
/// Returns the default value when the input is unset, and errors only if the value
/// is present but cannot be coerced to a boolean.
///
/// This is the preferred function for optional boolean component inputs where
/// unconnected pins should use a sensible default without generating errors.
fn coerce_optional_bool_with_default(
    value: Option<&Value>,
    default: bool,
    component: &str,
    name: &str,
) -> Result<bool, ComponentError> {
    match value {
        Some(Value::Null) | None => Ok(default),
        Some(v) => coerce_bool(v, component, name),
    }
}

/// Coerces an optional value to a positive integer (`usize`), treating `Value::Null`
/// and `None` as unset (returns `Ok(None)`).
///
/// Unlike using `.ok().map(|n| n.round() as usize)`, this function:
/// - Errors on invalid types (strings, etc.) instead of silently treating them as unset
/// - Explicitly rejects negative, NaN, and infinite values with clear error messages
/// - Safely converts to `usize` only after validation
///
/// # Arguments
/// * `value` - Optional input value from component pin
/// * `component` - Component name for error messages
/// * `name` - Pin/parameter name for error messages
///
/// # Returns
/// * `Ok(None)` - When input is `None` or `Value::Null` (unconnected pin)
/// * `Ok(Some(n))` - When input is a valid non-negative finite number
/// * `Err(_)` - When input is an invalid type or an invalid numeric value
fn coerce_optional_positive_integer(
    value: Option<&Value>,
    component: &str,
    name: &str,
) -> Result<Option<usize>, ComponentError> {
    match value {
        Some(Value::Null) | None => Ok(None),
        Some(v) => {
            let n = coerce_number(v, component, name)?;

            // Reject NaN and infinity with clear error messages
            if n.is_nan() {
                return Err(ComponentError::new(format!(
                    "{component}: {name} kan geen NaN zijn"
                )));
            }
            if n.is_infinite() {
                return Err(ComponentError::new(format!(
                    "{component}: {name} kan geen oneindig zijn"
                )));
            }

            // Reject negative values
            if n < 0.0 {
                return Err(ComponentError::new(format!(
                    "{component}: {name} moet een niet-negatief getal zijn, kreeg {n}"
                )));
            }

            // Safe conversion: n is finite and non-negative
            // Round to nearest integer for float inputs like 3.0 or 3.5
            let rounded = n.round();

            // Check for overflow before casting (usize::MAX as f64 may lose precision,
            // but any reasonable grid dimension will be far below this threshold)
            const MAX_REASONABLE_GRID_DIM: f64 = 1_000_000.0;
            if rounded > MAX_REASONABLE_GRID_DIM {
                return Err(ComponentError::new(format!(
                    "{component}: {name} waarde {rounded} is te groot (maximum: {MAX_REASONABLE_GRID_DIM})"
                )));
            }

            Ok(Some(rounded as usize))
        }
    }
}

fn coerce_number_list(
    value: &Value,
    component: &str,
    name: &str,
) -> Result<Vec<f64>, ComponentError> {
    match value {
        Value::Number(number) => Ok(vec![*number]),
        Value::List(values) => {
            let mut result = Vec::new();
            for entry in values {
                result.extend(coerce_number_list(entry, component, name)?);
            }
            Ok(result)
        }
        other => Err(ComponentError::new(format!(
            "{component} verwacht een (lijst) getallen voor {name}, kreeg {}",
            other.kind()
        ))),
    }
}

/// Applies Grasshopper-style "longest list" matching to two lists.
///
/// Grasshopper's default list matching behavior extends shorter lists by repeating
/// the last element until both lists have equal length. This allows users to provide:
/// - A single value that broadcasts to all positions
/// - A shorter list where the last value repeats for remaining positions
///
/// # Arguments
/// * `list_a` - First list (will be extended if shorter)
/// * `list_b` - Second list (will be extended if shorter)
///
/// # Returns
/// A tuple of two vectors with equal length, or the original lists if either is empty.
///
/// # Example
/// ```ignore
/// let params = vec![0.0, 0.5, 1.0];
/// let radii = vec![1.0];  // Single value
/// let (matched_params, matched_radii) = match_list_lengths_grasshopper(params, radii);
/// // matched_radii becomes [1.0, 1.0, 1.0]
/// ```
fn match_list_lengths_grasshopper(mut list_a: Vec<f64>, mut list_b: Vec<f64>) -> (Vec<f64>, Vec<f64>) {
    // Handle empty lists - return as-is (caller should validate)
    if list_a.is_empty() || list_b.is_empty() {
        return (list_a, list_b);
    }

    let target_len = list_a.len().max(list_b.len());

    // Extend list_a by repeating its last element
    if let Some(&last) = list_a.last() {
        list_a.resize(target_len, last);
    }

    // Extend list_b by repeating its last element
    if let Some(&last) = list_b.last() {
        list_b.resize(target_len, last);
    }

    (list_a, list_b)
}

/// Parameters extracted from an angle domain for revolution.
///
/// When a `Value::Domain` is provided, we preserve the full semantics:
/// - `start_angle`: Where the revolution seam starts (domain.start)
/// - `sweep_angle`: How much to revolve, signed (domain.end - domain.start)
///
/// The sign of `sweep_angle` determines the rotation direction:
/// - Positive: counter-clockwise around the axis (right-hand rule)
/// - Negative: clockwise around the axis
#[derive(Debug, Clone, Copy)]
struct AngleDomainParams {
    /// Starting angle offset for the seam (radians).
    pub start_angle: f64,
    /// Sweep angle, signed to indicate direction (radians).
    pub sweep_angle: f64,
}

impl AngleDomainParams {
    /// Full revolution (2π) with no start offset.
    pub const FULL_REVOLUTION: Self = Self {
        start_angle: 0.0,
        sweep_angle: 2.0 * std::f64::consts::PI,
    };

    /// Get the absolute sweep magnitude, clamped to [EPSILON, 2π].
    /// Zero sweep is promoted to full revolution.
    pub fn clamped_sweep_magnitude(&self) -> f64 {
        let abs_sweep = self.sweep_angle.abs();
        let clamped = abs_sweep.min(2.0 * std::f64::consts::PI);
        if clamped < EPSILON {
            2.0 * std::f64::consts::PI
        } else {
            clamped
        }
    }

    /// Returns `true` if the sweep is negative (clockwise rotation).
    pub fn is_clockwise(&self) -> bool {
        self.sweep_angle < 0.0
    }

    /// Returns `true` if the sweep magnitude is effectively a full 360° revolution.
    ///
    /// A full revolution means the revolved surface is inherently closed, so caps
    /// would create redundant/overlapping geometry and should be disabled.
    pub fn is_full_revolution(&self) -> bool {
        (self.clamped_sweep_magnitude() - 2.0 * std::f64::consts::PI).abs() < EPSILON
    }
}

/// Coerce a value to angle domain parameters for revolution.
///
/// Handles the following cases:
/// - `Value::Null`: Returns full revolution (2π), no start offset.
/// - `Value::Number(n)`: Treated as sweep angle from 0.
/// - `Value::Domain::One`: Uses `start` for seam location, `end - start` for signed sweep.
/// - `Value::Domain::Two`: Uses the U domain's parameters.
/// - `Value::List` (single element): Recursively unwrap.
///
/// This preserves both the start angle (seam location) and the sweep sign
/// (rotation direction), enabling Grasshopper-compatible behavior.
fn coerce_angle_domain_params(value: &Value, component: &str) -> Result<AngleDomainParams, ComponentError> {
    match value {
        Value::Null => Ok(AngleDomainParams::FULL_REVOLUTION),
        Value::Number(number) => Ok(AngleDomainParams {
            start_angle: 0.0,
            sweep_angle: *number,
        }),
        Value::Domain(Domain::One(domain)) => Ok(AngleDomainParams {
            start_angle: domain.start,
            sweep_angle: domain.end - domain.start,
        }),
        Value::Domain(Domain::Two(domain)) => Ok(AngleDomainParams {
            start_angle: domain.u.start,
            sweep_angle: domain.u.end - domain.u.start,
        }),
        Value::List(values) if values.len() == 1 => coerce_angle_domain_params(&values[0], component),
        other => Err(ComponentError::new(format!(
            "{component} verwacht een hoek of domein, kreeg {}",
            other.kind()
        ))),
    }
}

fn create_surface_from_points(
    points: &[[f64; 3]],
    component: &str,
) -> Result<Value, ComponentError> {
    create_surface_from_points_with_padding(points, 0.0, component)
}

fn create_surface_from_points_with_padding(
    points: &[[f64; 3]],
    padding: f64,
    component: &str,
) -> Result<Value, ComponentError> {
    if points.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist minstens twee unieke punten"
        )));
    }

    let mut min = points[0];
    let mut max = points[0];
    for point in points.iter().skip(1) {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }

    let padding = padding.max(0.0);
    for axis in 0..3 {
        min[axis] -= padding;
        max[axis] += padding;
    }

    let spans = [
        (max[0] - min[0], 0usize),
        (max[1] - min[1], 1usize),
        (max[2] - min[2], 2usize),
    ];

    let mut sorted = spans;
    sorted.sort_by(
        |a, b| match (a.0.partial_cmp(&b.0), b.0.partial_cmp(&a.0)) {
            (Some(order), _) => order.reverse(),
            (None, Some(order)) => order,
            _ => Ordering::Equal,
        },
    );

    let (primary_span, primary_axis) = sorted[0];
    if primary_span.abs() <= EPSILON {
        return Err(ComponentError::new(format!(
            "{component} kon geen oppervlak vormen uit samenvallende punten"
        )));
    }

    let secondary_axis = sorted
        .iter()
        .skip(1)
        .find(|(span, axis)| *axis != primary_axis && span.abs() > EPSILON)
        .map(|(_, axis)| *axis)
        .unwrap_or_else(|| if primary_axis != 0 { 0 } else { 1 });

    let mut min_secondary = min[secondary_axis];
    let mut max_secondary = max[secondary_axis];
    if (max_secondary - min_secondary).abs() <= EPSILON {
        min_secondary -= 0.5;
        max_secondary += 0.5;
    }

    let third_axis = (0..3)
        .find(|axis| *axis != primary_axis && *axis != secondary_axis)
        .unwrap_or(primary_axis);
    let mid_third = (min[third_axis] + max[third_axis]) * 0.5;

    let mut vertices = Vec::with_capacity(4);
    for &a in &[min[primary_axis], max[primary_axis]] {
        for &b in &[min_secondary, max_secondary] {
            let mut vertex = [0.0; 3];
            vertex[primary_axis] = a;
            vertex[secondary_axis] = b;
            vertex[third_axis] = mid_third;
            vertices.push(vertex);
        }
    }

    let faces = vec![vec![0, 1, 2], vec![0, 2, 3]];
    Ok(Value::Surface { vertices, faces })
}

/// Creates a surface from a closed curve, similar to the Surface component behavior.
/// This function is used when Sweep1 receives a closed curve primitive.
fn create_surface_from_closed_curve(
    polyline: &[[f64; 3]],
    component: &str,
) -> Result<Value, ComponentError> {
    // Remove duplicate closing point if it exists
    let mut points = polyline.to_vec();
    if points.len() > 1 && points_equal(points[0], *points.last().unwrap()) {
        points.pop();
    }

    if points.len() < 3 {
        return Err(ComponentError::new(format!(
            "{component} vereist minstens drie unieke punten voor een gesloten curve",
        )));
    }

    // Compute plane normal for the closed curve
    let normal = polyline_normal(polyline);
    if is_zero_vector(normal) {
        return Err(ComponentError::new(format!(
            "{component} kon geen geldige normaal berekenen voor de gesloten curve",
        )));
    }

    // Compute centroid
    let centroid = points.iter().fold([0.0; 3], |acc, p| add_vector(acc, *p));
    let n = points.len() as f64;
    let centroid = [centroid[0] / n, centroid[1] / n, centroid[2] / n];

    // Find plane axes
    let (axis_x, axis_y) = plane_basis(normal);

    // Sort points by angle around centroid for proper triangulation
    let mut entries: Vec<(f64, [f64; 3])> = points
        .iter()
        .map(|point| {
            let diff = subtract_points(*point, centroid);
            let x = dot_product(diff, axis_x);
            let y = dot_product(diff, axis_y);
            (y.atan2(x), *point)
        })
        .collect();

    entries.sort_by(|a, b| match a.0.partial_cmp(&b.0) {
        Some(order) => order,
        None => Ordering::Equal,
    });

    let sorted_points: Vec<[f64; 3]> = entries.into_iter().map(|entry| entry.1).collect();

    // Create triangulated faces for the planar surface
    let mut faces: Vec<Vec<u32>> = Vec::new();
    
    for i in 1..sorted_points.len().saturating_sub(1) {
        faces.push(vec![0, i as u32, (i + 1) as u32]);
    }

    // Create a surface value that can be used with sweep_surface_along_polyline
    Ok(Value::Surface {
        vertices: sorted_points,
        faces,
    })
}

fn expect_input<'a>(
    inputs: &'a [Value],
    index: usize,
    component: &str,
    description: &str,
) -> Result<&'a Value, ComponentError> {
    inputs.get(index).ok_or_else(|| {
        ComponentError::new(format!("{component} vereist een invoer voor {description}"))
    })
}

fn input_source_count(meta: &MetaMap, index: usize) -> usize {
    let key = format!("input.{index}.source_count");
    meta.get(&key)
        .and_then(meta_value_to_usize)
        .unwrap_or(0)
}

fn meta_value_to_usize(value: &MetaValue) -> Option<usize> {
    match value {
        MetaValue::Integer(i) => (*i).try_into().ok(),
        MetaValue::Number(n) if *n >= 0.0 => Some(*n as usize),
        _ => None,
    }
}

fn add_vector(point: [f64; 3], direction: [f64; 3]) -> [f64; 3] {
    [
        point[0] + direction[0],
        point[1] + direction[1],
        point[2] + direction[2],
    ]
}

fn subtract_points(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross_product(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[allow(dead_code)]
fn dot_product(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let mag = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if mag > EPSILON {
        [v[0] / mag, v[1] / mag, v[2] / mag]
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let delta = subtract_points(a, b);
    (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt()
}

fn is_zero_vector(vector: [f64; 3]) -> bool {
    vector.iter().all(|component| component.abs() < EPSILON)
}

fn offset_rail_polyline(
    rail_polyline: &[[f64; 3]],
    section_origin: [f64; 3],
) -> Vec<[f64; 3]> {
    if rail_polyline.is_empty() {
        return Vec::new();
    }

    let translation = subtract_points(section_origin, rail_polyline[0]);
    rail_polyline
        .iter()
        .map(|point| add_vector(*point, translation))
        .collect()
}

fn dedup_consecutive_points(mut points: Vec<[f64; 3]>, closed: bool) -> Vec<[f64; 3]> {
    let mut deduped = Vec::with_capacity(points.len());
    for point in points.drain(..) {
        if deduped
            .last()
            .map_or(true, |last| !points_equal(*last, point))
        {
            deduped.push(point);
        }
    }

    if closed && deduped.len() > 2 && points_equal(deduped[0], *deduped.last().unwrap()) {
        deduped.pop();
    }

    deduped
}

fn project_point_on_polyline(point: [f64; 3], polyline: &[[f64; 3]]) -> (f64, f64) {
    if polyline.len() < 2 {
        return (0.0, distance(point, polyline.get(0).copied().unwrap_or([0.0; 3])));
    }

    let mut best_t = 0.0;
    let mut best_dist = f64::MAX;
    let mut accumulated = 0.0;
    let total_length = polyline_length(polyline);

    for window in polyline.windows(2) {
        let a = window[0];
        let b = window[1];
        let ab = subtract_points(b, a);
        let ab_len_sq = dot_product(ab, ab);
        if ab_len_sq < EPSILON {
            continue;
        }
        let ap = subtract_points(point, a);
        let t_seg = (dot_product(ap, ab) / ab_len_sq).clamp(0.0, 1.0);
        let closest = add_vector(a, [
            ab[0] * t_seg,
            ab[1] * t_seg,
            ab[2] * t_seg,
        ]);
        let dist = distance(point, closest);
        if dist < best_dist {
            best_dist = dist;
            let seg_length = ab_len_sq.sqrt();
            let seg_t = accumulated + seg_length * t_seg;
            best_t = if total_length > 0.0 {
                seg_t / total_length
            } else {
                0.0
            };
        }
        accumulated += ab_len_sq.sqrt();
    }

    (best_t, best_dist)
}

fn plane_basis(normal: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let n = {
        let n = normalize(normal);
        if is_zero_vector(n) {
            [0.0, 0.0, 1.0]
        } else {
            n
        }
    };

    let mut tangent = cross_product(n, [1.0, 0.0, 0.0]);
    if is_zero_vector(tangent) {
        tangent = cross_product(n, [0.0, 1.0, 0.0]);
    }
    if is_zero_vector(tangent) {
        tangent = [1.0, 0.0, 0.0];
    }
    tangent = normalize(tangent);
    let bitangent = normalize(cross_product(n, tangent));
    (tangent, bitangent)
}

fn signed_area_in_plane(polyline: &[[f64; 3]], normal: [f64; 3]) -> f64 {
    if polyline.len() < 3 {
        return 0.0;
    }
    let (x_axis, y_axis) = plane_basis(normal);
    let origin = polyline[0];

    let mut area = 0.0;
    for i in 0..polyline.len() {
        let j = (i + 1) % polyline.len();
        let vi = subtract_points(polyline[i], origin);
        let vj = subtract_points(polyline[j], origin);
        let ui = dot_product(vi, x_axis);
        let wi = dot_product(vi, y_axis);
        let uj = dot_product(vj, x_axis);
        let wj = dot_product(vj, y_axis);
        area += ui * wj - uj * wi;
    }

    area * 0.5
}

fn into_output(pin: &str, value: Value) -> ComponentResult {
    let mut outputs = BTreeMap::new();
    outputs.insert(pin.to_owned(), value);
    Ok(outputs)
}

fn collect_surfaces_recursive<'a>(
    value: &'a Value,
    surfaces: &mut Vec<coerce::Surface<'a>>,
) -> Result<(), ComponentError> {
    match value {
        Value::Surface { .. } => surfaces.push(coerce::coerce_surface(value)?),
        Value::List(values) => {
            for entry in values {
                collect_surfaces_recursive(entry, surfaces)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn pick_longest_polyline(polylines: Vec<Vec<[f64; 3]>>) -> Option<Vec<[f64; 3]>> {
    polylines
        .into_iter()
        .max_by(|a, b| match (polyline_length(&a), polyline_length(&b)) {
            (x, y) if x.is_finite() && y.is_finite() => {
                x.partial_cmp(&y).unwrap_or(Ordering::Equal)
            }
            _ => Ordering::Equal,
        })
}

fn polyline_length(polyline: &[[f64; 3]]) -> f64 {
    polyline
        .windows(2)
        .map(|pair| distance(pair[0], pair[1]))
        .sum()
}

fn polyline_centroid(points: &[[f64; 3]]) -> [f64; 3] {
    if points.is_empty() {
        return [0.0, 0.0, 0.0];
    }

    let mut count = points.len();
    if count > 1 && points_equal(points[0], *points.last().unwrap()) {
        count = count.saturating_sub(1);
    }
    if count == 0 {
        return [0.0, 0.0, 0.0];
    }

    let mut sum = [0.0, 0.0, 0.0];
    for point in points.iter().take(count) {
        sum = add_vector(sum, *point);
    }

    let denom = count as f64;
    [sum[0] / denom, sum[1] / denom, sum[2] / denom]
}

fn find_boundary_polylines(surface: &coerce::Surface<'_>) -> Vec<Vec<u32>> {
    let mut edge_counts = HashMap::new();
    for face in surface.faces {
        if face.len() < 2 {
            continue;
        }
        for i in 0..face.len() {
            let p1_idx = face[i];
            let p2_idx = face[(i + 1) % face.len()];

            // Normaliseer de edge door de kleinste index eerst te plaatsen
            let edge = if p1_idx < p2_idx {
                (p1_idx, p2_idx)
            } else {
                (p2_idx, p1_idx)
            };
            *edge_counts.entry(edge).or_insert(0) += 1;
        }
    }

    let boundary_edges: Vec<_> = edge_counts
        .into_iter()
        .filter(|(_, count)| *count == 1)
        .map(|(edge, _)| edge)
        .collect();

    if boundary_edges.is_empty() {
        return Vec::new();
    }

    let mut adj_list: HashMap<u32, Vec<u32>> = HashMap::new();
    for (p1, p2) in boundary_edges {
        adj_list.entry(p1).or_default().push(p2);
        adj_list.entry(p2).or_default().push(p1);
    }

    let mut polylines = Vec::new();
    let mut visited = std::collections::HashSet::new();

    for start_node in adj_list.keys() {
        if visited.contains(start_node) {
            continue;
        }

        let mut current_polyline_indices = Vec::new();
        let mut current_node = *start_node;

        while !visited.contains(&current_node) {
            visited.insert(current_node);
            current_polyline_indices.push(current_node);

            let next_node = adj_list
                .get(&current_node)
                .unwrap()
                .iter()
                .find(|&node| !visited.contains(node));

            if let Some(node) = next_node {
                current_node = *node;
            } else {
                // Einde van een open polyline
                break;
            }
        }
        if current_polyline_indices.len() > 1 {
            polylines.push(current_polyline_indices);
        }
    }

    polylines
}

fn calculate_surface_normal(surface: &coerce::Surface<'_>) -> [f64; 3] {
    if surface.faces.is_empty() || surface.faces[0].len() < 3 {
        return [0.0, 0.0, 1.0]; // Standaard normaal als het oppervlak niet goed gedefinieerd is
    }

    let first_face_indices = &surface.faces[0];
    let p1 = surface.vertices[first_face_indices[0] as usize];
    let p2 = surface.vertices[first_face_indices[1] as usize];
    let p3 = surface.vertices[first_face_indices[2] as usize];

    let v1 = subtract_points(p2, p1);
    let v2 = subtract_points(p3, p1);

    normalize(cross_product(v1, v2))
}

/// Sweeps a surface along a rail polyline, ensuring proper positioning relative to the rail origin.
fn sweep_surface_along_polyline(
    surface: coerce::Surface<'_>,
    rail_polyline: &[[f64; 3]],
    component: &str,
    add_caps: bool,
) -> Result<Value, ComponentError> {
    if surface.vertices.is_empty() {
        return Err(ComponentError::new(format!(
            "{component} verwacht een surface met minstens één vertex",
        )));
    }
    if surface.faces.is_empty() {
        return Err(ComponentError::new(format!(
            "{component} verwacht een surface met minstens één face",
        )));
    }
    if rail_polyline.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist een rail met minstens twee punten",
        )));
    }

    let rail_polyline: Vec<[f64; 3]> = dedup_consecutive_points(rail_polyline.to_vec(), false);
    if rail_polyline.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist een rail met minstens twee unieke punten",
        )));
    }

    let surface_normal = calculate_surface_normal(&surface);
    let boundary_polylines_indices = find_boundary_polylines(&surface);

    let mut vertices: Vec<[f64; 3]> = surface.vertices.to_vec();
    let mut faces = if add_caps {
        surface.faces.clone()
    } else {
        Vec::new()
    };

    let mut last_layer_start = 0u32;
    let base_faces = if add_caps {
        Some(surface.faces.clone())
    } else {
        None
    };

    // Sweep along the rail by positioning the original surface at each rail point
    for (i, &rail_point) in rail_polyline.iter().enumerate().skip(1) {
        let prev_rail_point = rail_polyline[i - 1];
        let rail_direction = subtract_points(rail_point, prev_rail_point);
        
        if is_zero_vector(rail_direction) {
            continue;
        }

        // Calculate the transformation from the original section to the current rail position
        // Use the rail start point as reference, not the surface's first vertex
        let rail_start = rail_polyline[0];
        let translation = subtract_points(rail_point, rail_start);
        
        let new_layer_start = vertices.len() as u32;
        let new_layer_vertices: Vec<[f64; 3]> = surface.vertices
            .iter()
            .map(|vertex| add_vector(*vertex, translation))
            .collect();
        vertices.extend(new_layer_vertices.iter());

        for polyline_indices in &boundary_polylines_indices {
            let polyline_vertices: Vec<[f64; 3]> = polyline_indices
                .iter()
                .map(|&i| vertices[i as usize])
                .collect();

            // Bereken de normaal van de polyline
            let p1 = polyline_vertices[0];
            let p2 = polyline_vertices[1];
            let p3 = *polyline_vertices.get(2).unwrap_or(&p1);
            let v1 = subtract_points(p2, p1);
            let v2 = subtract_points(p3, p1);
            let polyline_normal = normalize(cross_product(v1, v2));

            let mut corrected_indices = polyline_indices.clone();
            // Keer de polyline om als de normaal in de tegenovergestelde richting van de oppervlaknormaal wijst
            if dot_product(polyline_normal, surface_normal) < 0.0 {
                corrected_indices.reverse();
            }

            let n = corrected_indices.len();
            if n < 2 {
                continue;
            }

            for j in 0..n {
                let current_idx = corrected_indices[j];
                let next_idx = corrected_indices[(j + 1) % n];

                let v1 = last_layer_start + current_idx;
                let v2 = last_layer_start + next_idx;
                let v3 = new_layer_start + next_idx;
                let v4 = new_layer_start + current_idx;

                // Gebruik een consistente winding order voor de vlakken
                faces.push(vec![v1, v4, v2]);
                faces.push(vec![v2, v4, v3]);
            }
        }

        last_layer_start = new_layer_start;
    }

    if let Some(base_faces) = base_faces {
        for face in &base_faces {
            if face.len() < 2 {
                continue;
            }
            let mut top_face = Vec::with_capacity(face.len());
            for &index in face.iter().rev() {
                top_face.push(last_layer_start + index);
            }
            faces.push(top_face);
        }
    }

    Ok(Value::Surface { vertices, faces })
}

fn sweep_polyline_along_rail(
    profile: &[[f64; 3]],
    rail_polyline: &[[f64; 3]],
    component: &str,
) -> Result<Value, ComponentError> {
    let mut profile = profile.to_vec();
    let mut profile_closed = false;
    if profile.len() >= 3 && points_equal(profile[0], *profile.last().unwrap()) {
        profile.pop(); // remove duplicate closing point, keep closed flag
        profile_closed = true;
    } else if profile.len() >= 2 && points_equal(profile[0], *profile.last().unwrap()) {
        profile.pop(); // degenerate "closed" with only two equal points -> treat as open
    }

    // Verwijder opeenvolgende dubbele punten om degeneratie te voorkomen.
    profile = dedup_consecutive_points(profile, profile_closed);

    // Zorg voor een consistente CCW-winding zoals in BoxRectangle zodat front-faces correct zijn.
    if profile_closed && profile.len() >= 3 {
        let normal = {
            let n = polyline_normal(&profile);
            if is_zero_vector(n) {
                [0.0, 0.0, 1.0]
            } else {
                n
            }
        };
        let signed_area = signed_area_in_plane(&profile, normal);
        if signed_area < 0.0 {
            profile.reverse();
        }
    }

    if profile.is_empty() {
        return Err(ComponentError::new(format!(
            "{component} verwacht een sectiepolyline",
        )));
    }

    if profile.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} verwacht een sectiepolyline met minstens twee punten",
        )));
    }
    if rail_polyline.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist een rail met minstens twee punten",
        )));
    }

    let rail_polyline: Vec<[f64; 3]> = dedup_consecutive_points(rail_polyline.to_vec(), false);
    if rail_polyline.len() < 2 {
        return Err(ComponentError::new(format!(
            "{component} vereist een rail met minstens twee unieke punten",
        )));
    }

    // Calculate the initial section origin (this will be kept at the rail start)
    let section_origin = profile[0];
    
    // Create a proper sweep by positioning section curves along the rail
    // while maintaining proper orientation and keeping the original section at the start
    let mut vertices = profile.clone();
    let mut faces: Vec<Vec<u32>> = Vec::new();

    let layer_size = profile.len();
    let profile_indices: Vec<u32> = (0..layer_size as u32).collect();
    let ordered_profile = if profile_closed && layer_size >= 3 {
        let normal = polyline_normal(&profile);
        let winding = polyline_winding_direction(&profile, normal);
        if winding < 0.0 {
            let mut reversed = profile_indices.clone();
            reversed.reverse();
            reversed
        } else {
            profile_indices.clone()
        }
    } else {
        profile_indices.clone()
    };

    if profile_closed && layer_size >= 3 {
        let mut bottom = ordered_profile.clone();
        bottom.reverse();
        faces.push(bottom);
    }

    let mut last_layer_start = 0u32;

    // Sweep along the rail by positioning sections at each rail point
    for (i, &rail_point) in rail_polyline.iter().enumerate().skip(1) {
        let prev_rail_point = rail_polyline[i - 1];
        let rail_direction = subtract_points(rail_point, prev_rail_point);
        
        if is_zero_vector(rail_direction) {
            continue;
        }

        // Calculate the transformation from the original section to the current rail position
        let translation = subtract_points(rail_point, section_origin);
        
        // Create the new layer by translating the original profile (not the previous layer)
        // This ensures the original section shape is maintained at each position
        let new_layer_start = vertices.len() as u32;
        let new_layer_vertices: Vec<[f64; 3]> = profile
            .iter()
            .map(|vertex| add_vector(*vertex, translation))
            .collect();

        vertices.extend(new_layer_vertices.iter());

        // Create faces between the current and previous layers
        let edge_count = if profile_closed { layer_size } else { layer_size.saturating_sub(1) };
        for j in 0..edge_count {
            let current_idx = ordered_profile[j];
            let next_idx = ordered_profile[(j + 1) % layer_size];
            let v1 = last_layer_start + current_idx;
            let v2 = last_layer_start + next_idx;
            let v3 = new_layer_start + next_idx;
            let v4 = new_layer_start + current_idx;
            faces.push(vec![v1, v2, v4]);
            faces.push(vec![v2, v3, v4]);
        }

        last_layer_start = new_layer_start;
    }

    if profile_closed && layer_size >= 3 {
        let mut top_face = Vec::with_capacity(layer_size);
        for &index in ordered_profile.iter() {
            top_face.push(last_layer_start + index);
        }
        faces.push(top_face);
    }

    Ok(Value::Surface { vertices, faces })
}


#[allow(dead_code)]
fn extrude_surface_along_vector(
    surface: coerce::Surface<'_>,
    direction: [f64; 3],
    component: &str,
) -> Result<Value, ComponentError> {
    if surface.vertices.is_empty() {
        return Err(ComponentError::new(format!(
            "{component} verwacht een surface met minstens één vertex"
        )));
    }
    if surface.faces.is_empty() {
        return Err(ComponentError::new(format!(
            "{component} verwacht een surface met minstens één face"
        )));
    }
    if is_zero_vector(direction) {
        return Err(ComponentError::new(format!(
            "{component} kan niet extruderen zonder railrichting"
        )));
    }

    let offset = surface.vertices.len() as u32;

    let mut vertices = surface.vertices.clone();
    vertices.extend(
        surface
            .vertices
            .iter()
            .map(|vertex| add_vector(*vertex, direction)),
    );

    let mut faces = Vec::new();
    for face in surface.faces.iter() {
        if face.len() < 2 {
            continue;
        }

        faces.push(face.clone());

        let mut top_face = Vec::with_capacity(face.len());
        for &index in face.iter().rev() {
            top_face.push(index + offset);
        }
        faces.push(top_face);

        for (current, next) in face
            .iter()
            .zip(face.iter().cycle().skip(1))
            .take(face.len())
        {
            faces.push(vec![*current, *next, *next + offset, *current + offset]);
        }
    }

    Ok(Value::Surface { vertices, faces })
}

/// Bepaalt of een polyline gesloten is door het eerste en laatste punt te vergelijken.
fn is_closed(polyline: &[[f64; 3]]) -> bool {
    if polyline.len() < 3 {
        return false;
    }
    points_equal(*polyline.first().unwrap(), *polyline.last().unwrap())
}

/// Result of processing boundary polylines for patch operations.
/// Tracks which polylines were auto-closed so warnings can be added to diagnostics.
#[derive(Debug, Default)]
struct PreparedBoundaryLoops {
    /// Closed polylines ready for patching (in [f64;3] format).
    closed_polylines: Vec<Vec<[f64; 3]>>,
    /// Indices of original polylines that were auto-closed (0-based).
    auto_closed_indices: Vec<usize>,
}

impl PreparedBoundaryLoops {
    /// Returns true if any polylines were auto-closed.
    fn has_auto_closed(&self) -> bool {
        !self.auto_closed_indices.is_empty()
    }

    /// Generates warning messages about auto-closed curves.
    /// Returns a vector of warning strings suitable for adding to diagnostics.
    fn auto_close_warnings(&self, component: &str) -> Vec<String> {
        if self.auto_closed_indices.is_empty() {
            return Vec::new();
        }
        let count = self.auto_closed_indices.len();
        if count == 1 {
            vec![format!(
                "{}: curve {} was open and has been automatically closed; \
                 provide closed curves to avoid unexpected surfaces",
                component,
                self.auto_closed_indices[0] + 1 // 1-based for user display
            )]
        } else if count <= 5 {
            let indices_str = self.auto_closed_indices
                .iter()
                .map(|i| (i + 1).to_string()) // 1-based for user display
                .collect::<Vec<_>>()
                .join(", ");
            vec![format!(
                "{}: curves [{}] were open and have been automatically closed; \
                 provide closed curves to avoid unexpected surfaces",
                component, indices_str
            )]
        } else {
            vec![format!(
                "{}: {} curves were open and have been automatically closed; \
                 provide closed curves to avoid unexpected surfaces",
                component, count
            )]
        }
    }
}

/// Prepares boundary polylines for patch operations by ensuring they are closed.
///
/// For each input polyline with >= 3 points:
/// - If already closed, it is kept as-is.
/// - If open, it is auto-closed by appending the first point, and tracked for warnings.
///
/// # Arguments
/// * `polylines` - Input boundary polylines from segment grouping.
///
/// # Returns
/// A `PreparedBoundaryLoops` containing the closed polylines and indices of auto-closed curves.
///
/// # Note
/// This function deliberately auto-closes open curves to match Grasshopper's Patch behavior,
/// but tracks them so warnings can be emitted. Users should ideally provide closed curves.
fn prepare_boundary_loops_for_patch(polylines: &[Vec<[f64; 3]>]) -> PreparedBoundaryLoops {
    let mut result = PreparedBoundaryLoops::default();

    for (idx, polyline) in polylines.iter().enumerate() {
        if polyline.len() < 3 {
            // Skip polylines with fewer than 3 points (cannot form a valid boundary)
            continue;
        }

        let mut closed = polyline.clone();
        let was_open = !is_closed(&closed);

        if was_open {
            // Auto-close the polyline by adding the first point at the end
            closed.push(closed[0]);
            result.auto_closed_indices.push(idx);
        }

        // Need at least 4 points for a valid closed loop (3 unique + closure)
        if closed.len() >= 4 {
            result.closed_polylines.push(closed);
        }
    }

    result
}

/// Berekent de gemiddelde normaal van een polyline.
/// Dit wordt gedaan door de normaal te berekenen voor elk segment ten opzichte van het centroïde
/// en deze te middelen. Dit geeft een robuuste normaal, zelfs voor niet-vlakke polylines.
fn polyline_normal(polyline: &[[f64; 3]]) -> [f64; 3] {
    if polyline.len() < 3 {
        return [0.0, 0.0, 1.0]; // Standaard Z-as voor onvoldoende punten
    }

    let centroid = polyline.iter().fold([0.0; 3], |acc, p| add_vector(acc, *p));
    let n = polyline.len() as f64;
    let centroid = [centroid[0] / n, centroid[1] / n, centroid[2] / n];

    let mut normal = [0.0; 3];
    for i in 0..polyline.len() {
        let p1 = polyline[i];
        let p2 = polyline[(i + 1) % polyline.len()];
        let v1 = subtract_points(p1, centroid);
        let v2 = subtract_points(p2, centroid);
        normal = add_vector(normal, cross_product(v1, v2));
    }

    normalize(normal)
}

/// Bepaalt de oriëntatie (winding direction) van een gesloten, vlakke polyline.
/// Retourneert een positieve waarde voor tegen de klok in (CCW), negatief voor met de klok mee (CW),
/// en nul als de oriëntatie niet bepaald kan worden.
fn polyline_winding_direction(polyline: &[[f64; 3]], normal: [f64; 3]) -> f64 {
    if polyline.len() < 3 {
        return 0.0;
    }

    let mut area_sum = 0.0;
    for i in 0..polyline.len() {
        let p1 = polyline[i];
        let p2 = polyline[(i + 1) % polyline.len()];
        let cross = cross_product(p1, p2);
        area_sum += dot_product(cross, normal);
    }

    area_sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_list_lengths_grasshopper_single_value_broadcast() {
        // Single radius value should broadcast to all parameter positions
        let params = vec![0.0, 0.5, 1.0];
        let radii = vec![1.0];

        let (matched_params, matched_radii) = match_list_lengths_grasshopper(params.clone(), radii);

        assert_eq!(matched_params.len(), 3);
        assert_eq!(matched_radii.len(), 3);
        assert_eq!(matched_radii, vec![1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_match_list_lengths_grasshopper_shorter_list_repeats_last() {
        // Shorter list's last value should repeat for remaining positions
        let params = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let radii = vec![1.0, 2.0, 3.0];

        let (matched_params, matched_radii) = match_list_lengths_grasshopper(params.clone(), radii);

        assert_eq!(matched_params.len(), 5);
        assert_eq!(matched_radii.len(), 5);
        assert_eq!(matched_radii, vec![1.0, 2.0, 3.0, 3.0, 3.0]);
    }

    #[test]
    fn test_match_list_lengths_grasshopper_equal_lengths_unchanged() {
        // Equal-length lists should remain unchanged
        let params = vec![0.0, 0.5, 1.0];
        let radii = vec![1.0, 2.0, 3.0];

        let (matched_params, matched_radii) = match_list_lengths_grasshopper(params.clone(), radii.clone());

        assert_eq!(matched_params, params);
        assert_eq!(matched_radii, radii);
    }

    #[test]
    fn test_match_list_lengths_grasshopper_shorter_params_extended() {
        // When parameters are shorter, they should be extended
        let params = vec![0.0];
        let radii = vec![1.0, 2.0, 3.0];

        let (matched_params, matched_radii) = match_list_lengths_grasshopper(params, radii.clone());

        assert_eq!(matched_params.len(), 3);
        assert_eq!(matched_radii.len(), 3);
        assert_eq!(matched_params, vec![0.0, 0.0, 0.0]);
        assert_eq!(matched_radii, radii);
    }

    #[test]
    fn test_match_list_lengths_grasshopper_empty_lists_unchanged() {
        // Empty lists should be returned as-is (caller validates)
        let params: Vec<f64> = vec![];
        let radii = vec![1.0];

        let (matched_params, matched_radii) = match_list_lengths_grasshopper(params, radii.clone());

        assert_eq!(matched_params.len(), 0);
        assert_eq!(matched_radii, radii);
    }

    // -------------------------------------------------------------------------
    // FourPointSurface input validation tests
    // -------------------------------------------------------------------------

    use crate::graph::node::MetaMap;

    #[test]
    fn test_four_point_surface_rejects_empty_inputs() {
        // No inputs at all should produce an error, not a surface with [0,0,0] defaults
        let inputs: Vec<Value> = vec![];
        let meta = MetaMap::new();
        let result = evaluate_four_point_surface(&inputs, &meta);
        assert!(result.is_err(), "should reject empty inputs");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("3"),
            "error should mention minimum 3 points: {}",
            err
        );
    }

    #[test]
    fn test_four_point_surface_rejects_single_point() {
        // Only one point provided, rest should NOT default to [0,0,0]
        let inputs = vec![Value::Point([1.0, 2.0, 3.0])];
        let meta = MetaMap::new();
        let result = evaluate_four_point_surface(&inputs, &meta);
        assert!(result.is_err(), "should reject single point input");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("1"),
            "error should mention that only 1 point was received: {}",
            err
        );
    }

    #[test]
    fn test_four_point_surface_rejects_two_points() {
        // Only two points provided
        let inputs = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
        ];
        let meta = MetaMap::new();
        let result = evaluate_four_point_surface(&inputs, &meta);
        assert!(result.is_err(), "should reject two points input");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("2"),
            "error should mention that only 2 points were received: {}",
            err
        );
    }

    #[test]
    fn test_four_point_surface_rejects_coincident_points() {
        // Three identical points should be rejected as degenerate
        let inputs = vec![
            Value::Point([1.0, 2.0, 3.0]),
            Value::Point([1.0, 2.0, 3.0]),
            Value::Point([1.0, 2.0, 3.0]),
        ];
        let meta = MetaMap::new();
        let result = evaluate_four_point_surface(&inputs, &meta);
        assert!(result.is_err(), "should reject coincident points");
        let err = result.unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("coincident")
                || err.to_string().to_lowercase().contains("samenvallend"),
            "error should mention coincident points: {}",
            err
        );
    }

    #[test]
    fn test_four_point_surface_rejects_collinear_three_points() {
        // Three collinear points cannot form a valid surface
        let inputs = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([2.0, 0.0, 0.0]), // All on the X axis
        ];
        let meta = MetaMap::new();
        let result = evaluate_four_point_surface(&inputs, &meta);
        assert!(result.is_err(), "should reject collinear points");
        let err = result.unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("colline")
                || err.to_string().to_lowercase().contains("lijn"),
            "error should mention collinear points: {}",
            err
        );
    }

    #[test]
    fn test_four_point_surface_accepts_valid_three_points() {
        // Three non-collinear points should work (fourth is computed)
        let inputs = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
        ];
        let meta = MetaMap::new();
        let result = evaluate_four_point_surface(&inputs, &meta);
        assert!(result.is_ok(), "should accept 3 valid points");
    }

    #[test]
    fn test_four_point_surface_accepts_valid_four_points() {
        // Four non-degenerate points should work
        let inputs = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
            Value::Point([1.0, 1.0, 0.0]),
        ];
        let meta = MetaMap::new();
        let result = evaluate_four_point_surface(&inputs, &meta);
        assert!(result.is_ok(), "should accept 4 valid points");
    }

    #[test]
    fn test_four_point_surface_rejects_degenerate_quad_with_collinear_subset() {
        // A quad where 3 of 4 points are collinear is degenerate
        let inputs = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([2.0, 0.0, 0.0]), // Collinear with first two
            Value::Point([0.0, 1.0, 0.0]), // Not collinear
        ];
        let meta = MetaMap::new();
        let result = evaluate_four_point_surface(&inputs, &meta);
        assert!(result.is_err(), "should reject quad with collinear subset");
    }

    #[test]
    fn test_four_point_surface_ignores_null_inputs() {
        // Null values in input slots should NOT count as points
        let inputs = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Null,
            Value::Point([1.0, 0.0, 0.0]),
            Value::Null,
        ];
        let meta = MetaMap::new();
        // Only 2 actual points provided
        let result = evaluate_four_point_surface(&inputs, &meta);
        assert!(result.is_err(), "should reject when nulls leave insufficient points");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("2"),
            "error should mention that only 2 points were received: {}",
            err
        );
    }

    #[test]
    fn test_four_point_surface_with_mixed_null_and_valid() {
        // Three valid points with one null should work
        let inputs = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Null,
            Value::Point([0.0, 1.0, 0.0]),
        ];
        let meta = MetaMap::new();
        let result = evaluate_four_point_surface(&inputs, &meta);
        assert!(result.is_ok(), "should accept 3 valid points even with null in between");
    }

    // -------------------------------------------------------------------------
    // Quality settings tests for surface builder components
    // -------------------------------------------------------------------------

    use crate::graph::node::MetaValue;

    /// Helper to count vertices in the mesh output
    fn count_mesh_vertices(result: &ComponentResult) -> usize {
        let out = result.as_ref().unwrap();
        if let Some(Value::Mesh { vertices, .. }) = out.get(PIN_OUTPUT_MESH) {
            vertices.len()
        } else if let Some(Value::Surface { vertices, .. }) = out.get(PIN_OUTPUT_SURFACE) {
            vertices.len()
        } else {
            0
        }
    }

    #[test]
    fn test_four_point_surface_respects_quality_settings() {
        // Test that different quality presets produce different mesh densities
        let inputs = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([10.0, 0.0, 0.0]),
            Value::Point([0.0, 10.0, 0.0]),
            Value::Point([10.0, 10.0, 0.0]),
        ];

        // Low quality: 4×4 = 16 vertices expected (4 subdivisions)
        let mut meta_low = MetaMap::new();
        meta_low.insert("mesh_quality".to_string(), MetaValue::Text("low".to_string()));
        let result_low = evaluate_four_point_surface(&inputs, &meta_low);
        assert!(result_low.is_ok());
        let verts_low = count_mesh_vertices(&result_low);

        // High quality: 20×20 = 400 vertices expected (20 subdivisions)
        let mut meta_high = MetaMap::new();
        meta_high.insert("mesh_quality".to_string(), MetaValue::Text("high".to_string()));
        let result_high = evaluate_four_point_surface(&inputs, &meta_high);
        assert!(result_high.is_ok());
        let verts_high = count_mesh_vertices(&result_high);

        // High quality should produce more vertices than low quality
        assert!(
            verts_high > verts_low,
            "high quality ({} vertices) should produce more vertices than low quality ({} vertices)",
            verts_high,
            verts_low
        );
    }

    #[test]
    fn test_four_point_surface_respects_explicit_subdivisions() {
        // Test that explicit subdivision settings are respected
        let inputs = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([10.0, 0.0, 0.0]),
            Value::Point([0.0, 10.0, 0.0]),
            Value::Point([10.0, 10.0, 0.0]),
        ];

        // Use explicit u/v subdivision settings
        let mut meta = MetaMap::new();
        meta.insert("u_subdivisions".to_string(), MetaValue::Integer(5));
        meta.insert("v_subdivisions".to_string(), MetaValue::Integer(8));
        let result = evaluate_four_point_surface(&inputs, &meta);
        assert!(result.is_ok());
        
        let verts = count_mesh_vertices(&result);
        // With 5×8 subdivisions, we expect 5×8 = 40 vertices
        assert_eq!(
            verts, 40,
            "expected 5×8 = 40 vertices, got {}",
            verts
        );
    }

    #[test]
    fn test_four_point_surface_default_quality_without_meta() {
        // Test that default quality is used when no meta settings are provided
        let inputs = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([10.0, 0.0, 0.0]),
            Value::Point([0.0, 10.0, 0.0]),
            Value::Point([10.0, 10.0, 0.0]),
        ];

        // Empty meta - should use default (10×10)
        let meta = MetaMap::new();
        let result = evaluate_four_point_surface(&inputs, &meta);
        assert!(result.is_ok());
        
        let verts = count_mesh_vertices(&result);
        // Default is 10×10 = 100 vertices
        assert_eq!(
            verts, 100,
            "expected default 10×10 = 100 vertices, got {}",
            verts
        );
    }

    // -------------------------------------------------------------------------
    // Sum Surface data-matching tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sum_surface_single_curves() {
        // Single U and V curves should produce a single surface
        let u_curve = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([2.0, 0.0, 0.0]),
        ]);
        let v_curve = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
            Value::Point([0.0, 2.0, 0.0]),
        ]);
        let inputs = vec![u_curve, v_curve];
        let meta = MetaMap::new();

        let result = evaluate_sum_surface(&inputs, &meta);
        assert!(result.is_ok(), "should create sum surface from single curves");
        
        let out = result.unwrap();
        // Single input curves should produce a single surface, not a list
        assert!(
            !matches!(out.get(PIN_OUTPUT_SURFACE), Some(Value::List(_))),
            "single curve pair should not return a list"
        );
    }

    #[test]
    fn test_sum_surface_list_data_matching() {
        // When given lists of curves, Sum Surface should create multiple surfaces
        // using Grasshopper data-matching semantics

        // Two U curves (list of lists of points)
        let u_curves = Value::List(vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
            ]),
            Value::List(vec![
                Value::Point([0.0, 0.0, 5.0]),
                Value::Point([1.0, 0.0, 5.0]),
            ]),
        ]);

        // Two V curves (list of lists of points)
        let v_curves = Value::List(vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([0.0, 1.0, 0.0]),
            ]),
            Value::List(vec![
                Value::Point([0.0, 0.0, 5.0]),
                Value::Point([0.0, 1.0, 5.0]),
            ]),
        ]);

        let inputs = vec![u_curves, v_curves];
        let meta = MetaMap::new();

        let result = evaluate_sum_surface(&inputs, &meta);
        assert!(result.is_ok(), "should create sum surfaces from curve lists");

        let out = result.unwrap();
        // Should return a list of surfaces (data-matched pairs)
        match out.get(PIN_OUTPUT_SURFACE) {
            Some(Value::List(surfaces)) => {
                assert_eq!(
                    surfaces.len(),
                    2,
                    "should create 2 surfaces from 2 matched curve pairs"
                );
            }
            _ => panic!("expected a list of surfaces for matched curve pairs"),
        }
    }

    #[test]
    fn test_sum_surface_single_u_broadcast_to_multiple_v() {
        // Single U curve should be broadcast to match multiple V curves
        let u_curve = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
        ]);

        // Three V curves
        let v_curves = Value::List(vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([0.0, 1.0, 0.0]),
            ]),
            Value::List(vec![
                Value::Point([0.0, 0.0, 2.0]),
                Value::Point([0.0, 1.0, 2.0]),
            ]),
            Value::List(vec![
                Value::Point([0.0, 0.0, 4.0]),
                Value::Point([0.0, 1.0, 4.0]),
            ]),
        ]);

        let inputs = vec![u_curve, v_curves];
        let meta = MetaMap::new();

        let result = evaluate_sum_surface(&inputs, &meta);
        assert!(result.is_ok(), "should broadcast single U curve to multiple V curves");

        let out = result.unwrap();
        match out.get(PIN_OUTPUT_SURFACE) {
            Some(Value::List(surfaces)) => {
                assert_eq!(
                    surfaces.len(),
                    3,
                    "single U curve broadcast to 3 V curves should create 3 surfaces"
                );
            }
            _ => panic!("expected a list of surfaces when broadcasting"),
        }
    }

    #[test]
    fn test_sum_surface_unequal_lists_use_min_length() {
        // When both inputs are lists of different lengths, match up to the shorter length
        let u_curves = Value::List(vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([1.0, 0.0, 0.0]),
            ]),
            Value::List(vec![
                Value::Point([0.0, 0.0, 2.0]),
                Value::Point([1.0, 0.0, 2.0]),
            ]),
            Value::List(vec![
                Value::Point([0.0, 0.0, 4.0]),
                Value::Point([1.0, 0.0, 4.0]),
            ]),
        ]);

        // Only 2 V curves
        let v_curves = Value::List(vec![
            Value::List(vec![
                Value::Point([0.0, 0.0, 0.0]),
                Value::Point([0.0, 1.0, 0.0]),
            ]),
            Value::List(vec![
                Value::Point([0.0, 0.0, 2.0]),
                Value::Point([0.0, 1.0, 2.0]),
            ]),
        ]);

        let inputs = vec![u_curves, v_curves];
        let meta = MetaMap::new();

        let result = evaluate_sum_surface(&inputs, &meta);
        assert!(result.is_ok(), "should handle unequal list lengths");

        let out = result.unwrap();
        match out.get(PIN_OUTPUT_SURFACE) {
            Some(Value::List(surfaces)) => {
                assert_eq!(
                    surfaces.len(),
                    2,
                    "3 U curves + 2 V curves should match to min(3,2) = 2 surfaces"
                );
            }
            _ => panic!("expected a list of surfaces for mismatched lists"),
        }
    }

    #[test]
    fn test_sum_surface_rejects_empty_curves() {
        // Empty curve inputs should produce an error
        let u_curve = Value::List(vec![]);
        let v_curve = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
        ]);
        let inputs = vec![u_curve, v_curve];
        let meta = MetaMap::new();

        let result = evaluate_sum_surface(&inputs, &meta);
        assert!(result.is_err(), "should reject empty curve inputs");
    }

    #[test]
    fn test_sum_surface_rejects_single_point_curves() {
        // Curves with only one point should be rejected
        let u_curve = Value::List(vec![Value::Point([0.0, 0.0, 0.0])]);
        let v_curve = Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
        ]);
        let inputs = vec![u_curve, v_curve];
        let meta = MetaMap::new();

        let result = evaluate_sum_surface(&inputs, &meta);
        assert!(result.is_err(), "should reject single-point curves");
    }

    // -------------------------------------------------------------------------
    // coerce_optional_positive_integer tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_coerce_optional_positive_integer_none_returns_none() {
        let result = coerce_optional_positive_integer(None, "Test", "input");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_coerce_optional_positive_integer_null_returns_none() {
        let result = coerce_optional_positive_integer(Some(&Value::Null), "Test", "input");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_coerce_optional_positive_integer_valid_positive() {
        let result = coerce_optional_positive_integer(Some(&Value::Number(5.0)), "Test", "input");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(5));
    }

    #[test]
    fn test_coerce_optional_positive_integer_zero() {
        let result = coerce_optional_positive_integer(Some(&Value::Number(0.0)), "Test", "input");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(0));
    }

    #[test]
    fn test_coerce_optional_positive_integer_rounds_half_up() {
        // 3.5 should round to 4
        let result = coerce_optional_positive_integer(Some(&Value::Number(3.5)), "Test", "input");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(4));
    }

    #[test]
    fn test_coerce_optional_positive_integer_rounds_down() {
        // 3.4 should round to 3
        let result = coerce_optional_positive_integer(Some(&Value::Number(3.4)), "Test", "input");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(3));
    }

    #[test]
    fn test_coerce_optional_positive_integer_negative_errors() {
        let result = coerce_optional_positive_integer(Some(&Value::Number(-1.0)), "Test", "U Count");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("niet-negatief"),
            "error should mention non-negative requirement: {}",
            err
        );
    }

    #[test]
    fn test_coerce_optional_positive_integer_nan_errors() {
        let result = coerce_optional_positive_integer(Some(&Value::Number(f64::NAN)), "Test", "U Count");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("NaN"),
            "error should mention NaN: {}",
            err
        );
    }

    #[test]
    fn test_coerce_optional_positive_integer_positive_infinity_errors() {
        let result = coerce_optional_positive_integer(Some(&Value::Number(f64::INFINITY)), "Test", "U Count");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("oneindig"),
            "error should mention infinity: {}",
            err
        );
    }

    #[test]
    fn test_coerce_optional_positive_integer_negative_infinity_errors() {
        let result = coerce_optional_positive_integer(Some(&Value::Number(f64::NEG_INFINITY)), "Test", "U Count");
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Should error on infinity before reaching the negative check
        assert!(
            err.to_string().contains("oneindig"),
            "error should mention infinity: {}",
            err
        );
    }

    #[test]
    fn test_coerce_optional_positive_integer_huge_value_errors() {
        // Values above MAX_REASONABLE_GRID_DIM should error
        let result = coerce_optional_positive_integer(Some(&Value::Number(2_000_000.0)), "Test", "U Count");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("te groot"),
            "error should mention value is too large: {}",
            err
        );
    }

    #[test]
    fn test_coerce_optional_positive_integer_invalid_type_errors() {
        // String input should produce a type error (not silently become None)
        let result = coerce_optional_positive_integer(Some(&Value::Text("five".to_string())), "Test", "U Count");
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Should mention expected a number
        assert!(
            err.to_string().contains("getal"),
            "error should mention expected number: {}",
            err
        );
    }

    #[test]
    fn test_coerce_optional_positive_integer_list_unwraps_single() {
        // Single-element list should unwrap to the value inside
        let result = coerce_optional_positive_integer(
            Some(&Value::List(vec![Value::Number(7.0)])),
            "Test",
            "input",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(7));
    }
}
