//! Implements parameter components for geometry types.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::components::{Component, ComponentError, ComponentResult};
use crate::graph::node::MetaMap;
use crate::graph::value::{Value, ValueKind};

const SURFACE_EPSILON: f64 = 1e-9;
const SURFACE_EPSILON_SQUARED: f64 = SURFACE_EPSILON * SURFACE_EPSILON;

/// Defines a component's registration information.
pub struct Registration<T> {
    /// The component's kind.
    pub kind: T,
    /// A list of GUIDs that identify the component.
    pub guids: &'static [&'static str],
    /// A list of names and nicknames for the component.
    pub names: &'static [&'static str],
}

impl<T: Copy> Registration<T> {
    /// Creates a new `Registration` instance.
    pub const fn new(
        kind: T,
        guids: &'static [&'static str],
        names: &'static [&'static str],
    ) -> Self {
        Self { kind, guids, names }
    }
}

// --- ComponentKind Enum ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentKind {
    Point,
    Vector,
    Line,
    Mesh,
    Surface,
    CircularArc,
    Transform,
    Field,
    Plane,
    TwistedBox,
    Location,
    SubD,
    Brep,
    Atom,
    Rectangle,
    Geometry,
    Group,
    GeometryPipeline,
    MesherSettings,
    Box,
    Circle,
    Curve,
    MeshFace,
    GeometryCache,
    MeshPoint,
}

impl ComponentKind {
    pub fn evaluate(self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Point => PointComponent.evaluate(inputs, meta),
            Self::Vector => VectorComponent.evaluate(inputs, meta),
            Self::Line => LineComponent.evaluate(inputs, meta),
            Self::Mesh => MeshComponent.evaluate(inputs, meta),
            Self::Surface => SurfaceComponent.evaluate(inputs, meta),
            Self::Curve => CurveComponent.evaluate(inputs, meta),
            Self::MeshFace => MeshFaceComponent.evaluate(inputs, meta),
            Self::Plane => PlaneComponent.evaluate(inputs, meta),
            // Placeholders
            Self::CircularArc => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::Transform => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::Field => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::TwistedBox => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::Location => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::SubD => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::Brep => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::Atom => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::Rectangle => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::Geometry => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::Group => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::GeometryPipeline => {
                Err(ComponentError::NotYetImplemented(self.name().to_string()))
            }
            Self::MesherSettings => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::Box => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::Circle => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::GeometryCache => Err(ComponentError::NotYetImplemented(self.name().to_string())),
            Self::MeshPoint => Err(ComponentError::NotYetImplemented(self.name().to_string())),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Point => "Point",
            Self::Vector => "Vector",
            Self::Line => "Line",
            Self::Mesh => "Mesh",
            Self::Surface => "Surface",
            Self::CircularArc => "Circular Arc",
            Self::Transform => "Transform",
            Self::Field => "Field",
            Self::Plane => "Plane",
            Self::TwistedBox => "Twisted Box",
            Self::Location => "Location",
            Self::SubD => "SubD",
            Self::Brep => "Brep",
            Self::Atom => "Atom",
            Self::Rectangle => "Rectangle",
            Self::Geometry => "Geometry",
            Self::Group => "Group",
            Self::GeometryPipeline => "Geometry Pipeline",
            Self::MesherSettings => "Mesher Settings",
            Self::Box => "Box",
            Self::Circle => "Circle",
            Self::Curve => "Curve",
            Self::MeshFace => "Mesh Face",
            Self::GeometryCache => "Geometry Cache",
            Self::MeshPoint => "Mesh Point",
        }
    }
}

// A macro to define a parameter component that passes through a specific `Value` type.
macro_rules! define_param_component {
    (
        $struct_name:ident,
        $output_pin:expr,
        $expected_kind:path
    ) => {
        #[derive(Debug, Default, Clone, Copy)]
        struct $struct_name;

        impl Component for $struct_name {
            fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
                if inputs.is_empty() {
                    let mut outputs = BTreeMap::new();
                    outputs.insert($output_pin.to_owned(), Value::Null);
                    return Ok(outputs);
                }

                let input_value = &inputs[0];

                let is_valid = match input_value {
                    Value::List(items) => items
                        .iter()
                        .all(|item| item.kind() == $expected_kind || matches!(item, Value::Null)),
                    value => value.kind() == $expected_kind || matches!(value, Value::Null),
                };

                if !is_valid {
                    return Err(ComponentError::new(format!(
                        "Expected {} or a List of {}, but got {}.",
                        $expected_kind,
                        $expected_kind,
                        input_value.kind()
                    )));
                }

                let mut outputs = BTreeMap::new();
                outputs.insert($output_pin.to_owned(), input_value.clone());
                Ok(outputs)
            }
        }
    };
}

// --- Implemented Components ---
define_param_component!(PointComponent, "Pt", ValueKind::Point);
define_param_component!(VectorComponent, "Vec", ValueKind::Vector);
define_param_component!(LineComponent, "Line", ValueKind::CurveLine);

// MeshComponent: Accept both Value::Mesh (preferred) and Value::Surface (legacy)
// This is implemented manually to handle both types properly.

/// The Mesh parameter component accepts both `Value::Mesh` (preferred) and
/// `Value::Surface` (legacy) inputs for backward compatibility.
///
/// When a `Value::Surface` is provided, it is converted to `Value::Mesh` format.
/// This ensures components downstream can rely on the modern mesh representation.
#[derive(Debug, Default, Clone, Copy)]
struct MeshComponent;

impl Component for MeshComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            let mut outputs = BTreeMap::new();
            outputs.insert("Mesh".to_owned(), Value::Null);
            return Ok(outputs);
        }

        let input_value = &inputs[0];
        let mesh_value = convert_to_mesh_value(input_value)?;

        let mut outputs = BTreeMap::new();
        outputs.insert("Mesh".to_owned(), mesh_value);
        Ok(outputs)
    }
}

/// The Surface parameter component accepts both `Value::Surface` (native) and
/// `Value::Mesh` (converted) inputs for interoperability.
///
/// When a `Value::Mesh` is provided, it is converted to `Value::Surface` format
/// to maintain backward compatibility with legacy consumers expecting surfaces.
#[derive(Debug, Default, Clone, Copy)]
struct SurfaceComponent;

impl Component for SurfaceComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            let mut outputs = BTreeMap::new();
            outputs.insert("Srf".to_owned(), Value::Null);
            return Ok(outputs);
        }

        let input_value = &inputs[0];
        let surface_value = convert_to_surface_value(input_value)?;

        let mut outputs = BTreeMap::new();
        outputs.insert("Srf".to_owned(), surface_value);
        Ok(outputs)
    }
}

// ============================================================================
// Mesh/Surface Conversion Helpers
// ============================================================================

/// Converts any mesh-like value (`Value::Mesh` or `Value::Surface`) to `Value::Mesh`.
///
/// This is the preferred conversion direction as `Value::Mesh` is the modern
/// representation with support for normals, UVs, and diagnostics.
///
/// # Conversion Rules
///
/// - `Value::Mesh` → passed through unchanged
/// - `Value::Surface` → faces are triangulated (first 3 vertices of each face)
/// - `Value::List` → each element is recursively converted
/// - `Value::Null` → passed through unchanged
fn convert_to_mesh_value(value: &Value) -> Result<Value, ComponentError> {
    match value {
        Value::Null => Ok(Value::Null),
        Value::Mesh { .. } => Ok(value.clone()),
        Value::Surface { vertices, faces } => {
            // Convert polygon faces to triangle indices (take first 3 vertices per face)
            let indices: Vec<u32> = faces
                .iter()
                .filter(|f| f.len() >= 3)
                .flat_map(|f| vec![f[0], f[1], f[2]])
                .collect();
            Ok(Value::Mesh {
                vertices: vertices.clone(),
                indices,
                normals: None,
                uvs: None,
                diagnostics: None,
            })
        }
        Value::List(items) => {
            if list_contains_only_geometry_like_or_null(value) {
                let converted: Result<Vec<Value>, ComponentError> = items
                    .iter()
                    .map(convert_to_mesh_value)
                    .collect();
                return Ok(Value::List(converted?));
            }
            // Try to create a surface from flat geometry list, then convert to mesh
            if is_flat_geometry_list(items) {
                let surface = create_surface_from_flat_list(items)?;
                return convert_to_mesh_value(&surface);
            }
            // Recursively convert list entries
            let converted: Result<Vec<Value>, ComponentError> = items
                .iter()
                .map(convert_to_mesh_value)
                .collect();
            Ok(Value::List(converted?))
        }
        other => Err(ComponentError::new(format!(
            "Expected Mesh, Surface, or a List of geometry, but got {}.",
            other.kind()
        ))),
    }
}

/// Converts any mesh-like value (`Value::Mesh` or `Value::Surface`) to `Value::Surface`.
///
/// This is provided for backward compatibility with components that expect
/// the legacy `Value::Surface` representation.
///
/// # Conversion Rules
///
/// - `Value::Surface` → passed through unchanged
/// - `Value::Mesh` → indices converted to face lists (triangles become 3-element face lists)
/// - `Value::List` → each element is recursively converted
/// - `Value::Null` → passed through unchanged
///
/// # Note
///
/// Converting from `Value::Mesh` to `Value::Surface` is lossy: normals, UVs,
/// and diagnostics are discarded.
fn convert_to_surface_value(value: &Value) -> Result<Value, ComponentError> {
    match value {
        Value::Null => Ok(Value::Null),
        Value::Surface { .. } => Ok(value.clone()),
        Value::Mesh { vertices, indices, .. } => {
            // Convert triangle indices to face lists
            let faces: Vec<Vec<u32>> = indices
                .chunks(3)
                .filter(|chunk| chunk.len() == 3)
                .map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
                .collect();
            Ok(Value::Surface {
                vertices: vertices.clone(),
                faces,
            })
        }
        Value::List(items) => {
            if list_contains_only_geometry_like_or_null(value) {
                let converted: Result<Vec<Value>, ComponentError> = items
                    .iter()
                    .map(convert_to_surface_value)
                    .collect();
                return Ok(Value::List(converted?));
            }
            // Try to create a surface from flat geometry list
            if is_flat_geometry_list(items) {
                return create_surface_from_flat_list(items);
            }
            // Recursively convert list entries (legacy behavior)
            convert_list_value_to_surface(value)
        }
        other => Err(ComponentError::new(format!(
            "Expected Surface, Mesh, or a List of geometry, but got {}.",
            other.kind()
        ))),
    }
}

/// Legacy convert_list_value that only outputs surfaces.
///
/// This handles the original behavior for backward compatibility.
fn convert_list_value_to_surface(value: &Value) -> Result<Value, ComponentError> {
    let entries = match value {
        Value::List(entries) => entries,
        _ => unreachable!(),
    };

    let mut converted = Vec::with_capacity(entries.len());
    for entry in entries {
        let converted_entry = match entry {
            Value::Surface { .. } => entry.clone(),
            Value::Mesh { vertices, indices, .. } => {
                // Convert Mesh to Surface for backward compatibility
                let faces: Vec<Vec<u32>> = indices
                    .chunks(3)
                    .filter(|chunk| chunk.len() == 3)
                    .map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
                    .collect();
                Value::Surface {
                    vertices: vertices.clone(),
                    faces,
                }
            }
            Value::Null => Value::Null,
            Value::List(_) => convert_list_value_to_surface(entry)?,
            other => {
                return Err(ComponentError::new(format!(
                    "Expected Surface, Mesh, or a List of geometry, but got {}.",
                    other.kind()
                )));
            }
        };
        converted.push(converted_entry);
    }

    Ok(Value::List(converted))
}

// ============================================================================
// List Content Validation Helpers
// ============================================================================

/// Checks if a list contains only geometry-like values (Mesh or Surface) or nulls.
///
/// This supports both the new `Value::Mesh` type and the legacy `Value::Surface` type
/// for backward compatibility.
fn list_contains_only_geometry_like_or_null(value: &Value) -> bool {
    matches!(value, Value::List(items) if items.iter().all(|item| matches!(item, Value::Surface { .. } | Value::Mesh { .. } | Value::Null)))
}

/// Legacy alias for backward compatibility - checks for surfaces or null only.
#[allow(dead_code)]
fn list_contains_only_surfaces_or_null(value: &Value) -> bool {
    matches!(value, Value::List(items) if items.iter().all(|item| matches!(item, Value::Surface { .. } | Value::Null)))
}

/// Legacy conversion function that handles list values for surface conversion.
///
/// This function is kept for backward compatibility but has been updated to
/// also handle `Value::Mesh` inputs by converting them to `Value::Surface`.
#[allow(dead_code)]
fn convert_list_value(value: &Value) -> Result<Value, ComponentError> {
    let entries = match value {
        Value::List(entries) => entries,
        _ => unreachable!(),
    };

    // Check if list contains only geometry-like values (Surface or Mesh)
    if list_contains_only_geometry_like_or_null(value) {
        return Ok(value.clone());
    }

    if is_flat_geometry_list(entries) {
        return create_surface_from_flat_list(entries);
    }

    let mut converted = Vec::with_capacity(entries.len());
    for entry in entries {
        let converted_entry = match entry {
            Value::Surface { .. } => entry.clone(),
            Value::Mesh { vertices, indices, .. } => {
                // Convert Mesh to Surface for backward compatibility
                let faces: Vec<Vec<u32>> = indices
                    .chunks(3)
                    .filter(|chunk| chunk.len() == 3)
                    .map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
                    .collect();
                Value::Surface {
                    vertices: vertices.clone(),
                    faces,
                }
            }
            Value::Null => Value::Null,
            Value::List(_) => convert_list_value(entry)?,
            other => {
                return Err(ComponentError::new(format!(
                    "Expected Surface, Mesh, or a List of geometry, but got {}.",
                    other.kind()
                )));
            }
        };
        converted.push(converted_entry);
    }

    Ok(Value::List(converted))
}

fn is_flat_geometry_list(entries: &[Value]) -> bool {
    let mut has_geometry = false;
    for entry in entries {
        match entry {
            Value::Point(_) | Value::Vector(_) | Value::CurveLine { .. } => has_geometry = true,
            Value::Null => {}
            _ => {
                return false;
            }
        }
    }
    has_geometry
}

fn create_surface_from_flat_list(entries: &[Value]) -> Result<Value, ComponentError> {
    let mut points = Vec::new();
    for entry in entries {
        match entry {
            Value::Point(point) | Value::Vector(point) => points.push(*point),
            Value::CurveLine { p1, p2 } => {
                points.push(*p1);
                points.push(*p2);
            }
            Value::Null => {}
            other => {
                return Err(ComponentError::new(format!(
                    "Surface expected points, but got {}.",
                    other.kind()
                )));
            }
        }
    }

    if points.len() > 1 && points.first() == points.last() {
        points.pop();
    }

    let unique_points = deduplicate_points(&points);
    if unique_points.len() < 3 {
        return Err(ComponentError::new(format!(
            "Surface requires at least three points, got {}.",
            unique_points.len()
        )));
    }

    let normal = compute_plane_normal(&unique_points).ok_or_else(|| {
        ComponentError::new("Surface requires at least three non-collinear points.")
    })?;
    let centroid = compute_centroid(&unique_points);
    let (axis_x, axis_y) = find_plane_axes(&unique_points, centroid, normal).ok_or_else(|| {
        ComponentError::new("Surface geometry could not determine an orientation.")
    })?;

    let sorted = sort_points_by_angle(&unique_points, centroid, axis_x, axis_y);
    let faces: Vec<Vec<u32>> = (1..sorted.len() - 1)
        .map(|i| vec![0, i as u32, (i + 1) as u32])
        .collect();

    Ok(Value::Surface {
        vertices: sorted,
        faces,
    })
}

fn deduplicate_points(points: &[[f64; 3]]) -> Vec<[f64; 3]> {
    let mut unique = Vec::new();
    'outer: for &point in points {
        for existing in &unique {
            if point_distance_squared(existing, &point) <= SURFACE_EPSILON_SQUARED {
                continue 'outer;
            }
        }
        unique.push(point);
    }
    unique
}

fn compute_centroid(points: &[[f64; 3]]) -> [f64; 3] {
    let mut centroid = [0.0; 3];
    for point in points {
        centroid[0] += point[0];
        centroid[1] += point[1];
        centroid[2] += point[2];
    }
    let count = points.len() as f64;
    centroid[0] /= count;
    centroid[1] /= count;
    centroid[2] /= count;
    centroid
}

fn compute_plane_normal(points: &[[f64; 3]]) -> Option<[f64; 3]> {
    for i in 1..points.len() {
        let a = subtract(points[i], points[0]);
        if vector_length_squared(a) <= SURFACE_EPSILON_SQUARED {
            continue;
        }
        for j in i + 1..points.len() {
            let b = subtract(points[j], points[0]);
            if vector_length_squared(b) <= SURFACE_EPSILON_SQUARED {
                continue;
            }
            let normal = cross(a, b);
            if vector_length_squared(normal) > SURFACE_EPSILON_SQUARED {
                return Some(normalize(normal));
            }
        }
    }
    None
}

fn find_plane_axes(
    points: &[[f64; 3]],
    centroid: [f64; 3],
    normal: [f64; 3],
) -> Option<([f64; 3], [f64; 3])> {
    for point in points {
        let diff = subtract(*point, centroid);
        if vector_length_squared(diff) <= SURFACE_EPSILON_SQUARED {
            continue;
        }
        let axis_x = normalize(diff);
        let axis_y = cross(normal, axis_x);
        if vector_length_squared(axis_y) <= SURFACE_EPSILON_SQUARED {
            continue;
        }
        return Some((axis_x, normalize(axis_y)));
    }
    None
}

fn sort_points_by_angle(
    points: &[[f64; 3]],
    centroid: [f64; 3],
    axis_x: [f64; 3],
    axis_y: [f64; 3],
) -> Vec<[f64; 3]> {
    let mut entries: Vec<(f64, [f64; 3])> = points
        .iter()
        .map(|point| {
            let diff = subtract(*point, centroid);
            let x = dot(diff, axis_x);
            let y = dot(diff, axis_y);
            (y.atan2(x), *point)
        })
        .collect();

    entries.sort_by(|a, b| match a.0.partial_cmp(&b.0) {
        Some(order) => order,
        None => Ordering::Equal,
    });

    entries.into_iter().map(|entry| entry.1).collect()
}

fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
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

fn vector_length_squared(v: [f64; 3]) -> f64 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

fn point_distance_squared(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = vector_length_squared(v).sqrt();
    if len <= SURFACE_EPSILON {
        [0.0, 0.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

define_param_component!(CurveComponent, "Crv", ValueKind::CurveLine);
define_param_component!(MeshFaceComponent, "Face", ValueKind::Text);

// --- Placeholder Components ---
// define_param_component!(CircularArcComponent, "Arc", ValueKind::CircularArc);
// define_param_component!(TransformComponent, "Transform", ValueKind::Transform);
// define_param_component!(FieldComponent, "Field", ValueKind::Field);
// define_param_component!(TwistedBoxComponent, "TBox", ValueKind::TwistedBox);

#[derive(Debug, Default, Clone, Copy)]
struct PlaneComponent;

impl Component for PlaneComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            let mut outputs = BTreeMap::new();
            outputs.insert("Pln".to_owned(), Value::Null);
            return Ok(outputs);
        }

        // We simply pass through values since we don't have a dedicated Plane type yet.
        // Validation logic is omitted for now.
        let mut outputs = BTreeMap::new();
        outputs.insert("Pln".to_owned(), inputs[0].clone());
        Ok(outputs)
    }
}
// define_param_component!(LocationComponent, "Loc", ValueKind::Location);
// define_param_component!(SubDComponent, "SubD", ValueKind::SubD);
// define_param_component!(BrepComponent, "Brep", ValueKind::Brep);
// define_param_component!(AtomComponent, "Atom", ValueKind::Atom);
// define_param_component!(RectangleComponent, "Rec", ValueKind::Rectangle);
// define_param_component!(GeometryComponent, "Geo", ValueKind::Geometry);
// define_param_component!(GroupComponent, "Grp", ValueKind::Group);
// define_param_component!(GeometryPipelineComponent, "Pipeline", ValueKind::GeometryPipeline);
// define_param_component!(MesherSettingsComponent, "Mesh", ValueKind::MesherSettings);
// define_param_component!(BoxComponent, "Box", ValueKind::Box);
// define_param_component!(CircleComponent, "Circle", ValueKind::Circle);
// define_param_component!(GeometryCacheComponent, "Geometry Cache", ValueKind::GeometryCache);
// define_param_component!(MeshPointComponent, "MPoint", ValueKind::MeshPoint);

// --- Registrations ---
pub const REGISTRATIONS: &[Registration<ComponentKind>] = &[
    // Implemented
    Registration::new(
        ComponentKind::Point,
        &["fbac3e32-f100-4292-8692-77240a42fd1a"],
        &["Point", "Pt"],
    ),
    Registration::new(
        ComponentKind::Vector,
        &["16ef3e75-e315-4899-b531-d3166b42dac9"],
        &["Vector", "Vec"],
    ),
    Registration::new(
        ComponentKind::Line,
        &["8529dbdf-9b6f-42e9-8e1f-c7a2bde56a70"],
        &["Line"],
    ),
    Registration::new(
        ComponentKind::Mesh,
        &["1e936df3-0eea-4246-8549-514cb8862b7a"],
        &["Mesh"],
    ),
    Registration::new(
        ComponentKind::Surface,
        &["deaf8653-5528-4286-807c-3de8b8dad781"],
        &["Surface", "Srf"],
    ),
    Registration::new(
        ComponentKind::Curve,
        &["d5967b9f-e8ee-436b-a8ad-29fdcecf32d5"],
        &["Curve", "Crv"],
    ),
    Registration::new(
        ComponentKind::MeshFace,
        &["e02b3da5-543a-46ac-a867-0ba6b0a524de"],
        &["Mesh Face", "Face"],
    ),
    // Placeholders
    Registration::new(
        ComponentKind::CircularArc,
        &["04d3eace-deaa-475e-9e69-8f804d687998"],
        &["Circular Arc", "Arc"],
    ),
    Registration::new(
        ComponentKind::Transform,
        &["28f40e48-e739-4211-91bd-f4aefa5965f8"],
        &["Transform"],
    ),
    Registration::new(
        ComponentKind::Field,
        &["3175e3eb-1ae0-4d0b-9395-53fd3e8f8a28"],
        &["Field"],
    ),
    Registration::new(
        ComponentKind::Plane,
        &["4f8984c4-7c7a-4d69-b0a2-183cbb330d20"],
        &["Plane", "Pln"],
    ),
    Registration::new(
        ComponentKind::TwistedBox,
        &["6db039c4-cad1-4549-bd45-e31cb0f71692"],
        &["Twisted Box", "TBox"],
    ),
    Registration::new(
        ComponentKind::Location,
        &["87391af3-35fe-4a40-b001-2bd4547ccd45"],
        &["Location", "Loc"],
    ),
    Registration::new(
        ComponentKind::SubD,
        &["89cd1a12-0007-4581-99ba-66578665e610"],
        &["SubD"],
    ),
    Registration::new(
        ComponentKind::Brep,
        &["919e146f-30ae-4aae-be34-4d72f555e7da"],
        &["Brep"],
    ),
    Registration::new(
        ComponentKind::Atom,
        &["a80395af-f134-4d6a-9b89-15edf3161619"],
        &["Atom"],
    ),
    Registration::new(
        ComponentKind::Rectangle,
        &["abf9c670-5462-4cd8-acb3-f1ab0256dbf3"],
        &["Rectangle", "Rec"],
    ),
    Registration::new(
        ComponentKind::Geometry,
        &["ac2bc2cb-70fb-4dd5-9c78-7e1ea97fe278"],
        &["Geometry", "Geo"],
    ),
    Registration::new(
        ComponentKind::Group,
        &["b0851fc0-ab55-47d8-bdda-cc6306a40176"],
        &["Group", "Grp"],
    ),
    Registration::new(
        ComponentKind::GeometryPipeline,
        &["b341e2e5-c4b3-49a3-b3a4-b4e6e2054516"],
        &["Geometry Pipeline", "Pipeline"],
    ),
    Registration::new(
        ComponentKind::MesherSettings,
        &["c3407fda-b505-4686-9165-38fe7a9274cf"],
        &["Mesher Settings"],
    ),
    Registration::new(
        ComponentKind::Box,
        &["c9482db6-bea9-448d-98ff-fed6d69a8efc"],
        &["Box"],
    ),
    Registration::new(
        ComponentKind::Circle,
        &["d1028c72-ff86-4057-9eb0-36c687a4d98c"],
        &["Circle"],
    ),
    Registration::new(
        ComponentKind::GeometryCache,
        &["f91778ca-2700-42fc-8ee6-74049a2292b5"],
        &["Geometry Cache"],
    ),
    Registration::new(
        ComponentKind::MeshPoint,
        &["fa20fe95-5775-417b-92ff-b77c13cbf40c"],
        &["Mesh Point", "MPoint"],
    ),
];

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::value::Value;

    // -------------------------------------------------------------------------
    // MeshComponent Tests
    // -------------------------------------------------------------------------

    #[test]
    fn mesh_component_accepts_value_mesh() {
        let mesh = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        
        let meta = MetaMap::new();
        let result = MeshComponent.evaluate(&[mesh.clone()], &meta);
        
        assert!(result.is_ok(), "MeshComponent should accept Value::Mesh");
        let outputs = result.unwrap();
        assert!(outputs.contains_key("Mesh"));
        
        // The output should be a Mesh
        match &outputs["Mesh"] {
            Value::Mesh { vertices, indices, .. } => {
                assert_eq!(vertices.len(), 3);
                assert_eq!(indices.len(), 3);
            }
            _ => panic!("Expected Value::Mesh output"),
        }
    }

    #[test]
    fn mesh_component_accepts_value_surface_and_converts() {
        let surface = Value::Surface {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            faces: vec![vec![0, 1, 2]],
        };
        
        let meta = MetaMap::new();
        let result = MeshComponent.evaluate(&[surface], &meta);
        
        assert!(result.is_ok(), "MeshComponent should accept Value::Surface");
        let outputs = result.unwrap();
        
        // The output should be converted to Value::Mesh
        match &outputs["Mesh"] {
            Value::Mesh { vertices, indices, .. } => {
                assert_eq!(vertices.len(), 3);
                assert_eq!(indices.len(), 3);
                assert_eq!(indices, &[0, 1, 2]);
            }
            _ => panic!("Expected Value::Mesh output"),
        }
    }

    #[test]
    fn mesh_component_handles_null() {
        let meta = MetaMap::new();
        let result = MeshComponent.evaluate(&[Value::Null], &meta);
        
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert!(matches!(outputs["Mesh"], Value::Null));
    }

    #[test]
    fn mesh_component_handles_empty_inputs() {
        let meta = MetaMap::new();
        let result = MeshComponent.evaluate(&[], &meta);
        
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert!(matches!(outputs["Mesh"], Value::Null));
    }

    #[test]
    fn mesh_component_handles_list_of_meshes() {
        let mesh1 = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        let mesh2 = Value::Mesh {
            vertices: vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        let list = Value::List(vec![mesh1, mesh2]);
        
        let meta = MetaMap::new();
        let result = MeshComponent.evaluate(&[list], &meta);
        
        assert!(result.is_ok(), "MeshComponent should accept list of meshes");
        let outputs = result.unwrap();
        
        match &outputs["Mesh"] {
            Value::List(items) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(items[0], Value::Mesh { .. }));
                assert!(matches!(items[1], Value::Mesh { .. }));
            }
            _ => panic!("Expected Value::List output"),
        }
    }

    #[test]
    fn mesh_component_handles_mixed_mesh_surface_list() {
        let mesh = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        let surface = Value::Surface {
            vertices: vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.5, 1.0, 0.0]],
            faces: vec![vec![0, 1, 2]],
        };
        let list = Value::List(vec![mesh, surface]);
        
        let meta = MetaMap::new();
        let result = MeshComponent.evaluate(&[list], &meta);
        
        assert!(result.is_ok(), "MeshComponent should accept mixed mesh/surface list");
        let outputs = result.unwrap();
        
        match &outputs["Mesh"] {
            Value::List(items) => {
                assert_eq!(items.len(), 2);
                // Both should be converted to Mesh
                assert!(matches!(items[0], Value::Mesh { .. }));
                assert!(matches!(items[1], Value::Mesh { .. }));
            }
            _ => panic!("Expected Value::List output"),
        }
    }

    // -------------------------------------------------------------------------
    // SurfaceComponent Tests
    // -------------------------------------------------------------------------

    #[test]
    fn surface_component_accepts_value_surface() {
        let surface = Value::Surface {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            faces: vec![vec![0, 1, 2]],
        };
        
        let meta = MetaMap::new();
        let result = SurfaceComponent.evaluate(&[surface.clone()], &meta);
        
        assert!(result.is_ok(), "SurfaceComponent should accept Value::Surface");
        let outputs = result.unwrap();
        assert!(outputs.contains_key("Srf"));
        
        match &outputs["Srf"] {
            Value::Surface { vertices, faces } => {
                assert_eq!(vertices.len(), 3);
                assert_eq!(faces.len(), 1);
            }
            _ => panic!("Expected Value::Surface output"),
        }
    }

    #[test]
    fn surface_component_accepts_value_mesh_and_converts() {
        let mesh = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: None,
            diagnostics: None,
        };
        
        let meta = MetaMap::new();
        let result = SurfaceComponent.evaluate(&[mesh], &meta);
        
        assert!(result.is_ok(), "SurfaceComponent should accept Value::Mesh");
        let outputs = result.unwrap();
        
        // The output should be converted to Value::Surface
        match &outputs["Srf"] {
            Value::Surface { vertices, faces } => {
                assert_eq!(vertices.len(), 3);
                assert_eq!(faces.len(), 1);
                assert_eq!(faces[0], vec![0, 1, 2]);
            }
            _ => panic!("Expected Value::Surface output"),
        }
    }

    #[test]
    fn surface_component_handles_null() {
        let meta = MetaMap::new();
        let result = SurfaceComponent.evaluate(&[Value::Null], &meta);
        
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert!(matches!(outputs["Srf"], Value::Null));
    }

    #[test]
    fn surface_component_handles_empty_inputs() {
        let meta = MetaMap::new();
        let result = SurfaceComponent.evaluate(&[], &meta);
        
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert!(matches!(outputs["Srf"], Value::Null));
    }

    #[test]
    fn surface_component_handles_list_of_surfaces() {
        let surface1 = Value::Surface {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            faces: vec![vec![0, 1, 2]],
        };
        let surface2 = Value::Surface {
            vertices: vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.5, 1.0, 0.0]],
            faces: vec![vec![0, 1, 2]],
        };
        let list = Value::List(vec![surface1, surface2]);
        
        let meta = MetaMap::new();
        let result = SurfaceComponent.evaluate(&[list], &meta);
        
        assert!(result.is_ok(), "SurfaceComponent should accept list of surfaces");
        let outputs = result.unwrap();
        
        match &outputs["Srf"] {
            Value::List(items) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(items[0], Value::Surface { .. }));
                assert!(matches!(items[1], Value::Surface { .. }));
            }
            _ => panic!("Expected Value::List output"),
        }
    }

    #[test]
    fn surface_component_handles_mixed_mesh_surface_list() {
        let surface = Value::Surface {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            faces: vec![vec![0, 1, 2]],
        };
        let mesh = Value::Mesh {
            vertices: vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        let list = Value::List(vec![surface, mesh]);
        
        let meta = MetaMap::new();
        let result = SurfaceComponent.evaluate(&[list], &meta);
        
        assert!(result.is_ok(), "SurfaceComponent should accept mixed mesh/surface list");
        let outputs = result.unwrap();
        
        match &outputs["Srf"] {
            Value::List(items) => {
                assert_eq!(items.len(), 2);
                // Both should be converted to Surface
                assert!(matches!(items[0], Value::Surface { .. }));
                assert!(matches!(items[1], Value::Surface { .. }));
            }
            _ => panic!("Expected Value::List output"),
        }
    }

    // -------------------------------------------------------------------------
    // Conversion Helper Tests
    // -------------------------------------------------------------------------

    #[test]
    fn convert_to_mesh_preserves_mesh_attributes() {
        let original = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: Some(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]),
            diagnostics: None,
        };
        
        let converted = convert_to_mesh_value(&original).unwrap();
        
        match converted {
            Value::Mesh { vertices, indices, normals, uvs, .. } => {
                assert_eq!(vertices.len(), 3);
                assert_eq!(indices.len(), 3);
                assert!(normals.is_some());
                assert!(uvs.is_some());
            }
            _ => panic!("Expected Value::Mesh"),
        }
    }

    #[test]
    fn convert_to_surface_from_mesh_is_lossy() {
        let original = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: Some(vec![[0.0, 0.0, 1.0]; 3]),
            uvs: Some(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]),
            diagnostics: None,
        };
        
        let converted = convert_to_surface_value(&original).unwrap();
        
        match converted {
            Value::Surface { vertices, faces } => {
                assert_eq!(vertices.len(), 3);
                assert_eq!(faces.len(), 1);
                assert_eq!(faces[0], vec![0, 1, 2]);
                // Note: normals and uvs are lost in this conversion
            }
            _ => panic!("Expected Value::Surface"),
        }
    }

    #[test]
    fn list_contains_geometry_detects_mixed_list() {
        let mesh = Value::Mesh {
            vertices: vec![[0.0, 0.0, 0.0]],
            indices: vec![],
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        let surface = Value::Surface {
            vertices: vec![[0.0, 0.0, 0.0]],
            faces: vec![],
        };
        
        let list = Value::List(vec![mesh, surface, Value::Null]);
        assert!(list_contains_only_geometry_like_or_null(&list));
        
        let bad_list = Value::List(vec![Value::Number(42.0)]);
        assert!(!list_contains_only_geometry_like_or_null(&bad_list));
    }

    #[test]
    fn mesh_component_rejects_invalid_types() {
        let meta = MetaMap::new();
        let result = MeshComponent.evaluate(&[Value::Number(42.0)], &meta);
        
        assert!(result.is_err(), "MeshComponent should reject Value::Number");
    }

    #[test]
    fn surface_component_rejects_invalid_types() {
        let meta = MetaMap::new();
        let result = SurfaceComponent.evaluate(&[Value::Number(42.0)], &meta);
        
        assert!(result.is_err(), "SurfaceComponent should reject Value::Number");
    }
}