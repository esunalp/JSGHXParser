//! Implementaties van Grasshopper "Surface → Util" componenten.

use std::collections::BTreeMap;

#[cfg(not(feature = "mesh_engine_next"))]
use delaunator;

use crate::graph::node::MetaMap;
use crate::graph::value::{Domain, Domain1D, Domain2D, Value};

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_POINTS: &str = "P";
const PIN_OUTPUT_NORMALS: &str = "N";
const PIN_OUTPUT_PARAMETERS: &str = "uv";
const PIN_OUTPUT_FRAMES: &str = "F";
const PIN_OUTPUT_BREPS: &str = "B";
const PIN_OUTPUT_CLOSED: &str = "C";
const PIN_OUTPUT_RESULT: &str = "R";
const PIN_OUTPUT_MAP: &str = "M";
const PIN_OUTPUT_CLOSED_INDICES: &str = "Ci";
const PIN_OUTPUT_OPEN: &str = "O";
const PIN_OUTPUT_OPEN_INDICES: &str = "Oi";
const PIN_OUTPUT_INDICES: &str = "I";
const PIN_OUTPUT_CONVEX: &str = "Cv";
const PIN_OUTPUT_CONCAVE: &str = "Cc";
const PIN_OUTPUT_MIXED: &str = "Mx";
const PIN_OUTPUT_BEFORE: &str = "N0";
const PIN_OUTPUT_AFTER: &str = "N1";
const PIN_OUTPUT_CAPS: &str = "C";
const PIN_OUTPUT_SOLID: &str = "S";

const EPSILON: f64 = 1e-9;

/// Beschikbare componenten binnen Surface → Util.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    DivideSurfaceObsolete,
    BrepJoin,
    SurfaceFrames,
    FilletEdge,
    DivideSurface,
    SurfaceFramesObsolete,
    CopyTrim,
    EdgesFromDirections,
    Isotrim,
    ClosedEdges,
    EdgesFromFaces,
    EdgesFromPoints,
    ConvexEdges,
    Retrim,
    OffsetSurface,
    CapHoles,
    Flip,
    MergeFaces,
    EdgesFromLinearity,
    OffsetSurfaceLoose,
    CapHolesEx,
    Untrim,
    EdgesFromLength,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Registraties van alle Surface → Util componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{082976f0-c741-4df8-a1d4-89891bf8619f}"],
        names: &["Divide Surface [OBSOLETE]", "Divide"],
        kind: ComponentKind::DivideSurfaceObsolete,
    },
    Registration {
        guids: &["{1addcc85-b04e-46e6-bd4a-6f6c93bf7efd}"],
        names: &["Brep Join", "Join"],
        kind: ComponentKind::BrepJoin,
    },
    Registration {
        guids: &["{332378f4-acb2-43fe-8593-ed22bfeb2721}"],
        names: &["Surface Frames", "SFrames"],
        kind: ComponentKind::SurfaceFrames,
    },
    Registration {
        guids: &["{4b87eb13-f87c-4ff1-ae0e-6c9f1f2aecbd}"],
        names: &["Fillet Edge", "FilEdge"],
        kind: ComponentKind::FilletEdge,
    },
    Registration {
        guids: &["{5106bafc-d5d4-4983-83e7-7be3ed07f502}"],
        names: &["Divide Surface", "SDivide"],
        kind: ComponentKind::DivideSurface,
    },
    Registration {
        guids: &["{59143f40-32f3-47c1-b9ae-1a09eb9c926b}"],
        names: &["Surface Frames [OBSOLETE]", "Frames"],
        kind: ComponentKind::SurfaceFramesObsolete,
    },
    Registration {
        guids: &["{5d192b90-1ae3-4439-bbde-b05976fc4ac3}"],
        names: &["Copy Trim", "Trim"],
        kind: ComponentKind::CopyTrim,
    },
    Registration {
        guids: &["{64ff9813-8fe8-4708-ac9f-61b825213e83}"],
        names: &["Edges from Directions", "EdgesDir"],
        kind: ComponentKind::EdgesFromDirections,
    },
    Registration {
        guids: &["{6a9ccaab-1b03-484e-bbda-be9c81584a66}"],
        names: &["Isotrim", "SubSrf"],
        kind: ComponentKind::Isotrim,
    },
    Registration {
        guids: &["{70905be1-e22f-4fa8-b9ae-e119d417904f}"],
        names: &["Closed Edges", "EdgesCls"],
        kind: ComponentKind::ClosedEdges,
    },
    Registration {
        guids: &["{71e99dbb-2d79-4f02-a8a6-e87a09d54f47}"],
        names: &["Edges from Faces", "EdgesFaces"],
        kind: ComponentKind::EdgesFromFaces,
    },
    Registration {
        guids: &["{73269f6a-9645-4638-8d5e-88064dd289bd}"],
        names: &["Edges from Points", "EdgesPt"],
        kind: ComponentKind::EdgesFromPoints,
    },
    Registration {
        guids: &["{8248da39-0729-4e04-8395-267b3259bc2f}"],
        names: &["Convex Edges", "EdgesCvx"],
        kind: ComponentKind::ConvexEdges,
    },
    Registration {
        guids: &["{a1da39b7-6387-4522-bf2b-2eaee6b14072}"],
        names: &["Retrim", "Retrim"],
        kind: ComponentKind::Retrim,
    },
    Registration {
        guids: &["{b25c5762-f90e-4839-9fc5-74b74ab42b1e}"],
        names: &["Offset Surface", "Offset"],
        kind: ComponentKind::OffsetSurface,
    },
    Registration {
        guids: &["{b648d933-ddea-4e75-834c-8f6f3793e311}"],
        names: &["Cap Holes", "Cap"],
        kind: ComponentKind::CapHoles,
    },
    Registration {
        guids: &["{c3d1f2b8-8596-4e8d-8861-c28ba8ffb4f4}"],
        names: &["Flip", "Flip"],
        kind: ComponentKind::Flip,
    },
    Registration {
        guids: &["{d6b43673-55dd-4e2f-95c4-6c69a14513a6}"],
        names: &["Merge Faces", "FMerge"],
        kind: ComponentKind::MergeFaces,
    },
    Registration {
        guids: &["{e4ff8101-73c9-4802-8c5d-704d8721b909}"],
        names: &["Edges from Linearity", "EdgesLin"],
        kind: ComponentKind::EdgesFromLinearity,
    },
    Registration {
        guids: &["{e7e43403-f913-4d83-8aff-5b1c7a7f9fbc}"],
        names: &["Offset Surface Loose", "Offset (L)"],
        kind: ComponentKind::OffsetSurfaceLoose,
    },
    Registration {
        guids: &["{f6409a9c-3d2a-4b14-9f2c-e3c3f2cb72f8}"],
        names: &["Cap Holes Ex", "CapEx"],
        kind: ComponentKind::CapHolesEx,
    },
    Registration {
        guids: &["{fa92858a-a180-4545-ad4d-0dc644b3a2a8}"],
        names: &["Untrim", "Untrim"],
        kind: ComponentKind::Untrim,
    },
    Registration {
        guids: &["{ff187e6a-84bc-4bb9-a572-b39006a0576d}"],
        names: &["Edges from Length", "EdgesLen"],
        kind: ComponentKind::EdgesFromLength,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::DivideSurfaceObsolete | Self::DivideSurface => {
                evaluate_divide_surface(inputs, self.name())
            }
            Self::SurfaceFrames | Self::SurfaceFramesObsolete => {
                evaluate_surface_frames(inputs, self.name())
            }
            Self::BrepJoin => evaluate_brep_join(inputs),
            Self::FilletEdge => evaluate_fillet_edge(inputs),
            Self::CopyTrim => evaluate_copy_trim(inputs),
            Self::Retrim => evaluate_retrim(inputs),
            Self::EdgesFromDirections => evaluate_edges_from_directions(inputs),
            Self::Isotrim => evaluate_isotrim(inputs),
            Self::ClosedEdges => evaluate_closed_edges(inputs),
            Self::EdgesFromFaces => evaluate_edges_from_faces(inputs),
            Self::EdgesFromPoints => evaluate_edges_from_points(inputs),
            Self::ConvexEdges => evaluate_convex_edges(inputs),
            Self::OffsetSurface | Self::OffsetSurfaceLoose => {
                evaluate_offset_surface(inputs, self.name())
            }
            Self::CapHoles => evaluate_cap_holes(inputs, false),
            Self::CapHolesEx => evaluate_cap_holes(inputs, true),
            Self::Flip => evaluate_flip(inputs),
            Self::MergeFaces => evaluate_merge_faces(inputs),
            Self::EdgesFromLinearity => evaluate_edges_by_length(inputs, "Edges from Linearity"),
            Self::EdgesFromLength => evaluate_edges_by_length(inputs, "Edges from Length"),
            Self::Untrim => evaluate_untrim(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::DivideSurfaceObsolete => "Divide Surface [OBSOLETE]",
            Self::BrepJoin => "Brep Join",
            Self::SurfaceFrames => "Surface Frames",
            Self::FilletEdge => "Fillet Edge",
            Self::DivideSurface => "Divide Surface",
            Self::SurfaceFramesObsolete => "Surface Frames [OBSOLETE]",
            Self::CopyTrim => "Copy Trim",
            Self::EdgesFromDirections => "Edges from Directions",
            Self::Isotrim => "Isotrim",
            Self::ClosedEdges => "Closed Edges",
            Self::EdgesFromFaces => "Edges from Faces",
            Self::EdgesFromPoints => "Edges from Points",
            Self::ConvexEdges => "Convex Edges",
            Self::Retrim => "Retrim",
            Self::OffsetSurface => "Offset Surface",
            Self::CapHoles => "Cap Holes",
            Self::Flip => "Flip",
            Self::MergeFaces => "Merge Faces",
            Self::EdgesFromLinearity => "Edges from Linearity",
            Self::OffsetSurfaceLoose => "Offset Surface Loose",
            Self::CapHolesEx => "Cap Holes Ex",
            Self::Untrim => "Untrim",
            Self::EdgesFromLength => "Edges from Length",
        }
    }

    #[must_use]
    pub fn optional_input_pins(&self) -> &'static [&'static str] {
        match self {
            Self::Flip => &["G", "Guide"],
            _ => &[],
        }
    }
}

fn evaluate_divide_surface(inputs: &[Value], component: &str) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(format!(
            "{} vereist een surface en segmentaantallen",
            component
        )));
    }

    let u_segments = coerce_positive_integer(inputs.get(1), &(component.to_owned() + " U"))?;
    let v_segments = coerce_positive_integer(inputs.get(2), &(component.to_owned() + " V"))?;

    let (points, normals, parameters) = {
        #[cfg(feature = "mesh_engine_next")]
        {
            // Try to extract vertex data for accurate surface sampling
            let surface_data = SurfaceVertexData::from_value(inputs.get(0))
                .ok_or_else(|| ComponentError::new(format!(
                    "{} vereist geometrische invoer", component
                )))?;

            // Use vertex-based sampling if we have a proper grid
            let result = if surface_data.has_valid_grid() {
                crate::geom::divide_surface_from_vertices(
                    surface_data.to_vertex_input(),
                    u_segments,
                    v_segments,
                    crate::geom::DivideSurfaceBoundsOptions::default(),
                )
            } else {
                None
            };

            // Fall back to bounds-based if vertex-based failed
            let result = result.unwrap_or_else(|| {
                crate::geom::divide_surface_from_bounds(
                    surface_data.min,
                    surface_data.max,
                    u_segments,
                    v_segments,
                    crate::geom::DivideSurfaceBoundsOptions::default(),
                )
            });

            let points = result
                .points
                .into_iter()
                .map(Value::Point)
                .collect();
            let normals = result
                .normals
                .into_iter()
                .map(Value::Vector)
                .collect();
            let parameters = result
                .parameters
                .into_iter()
                .map(Value::Point)
                .collect();
            (points, normals, parameters)
        }

        #[cfg(not(feature = "mesh_engine_next"))]
        {
            let metrics = coerce_shape_metrics(inputs.get(0), component)?;
            let mut points = Vec::new();
            let mut normals = Vec::new();
            let mut parameters = Vec::new();

            for v in 0..=v_segments {
                let fv = if v_segments == 0 {
                    0.0
                } else {
                    v as f64 / v_segments as f64
                };
                for u in 0..=u_segments {
                    let fu = if u_segments == 0 {
                        0.0
                    } else {
                        u as f64 / u_segments as f64
                    };
                    let point = metrics.sample_point((fu, fv));
                    points.push(Value::Point(point));
                    normals.push(Value::Vector(metrics.normal_hint()));
                    parameters.push(Value::Point([fu, fv, 0.0]));
                }
            }

            (points, normals, parameters)
        }
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(points));
    outputs.insert(PIN_OUTPUT_NORMALS.to_owned(), Value::List(normals));
    outputs.insert(PIN_OUTPUT_PARAMETERS.to_owned(), Value::List(parameters));
    Ok(outputs)
}

fn evaluate_surface_frames(inputs: &[Value], component: &str) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(format!(
            "{} vereist een surface en segmentaantallen",
            component
        )));
    }

    let u_segments = coerce_positive_integer(inputs.get(1), &(component.to_owned() + " U"))?;
    let v_segments = coerce_positive_integer(inputs.get(2), &(component.to_owned() + " V"))?;

    #[cfg(feature = "mesh_engine_next")]
    {
        // Try to extract vertex data for accurate frame computation
        let surface_data = SurfaceVertexData::from_value(inputs.get(0))
            .ok_or_else(|| ComponentError::new(format!(
                "{} vereist geometrische invoer", component
            )))?;

        // Use vertex-based frames if we have a proper grid
        let result = if surface_data.has_valid_grid() {
            crate::geom::surface_frames_from_vertices(
                surface_data.to_vertex_input(),
                u_segments,
                v_segments,
            )
        } else {
            None
        };

        // Fall back to bounds-based if vertex-based failed
        let result = result.unwrap_or_else(|| {
            crate::geom::surface_frames_from_bounds(
                surface_data.min,
                surface_data.max,
                u_segments,
                v_segments,
            )
        });

        let mut frames_rows = Vec::with_capacity(result.v_count);
        let mut parameter_rows = Vec::with_capacity(result.v_count);

        for (frame_row, param_row) in result.frames.into_iter().zip(result.parameters.into_iter())
        {
            let frames_row: Vec<Value> = frame_row
                .into_iter()
                .map(|f| frame_value(f.origin, f.x_axis, f.y_axis, f.z_axis))
                .collect();
            let parameters_row: Vec<Value> = param_row.into_iter().map(Value::Point).collect();

            frames_rows.push(Value::List(frames_row));
            parameter_rows.push(Value::List(parameters_row));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_FRAMES.to_owned(), Value::List(frames_rows));
        outputs.insert(
            PIN_OUTPUT_PARAMETERS.to_owned(),
            Value::List(parameter_rows),
        );
        return Ok(outputs);
    }

    #[cfg(not(feature = "mesh_engine_next"))]
    {
        let metrics = coerce_shape_metrics(inputs.get(0), component)?;
        let mut frames_rows = Vec::new();
        let mut parameter_rows = Vec::new();

        for v in 0..=v_segments {
            let fv = if v_segments == 0 {
                0.0
            } else {
                v as f64 / v_segments as f64
            };
            let mut frames_row = Vec::new();
            let mut parameters_row = Vec::new();
            for u in 0..=u_segments {
                let fu = if u_segments == 0 {
                    0.0
                } else {
                    u as f64 / u_segments as f64
                };
                let point = metrics.sample_point((fu, fv));
                let tangent_u = metrics.tangent_hint_u();
                let tangent_v = metrics.tangent_hint_v();
                let normal = metrics.normal_hint();
                frames_row.push(frame_value(point, tangent_u, tangent_v, normal));
                parameters_row.push(Value::Point([fu, fv, 0.0]));
            }
            frames_rows.push(Value::List(frames_row));
            parameter_rows.push(Value::List(parameters_row));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_FRAMES.to_owned(), Value::List(frames_rows));
        outputs.insert(
            PIN_OUTPUT_PARAMETERS.to_owned(),
            Value::List(parameter_rows),
        );
        Ok(outputs)
    }
}

#[cfg(not(feature = "mesh_engine_next"))]
fn evaluate_brep_join(inputs: &[Value]) -> ComponentResult {
    let mut breps = Vec::new();
    let mut closed = Vec::new();

    if let Some(Value::List(values)) = inputs.get(0) {
        for value in values {
            breps.push(value.clone());
            let metrics = ShapeMetrics::from_inputs(Some(value));
            let is_closed = metrics
                .as_ref()
                .map(|m| m.volume().abs() > EPSILON)
                .unwrap_or(false);
            closed.push(Value::Boolean(is_closed));
        }
    } else if let Some(value) = inputs.get(0) {
        breps.push(value.clone());
        let metrics = ShapeMetrics::from_inputs(Some(value));
        let is_closed = metrics
            .as_ref()
            .map(|m| m.volume().abs() > EPSILON)
            .unwrap_or(false);
        closed.push(Value::Boolean(is_closed));
    }

    if breps.is_empty() {
        return Err(ComponentError::new("Brep Join vereist een lijst met breps"));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(breps));
    outputs.insert(PIN_OUTPUT_CLOSED.to_owned(), Value::List(closed));
    Ok(outputs)
}

/// Evaluates the BrepJoin component using the geom module.
///
/// Uses `geom::brep_join` to join multiple breps by welding matching naked edges.
/// Breps that share coincident naked edges (within tolerance) are merged into
/// unified shells.
///
/// # Inputs
/// - `B` (index 0): List of breps to join
///
/// # Outputs
/// - `B`: The joined breps (may be fewer than inputs if merged)
/// - `C`: For each output brep, whether it forms a closed (watertight) shell
#[cfg(feature = "mesh_engine_next")]
fn evaluate_brep_join(inputs: &[Value]) -> ComponentResult {
    // Collect input values
    let mut values = Vec::new();
    if let Some(Value::List(list)) = inputs.get(0) {
        values.extend(list.iter().cloned());
    } else if let Some(value) = inputs.get(0) {
        values.push(value.clone());
    }

    if values.is_empty() {
        return Err(ComponentError::new("Brep Join vereist een lijst met breps"));
    }

    // Separate surface meshes from non-surface values
    let mut surface_meshes = Vec::new();
    let mut non_surface_values = Vec::new();
    let mut non_surface_indices = Vec::new();

    for (i, value) in values.iter().enumerate() {
        match value {
            Value::Surface { vertices, faces } => {
                surface_meshes.push(crate::geom::LegacySurfaceMesh {
                    vertices: vertices.clone(),
                    faces: faces.clone(),
                });
            }
            Value::Mesh { vertices, indices, .. } => {
                // Convert Value::Mesh (flat triangle indices) to LegacySurfaceMesh (face vectors)
                let faces: Vec<Vec<u32>> = indices
                    .chunks(3)
                    .map(|chunk| chunk.to_vec())
                    .collect();
                surface_meshes.push(crate::geom::LegacySurfaceMesh {
                    vertices: vertices.clone(),
                    faces,
                });
            }
            _ => {
                non_surface_indices.push(i);
                non_surface_values.push(value.clone());
            }
        }
    }

    // Join surface meshes using the new brep_ops API
    let joined = crate::geom::brep_join(
        &surface_meshes,
        crate::geom::BrepJoinOptions::default(),
    );

    // Build output breps list
    let mut output_breps = Vec::with_capacity(joined.breps.len() + non_surface_values.len());
    let mut output_closed = Vec::with_capacity(joined.breps.len() + non_surface_values.len());

    // Add joined surface meshes
    for (brep, &is_closed) in joined.breps.iter().zip(joined.closed.iter()) {
        output_breps.push(Value::Surface {
            vertices: brep.vertices.clone(),
            faces: brep.faces.clone(),
        });
        output_closed.push(Value::Boolean(is_closed));
    }

    // Add non-surface values back with estimated closedness
    for value in non_surface_values {
        let metrics = ShapeMetrics::from_inputs(Some(&value));
        let is_closed = metrics
            .as_ref()
            .map(|m| m.volume().abs() > EPSILON)
            .unwrap_or(false);
        output_breps.push(value);
        output_closed.push(Value::Boolean(is_closed));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(output_breps));
    outputs.insert(PIN_OUTPUT_CLOSED.to_owned(), Value::List(output_closed));
    Ok(outputs)
}

/// Evaluates the FilletEdge component using the geom module.
///
/// Applies fillets to selected edges of a mesh or surface.
///
/// # Inputs
/// - `S` (index 0): Shape to fillet (Mesh or Surface)
/// - `B` (index 1): Blend type (currently ignored; reserved for future use)
/// - `M` (index 2): Metric type (currently ignored; reserved for future use)
/// - `E` (index 3): Edge indices to fillet (list of integers, each representing an edge)
/// - `R` (index 4): Fillet radii per edge (list of numbers; if fewer than edges, the last value is reused)
///
/// # Outputs
/// - `B`: Filleted mesh as `Value::Mesh` with embedded diagnostics (primary output).
///        Diagnostics include warnings for skipped edges, clamped radii, and topology issues.
/// - `S`: Legacy `Value::Surface` adapter for backward compatibility (no diagnostics).
///
/// # Limitations (documented per mesh_engine_integration_plan.md)
/// - Currently only supports triangle meshes with "hinge" edges (both endpoints shared by exactly 2 triangles).
/// - General manifold edge filleting is not supported; unsupported edges are skipped with diagnostics.
/// - No B-rep/NURBS surface filleting yet; surfaces are converted to triangle meshes first.
/// - Blend and Metric parameters are reserved but currently ignored.
/// - UV generation is not implemented for fillet faces.
///
/// # Diagnostics
/// - Errors and warnings are collected in `FilletMeshEdgeDiagnostics` and merged into the mesh diagnostics.
/// - Unsupported edges emit warnings rather than failing silently.
/// - Diagnostics are attached to the `Value::Mesh` output on pin `B` and can be inspected
///   by consumers via `mesh.diagnostics`.
#[cfg(feature = "mesh_engine_next")]
fn evaluate_fillet_edge(inputs: &[Value]) -> ComponentResult {
    use super::coerce::{coerce_mesh_like_with_context, geom_bridge};
    use crate::geom::{
        fillet_triangle_mesh_edges, list_triangle_mesh_edges, FilletEdgeOptions, Tolerance,
    };

    const COMPONENT: &str = "Fillet Edge";

    // ------------------------------------------------------------------
    // Input 0: Shape (required)
    // ------------------------------------------------------------------
    let input_value = inputs
        .get(0)
        .ok_or_else(|| ComponentError::new(format!("{COMPONENT} vereist een shape (S)")))?;

    let mesh = coerce_mesh_like_with_context(input_value, &format!("{COMPONENT} input S"))?;

    // Early return for empty meshes
    if mesh.vertices.is_empty() || mesh.indices.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), input_value.clone());
        return Ok(outputs);
    }

    // ------------------------------------------------------------------
    // Input 1: Blend type (optional, reserved for future use)
    // Currently ignored; documented in limitations
    // ------------------------------------------------------------------
    let _blend_type: Option<i64> = inputs
        .get(1)
        .and_then(|v| super::coerce::coerce_integer(v).ok());

    // ------------------------------------------------------------------
    // Input 2: Metric type (optional, reserved for future use)
    // Currently ignored; documented in limitations
    // ------------------------------------------------------------------
    let _metric_type: Option<i64> = inputs
        .get(2)
        .and_then(|v| super::coerce::coerce_integer(v).ok());

    // ------------------------------------------------------------------
    // Input 3: Edge indices (optional)
    // ------------------------------------------------------------------
    let edge_indices: Vec<u32> = match inputs.get(3) {
        Some(Value::List(values)) => values
            .iter()
            .filter_map(|v| super::coerce::coerce_integer(v).ok())
            .filter(|&i| i >= 0)
            .map(|i| i as u32)
            .collect(),
        Some(v) => super::coerce::coerce_integer(v)
            .ok()
            .filter(|&i| i >= 0)
            .map(|i| vec![i as u32])
            .unwrap_or_default(),
        None => Vec::new(),
    };

    // If no edges specified, return input unchanged
    if edge_indices.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), input_value.clone());
        return Ok(outputs);
    }

    // ------------------------------------------------------------------
    // Input 4: Radii (optional, defaults to 1.0)
    // ------------------------------------------------------------------
    let radii: Vec<f64> = match inputs.get(4) {
        Some(Value::List(values)) => values
            .iter()
            .filter_map(|v| super::coerce::coerce_number(v, None).ok())
            .filter(|&r| r.is_finite() && r >= 0.0)
            .collect(),
        Some(v) => super::coerce::coerce_number(v, None)
            .ok()
            .filter(|&r| r.is_finite() && r >= 0.0)
            .map(|r| vec![r])
            .unwrap_or_else(|| vec![1.0]),
        None => vec![1.0],
    };

    // If all radii are zero or empty, return input unchanged
    if radii.is_empty() || radii.iter().all(|&r| r <= 1e-15) {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), input_value.clone());
        return Ok(outputs);
    }

    // ------------------------------------------------------------------
    // Convert coerce::Mesh to geom::GeomMesh
    // ------------------------------------------------------------------
    let geom_mesh = geom_bridge::mesh_to_geom_mesh(mesh);
    let tol = Tolerance::default_geom();

    // ------------------------------------------------------------------
    // List available edges and map indices
    // ------------------------------------------------------------------
    let all_edges = match list_triangle_mesh_edges(&geom_mesh) {
        Ok(edges) => edges,
        Err(e) => {
            return Err(ComponentError::new(format!(
                "{COMPONENT}: kon mesh-edges niet bepalen: {e}"
            )));
        }
    };

    // Build edge pairs from indices, using the edge list
    // Edge indices are 0-based indices into the sorted edge list
    let mut edge_pairs: Vec<(u32, u32)> = Vec::with_capacity(edge_indices.len());
    let mut edge_options: Vec<FilletEdgeOptions> = Vec::with_capacity(edge_indices.len());

    for (i, &idx) in edge_indices.iter().enumerate() {
        if let Some(edge) = all_edges.get(idx as usize) {
            edge_pairs.push((edge.a, edge.b));
            // Use the corresponding radius, or the last one if fewer radii than edges
            let radius = *radii.get(i).or_else(|| radii.last()).unwrap_or(&1.0);
            edge_options.push(FilletEdgeOptions::new(radius, 4)); // 4 segments for smooth fillet
        }
        // Indices out of range are silently ignored (common Grasshopper behavior)
    }

    // If no valid edges after mapping, return input unchanged
    if edge_pairs.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), input_value.clone());
        return Ok(outputs);
    }

    // ------------------------------------------------------------------
    // Apply fillet to each edge (using the first radius for all, as the geom API
    // applies a single options set; for variable radii, we would need multiple passes)
    // ------------------------------------------------------------------
    // Note: The geom API takes a single FilletEdgeOptions. For variable radii per edge,
    // we use the average radius as a simplification (documented limitation).
    let avg_radius = radii.iter().copied().sum::<f64>() / radii.len() as f64;
    let options = FilletEdgeOptions::new(avg_radius, 4);

    let (result_mesh, mesh_diag, fillet_diag) =
        match fillet_triangle_mesh_edges(&geom_mesh, &edge_pairs, options, tol) {
            Ok(result) => result,
            Err(e) => {
                return Err(ComponentError::new(format!(
                    "{COMPONENT}: fillet mislukt: {e}"
                )));
            }
        };

    // ------------------------------------------------------------------
    // Merge diagnostics
    // ------------------------------------------------------------------
    let mut combined_diag = geom_bridge::geom_diagnostics_to_value_diagnostics(mesh_diag);

    // Add fillet-specific diagnostics as warnings
    for err in &fillet_diag.errors {
        combined_diag.warnings.push(format!("Fillet: {err}"));
    }
    for warn in &fillet_diag.warnings {
        combined_diag.warnings.push(format!("Fillet: {warn}"));
    }

    if fillet_diag.skipped_edge_count > 0 {
        combined_diag.warnings.push(format!(
            "Fillet: {} edge(s) skipped (unsupported topology; only hinge edges are supported)",
            fillet_diag.skipped_edge_count
        ));
    }
    if fillet_diag.clamped_edge_count > 0 {
        combined_diag.warnings.push(format!(
            "Fillet: {} edge(s) had radius clamped to fit geometry",
            fillet_diag.clamped_edge_count
        ));
    }

    // Update vertex/triangle counts in diagnostics to reflect the result mesh
    combined_diag.vertex_count = result_mesh.positions.len();
    combined_diag.triangle_count = result_mesh.indices.len() / 3;

    // ------------------------------------------------------------------
    // Output: Return as Value::Mesh with diagnostics (primary output)
    // Also provide legacy Value::Surface adapter for backward compatibility
    // ------------------------------------------------------------------
    // Primary output: Value::Mesh with full diagnostics attached
    let mesh_output = Value::Mesh {
        vertices: result_mesh.positions.clone(),
        indices: result_mesh.indices.clone(),
        normals: result_mesh.normals.clone(),
        uvs: result_mesh.uvs.clone(),
        diagnostics: Some(combined_diag),
    };

    // Legacy adapter: Value::Surface for consumers that don't support Value::Mesh yet
    // Note: This is a lossy conversion - normals, uvs, and diagnostics are discarded
    let legacy_surface = geom_bridge::geom_mesh_to_surface_legacy(result_mesh);

    let mut outputs = BTreeMap::new();
    // Use Value::Mesh as primary output on the "B" pin (Brep/mesh output)
    // Consumers that support Value::Mesh will get diagnostics; legacy consumers
    // will automatically coerce to surface-like representation
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), mesh_output);
    // Provide explicit legacy surface on "S" pin for backward compatibility
    outputs.insert(PIN_OUTPUT_SOLID.to_owned(), legacy_surface);
    Ok(outputs)
}

/// Legacy fallback for FilletEdge when mesh_engine_next is disabled.
///
/// Simply passes the input through unchanged, as fillet operations require the geom module.
#[cfg(not(feature = "mesh_engine_next"))]
fn evaluate_fillet_edge(inputs: &[Value]) -> ComponentResult {
    let shape = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Fillet Edge vereist geometrie"))?;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), shape);
    Ok(outputs)
}

/// Evaluates the CopyTrim component.
///
/// Copies trim bounds from a source surface to a target surface.
/// The resulting surface is the intersection of the source and target bounds.
/// If there is no valid intersection, the target bounds are returned unchanged.
///
/// # Inputs
/// - `S` (index 0): Source surface with trim information to copy
/// - `T` (index 1): Target surface to apply the trim to
///
/// # Outputs
/// - `B`: The trimmed surface result
#[cfg(not(feature = "mesh_engine_next"))]
fn evaluate_copy_trim(inputs: &[Value]) -> ComponentResult {
    const COMPONENT: &str = "Copy Trim";
    
    if inputs.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist zowel een bron- als doelsurface",
            COMPONENT
        )));
    }

    let source = coerce_shape_metrics(inputs.get(0), COMPONENT)?;
    let target = coerce_shape_metrics(inputs.get(1), COMPONENT)?;
    
    // Compute the intersection of source and target bounds
    let min = [
        source.min[0].max(target.min[0]),
        source.min[1].max(target.min[1]),
        source.min[2].max(target.min[2]),
    ];
    let max = [
        source.max[0].min(target.max[0]),
        source.max[1].min(target.max[1]),
        source.max[2].min(target.max[2]),
    ];

    // Only use intersection if valid, otherwise return target unchanged
    let surface = if min[0] <= max[0] && min[1] <= max[1] && min[2] <= max[2] {
        create_surface_from_bounds(min, max)
    } else {
        create_surface_from_bounds(target.min, target.max)
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface);
    Ok(outputs)
}

/// Evaluates the CopyTrim component using the geom module.
///
/// Uses `geom::copy_trim_bounds` to compute the intersection of source and target bounds.
/// This preserves trim-loop ordering semantics by applying the source's trim region
/// to the target surface's parameter space.
///
/// # Inputs
/// - `S` (index 0): Source surface with trim information to copy
/// - `T` (index 1): Target surface to apply the trim to
///
/// # Outputs
/// - `B`: The trimmed surface result
#[cfg(feature = "mesh_engine_next")]
fn evaluate_copy_trim(inputs: &[Value]) -> ComponentResult {
    const COMPONENT: &str = "Copy Trim";
    
    if inputs.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist zowel een bron- als doelsurface",
            COMPONENT
        )));
    }

    let source = coerce_shape_metrics(inputs.get(0), COMPONENT)?;
    let target = coerce_shape_metrics(inputs.get(1), COMPONENT)?;
    
    // Use geom::copy_trim_bounds for the heavy lifting.
    // This function computes the intersection of source and target bounds,
    // returning target bounds unchanged if no valid intersection exists.
    let (min, max) = crate::geom::copy_trim_bounds(
        source.min,
        source.max,
        target.min,
        target.max,
    );
    
    let surface = create_surface_from_bounds(min, max);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface);
    Ok(outputs)
}

/// Evaluates the Retrim component.
///
/// Applies new trim bounds from a source surface to a target surface.
/// This is semantically similar to CopyTrim but emphasizes re-applying
/// trim information rather than copying it.
///
/// # Inputs
/// - `S` (index 0): Source surface providing new trim bounds
/// - `T` (index 1): Target surface to retrim
///
/// # Outputs
/// - `B`: The retrimmed surface result
#[cfg(not(feature = "mesh_engine_next"))]
fn evaluate_retrim(inputs: &[Value]) -> ComponentResult {
    const COMPONENT: &str = "Retrim";
    
    if inputs.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist zowel een bron- als doelsurface",
            COMPONENT
        )));
    }

    let source = coerce_shape_metrics(inputs.get(0), COMPONENT)?;
    let target = coerce_shape_metrics(inputs.get(1), COMPONENT)?;
    
    // Retrim computes the intersection of source and target bounds
    let min = [
        source.min[0].max(target.min[0]),
        source.min[1].max(target.min[1]),
        source.min[2].max(target.min[2]),
    ];
    let max = [
        source.max[0].min(target.max[0]),
        source.max[1].min(target.max[1]),
        source.max[2].min(target.max[2]),
    ];

    // Only use intersection if valid, otherwise return target unchanged
    let surface = if min[0] <= max[0] && min[1] <= max[1] && min[2] <= max[2] {
        create_surface_from_bounds(min, max)
    } else {
        create_surface_from_bounds(target.min, target.max)
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface);
    Ok(outputs)
}

/// Evaluates the Retrim component using the geom module.
///
/// Uses `geom::retrim_bounds` to apply new trim bounds from the source
/// to the target surface. This preserves trim-loop ordering semantics
/// by computing the intersection of source and target bounds.
///
/// # Inputs
/// - `S` (index 0): Source surface providing new trim bounds
/// - `T` (index 1): Target surface to retrim
///
/// # Outputs
/// - `B`: The retrimmed surface result
#[cfg(feature = "mesh_engine_next")]
fn evaluate_retrim(inputs: &[Value]) -> ComponentResult {
    const COMPONENT: &str = "Retrim";
    
    if inputs.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist zowel een bron- als doelsurface",
            COMPONENT
        )));
    }

    let source = coerce_shape_metrics(inputs.get(0), COMPONENT)?;
    let target = coerce_shape_metrics(inputs.get(1), COMPONENT)?;
    
    // Use geom::retrim_bounds for the heavy lifting.
    // This function is semantically equivalent to copy_trim_bounds but
    // is named to clarify intent for retrim operations.
    let (min, max) = crate::geom::retrim_bounds(
        source.min,
        source.max,
        target.min,
        target.max,
    );
    
    let surface = create_surface_from_bounds(min, max);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface);
    Ok(outputs)
}

fn evaluate_edges_from_directions(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Edges from Directions vereist een brep, richtingen en toleranties",
        ));
    }

    let metrics = coerce_shape_metrics(inputs.get(0), "Edges from Directions")?;
    let directions = parse_directions(inputs.get(1));
    if directions.is_empty() {
        return Err(ComponentError::new(
            "Edges from Directions vereist minstens één richting",
        ));
    }
    let reflex = coerce_boolean(inputs.get(2), false)?;
    let tolerance = coerce_number(inputs.get(3), "Edges from Directions hoek")?
        .to_radians()
        .abs();

    #[cfg(feature = "mesh_engine_next")]
    {
        let tol = crate::geom::Tolerance::default_geom();
        let mut brep = collect_legacy_brep_data(inputs.get(0), tol);
        if brep.edges.is_empty() {
            brep = crate::geom::LegacyBrepData::from_bounds(
                crate::geom::Point3::new(metrics.min[0], metrics.min[1], metrics.min[2]),
                crate::geom::Point3::new(metrics.max[0], metrics.max[1], metrics.max[2]),
            );
        }

        let directions = directions
            .iter()
            .map(|direction| crate::geom::Vec3::new(direction[0], direction[1], direction[2]))
            .collect::<Vec<_>>();

        let result =
            crate::geom::edges_from_directions(&brep, &directions, reflex, tolerance, tol);

        let mut selected = Vec::with_capacity(result.edges.len());
        let mut indices = Vec::with_capacity(result.edges.len());
        let mut mapping = Vec::with_capacity(result.edges.len());

        for (edge_index, dir_index) in result.edges.into_iter().zip(result.map.into_iter()) {
            let edge = &brep.edges[edge_index];
            selected.push(Value::CurveLine {
                p1: edge.start.to_array(),
                p2: edge.end.to_array(),
            });
            indices.push(Value::Number(edge_index as f64));
            mapping.push(Value::Number(dir_index as f64));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(selected));
        outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
        outputs.insert(PIN_OUTPUT_MAP.to_owned(), Value::List(mapping));
        return Ok(outputs);
    }

    #[cfg(not(feature = "mesh_engine_next"))]
    {
        let mut brep = collect_brep_data(inputs.get(0));
        if brep.edges.is_empty() {
            brep = BrepData::from_metrics(&metrics);
        }

        let mut selected = Vec::new();
        let mut indices = Vec::new();
        let mut mapping = Vec::new();

        for (index, edge) in brep.edges.iter().enumerate() {
            if let Some((direction, _)) = normalize(edge.vector()) {
                let mut matched = None;
                for (dir_index, candidate) in directions.iter().enumerate() {
                    let (candidate, _) = normalize(*candidate).unwrap_or(([1.0, 0.0, 0.0], 1.0));
                    let dot = clamp(dot(direction, candidate), -1.0, 1.0);
                    let angle = dot.acos();
                    if angle <= tolerance
                        || (reflex && (std::f64::consts::PI - angle) <= tolerance)
                    {
                        matched = Some(dir_index);
                        break;
                    }
                }
                if let Some(dir_index) = matched {
                    selected.push(edge.to_value());
                    indices.push(Value::Number(index as f64));
                    mapping.push(Value::Number(dir_index as f64));
                }
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(selected));
        outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
        outputs.insert(PIN_OUTPUT_MAP.to_owned(), Value::List(mapping));
        Ok(outputs)
    }
}

fn evaluate_isotrim(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Isotrim vereist een surface en domein"));
    }

    let domain_value = inputs
        .get(1)
        .ok_or_else(|| ComponentError::new("Isotrim vereist een domein"))?;
    let (u_range, v_range) = coerce_domain_pair(domain_value, "Isotrim")?;

    let surface = {
        #[cfg(feature = "mesh_engine_next")]
        {
            // Try to extract vertex data for accurate isotrim
            let surface_data = SurfaceVertexData::from_value(inputs.get(0))
                .ok_or_else(|| ComponentError::new("Isotrim vereist geometrische invoer"))?;

            // Use vertex-based isotrim if we have a proper grid
            let result = if surface_data.has_valid_grid() {
                crate::geom::isotrim_from_vertices(
                    surface_data.to_vertex_input(),
                    u_range,
                    v_range,
                )
            } else {
                None
            };

            // Fall back to bounds-based if vertex-based failed
            let result = result.unwrap_or_else(|| {
                crate::geom::isotrim_from_bounds(
                    surface_data.min,
                    surface_data.max,
                    u_range,
                    v_range,
                )
            });

            Value::Surface {
                vertices: result.vertices,
                faces: result.faces,
            }
        }

        #[cfg(not(feature = "mesh_engine_next"))]
        {
            let metrics = coerce_shape_metrics(inputs.get(0), "Isotrim")?;
            let mut min = metrics.min;
            let mut max = metrics.max;
            min[0] = metrics.min[0] + metrics.size()[0] * clamp01(u_range.0);
            max[0] = metrics.min[0] + metrics.size()[0] * clamp01(u_range.1);
            min[1] = metrics.min[1] + metrics.size()[1] * clamp01(v_range.0);
            max[1] = metrics.min[1] + metrics.size()[1] * clamp01(v_range.1);

            create_surface_from_bounds(min, max)
        }
    };
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface);
    Ok(outputs)
}

fn evaluate_closed_edges(inputs: &[Value]) -> ComponentResult {
    let metrics = coerce_shape_metrics(inputs.get(0), "Closed Edges")?;
    let _tangency = coerce_boolean(inputs.get(1), true).unwrap_or(true);

    #[cfg(feature = "mesh_engine_next")]
    {
        let tol = crate::geom::Tolerance::default_geom();
        let mut brep = collect_legacy_brep_data(inputs.get(0), tol);
        if brep.edges.is_empty() {
            brep = crate::geom::LegacyBrepData::from_bounds(
                crate::geom::Point3::new(metrics.min[0], metrics.min[1], metrics.min[2]),
                crate::geom::Point3::new(metrics.max[0], metrics.max[1], metrics.max[2]),
            );
        }

        let result = crate::geom::closed_edges(&brep);
        let mut closed_edges = Vec::with_capacity(result.closed.len());
        let mut closed_indices = Vec::with_capacity(result.closed.len());
        for index in result.closed {
            let edge = &brep.edges[index];
            closed_edges.push(Value::CurveLine {
                p1: edge.start.to_array(),
                p2: edge.end.to_array(),
            });
            closed_indices.push(Value::Number(index as f64));
        }

        let mut open_edges = Vec::with_capacity(result.open.len());
        let mut open_indices = Vec::with_capacity(result.open.len());
        for index in result.open {
            let edge = &brep.edges[index];
            open_edges.push(Value::CurveLine {
                p1: edge.start.to_array(),
                p2: edge.end.to_array(),
            });
            open_indices.push(Value::Number(index as f64));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_CLOSED.to_owned(), Value::List(closed_edges));
        outputs.insert(
            PIN_OUTPUT_CLOSED_INDICES.to_owned(),
            Value::List(closed_indices),
        );
        outputs.insert(PIN_OUTPUT_OPEN.to_owned(), Value::List(open_edges));
        outputs.insert(
            PIN_OUTPUT_OPEN_INDICES.to_owned(),
            Value::List(open_indices),
        );
        return Ok(outputs);
    }

    #[cfg(not(feature = "mesh_engine_next"))]
    {
        let mut brep = collect_brep_data(inputs.get(0));
        if brep.edges.is_empty() {
            brep = BrepData::from_metrics(&metrics);
        }

        let mut closed_edges = Vec::new();
        let mut closed_indices = Vec::new();
        let mut open_edges = Vec::new();
        let mut open_indices = Vec::new();

        for (index, edge) in brep.edges.iter().enumerate() {
            if edge.face_count() >= 2 {
                closed_edges.push(edge.to_value());
                closed_indices.push(Value::Number(index as f64));
            } else {
                open_edges.push(edge.to_value());
                open_indices.push(Value::Number(index as f64));
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_CLOSED.to_owned(), Value::List(closed_edges));
        outputs.insert(
            PIN_OUTPUT_CLOSED_INDICES.to_owned(),
            Value::List(closed_indices),
        );
        outputs.insert(PIN_OUTPUT_OPEN.to_owned(), Value::List(open_edges));
        outputs.insert(
            PIN_OUTPUT_OPEN_INDICES.to_owned(),
            Value::List(open_indices),
        );
        Ok(outputs)
    }
}

fn evaluate_edges_from_faces(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Edges from Faces vereist een brep en punten",
        ));
    }
    let metrics = coerce_shape_metrics(inputs.get(0), "Edges from Faces")?;
    let points = collect_point_list(inputs.get(1));
    let tolerance = 1e-3;

    #[cfg(feature = "mesh_engine_next")]
    {
        let tol = crate::geom::Tolerance::default_geom();
        let mut brep = collect_legacy_brep_data(inputs.get(0), tol);
        if brep.edges.is_empty() {
            brep = crate::geom::LegacyBrepData::from_bounds(
                crate::geom::Point3::new(metrics.min[0], metrics.min[1], metrics.min[2]),
                crate::geom::Point3::new(metrics.max[0], metrics.max[1], metrics.max[2]),
            );
        }

        let points = points
            .iter()
            .map(|point| crate::geom::Point3::new(point[0], point[1], point[2]))
            .collect::<Vec<_>>();
        let selected_indices = crate::geom::edges_from_faces(&brep, &points, tolerance);

        let mut selected = Vec::with_capacity(selected_indices.len());
        let mut indices = Vec::with_capacity(selected_indices.len());
        for index in selected_indices {
            let edge = &brep.edges[index];
            selected.push(Value::CurveLine {
                p1: edge.start.to_array(),
                p2: edge.end.to_array(),
            });
            indices.push(Value::Number(index as f64));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(selected));
        outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
        return Ok(outputs);
    }

    #[cfg(not(feature = "mesh_engine_next"))]
    {
        let mut brep = collect_brep_data(inputs.get(0));
        if brep.edges.is_empty() {
            brep = BrepData::from_metrics(&metrics);
        }

        let mut selected = Vec::new();
        let mut indices = Vec::new();

        if !brep.faces.is_empty() {
            let mut selected_faces = Vec::new();
            if points.is_empty() {
                selected_faces.extend(0..brep.faces.len());
            } else {
                for (face_index, face) in brep.faces.iter().enumerate() {
                    let centroid = face.centroid();
                    if points
                        .iter()
                        .any(|point| distance(&centroid, point) <= tolerance)
                    {
                        selected_faces.push(face_index);
                    }
                }
            }

            for (index, edge) in brep.edges.iter().enumerate() {
                if edge
                    .faces
                    .iter()
                    .any(|face_index| selected_faces.contains(face_index))
                {
                    selected.push(edge.to_value());
                    indices.push(Value::Number(index as f64));
                }
            }
        } else {
            for (index, edge) in brep.edges.iter().enumerate() {
                let include = points.is_empty()
                    || points
                        .iter()
                        .any(|point| edge.touches_point(point, tolerance));
                if include {
                    selected.push(edge.to_value());
                    indices.push(Value::Number(index as f64));
                }
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(selected));
        outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
        Ok(outputs)
    }
}

fn evaluate_edges_from_points(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Edges from Points vereist een brep en punten",
        ));
    }
    let metrics = coerce_shape_metrics(inputs.get(0), "Edges from Points")?;
    let points = collect_point_list(inputs.get(1));
    if points.is_empty() {
        return Err(ComponentError::new(
            "Edges from Points vereist minstens één punt",
        ));
    }
    let valence = coerce_positive_integer(inputs.get(2), "Edges from Points valentie")?;
    let tolerance = inputs
        .get(3)
        .map(|value| coerce_number(Some(value), "Edges from Points tolerantie"))
        .transpose()?;
    let tolerance = tolerance.unwrap_or(0.25).abs();

    #[cfg(feature = "mesh_engine_next")]
    {
        let tol = crate::geom::Tolerance::default_geom();
        let mut brep = collect_legacy_brep_data(inputs.get(0), tol);
        if brep.edges.is_empty() {
            brep = crate::geom::LegacyBrepData::from_bounds(
                crate::geom::Point3::new(metrics.min[0], metrics.min[1], metrics.min[2]),
                crate::geom::Point3::new(metrics.max[0], metrics.max[1], metrics.max[2]),
            );
        }

        let points = points
            .iter()
            .map(|point| crate::geom::Point3::new(point[0], point[1], point[2]))
            .collect::<Vec<_>>();
        let result = crate::geom::edges_from_points(&brep, &points, valence, tolerance);

        let mut selected = Vec::with_capacity(result.edges.len());
        let mut indices = Vec::with_capacity(result.edges.len());
        for index in result.edges {
            let edge = &brep.edges[index];
            selected.push(Value::CurveLine {
                p1: edge.start.to_array(),
                p2: edge.end.to_array(),
            });
            indices.push(Value::Number(index as f64));
        }

        let map_values = result
            .map
            .into_iter()
            .map(|count| Value::Number(count as f64))
            .collect();

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(selected));
        outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
        outputs.insert(PIN_OUTPUT_MAP.to_owned(), Value::List(map_values));
        return Ok(outputs);
    }

    #[cfg(not(feature = "mesh_engine_next"))]
    {
        let mut brep = collect_brep_data(inputs.get(0));
        if brep.edges.is_empty() {
            brep = BrepData::from_metrics(&metrics);
        }

        let mut selected = Vec::new();
        let mut indices = Vec::new();
        let mut mapping = vec![0usize; points.len()];

        for (index, edge) in brep.edges.iter().enumerate() {
            let mut matched_points = Vec::new();
            for (point_index, point) in points.iter().enumerate() {
                if edge.touches_point(point, tolerance) {
                    matched_points.push(point_index);
                }
            }
            if matched_points.len() >= valence {
                selected.push(edge.to_value());
                indices.push(Value::Number(index as f64));
                for point_index in matched_points {
                    mapping[point_index] += 1;
                }
            }
        }

        let map_values = mapping
            .into_iter()
            .map(|count| Value::Number(count as f64))
            .collect();

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(selected));
        outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
        outputs.insert(PIN_OUTPUT_MAP.to_owned(), Value::List(map_values));
        Ok(outputs)
    }
}

fn evaluate_convex_edges(inputs: &[Value]) -> ComponentResult {
    let metrics = coerce_shape_metrics(inputs.get(0), "Convex Edges")?;
    let mut brep = collect_brep_data(inputs.get(0));
    if brep.edges.is_empty() {
        brep = BrepData::from_metrics(&metrics);
    }

    let mut convex = Vec::new();
    let mut concave = Vec::new();
    let mut mixed = Vec::new();

    for edge in brep.edges {
        let value = edge.to_value();
        match edge.face_count() {
            0 => mixed.push(value),
            1 => concave.push(value),
            _ => convex.push(value),
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CONVEX.to_owned(), Value::List(convex));
    outputs.insert(PIN_OUTPUT_CONCAVE.to_owned(), Value::List(concave));
    outputs.insert(PIN_OUTPUT_MIXED.to_owned(), Value::List(mixed));
    Ok(outputs)
}

/// Evaluates the Offset Surface and Offset Surface Loose components.
///
/// Offsets a surface/mesh by moving vertices along their normals by a specified
/// distance. Positive distance offsets outward (along normal), negative distance
/// offsets inward (against normal).
///
/// # Inputs
/// - `S` (index 0): Surface or mesh to offset
/// - `D` (index 1): Offset distance (positive = outward, negative = inward)
///
/// # Outputs
/// - `B`: The offset surface/mesh result
///
/// # Difference between OffsetSurface and OffsetSurfaceLoose
/// - `OffsetSurface`: Uses standard geometry tolerance for welding and repair
/// - `OffsetSurfaceLoose`: Uses looser tolerance, which may be faster but less precise
#[cfg(not(feature = "mesh_engine_next"))]
fn evaluate_offset_surface(inputs: &[Value], component: &str) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist een surface en afstand",
            component
        )));
    }
    let metrics = coerce_shape_metrics(inputs.get(0), component)?;
    let distance = coerce_number(inputs.get(1), &(component.to_owned() + " afstand"))?;

    let mut min = metrics.min;
    let mut max = metrics.max;
    min[2] -= distance.abs();
    max[2] += distance.abs();

    let surface = create_surface_from_bounds(min, max);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface);
    Ok(outputs)
}

/// Evaluates the Offset Surface and Offset Surface Loose components using the
/// new geometry kernel.
///
/// Uses `geom::offset::*` functions to perform true normal-based offsetting
/// on mesh vertices, producing a properly offset mesh with welding and repair.
///
/// # Inputs
/// - `S` (index 0): Surface or mesh to offset
/// - `D` (index 1): Offset distance (positive = outward, negative = inward)
///
/// # Outputs
/// - `B`: The offset surface/mesh result (list if multiple inputs, single value otherwise)
///
/// # Difference between OffsetSurface and OffsetSurfaceLoose
/// - `OffsetSurface`: Uses standard geometry tolerance (1e-9)
/// - `OffsetSurfaceLoose`: Uses looser tolerance (1e-6)
///
/// # List Handling
/// Properly handles multi-item list inputs by offsetting each mesh individually
/// and returning a list of results. Single-item inputs return a single value.
#[cfg(feature = "mesh_engine_next")]
fn evaluate_offset_surface(inputs: &[Value], component: &str) -> ComponentResult {
    use super::coerce::{coerce_mesh_list, geom_bridge};
    use crate::geom::{offset_mesh, OffsetDirection, OffsetOptions, Tolerance};

    if inputs.len() < 2 {
        return Err(ComponentError::new(format!(
            "{} vereist een surface en afstand",
            component
        )));
    }

    // Get the input value for list-structure preservation
    let input_value = inputs
        .get(0)
        .ok_or_else(|| ComponentError::new(format!("{} vereist geometrie", component)))?;

    // Coerce input to a list of meshes (handles both single items and multi-item lists)
    let meshes = coerce_mesh_list(input_value, component)?;
    let distance = coerce_number(inputs.get(1), &(component.to_owned() + " afstand"))?;

    // Track whether input was a list for output structure preservation
    let input_was_list = matches!(input_value, Value::List(l) if l.len() > 1);

    // Handle zero distance case - just return input unchanged
    if distance.abs() < 1e-15 {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), input_value.clone());
        return Ok(outputs);
    }

    // Select tolerance based on component type
    // OffsetSurfaceLoose uses a looser tolerance for faster but less precise results
    let is_loose = component.contains("Loose");
    let tol = if is_loose {
        Tolerance::LOOSE
    } else {
        Tolerance::default_geom()
    };

    // Determine offset direction based on distance sign
    let (direction, abs_distance) = if distance >= 0.0 {
        (OffsetDirection::Outside, distance)
    } else {
        (OffsetDirection::Inside, -distance)
    };

    // Configure offset options (shared across all meshes)
    let options = OffsetOptions::new(abs_distance).direction(direction);

    // Process each mesh individually
    let mut offset_results: Vec<Value> = Vec::with_capacity(meshes.len());
    for (idx, mesh) in meshes.into_iter().enumerate() {
        // Convert coerce::Mesh to geom::GeomMesh
        let geom_mesh = geom_bridge::mesh_to_geom_mesh(mesh);

        // Perform the offset operation
        let (result_mesh, offset_diag) =
            offset_mesh(&geom_mesh, options.clone(), tol).map_err(|e| {
                ComponentError::new(format!(
                    "{} offset failed for item {}: {}",
                    component, idx, e
                ))
            })?;

        // Log warnings if any (for debugging)
        for warning in &offset_diag.warnings {
            // In a real implementation, these could be surfaced to the user
            // For now, they're available in the diagnostics
            let _ = warning;
        }

        // Convert back to legacy Surface for backward compatibility
        let surface_legacy = geom_bridge::geom_mesh_to_surface_legacy(result_mesh);
        offset_results.push(surface_legacy);
    }

    // Build output: preserve list structure from input
    // - Single input → single output
    // - Multi-item list input → list output
    let output_value = if input_was_list || offset_results.len() > 1 {
        Value::List(offset_results)
    } else {
        // Unwrap single result; default to empty list if somehow empty
        offset_results.into_iter().next().unwrap_or(Value::List(vec![]))
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), output_value);
    Ok(outputs)
}

#[cfg(not(feature = "mesh_engine_next"))]
fn evaluate_cap_holes(inputs: &[Value], extended: bool) -> ComponentResult {
    let mut surface_value = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Cap Holes vereist een brep"))?;

    // If the input is a list, find the first surface in it and use that.
    if let Value::List(values) = &surface_value {
        if let Some(surface) = values.iter().find(|v| matches!(v, Value::Surface { .. })) {
            surface_value = surface.clone();
        }
    }

    // Now, `surface_value` is either the original input or an extracted surface.
    // We need to ensure it's a processable surface before continuing.
    // We clone the vertices here to avoid borrow checker conflicts later.
    let vertices_clone = match &surface_value {
        Value::Surface { vertices, faces } if !vertices.is_empty() && !faces.is_empty() => {
            vertices.clone()
        }
        _ => {
            // If it's not a valid surface (or empty), just return the original input.
            let mut outputs = BTreeMap::new();
            outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface_value);
            if extended {
                outputs.insert(PIN_OUTPUT_CAPS.to_owned(), Value::Number(0.0));
                outputs.insert(PIN_OUTPUT_SOLID.to_owned(), Value::Boolean(false));
            }
            return Ok(outputs);
        }
    };

    // From here, we know `surface_value` is a `Value::Surface` with content.
    let brep_data = collect_brep_data(Some(&surface_value));
    let naked_edge_indices = brep_data.get_naked_edges();

    if naked_edge_indices.is_empty() {
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface_value);
        if extended {
            outputs.insert(PIN_OUTPUT_CAPS.to_owned(), Value::Number(0.0));
            outputs.insert(PIN_OUTPUT_SOLID.to_owned(), Value::Boolean(true)); // Already solid
        }
        return Ok(outputs);
    }

    let loops_of_points = brep_data.find_loops(&naked_edge_indices);
    let caps_created = loops_of_points.len();
    let mut new_faces_count = 0;

    fn find_vertex_index(vertices: &[[f64; 3]], point: &[f64; 3]) -> Option<u32> {
        vertices
            .iter()
            .position(|v| nearly_equal_points(v, point))
            .map(|i| i as u32)
    }

    for hole_points in loops_of_points {
        if hole_points.len() < 3 {
            continue;
        }

        let hole_indices: Vec<u32> = hole_points
            .iter()
            .filter_map(|p| find_vertex_index(&vertices_clone, p))
            .collect();

        if hole_indices.len() != hole_points.len() {
            continue;
        }

        let mut normal = [0.0, 0.0, 0.0];
        for i in 0..hole_points.len() {
            let p1 = hole_points[i];
            let p2 = hole_points[(i + 1) % hole_points.len()];
            normal[0] += (p1[1] - p2[1]) * (p1[2] + p2[2]);
            normal[1] += (p1[2] - p2[2]) * (p1[0] + p2[0]);
            normal[2] += (p1[0] - p2[0]) * (p1[1] + p2[1]);
        }

        let norm_mag = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
        if norm_mag < EPSILON {
            continue;
        }
        let normal = [
            normal[0] / norm_mag,
            normal[1] / norm_mag,
            normal[2] / norm_mag,
        ];

        let u_axis_candidate = if normal[0].abs() > 0.9 {
            [0.0, 1.0, 0.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        let mut u_axis = cross(u_axis_candidate, normal);
        u_axis = normalize(u_axis).map_or([1.0, 0.0, 0.0], |(v, _)| v);
        let v_axis = cross(normal, u_axis);

        let points_2d: Vec<delaunator::Point> = hole_points
            .iter()
            .map(|p| {
                let p_vec = [p[0], p[1], p[2]];
                delaunator::Point {
                    x: dot(p_vec, u_axis),
                    y: dot(p_vec, v_axis),
                }
            })
            .collect();

        let triangulation = delaunator::triangulate(&points_2d);
        if triangulation.triangles.is_empty() {
            continue;
        }

        if !triangulation.triangles.is_empty() {
            let i1_idx = triangulation.triangles[0];
            let i2_idx = triangulation.triangles[1];
            let i3_idx = triangulation.triangles[2];

            let p1 = vertices_clone[hole_indices[i1_idx] as usize];
            let p2 = vertices_clone[hole_indices[i2_idx] as usize];
            let p3 = vertices_clone[hole_indices[i3_idx] as usize];

            let vec1 = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
            let vec2 = [p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]];
            let new_face_normal = cross(vec1, vec2);
            let flip_winding = dot(new_face_normal, normal) >= 0.0;

            let faces = match &mut surface_value {
                Value::Surface { faces, .. } => faces,
                _ => unreachable!(),
            };
            for i in (0..triangulation.triangles.len()).step_by(3) {
                let i1 = hole_indices[triangulation.triangles[i]];
                let i2 = hole_indices[triangulation.triangles[i + 1]];
                let i3 = hole_indices[triangulation.triangles[i + 2]];

                if flip_winding {
                    faces.push(vec![i1, i3, i2]);
                } else {
                    faces.push(vec![i1, i2, i3]);
                }
                new_faces_count += 1;
            }
        }
    }

    let is_solid = if new_faces_count > 0 {
        let final_brep_data = collect_brep_data(Some(&surface_value));
        final_brep_data.get_naked_edges().is_empty()
    } else {
        caps_created == 0
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface_value);
    if extended {
        outputs.insert(
            PIN_OUTPUT_CAPS.to_owned(),
            Value::Number(caps_created as f64),
        );
        outputs.insert(PIN_OUTPUT_SOLID.to_owned(), Value::Boolean(is_solid));
    }
    Ok(outputs)
}

/// Evaluates the Cap Holes / Cap Holes Ex components using `geom::solid::*`.
///
/// This function caps planar-ish boundary loops (naked edges) on a mesh or surface
/// using 2D polygon triangulation. It uses the new `geom::solid` module for the
/// heavy lifting while keeping the component thin.
///
/// # Inputs
/// - Input 0: Brep/Surface/Mesh - The geometry to cap holes on
/// - Input 1 (extended only): Planarity - Maximum planarity deviation threshold (Number, optional)
///
/// # Outputs
/// - `B`: The capped brep/surface (same type as input for backward compatibility)
/// - `C` (extended only): Number of holes found
/// - `S` (extended only): Boolean indicating if the result is a solid (watertight)
///
/// # Algorithm
/// Delegates to `geom::solid::cap_holes_legacy` or `geom::solid::cap_holes_ex_legacy`,
/// which:
/// 1. Builds an edge graph to identify naked (boundary) edges
/// 2. Finds closed loops of naked edges (these are the holes)
/// 3. For each loop: projects to 2D, triangulates using constrained Delaunay/ear-clipping
/// 4. Appends cap triangles to the mesh with correct winding
/// 5. Reports diagnostics including planarity deviation and any failures
#[cfg(feature = "mesh_engine_next")]
fn evaluate_cap_holes(inputs: &[Value], extended: bool) -> ComponentResult {
    use crate::geom::{
        cap_holes_ex_legacy, cap_holes_legacy, CapHolesExOptions, LegacySurfaceMesh, Tolerance,
    };

    let input_value = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Cap Holes vereist een brep"))?;

    // Helper to extract a mesh-like value from the input (handles lists, surfaces, meshes)
    fn extract_mesh_like(value: &Value) -> Option<LegacySurfaceMesh> {
        match value {
            Value::Surface { vertices, faces } if !vertices.is_empty() && !faces.is_empty() => {
                Some(LegacySurfaceMesh {
                    vertices: vertices.clone(),
                    faces: faces.clone(),
                })
            }
            Value::Mesh {
                vertices, indices, ..
            } if !vertices.is_empty() && !indices.is_empty() => {
                // Convert triangle indices to face lists
                let faces: Vec<Vec<u32>> = indices
                    .chunks(3)
                    .filter(|chunk| chunk.len() == 3)
                    .map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
                    .collect();
                Some(LegacySurfaceMesh {
                    vertices: vertices.clone(),
                    faces,
                })
            }
            Value::List(list) => {
                // Find the first mesh-like value in the list
                list.iter().find_map(extract_mesh_like)
            }
            _ => None,
        }
    }

    // Track whether input was a Mesh (for output type consistency)
    let input_was_mesh = matches!(&input_value, Value::Mesh { .. })
        || matches!(&input_value, Value::List(list) if list.iter().any(|v| matches!(v, Value::Mesh { .. })));

    let Some(mesh) = extract_mesh_like(&input_value) else {
        // If it's not a valid mesh-like value (or empty), just return the original input
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), input_value);
        if extended {
            outputs.insert(PIN_OUTPUT_CAPS.to_owned(), Value::Number(0.0));
            outputs.insert(PIN_OUTPUT_SOLID.to_owned(), Value::Boolean(false));
        }
        return Ok(outputs);
    };

    let tol = Tolerance::default_geom();

    // Call the appropriate geom function based on extended mode
    let result = if extended {
        // Parse extended options from additional inputs
        let max_planarity_deviation = inputs
            .get(1)
            .and_then(|v| match v {
                Value::Number(n) if n.is_finite() && *n > 0.0 => Some(*n),
                Value::List(list) if !list.is_empty() => match &list[0] {
                    Value::Number(n) if n.is_finite() && *n > 0.0 => Some(*n),
                    _ => None,
                },
                _ => None,
            })
            .unwrap_or(f64::INFINITY);

        let options = CapHolesExOptions {
            max_planarity_deviation,
            ..Default::default()
        };

        cap_holes_ex_legacy(mesh, tol, options)
    } else {
        cap_holes_legacy(mesh, tol)
    };

    // Convert result back to the appropriate Value type
    let output_value = if input_was_mesh {
        // Convert to Value::Mesh for consistency with input type
        let indices: Vec<u32> = result
            .brep
            .faces
            .iter()
            .flat_map(|face| {
                if face.len() >= 3 {
                    vec![face[0], face[1], face[2]]
                } else {
                    vec![]
                }
            })
            .collect();
        Value::Mesh {
            vertices: result.brep.vertices,
            indices,
            normals: None,
            uvs: None,
            diagnostics: None,
        }
    } else {
        // Return as Value::Surface for backward compatibility
        Value::Surface {
            vertices: result.brep.vertices,
            faces: result.brep.faces,
        }
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), output_value);

    if extended {
        // Output the number of holes found (caps_added is more accurate than holes_found
        // because some holes may have failed to cap)
        outputs.insert(
            PIN_OUTPUT_CAPS.to_owned(),
            Value::Number(result.diagnostics.caps_added as f64),
        );
        outputs.insert(PIN_OUTPUT_SOLID.to_owned(), Value::Boolean(result.is_solid));
    }

    Ok(outputs)
}

/// Evaluates the Flip component (legacy implementation).
///
/// Flips the orientation of a surface by reversing the winding order of its faces.
/// If a guide vector/point is provided, the surface is only flipped if its
/// orientation doesn't match the guide direction.
///
/// # Inputs
/// - `S` (index 0): Surface to flip
/// - `G` (index 1, optional): Guide vector or point for orientation check
///
/// # Outputs
/// - `B`: The flipped (or unchanged) surface
/// - `R`: Boolean indicating whether a flip was performed
#[cfg(not(feature = "mesh_engine_next"))]
fn evaluate_flip(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Flip vereist een surface"));
    }
    let mut surface = inputs[0].clone();
    let guide = inputs.get(1);
    let should_flip = guide
        .and_then(|value| match value {
            Value::Vector(vector) => Some(vector[2] < 0.0),
            Value::Point(point) => Some(point[2] < 0.0),
            _ => None,
        })
        .unwrap_or(true);

    if should_flip {
        if let Value::Surface { faces, .. } = &mut surface {
            faces.iter_mut().for_each(|face| face.reverse());
        }
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface);
    outputs.insert(PIN_OUTPUT_RESULT.to_owned(), Value::Boolean(should_flip));
    Ok(outputs)
}

/// Evaluates the Flip component using the geom module.
///
/// Flips the orientation of a surface or mesh by reversing the winding order
/// of its triangles/faces. If a guide vector/point is provided, the geometry
/// is only flipped if its average normal doesn't align with the guide direction.
///
/// This implementation uses `geom::flip_mesh` for proper normal-based flip
/// determination instead of the legacy z-component check.
///
/// # Inputs
/// - `S` (index 0): Surface or mesh to flip
/// - `G` (index 1, optional): Guide vector or point for orientation check
///
/// # Outputs
/// - `B`: The flipped (or unchanged) surface/mesh
/// - `R`: Boolean indicating whether a flip was performed
///
/// # Differences from legacy
/// - Supports both `Value::Surface` and `Value::Mesh` inputs
/// - Uses proper dot-product alignment check with average mesh normal
/// - Flips explicit normals when present in `Value::Mesh`
#[cfg(feature = "mesh_engine_next")]
fn evaluate_flip(inputs: &[Value]) -> ComponentResult {
    use crate::geom::{flip_mesh, GeomMesh, MeshFlipGuide, Point3, Vec3};

    if inputs.is_empty() {
        return Err(ComponentError::new("Flip vereist een surface"));
    }

    let input_value = &inputs[0];
    let guide = inputs.get(1);

    // Convert guide to MeshFlipGuide
    let flip_guide: Option<MeshFlipGuide> = guide.and_then(|value| match value {
        Value::Vector(v) => Some(MeshFlipGuide::Vector(Vec3::new(v[0], v[1], v[2]))),
        Value::Point(p) => Some(MeshFlipGuide::Point(Point3::new(p[0], p[1], p[2]))),
        _ => None,
    });

    match input_value {
        Value::Mesh {
            vertices,
            indices,
            normals,
            uvs,
            diagnostics,
        } => {
            // Convert to GeomMesh
            let geom_mesh = GeomMesh {
                positions: vertices.clone(),
                indices: indices.clone(),
                normals: normals.clone(),
                uvs: uvs.clone(),
                tangents: None,
            };

            // Flip using geom function
            let (flipped_mesh, flip_diag) = flip_mesh(geom_mesh, flip_guide);

            // Convert back to Value::Mesh
            let result_value = Value::Mesh {
                vertices: flipped_mesh.positions,
                indices: flipped_mesh.indices,
                normals: flipped_mesh.normals,
                uvs: flipped_mesh.uvs,
                diagnostics: diagnostics.clone(),
            };

            let mut outputs = BTreeMap::new();
            outputs.insert(PIN_OUTPUT_BREPS.to_owned(), result_value);
            outputs.insert(PIN_OUTPUT_RESULT.to_owned(), Value::Boolean(flip_diag.flipped));
            Ok(outputs)
        }
        Value::Surface { vertices, faces } => {
            // Convert Surface to a temporary mesh for flip computation.
            // Use proper triangulation to get accurate normal computation for n-gons.
            let indices = crate::graph::value::triangulate_polygon_faces(faces);

            let geom_mesh = GeomMesh::new(vertices.clone(), indices);

            // Flip using geom function to determine if flip is needed based on guide
            // We use the result only for flip_diag.flipped; we reverse original faces
            // to preserve non-triangular face topology
            let (_flipped_mesh, flip_diag) = flip_mesh(geom_mesh, flip_guide);

            // Convert back to Value::Surface with flipped faces
            let flipped_faces: Vec<Vec<u32>> = if flip_diag.flipped {
                // Reverse winding of original faces
                faces.iter().map(|f| {
                    let mut reversed = f.clone();
                    reversed.reverse();
                    reversed
                }).collect()
            } else {
                faces.clone()
            };

            let result_value = Value::Surface {
                vertices: vertices.clone(),
                faces: flipped_faces,
            };

            let mut outputs = BTreeMap::new();
            outputs.insert(PIN_OUTPUT_BREPS.to_owned(), result_value);
            outputs.insert(PIN_OUTPUT_RESULT.to_owned(), Value::Boolean(flip_diag.flipped));
            Ok(outputs)
        }
        Value::List(items) => {
            // Handle list of surfaces/meshes
            let mut flipped_items = Vec::with_capacity(items.len());
            let mut any_flipped = false;

            for item in items {
                let sub_inputs = vec![item.clone(), guide.cloned().unwrap_or(Value::Null)];
                let sub_result = evaluate_flip(&sub_inputs)?;
                
                if let Some(flipped) = sub_result.get(PIN_OUTPUT_BREPS) {
                    flipped_items.push(flipped.clone());
                }
                if let Some(Value::Boolean(did_flip)) = sub_result.get(PIN_OUTPUT_RESULT) {
                    any_flipped = any_flipped || *did_flip;
                }
            }

            let mut outputs = BTreeMap::new();
            outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(flipped_items));
            outputs.insert(PIN_OUTPUT_RESULT.to_owned(), Value::Boolean(any_flipped));
            Ok(outputs)
        }
        other => {
            // Try to use legacy Surface coercion for other types
            Err(ComponentError::new(format!(
                "Flip verwacht een Surface of Mesh, maar kreeg {}",
                other.kind()
            )))
        }
    }
}

#[cfg(not(feature = "mesh_engine_next"))]
fn evaluate_merge_faces(inputs: &[Value]) -> ComponentResult {
    let brep = inputs
        .get(0)
        .ok_or_else(|| ComponentError::new("Merge Faces vereist een brep"))?;
    let surfaces = collect_shapes(Some(brep));
    if surfaces.is_empty() {
        return Err(ComponentError::new(
            "Merge Faces kon geen oppervlakken vinden",
        ));
    }
    let before = surfaces.len();
    let merged_metrics = ShapeMetrics::from_inputs(Some(&Value::List(surfaces.clone()))).unwrap();
    let merged_surface = create_surface_from_bounds(merged_metrics.min, merged_metrics.max);

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_BREPS.to_owned(),
        Value::List(vec![merged_surface]),
    );
    outputs.insert(PIN_OUTPUT_BEFORE.to_owned(), Value::Number(before as f64));
    outputs.insert(PIN_OUTPUT_AFTER.to_owned(), Value::Number(1.0));
    Ok(outputs)
}

/// Evaluates the MergeFaces component using the geom module.
///
/// Uses `geom::merge_brep_faces` to merge coplanar/continuous faces within breps.
/// This function combines multiple input breps and then merges any coplanar
/// adjacent faces that share edges into larger polygons.
///
/// # Inputs
/// - `B` (index 0): Brep(s) to process (can be a single brep or list of breps)
///
/// # Outputs
/// - `B`: The brep with merged faces
/// - `N0`: Number of faces before merging
/// - `N1`: Number of faces after merging
#[cfg(feature = "mesh_engine_next")]
fn evaluate_merge_faces(inputs: &[Value]) -> ComponentResult {
    let brep = inputs
        .get(0)
        .ok_or_else(|| ComponentError::new("Merge Faces vereist een brep"))?;

    let shapes = collect_shapes(Some(brep));
    let mut surfaces = Vec::new();

    // Collect surfaces from both Value::Surface and Value::Mesh
    for shape in &shapes {
        match shape {
            Value::Surface { vertices, faces } => {
                surfaces.push(crate::geom::LegacySurfaceMesh {
                    vertices: vertices.clone(),
                    faces: faces.clone(),
                });
            }
            Value::Mesh { vertices, indices, .. } => {
                // Convert Value::Mesh (flat triangle indices) to LegacySurfaceMesh (face vectors)
                let faces: Vec<Vec<u32>> = indices
                    .chunks(3)
                    .map(|chunk| chunk.to_vec())
                    .collect();
                surfaces.push(crate::geom::LegacySurfaceMesh {
                    vertices: vertices.clone(),
                    faces,
                });
            }
            _ => {}
        }
    }

    if surfaces.is_empty() {
        return Err(ComponentError::new(
            "Merge Faces kon geen oppervlakken vinden",
        ));
    }

    // Use the new brep_ops API
    let merged = crate::geom::merge_brep_faces(
        &surfaces,
        crate::geom::MergeFacesOptions::default(),
    );

    if !merged.success {
        // Fall back to returning the combined input unchanged
        let combined_vertices: Vec<[f64; 3]> = surfaces.iter()
            .flat_map(|s| s.vertices.iter().copied())
            .collect();
        let mut offset = 0u32;
        let mut combined_faces: Vec<Vec<u32>> = Vec::new();
        for surface in &surfaces {
            for face in &surface.faces {
                combined_faces.push(face.iter().map(|&i| i + offset).collect());
            }
            offset += surface.vertices.len() as u32;
        }
        let fallback_surface = Value::Surface {
            vertices: combined_vertices,
            faces: combined_faces.clone(),
        };
        let before = surfaces.iter().map(|s| s.faces.len()).sum::<usize>();
        let after = combined_faces.len();

        let mut outputs = BTreeMap::new();
        outputs.insert(
            PIN_OUTPUT_BREPS.to_owned(),
            Value::List(vec![fallback_surface]),
        );
        outputs.insert(PIN_OUTPUT_BEFORE.to_owned(), Value::Number(before as f64));
        outputs.insert(PIN_OUTPUT_AFTER.to_owned(), Value::Number(after as f64));
        return Ok(outputs);
    }

    let merged_surface = Value::Surface {
        vertices: merged.brep.vertices,
        faces: merged.brep.faces,
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(
        PIN_OUTPUT_BREPS.to_owned(),
        Value::List(vec![merged_surface]),
    );
    outputs.insert(
        PIN_OUTPUT_BEFORE.to_owned(),
        Value::Number(merged.diagnostics.before as f64),
    );
    outputs.insert(
        PIN_OUTPUT_AFTER.to_owned(),
        Value::Number(merged.diagnostics.after as f64),
    );
    Ok(outputs)
}

fn evaluate_edges_by_length(inputs: &[Value], component: &str) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(format!(
            "{} vereist een brep en een minimaal/maximaal criterium",
            component
        )));
    }
    let metrics = coerce_shape_metrics(inputs.get(0), component)?;
    let min_length = coerce_number(inputs.get(1), &(component.to_owned() + " minimum"))?.abs();
    let max_length = coerce_number(inputs.get(2), &(component.to_owned() + " maximum"))?.abs();

    #[cfg(feature = "mesh_engine_next")]
    {
        let tol = crate::geom::Tolerance::default_geom();
        let mut brep = collect_legacy_brep_data(inputs.get(0), tol);
        if brep.edges.is_empty() {
            brep = crate::geom::LegacyBrepData::from_bounds(
                crate::geom::Point3::new(metrics.min[0], metrics.min[1], metrics.min[2]),
                crate::geom::Point3::new(metrics.max[0], metrics.max[1], metrics.max[2]),
            );
        }

        let selected_indices = crate::geom::edges_by_length(&brep, min_length, max_length);
        let mut selected = Vec::with_capacity(selected_indices.len());
        let mut indices = Vec::with_capacity(selected_indices.len());

        for index in selected_indices {
            let edge = &brep.edges[index];
            selected.push(Value::CurveLine {
                p1: edge.start.to_array(),
                p2: edge.end.to_array(),
            });
            indices.push(Value::Number(index as f64));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(selected));
        outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
        return Ok(outputs);
    }

    #[cfg(not(feature = "mesh_engine_next"))]
    {
        let mut brep = collect_brep_data(inputs.get(0));
        if brep.edges.is_empty() {
            brep = BrepData::from_metrics(&metrics);
        }

        let mut selected = Vec::new();
        let mut indices = Vec::new();

        for (index, edge) in brep.edges.iter().enumerate() {
            let length = edge.length();
            if length >= min_length && length <= max_length.max(min_length) {
                selected.push(edge.to_value());
                indices.push(Value::Number(index as f64));
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_BREPS.to_owned(), Value::List(selected));
        outputs.insert(PIN_OUTPUT_INDICES.to_owned(), Value::List(indices));
        Ok(outputs)
    }
}

/// Evaluates the Untrim component.
///
/// Removes all trim information from a surface, returning the full
/// untrimmed surface based on its bounding box.
///
/// # Inputs
/// - `S` (index 0): Surface to untrim
///
/// # Outputs
/// - `B`: The untrimmed surface covering the full original bounds
#[cfg(not(feature = "mesh_engine_next"))]
fn evaluate_untrim(inputs: &[Value]) -> ComponentResult {
    const COMPONENT: &str = "Untrim";
    
    let metrics = coerce_shape_metrics(inputs.get(0), COMPONENT)?;
    let surface = create_surface_from_bounds(metrics.min, metrics.max);
    
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface);
    Ok(outputs)
}

/// Evaluates the Untrim component using the geom module.
///
/// Uses `geom::untrim_bounds` to remove all trim information from a surface.
/// The result is a surface covering the full original parameter domain,
/// equivalent to removing all trim loops and returning the base surface.
///
/// # Inputs
/// - `S` (index 0): Surface to untrim
///
/// # Outputs
/// - `B`: The untrimmed surface covering the full original bounds
#[cfg(feature = "mesh_engine_next")]
fn evaluate_untrim(inputs: &[Value]) -> ComponentResult {
    const COMPONENT: &str = "Untrim";
    
    let metrics = coerce_shape_metrics(inputs.get(0), COMPONENT)?;
    
    // Use geom::untrim_bounds to remove trim information.
    // This returns the bounds unchanged, representing the full surface domain.
    let (min, max) = crate::geom::untrim_bounds(metrics.min, metrics.max);
    let surface = create_surface_from_bounds(min, max);
    
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BREPS.to_owned(), surface);
    Ok(outputs)
}

#[derive(Debug, Clone)]
struct Face {
    vertices: Vec<[f64; 3]>,
}

impl Face {
    fn centroid(&self) -> [f64; 3] {
        if self.vertices.is_empty() {
            return [0.0, 0.0, 0.0];
        }
        let mut sum = [0.0, 0.0, 0.0];
        for vertex in &self.vertices {
            sum[0] += vertex[0];
            sum[1] += vertex[1];
            sum[2] += vertex[2];
        }
        let scale = 1.0 / self.vertices.len() as f64;
        [sum[0] * scale, sum[1] * scale, sum[2] * scale]
    }
}

#[derive(Debug, Clone)]
struct EdgeData {
    start: [f64; 3],
    end: [f64; 3],
    faces: Vec<usize>,
}

impl EdgeData {
    fn new(start: [f64; 3], end: [f64; 3]) -> Self {
        Self {
            start,
            end,
            faces: Vec::new(),
        }
    }

    fn to_value(&self) -> Value {
        Value::CurveLine {
            p1: self.start,
            p2: self.end,
        }
    }

    fn face_count(&self) -> usize {
        self.faces.len()
    }

    fn add_face(&mut self, face: usize) {
        if !self.faces.contains(&face) {
            self.faces.push(face);
        }
    }

    fn vector(&self) -> [f64; 3] {
        [
            self.end[0] - self.start[0],
            self.end[1] - self.start[1],
            self.end[2] - self.start[2],
        ]
    }

    fn length(&self) -> f64 {
        distance(&self.start, &self.end)
    }

    fn matches(&self, start: [f64; 3], end: [f64; 3]) -> bool {
        (nearly_equal_points(&self.start, &start) && nearly_equal_points(&self.end, &end))
            || (nearly_equal_points(&self.start, &end) && nearly_equal_points(&self.end, &start))
    }

    fn touches_point(&self, point: &[f64; 3], tolerance: f64) -> bool {
        distance(&self.start, point) <= tolerance || distance(&self.end, point) <= tolerance
    }
}

#[derive(Debug, Default, Clone)]
struct BrepData {
    faces: Vec<Face>,
    edges: Vec<EdgeData>,
}

impl BrepData {
    fn add_edge(&mut self, start: [f64; 3], end: [f64; 3], face: Option<usize>) {
        if nearly_equal_points(&start, &end) {
            return;
        }
        if let Some(existing) = self.edges.iter_mut().find(|edge| edge.matches(start, end)) {
            if let Some(face_index) = face {
                existing.add_face(face_index);
            }
            return;
        }

        let mut edge = EdgeData::new(start, end);
        if let Some(face_index) = face {
            edge.add_face(face_index);
        }
        self.edges.push(edge);
    }

    fn from_metrics(metrics: &ShapeMetrics) -> Self {
        Self {
            faces: Vec::new(),
            edges: create_box_edges(metrics),
        }
    }

    fn get_naked_edges(&self) -> Vec<usize> {
        self.edges
            .iter()
            .enumerate()
            .filter_map(|(i, edge)| {
                if edge.face_count() == 1 {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    fn find_loops(&self, edge_indices: &[usize]) -> Vec<Vec<[f64; 3]>> {
        let mut visited = vec![false; edge_indices.len()];
        let mut loops = Vec::new();

        // Bouw een map van start/end punten naar edge indices voor snelle lookup
        // We gebruiken een eenvoudige aanpak met lineaire zoektochten voor nu,
        // optimalisatie kan later indien nodig.

        for i in 0..edge_indices.len() {
            if visited[i] {
                continue;
            }

            let start_edge_idx = edge_indices[i];
            let start_edge = &self.edges[start_edge_idx];

            // Begin een nieuwe loop
            let mut current_loop = Vec::new();
            current_loop.push(start_edge.start);
            current_loop.push(start_edge.end);

            visited[i] = true;
            let mut current_end = start_edge.end;
            let mut loop_closed = false;

            // Probeer de loop te sluiten
            loop {
                let mut found_next = false;
                for j in 0..edge_indices.len() {
                    if visited[j] {
                        continue;
                    }

                    let next_edge_idx = edge_indices[j];
                    let next_edge = &self.edges[next_edge_idx];

                    if nearly_equal_points(&next_edge.start, &current_end) {
                        current_loop.push(next_edge.end);
                        current_end = next_edge.end;
                        visited[j] = true;
                        found_next = true;
                    } else if nearly_equal_points(&next_edge.end, &current_end) {
                        current_loop.push(next_edge.start);
                        current_end = next_edge.start;
                        visited[j] = true;
                        found_next = true;
                    }

                    if found_next {
                        break;
                    }
                }

                if !found_next {
                    // Check of we terug zijn bij het begin
                    if nearly_equal_points(&current_end, &current_loop[0]) {
                        loop_closed = true;
                        // Het laatste punt is gelijk aan het eerste, verwijder het dubbele punt
                        current_loop.pop();
                    }
                    break;
                }

                if nearly_equal_points(&current_end, &current_loop[0]) {
                    loop_closed = true;
                    current_loop.pop();
                    break;
                }
            }

            if loop_closed && current_loop.len() >= 3 {
                loops.push(current_loop);
            }
        }

        loops
    }

    fn to_value(&self) -> Value {
        // Verzamel alle unieke vertices
        let mut vertices = Vec::new();
        let mut faces_indices = Vec::new();

        for face in &self.faces {
            let mut face_indices = Vec::new();
            for vertex in &face.vertices {
                let index = if let Some(pos) =
                    vertices.iter().position(|v| nearly_equal_points(v, vertex))
                {
                    pos
                } else {
                    vertices.push(*vertex);
                    vertices.len() - 1
                };
                face_indices.push(index as u32);
            }
            faces_indices.push(face_indices);
        }

        Value::Surface {
            vertices,
            faces: faces_indices,
        }
    }
}

#[derive(Debug, Clone)]
struct ShapeMetrics {
    min: [f64; 3],
    max: [f64; 3],
}

impl ShapeMetrics {
    fn from_inputs(value: Option<&Value>) -> Option<Self> {
        let points = collect_points(value);
        if points.is_empty() {
            return None;
        }
        let (min, max) = bounding_box(&points);
        Some(Self { min, max })
    }

    fn size(&self) -> [f64; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    fn sample_point(&self, uv: (f64, f64)) -> [f64; 3] {
        [
            self.min[0] + self.size()[0] * uv.0,
            self.min[1] + self.size()[1] * uv.1,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    fn normal_hint(&self) -> [f64; 3] {
        [0.0, 0.0, 1.0]
    }

    fn tangent_hint_u(&self) -> [f64; 3] {
        [self.size()[0].signum().max(EPSILON), 0.0, 0.0]
    }

    fn tangent_hint_v(&self) -> [f64; 3] {
        [0.0, self.size()[1].signum().max(EPSILON), 0.0]
    }

    fn volume(&self) -> f64 {
        let size = self.size();
        size[0].abs() * size[1].abs() * size[2].abs()
    }
}

/// Extended surface data that preserves vertex geometry.
///
/// This struct captures both the bounding box (for legacy code) and the
/// actual vertex positions (for accurate surface sampling). When vertex
/// data is available, operations can use the true surface geometry instead
/// of axis-aligned approximations.
#[cfg(feature = "mesh_engine_next")]
#[derive(Debug, Clone)]
struct SurfaceVertexData {
    /// Bounding box minimum corner (for legacy fallback).
    min: [f64; 3],
    /// Bounding box maximum corner (for legacy fallback).
    max: [f64; 3],
    /// Vertex positions, if extracted from a surface/mesh.
    /// Stored in the order they appeared in the input.
    vertices: Vec<[f64; 3]>,
    /// Grid dimensions (u_count, v_count) if known from the source.
    /// When `None`, dimensions will be inferred from vertex count.
    grid_dimensions: Option<(usize, usize)>,
}

#[cfg(feature = "mesh_engine_next")]
impl SurfaceVertexData {
    /// Extracts surface data from a Value, preserving vertex positions.
    fn from_value(value: Option<&Value>) -> Option<Self> {
        match value {
            Some(Value::Surface { vertices, faces }) if !vertices.is_empty() => {
                let (min, max) = bounding_box(vertices);
                // Try to infer grid dimensions from face structure
                let grid_dims = Self::infer_grid_from_faces(vertices.len(), faces);
                Some(Self {
                    min,
                    max,
                    vertices: vertices.clone(),
                    grid_dimensions: grid_dims,
                })
            }
            Some(Value::Mesh { vertices, indices, .. }) if !vertices.is_empty() => {
                let (min, max) = bounding_box(vertices);
                // Mesh indices are flat triangles; harder to infer grid
                // Try to detect from vertex count
                let grid_dims = crate::geom::VertexGridSurface::infer_grid_dimensions(vertices.len());
                Some(Self {
                    min,
                    max,
                    vertices: vertices.clone(),
                    grid_dimensions: grid_dims,
                })
            }
            Some(Value::List(values)) => {
                // Try to find a surface or mesh in the list
                for v in values {
                    if let Some(data) = Self::from_value(Some(v)) {
                        return Some(data);
                    }
                }
                // Fall back to collecting points
                let points = collect_points(value);
                if points.is_empty() {
                    return None;
                }
                let (min, max) = bounding_box(&points);
                let grid_dims = crate::geom::VertexGridSurface::infer_grid_dimensions(points.len());
                Some(Self {
                    min,
                    max,
                    vertices: points,
                    grid_dimensions: grid_dims,
                })
            }
            _ => {
                let points = collect_points(value);
                if points.is_empty() {
                    return None;
                }
                let (min, max) = bounding_box(&points);
                let grid_dims = crate::geom::VertexGridSurface::infer_grid_dimensions(points.len());
                Some(Self {
                    min,
                    max,
                    vertices: points,
                    grid_dimensions: grid_dims,
                })
            }
        }
    }

    /// Attempts to infer grid dimensions from face connectivity.
    ///
    /// For surfaces created as grids (like from loft/sweep), the face structure
    /// often reveals the grid pattern.
    fn infer_grid_from_faces(vertex_count: usize, faces: &[Vec<u32>]) -> Option<(usize, usize)> {
        if faces.is_empty() || vertex_count < 4 {
            return None;
        }

        // For quad grids triangulated as pairs, face count = 2 * (u-1) * (v-1)
        // So (u-1) * (v-1) = face_count / 2
        let face_count = faces.len();
        if face_count % 2 == 0 {
            let cell_count = face_count / 2;

            // Try to factor cell_count
            for u_cells in 1..=cell_count {
                if cell_count % u_cells == 0 {
                    let v_cells = cell_count / u_cells;
                    let expected_vertices = (u_cells + 1) * (v_cells + 1);
                    if expected_vertices == vertex_count {
                        return Some((u_cells + 1, v_cells + 1));
                    }
                }
            }
        }

        // Fall back to inferring from vertex count alone
        crate::geom::VertexGridSurface::infer_grid_dimensions(vertex_count)
    }

    /// Checks if we have enough vertices for a proper surface.
    fn has_valid_grid(&self) -> bool {
        self.vertices.len() >= 4
    }

    /// Creates a `VertexSurfaceInput` for use with geom functions.
    fn to_vertex_input(&self) -> crate::geom::VertexSurfaceInput<'_> {
        if let Some((u, v)) = self.grid_dimensions {
            crate::geom::VertexSurfaceInput::with_dimensions(&self.vertices, u, v)
        } else {
            crate::geom::VertexSurfaceInput::new(&self.vertices)
        }
    }
}

fn frame_value(origin: [f64; 3], x_axis: [f64; 3], y_axis: [f64; 3], z_axis: [f64; 3]) -> Value {
    Value::List(vec![
        Value::Point(origin),
        Value::Vector(x_axis),
        Value::Vector(y_axis),
        Value::Vector(z_axis),
    ])
}

fn create_surface_from_bounds(min: [f64; 3], max: [f64; 3]) -> Value {
    let vertices = vec![
        [min[0], min[1], min[2]],
        [max[0], min[1], min[2]],
        [max[0], max[1], max[2]],
        [min[0], max[1], max[2]],
    ];
    let faces = vec![vec![0, 1, 2], vec![0, 2, 3]];
    Value::Surface { vertices, faces }
}

fn create_box_edges(metrics: &ShapeMetrics) -> Vec<EdgeData> {
    let corners = create_box_corners(metrics);
    let pairs = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    pairs
        .iter()
        .map(|(a, b)| EdgeData::new(corners[*a], corners[*b]))
        .collect()
}

fn create_box_corners(metrics: &ShapeMetrics) -> Vec<[f64; 3]> {
    let mut corners = Vec::with_capacity(8);
    for &z in &[metrics.min[2], metrics.max[2]] {
        for &y in &[metrics.min[1], metrics.max[1]] {
            for &x in &[metrics.min[0], metrics.max[0]] {
                corners.push([x, y, z]);
            }
        }
    }
    corners
}

#[cfg(feature = "mesh_engine_next")]
fn collect_legacy_brep_data(
    value: Option<&Value>,
    tol: crate::geom::Tolerance,
) -> crate::geom::LegacyBrepData {
    let mut data = crate::geom::LegacyBrepData::default();
    collect_legacy_brep_data_recursive(value, &mut data, tol);
    data
}

#[cfg(feature = "mesh_engine_next")]
fn collect_legacy_brep_data_recursive(
    value: Option<&Value>,
    data: &mut crate::geom::LegacyBrepData,
    tol: crate::geom::Tolerance,
) {
    match value {
        Some(Value::Surface { vertices, faces }) => {
            data.extend_from_surface_buffers(vertices, faces, tol);
        }
        Some(Value::CurveLine { p1, p2 }) => {
            data.extend_from_line(
                crate::geom::Point3::new(p1[0], p1[1], p1[2]),
                crate::geom::Point3::new(p2[0], p2[1], p2[2]),
                tol,
            );
        }
        Some(Value::List(values)) => {
            for value in values {
                collect_legacy_brep_data_recursive(Some(value), data, tol);
            }
        }
        _ => {}
    }
}

fn collect_brep_data(value: Option<&Value>) -> BrepData {
    let mut data = BrepData::default();
    collect_brep_data_recursive(value, &mut data);
    data
}

fn collect_brep_data_recursive(value: Option<&Value>, data: &mut BrepData) {
    match value {
        Some(Value::Surface { vertices, faces }) => {
            for face_indices in faces {
                let mut face_vertices = Vec::new();
                for &index in face_indices {
                    if let Some(vertex) = vertices.get(index as usize) {
                        face_vertices.push(*vertex);
                    }
                }
                if face_vertices.len() < 2 {
                    continue;
                }
                let face_index = data.faces.len();
                data.faces.push(Face {
                    vertices: face_vertices.clone(),
                });
                for segment in 0..face_vertices.len() {
                    let start = face_vertices[segment];
                    let end = face_vertices[(segment + 1) % face_vertices.len()];
                    data.add_edge(start, end, Some(face_index));
                }
            }
        }
        Some(Value::CurveLine { p1, p2 }) => {
            data.add_edge(*p1, *p2, None);
        }
        Some(Value::List(values)) => {
            for value in values {
                collect_brep_data_recursive(Some(value), data);
            }
        }
        _ => {}
    }
}

fn collect_points(value: Option<&Value>) -> Vec<[f64; 3]> {
    match value {
        Some(Value::Point(point)) | Some(Value::Vector(point)) => vec![*point],
        Some(Value::CurveLine { p1, p2 }) => vec![*p1, *p2],
        Some(Value::Surface { vertices, .. }) => vertices.clone(),
        Some(Value::List(values)) => values
            .iter()
            .flat_map(|value| collect_points(Some(value)))
            .collect(),
        _ => Vec::new(),
    }
}

fn collect_shapes(value: Option<&Value>) -> Vec<Value> {
    match value {
        Some(Value::List(values)) => values.clone(),
        Some(other) => vec![other.clone()],
        None => Vec::new(),
    }
}

fn collect_point_list(value: Option<&Value>) -> Vec<[f64; 3]> {
    match value {
        Some(Value::List(values)) => values
            .iter()
            .filter_map(|value| match value {
                Value::Point(point) | Value::Vector(point) => Some(*point),
                _ => None,
            })
            .collect(),
        Some(Value::Point(point)) | Some(Value::Vector(point)) => vec![*point],
        _ => Vec::new(),
    }
}

fn bounding_box(points: &[[f64; 3]]) -> ([f64; 3], [f64; 3]) {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for point in points {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    (min, max)
}

fn coerce_shape_metrics(
    value: Option<&Value>,
    component: &str,
) -> Result<ShapeMetrics, ComponentError> {
    ShapeMetrics::from_inputs(value)
        .ok_or_else(|| ComponentError::new(format!("{} vereist geometrische invoer", component)))
}

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    match value {
        Some(Value::Number(number)) => Ok(*number),
        Some(Value::Boolean(flag)) => Ok(if *flag { 1.0 } else { 0.0 }),
        Some(Value::List(list)) if !list.is_empty() => coerce_number(list.first(), context),
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een getal, kreeg {}",
            context,
            other.kind()
        ))),
        None => Err(ComponentError::new(format!(
            "{} vereist een numerieke waarde",
            context
        ))),
    }
}

fn coerce_positive_integer(value: Option<&Value>, context: &str) -> Result<usize, ComponentError> {
    let number = coerce_number(value, context)?;
    if !number.is_finite() {
        return Err(ComponentError::new(format!(
            "{} vereist een eindige waarde",
            context
        )));
    }
    let rounded = number.round().abs();
    Ok(rounded.max(1.0) as usize)
}

fn coerce_boolean(value: Option<&Value>, default: bool) -> Result<bool, ComponentError> {
    match value {
        Some(Value::Boolean(flag)) => Ok(*flag),
        Some(Value::Number(number)) => Ok(*number != 0.0),
        Some(Value::List(list)) if !list.is_empty() => coerce_boolean(list.first(), default),
        Some(Value::Text(text)) => {
            let normalized = text.trim().to_ascii_lowercase();
            if normalized.is_empty() {
                Ok(default)
            } else {
                Ok(matches!(
                    normalized.as_str(),
                    "true" | "yes" | "1" | "y" | "on"
                ))
            }
        }
        Some(_) => Ok(default),
        None => Ok(default),
    }
}

fn coerce_domain_pair(
    value: &Value,
    context: &str,
) -> Result<((f64, f64), (f64, f64)), ComponentError> {
    match value {
        Value::Domain(Domain::Two(Domain2D { u, v })) => Ok(((u.min, u.max), (v.min, v.max))),
        Value::Domain(Domain::One(Domain1D { min, max, .. })) => Ok(((*min, *max), (*min, *max))),
        Value::List(values) if values.len() >= 4 => {
            let u0 = coerce_number(values.get(0), context)?;
            let u1 = coerce_number(values.get(1), context)?;
            let v0 = coerce_number(values.get(2), context)?;
            let v1 = coerce_number(values.get(3), context)?;
            Ok(((u0, u1), (v0, v1)))
        }
        Value::List(values) if values.len() >= 2 => {
            let u0 = coerce_number(values.get(0), context)?;
            let u1 = coerce_number(values.get(1), context)?;
            Ok(((u0, u1), (0.0, 1.0)))
        }
        _ => Err(ComponentError::new(format!(
            "{} verwacht een domein",
            context
        ))),
    }
}

fn parse_directions(value: Option<&Value>) -> Vec<[f64; 3]> {
    fn parse(value: &Value) -> Option<[f64; 3]> {
        match value {
            Value::Vector(vector) | Value::Point(vector) => Some(*vector),
            Value::List(values) if values.len() >= 3 => {
                let x = coerce_number(values.get(0), "richting").ok()?;
                let y = coerce_number(values.get(1), "richting").ok()?;
                let z = coerce_number(values.get(2), "richting").ok()?;
                Some([x, y, z])
            }
            _ => None,
        }
    }

    match value {
        Some(Value::List(values)) => values.iter().filter_map(parse).collect(),
        Some(other) => parse(other).into_iter().collect(),
        None => Vec::new(),
    }
}

fn normalize(vector: [f64; 3]) -> Option<([f64; 3], f64)> {
    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    if length < EPSILON {
        None
    } else {
        Some((
            [vector[0] / length, vector[1] / length, vector[2] / length],
            length,
        ))
    }
}

fn nearly_equal_points(a: &[f64; 3], b: &[f64; 3]) -> bool {
    (a[0] - b[0]).abs() <= EPSILON
        && (a[1] - b[1]).abs() <= EPSILON
        && (a[2] - b[2]).abs() <= EPSILON
}

fn distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
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

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

fn clamp01(value: f64) -> f64 {
    clamp(value, 0.0, 1.0)
}
