//! Componenten voor weergave en preview in de GHX-engine.
//!
//! This module provides components for geometry display and preview, including:
//!
//! - **Custom Preview**: Apply materials to any geometry (Mesh or Surface)
//! - **Mesh Preview**: Preview meshes with optional diagnostics visualization
//! - **Cloud/Dot Display**: Point cloud visualization
//! - **Symbol Display**: 2D symbol display at locations
//! - **Create Material**: Material construction
//!
//! # Mesh Support
//!
//! Components in this module support both `Value::Mesh` (preferred) and
//! `Value::Surface` (legacy). When a `Value::Mesh` is provided, it is rendered
//! directly with full attribute support (normals, UVs). For `Value::Surface`,
//! the existing rendering path is maintained for backward compatibility.
//!
//! # Diagnostics Visualization
//!
//! The `MeshPreview` component can optionally visualize mesh diagnostics:
//! - Open edges (boundary edges) as colored lines
//! - Non-manifold edges as colored lines
//! - Self-intersection regions (when detected)

use super::{Component, ComponentError, ComponentResult};
use crate::components::coerce::coerce_mesh_like;
use crate::components::vector_point::parse_color_value;
use crate::graph::node::MetaMap;
use crate::graph::value::{ColorValue, MaterialValue, MeshDiagnostics, SymbolValue, Value};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    CloudDisplay,
    CustomPreview,
    MeshPreview,
    SymbolDisplay,
    DotDisplay,
    CreateMaterial,
    SymbolSimple,
    SymbolAdvanced,
}

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::CloudDisplay => cloud_display(inputs, meta),
            Self::CustomPreview => custom_preview(inputs, meta),
            Self::MeshPreview => mesh_preview(inputs, meta),
            Self::SymbolDisplay => symbol_display(inputs, meta),
            Self::DotDisplay => dot_display(inputs, meta),
            Self::CreateMaterial => create_material(inputs, meta),
            Self::SymbolSimple => symbol_simple(inputs, meta),
            Self::SymbolAdvanced => symbol_advanced(inputs, meta),
        }
    }
}

impl ComponentKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::CloudDisplay => "Cloud Display",
            Self::CustomPreview => "Custom Preview",
            Self::MeshPreview => "Mesh Preview",
            Self::SymbolDisplay => "Symbol Display",
            Self::DotDisplay => "Dot Display",
            Self::CreateMaterial => "Create Material",
            Self::SymbolSimple => "Symbol (Simple)",
            Self::SymbolAdvanced => "Symbol (Advanced)",
        }
    }
}

fn cloud_display(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Expected 3 inputs: Points, Colours, Size",
        ));
    }
    let points = collect_points(&inputs[0])?;
    let colors = collect_colors(&inputs[1])?;
    let sizes = collect_numbers(&inputs[2])?;

    let mut tags = Vec::new();
    for i in 0..points.len() {
        let point = points[i];
        let color = colors
            .get(i)
            .cloned()
            .unwrap_or_else(|| ColorValue::from_rgb255(0.0, 0.0, 0.0));
        let size = sizes.get(i).cloned().unwrap_or(1.0);

        let tag = crate::graph::value::TextTagValue {
            plane: crate::graph::value::PlaneValue {
                origin: point,
                x_axis: [1.0, 0.0, 0.0],
                y_axis: [0.0, 1.0, 0.0],
                z_axis: [0.0, 0.0, 1.0],
            },
            text: "cloud".to_string(),
            size,
            color: Some(color),
        };
        tags.push(Value::Tag(tag));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert("Tags".to_string(), Value::List(tags));
    Ok(outputs)
}

fn custom_preview(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Expected 2 inputs: Geometry, Material"));
    }

    // Normalize the geometry input: handle both Value::Mesh and Value::Surface directly
    // Both types are passed through unchanged - the rendering layer handles both.
    let geometry = normalize_geometry_for_preview(&inputs[0]);
    let material = coerce_material(&inputs[1])?;

    let mut outputs = BTreeMap::new();
    outputs.insert("Geometry".to_string(), geometry);
    outputs.insert("Material".to_string(), Value::Material(material));
    Ok(outputs)
}

/// Mesh Preview component with optional diagnostics visualization.
///
/// Inputs:
/// - `M` (0): Mesh geometry (`Value::Mesh` or `Value::Surface`)
/// - `Mat` (1): Optional material (`Value::Material` or `Value::Color`)
/// - `D` (2): Optional show diagnostics flag (`Value::Boolean`, default: false)
///
/// Outputs:
/// - `Geometry`: The mesh geometry (passed through)
/// - `Material`: The applied material
/// - `OpenEdges`: Lines representing open (boundary) edges (if diagnostics enabled)
/// - `NonManifoldEdges`: Lines representing non-manifold edges (if diagnostics enabled)
/// - `Diagnostics`: The mesh diagnostics summary (if available)
///
/// When the diagnostics flag is enabled, this component extracts edge information
/// from the mesh and outputs them as line lists that can be visualized separately.
fn mesh_preview(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Expected at least 1 input: Mesh"));
    }

    // Get the mesh geometry - accept both Value::Mesh and Value::Surface
    let geometry = normalize_geometry_for_preview(&inputs[0]);

    // Optional material (default to a neutral gray material)
    let material = if inputs.len() > 1 && !matches!(inputs[1], Value::Null) {
        coerce_material(&inputs[1])?
    } else {
        MaterialValue {
            diffuse: ColorValue::new(0.7, 0.7, 0.7),
            specular: ColorValue::new(1.0, 1.0, 1.0),
            emission: ColorValue::new(0.0, 0.0, 0.0),
            transparency: 0.0,
            shine: 30.0,
        }
    };

    // Optional diagnostics display flag (default: false)
    let show_diagnostics = if inputs.len() > 2 {
        coerce_boolean_or_default(&inputs[2], false)
    } else {
        false
    };

    let mut outputs = BTreeMap::new();
    outputs.insert("Geometry".to_string(), geometry.clone());
    outputs.insert("Material".to_string(), Value::Material(material));

    // Extract diagnostics and edge visualization if requested
    if show_diagnostics {
        let (open_edges, non_manifold_edges, diagnostics) =
            extract_mesh_diagnostics_edges(&geometry);
        outputs.insert("OpenEdges".to_string(), Value::List(open_edges));
        outputs.insert(
            "NonManifoldEdges".to_string(),
            Value::List(non_manifold_edges),
        );
        if let Some(diag) = diagnostics {
            outputs.insert("Diagnostics".to_string(), Value::Text(diag.summary()));
        } else {
            outputs.insert("Diagnostics".to_string(), Value::Null);
        }
    }

    Ok(outputs)
}

/// Normalizes geometry input for preview, handling both Mesh and Surface types.
///
/// For `Value::Mesh`: passed through unchanged (preferred path)
/// For `Value::Surface`: passed through unchanged (legacy path)
/// For `Value::List`: recursively normalizes each element
/// For other types: passed through unchanged (may be curves, points, etc.)
fn normalize_geometry_for_preview(value: &Value) -> Value {
    match value {
        // Preferred path: Value::Mesh is passed through directly
        Value::Mesh { .. } => value.clone(),

        // Legacy path: Value::Surface is passed through for backward compatibility
        Value::Surface { .. } => value.clone(),

        // Handle lists of geometry by recursively normalizing each element
        Value::List(items) => {
            let normalized: Vec<Value> = items
                .iter()
                .map(normalize_geometry_for_preview)
                .collect();
            Value::List(normalized)
        }

        // Other geometry types (curves, points, etc.) pass through unchanged
        _ => value.clone(),
    }
}

/// Extracts diagnostic edge visualization from a mesh-like value.
///
/// Returns:
/// - `open_edges`: List of `Value::CurveLine` representing boundary edges (edges with only one adjacent face)
/// - `non_manifold_edges`: List of `Value::CurveLine` representing non-manifold edges (edges with >2 adjacent faces)
/// - `diagnostics`: Optional `MeshDiagnostics` if the value is a `Value::Mesh` with diagnostics
///
/// For `Value::Surface` or `Value::Mesh` without embedded diagnostics, edges are computed
/// from the mesh topology directly.
fn extract_mesh_diagnostics_edges(value: &Value) -> (Vec<Value>, Vec<Value>, Option<MeshDiagnostics>) {
    // Try to get embedded diagnostics from Value::Mesh
    let embedded_diagnostics = match value {
        Value::Mesh { diagnostics, .. } => diagnostics.clone(),
        _ => None,
    };

    // Try to coerce to a mesh-like structure to analyze edges
    let mesh = match coerce_mesh_like(value) {
        Ok(m) => m,
        Err(_) => {
            // Not a mesh-like value, return empty results
            return (Vec::new(), Vec::new(), embedded_diagnostics);
        }
    };

    // Build edge count map: edge -> number of adjacent triangles
    let mut edge_counts: HashMap<(u32, u32), u32> = HashMap::new();
    for chunk in mesh.indices.chunks(3) {
        if chunk.len() < 3 {
            continue;
        }
        let face = [chunk[0], chunk[1], chunk[2]];
        for i in 0..3 {
            let v1 = face[i];
            let v2 = face[(i + 1) % 3];
            // Normalize edge direction for consistent counting
            let edge = if v1 < v2 { (v1, v2) } else { (v2, v1) };
            *edge_counts.entry(edge).or_insert(0) += 1;
        }
    }

    // Classify edges by their adjacency count
    let mut open_edges = Vec::new();
    let mut non_manifold_edges = Vec::new();

    for ((v1, v2), count) in edge_counts {
        let p1 = mesh.vertices.get(v1 as usize).copied().unwrap_or([0.0, 0.0, 0.0]);
        let p2 = mesh.vertices.get(v2 as usize).copied().unwrap_or([0.0, 0.0, 0.0]);
        let line = Value::CurveLine { p1, p2 };

        match count {
            1 => open_edges.push(line),      // Boundary edge (open)
            2 => {}                           // Interior edge (manifold) - skip
            _ => non_manifold_edges.push(line), // Non-manifold edge (>2 faces)
        }
    }

    // Build or update diagnostics with computed values
    let diagnostics = match embedded_diagnostics {
        Some(mut diag) => {
            // Update with computed values if they weren't set
            if diag.open_edge_count == 0 && !open_edges.is_empty() {
                diag.open_edge_count = open_edges.len();
            }
            if diag.non_manifold_edge_count == 0 && !non_manifold_edges.is_empty() {
                diag.non_manifold_edge_count = non_manifold_edges.len();
            }
            Some(diag)
        }
        None if !open_edges.is_empty() || !non_manifold_edges.is_empty() => {
            // Create diagnostics from computed values
            Some(MeshDiagnostics {
                vertex_count: mesh.vertices.len(),
                triangle_count: mesh.indices.len() / 3,
                open_edge_count: open_edges.len(),
                non_manifold_edge_count: non_manifold_edges.len(),
                ..Default::default()
            })
        }
        None => None,
    };

    (open_edges, non_manifold_edges, diagnostics)
}

/// Coerces a boolean value with a default fallback.
fn coerce_boolean_or_default(value: &Value, default: bool) -> bool {
    match value {
        Value::Boolean(b) => *b,
        Value::Number(n) => *n != 0.0,
        Value::Null => default,
        _ => default,
    }
}

fn symbol_display(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Expected 2 inputs: Location, Display"));
    }
    let location = inputs[0].clone();
    let symbol = coerce_symbol(&inputs[1])?;

    let mut outputs = BTreeMap::new();
    outputs.insert("Location".to_string(), location);
    outputs.insert("Symbol".to_string(), Value::Symbol(symbol));
    Ok(outputs)
}

fn dot_display(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Expected 3 inputs: Point, Colour, Size",
        ));
    }
    let points = collect_points(&inputs[0])?;
    let colors = collect_colors(&inputs[1])?;
    let sizes = collect_numbers(&inputs[2])?;

    let mut tags = Vec::new();
    for i in 0..points.len() {
        let point = points[i];
        let color = colors
            .get(i)
            .cloned()
            .unwrap_or_else(|| ColorValue::from_rgb255(0.0, 0.0, 0.0));
        let size = sizes.get(i).cloned().unwrap_or(1.0);

        let tag = crate::graph::value::TextTagValue {
            plane: crate::graph::value::PlaneValue {
                origin: point,
                x_axis: [1.0, 0.0, 0.0],
                y_axis: [0.0, 1.0, 0.0],
                z_axis: [0.0, 0.0, 1.0],
            },
            text: "".to_string(),
            size,
            color: Some(color),
        };
        tags.push(Value::Tag(tag));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert("Tags".to_string(), Value::List(tags));
    Ok(outputs)
}

fn create_material(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 5 {
        return Err(ComponentError::new(
            "Expected 5 inputs: Diffuse, Specular, Emission, Transparency, Shine",
        ));
    }
    let diffuse = coerce_color(&inputs[0])?;
    let specular = coerce_color(&inputs[1])?;
    let emission = coerce_color(&inputs[2])?;
    let transparency = coerce_number(&inputs[3])?;
    let shine = coerce_number(&inputs[4])?;

    let material = MaterialValue {
        diffuse,
        specular,
        emission,
        transparency,
        shine,
    };

    let mut outputs = BTreeMap::new();
    outputs.insert("M".to_string(), Value::Material(material));
    Ok(outputs)
}

fn symbol_simple(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Expected 4 inputs: Style, Size, Rotation, Colour",
        ));
    }
    let style = coerce_text(&inputs[0])?;
    let size = coerce_number(&inputs[1])?;
    let rotation = coerce_number(&inputs[2])?;
    let color = coerce_color(&inputs[3])?;

    let symbol = SymbolValue {
        style,
        size_primary: size,
        size_secondary: None,
        rotation,
        fill: color,
        edge: None,
        width: 1.0,
        adjust: false,
    };

    let mut outputs = BTreeMap::new();
    outputs.insert("D".to_string(), Value::Symbol(symbol));
    Ok(outputs)
}

fn symbol_advanced(inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
    if inputs.len() < 8 {
        return Err(ComponentError::new(
            "Expected 8 inputs: Style, Size Primary, Size Secondary, Rotation, Fill, Edge, Width, Adjust",
        ));
    }
    let style = coerce_text(&inputs[0])?;
    let size_primary = coerce_number(&inputs[1])?;
    let size_secondary = Some(coerce_number(&inputs[2])?);
    let rotation = coerce_number(&inputs[3])?;
    let fill = coerce_color(&inputs[4])?;
    let edge = Some(coerce_color(&inputs[5])?);
    let width = coerce_number(&inputs[6])?;
    let adjust = coerce_boolean(&inputs[7])?;

    let symbol = SymbolValue {
        style,
        size_primary,
        size_secondary,
        rotation,
        fill,
        edge,
        width,
        adjust,
    };

    let mut outputs = BTreeMap::new();
    outputs.insert("D".to_string(), Value::Symbol(symbol));
    Ok(outputs)
}

fn coerce_number(value: &Value) -> Result<f64, ComponentError> {
    match value {
        Value::Number(n) => Ok(*n),
        other => Err(ComponentError::new(format!(
            "Expected a number, got {}",
            other.kind()
        ))),
    }
}

fn coerce_color(value: &Value) -> Result<ColorValue, ComponentError> {
    match value {
        Value::Color(c) => Ok(*c),
        other => parse_color_value(other)
            .ok_or_else(|| ComponentError::new(format!("Expected a color, got {}", other.kind()))),
    }
}

fn coerce_text(value: &Value) -> Result<String, ComponentError> {
    match value {
        Value::Text(t) => Ok(t.clone()),
        other => Err(ComponentError::new(format!(
            "Expected text, got {}",
            other.kind()
        ))),
    }
}

fn coerce_boolean(value: &Value) -> Result<bool, ComponentError> {
    match value {
        Value::Boolean(b) => Ok(*b),
        other => Err(ComponentError::new(format!(
            "Expected a boolean, got {}",
            other.kind()
        ))),
    }
}

pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["059b72b0-9bb3-4542-a805-2dcd27493164"],
        names: &["Cloud Display", "Cloud"],
        kind: ComponentKind::CloudDisplay,
    },
    Registration {
        guids: &["537b0419-bbc2-4ff4-bf08-afe526367b2c"],
        names: &["Custom Preview", "Preview"],
        kind: ComponentKind::CustomPreview,
    },
    // MeshPreview is a new component for mesh-specific preview with diagnostics.
    // It uses a generated GUID since this is a new component not present in
    // the original Grasshopper component set.
    Registration {
        guids: &["a1b2c3d4-mesh-prev-diag-000000000001"],
        names: &["Mesh Preview", "MeshPrev", "MPreview"],
        kind: ComponentKind::MeshPreview,
    },
    Registration {
        guids: &["62d5ead4-53c4-4d0b-b5ce-6bd6e0850ab8"],
        names: &["Symbol Display", "Symbol"],
        kind: ComponentKind::SymbolDisplay,
    },
    Registration {
        guids: &["6b1bd8b2-47a4-4aa6-a471-3fd91c62a486"],
        names: &["Dot Display", "Dots"],
        kind: ComponentKind::DotDisplay,
    },
    Registration {
        guids: &["76975309-75a6-446a-afed-f8653720a9f2"],
        names: &["Create Material", "Material"],
        kind: ComponentKind::CreateMaterial,
    },
    Registration {
        guids: &["79747717-1874-4c34-b790-faef53b50569"],
        names: &["Symbol (Simple)", "SymSim"],
        kind: ComponentKind::SymbolSimple,
    },
    Registration {
        guids: &["e5c82975-8011-412c-b56d-bb7fc9e7f28d"],
        names: &["Symbol (Advanced)", "SymAdv"],
        kind: ComponentKind::SymbolAdvanced,
    },
];

fn coerce_symbol(value: &Value) -> Result<SymbolValue, ComponentError> {
    match value {
        Value::Symbol(s) => Ok(s.clone()),
        other => Err(ComponentError::new(format!(
            "Expected a symbol, got {}",
            other.kind()
        ))),
    }
}

fn coerce_material(value: &Value) -> Result<MaterialValue, ComponentError> {
    match value {
        Value::Material(m) => Ok(*m),
        Value::Color(c) => Ok(MaterialValue {
            diffuse: *c,
            specular: ColorValue::new(1.0, 1.0, 1.0),
            emission: ColorValue::new(0.0, 0.0, 0.0),
            transparency: 0.0,
            shine: 10.0,
        }),
        other => Err(ComponentError::new(format!(
            "Expected a material, got {}",
            other.kind()
        ))),
    }
}

fn collect_points(value: &Value) -> Result<Vec<[f64; 3]>, ComponentError> {
    let mut points = Vec::new();
    collect_points_into(value, &mut points)?;
    Ok(points)
}

fn collect_points_into(value: &Value, output: &mut Vec<[f64; 3]>) -> Result<(), ComponentError> {
    match value {
        Value::Point(p) => {
            output.push(*p);
            Ok(())
        }
        Value::List(values) => {
            for value in values {
                collect_points_into(value, output)?;
            }
            Ok(())
        }
        _ => Err(ComponentError::new(format!(
            "Expected a point, got {}",
            value.kind()
        ))),
    }
}

fn collect_colors(value: &Value) -> Result<Vec<ColorValue>, ComponentError> {
    let mut colors = Vec::new();
    collect_colors_into(value, &mut colors)?;
    Ok(colors)
}

fn collect_colors_into(value: &Value, output: &mut Vec<ColorValue>) -> Result<(), ComponentError> {
    match value {
        Value::Color(c) => {
            output.push(*c);
            Ok(())
        }
        Value::List(values) => {
            for value in values {
                collect_colors_into(value, output)?;
            }
            Ok(())
        }
        _ => Err(ComponentError::new(format!(
            "Expected a color, got {}",
            value.kind()
        ))),
    }
}

fn collect_numbers(value: &Value) -> Result<Vec<f64>, ComponentError> {
    let mut numbers = Vec::new();
    collect_numbers_into(value, &mut numbers)?;
    Ok(numbers)
}

fn collect_numbers_into(value: &Value, output: &mut Vec<f64>) -> Result<(), ComponentError> {
    match value {
        Value::Number(n) => {
            output.push(*n);
            Ok(())
        }
        Value::List(values) => {
            for value in values {
                collect_numbers_into(value, output)?;
            }
            Ok(())
        }
        _ => Err(ComponentError::new(format!(
            "Expected a number, got {}",
            value.kind()
        ))),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::MetaMap;

    /// Creates a simple test mesh (triangle)
    fn test_mesh() -> Value {
        Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2],
            normals: Some(vec![
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ]),
            uvs: None,
            diagnostics: None,
        }
    }

    /// Creates a simple test surface (quad, legacy format)
    fn test_surface() -> Value {
        Value::Surface {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![vec![0, 1, 2, 3]],
        }
    }

    /// Creates a mesh with an open edge (boundary)
    fn test_open_mesh() -> Value {
        Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            diagnostics: Some(MeshDiagnostics {
                vertex_count: 3,
                triangle_count: 1,
                open_edge_count: 3, // All edges are open (single triangle)
                ..Default::default()
            }),
        }
    }

    /// Creates a test material
    fn test_material() -> Value {
        Value::Material(MaterialValue {
            diffuse: ColorValue::new(1.0, 0.0, 0.0),
            specular: ColorValue::new(1.0, 1.0, 1.0),
            emission: ColorValue::new(0.0, 0.0, 0.0),
            transparency: 0.0,
            shine: 50.0,
        })
    }

    #[test]
    fn test_custom_preview_with_mesh() {
        let inputs = vec![test_mesh(), test_material()];
        let meta = MetaMap::new();

        let result = custom_preview(&inputs, &meta);
        assert!(result.is_ok());

        let outputs = result.unwrap();
        assert!(outputs.contains_key("Geometry"));
        assert!(outputs.contains_key("Material"));

        // Verify the geometry is preserved as Value::Mesh
        match &outputs["Geometry"] {
            Value::Mesh { vertices, indices, normals, .. } => {
                assert_eq!(vertices.len(), 3);
                assert_eq!(indices.len(), 3);
                assert!(normals.is_some());
            }
            other => panic!("Expected Value::Mesh, got {:?}", other),
        }
    }

    #[test]
    fn test_custom_preview_with_surface() {
        let inputs = vec![test_surface(), test_material()];
        let meta = MetaMap::new();

        let result = custom_preview(&inputs, &meta);
        assert!(result.is_ok());

        let outputs = result.unwrap();

        // Verify the geometry is preserved as Value::Surface
        match &outputs["Geometry"] {
            Value::Surface { vertices, faces } => {
                assert_eq!(vertices.len(), 4);
                assert_eq!(faces.len(), 1);
            }
            other => panic!("Expected Value::Surface, got {:?}", other),
        }
    }

    #[test]
    fn test_mesh_preview_basic() {
        let inputs = vec![test_mesh()];
        let meta = MetaMap::new();

        let result = mesh_preview(&inputs, &meta);
        assert!(result.is_ok());

        let outputs = result.unwrap();
        assert!(outputs.contains_key("Geometry"));
        assert!(outputs.contains_key("Material"));
        // Diagnostics outputs should not be present when flag is false
        assert!(!outputs.contains_key("OpenEdges"));
    }

    #[test]
    fn test_mesh_preview_with_diagnostics() {
        let inputs = vec![
            test_open_mesh(),
            test_material(),
            Value::Boolean(true), // Enable diagnostics
        ];
        let meta = MetaMap::new();

        let result = mesh_preview(&inputs, &meta);
        assert!(result.is_ok());

        let outputs = result.unwrap();
        assert!(outputs.contains_key("OpenEdges"));
        assert!(outputs.contains_key("NonManifoldEdges"));
        assert!(outputs.contains_key("Diagnostics"));

        // Verify open edges were extracted
        if let Value::List(edges) = &outputs["OpenEdges"] {
            assert_eq!(edges.len(), 3, "Triangle should have 3 open edges");
            // Verify each edge is a CurveLine
            for edge in edges {
                assert!(matches!(edge, Value::CurveLine { .. }));
            }
        } else {
            panic!("Expected Value::List for OpenEdges");
        }
    }

    #[test]
    fn test_extract_mesh_diagnostics_edges_from_mesh() {
        let mesh = test_mesh();
        let (open_edges, non_manifold_edges, diagnostics) =
            extract_mesh_diagnostics_edges(&mesh);

        // Single triangle has 3 boundary edges
        assert_eq!(open_edges.len(), 3);
        assert_eq!(non_manifold_edges.len(), 0);

        // Diagnostics should be created
        assert!(diagnostics.is_some());
        let diag = diagnostics.unwrap();
        assert_eq!(diag.open_edge_count, 3);
    }

    #[test]
    fn test_extract_mesh_diagnostics_edges_from_surface() {
        let surface = test_surface();
        let (open_edges, non_manifold_edges, diagnostics) =
            extract_mesh_diagnostics_edges(&surface);

        // Quad surface converted to triangle has edges
        // (after triangulation, we take first 3 vertices as one triangle)
        assert!(!open_edges.is_empty() || diagnostics.is_some());
        assert_eq!(non_manifold_edges.len(), 0);
    }

    #[test]
    fn test_normalize_geometry_preserves_mesh() {
        let mesh = test_mesh();
        let normalized = normalize_geometry_for_preview(&mesh);

        match normalized {
            Value::Mesh { vertices, indices, normals, .. } => {
                assert_eq!(vertices.len(), 3);
                assert_eq!(indices.len(), 3);
                assert!(normals.is_some());
            }
            _ => panic!("Expected Value::Mesh to be preserved"),
        }
    }

    #[test]
    fn test_normalize_geometry_preserves_surface() {
        let surface = test_surface();
        let normalized = normalize_geometry_for_preview(&surface);

        match normalized {
            Value::Surface { vertices, faces } => {
                assert_eq!(vertices.len(), 4);
                assert_eq!(faces.len(), 1);
            }
            _ => panic!("Expected Value::Surface to be preserved"),
        }
    }

    #[test]
    fn test_normalize_geometry_handles_list() {
        let mesh = test_mesh();
        let surface = test_surface();
        let list = Value::List(vec![mesh, surface]);

        let normalized = normalize_geometry_for_preview(&list);

        match normalized {
            Value::List(items) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(items[0], Value::Mesh { .. }));
                assert!(matches!(items[1], Value::Surface { .. }));
            }
            _ => panic!("Expected Value::List to be preserved"),
        }
    }

    #[test]
    fn test_closed_mesh_no_open_edges() {
        // Create a tetrahedron (closed mesh with no open edges)
        let tetra = Value::Mesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            indices: vec![
                0, 1, 2, // Bottom
                0, 1, 3, // Front
                1, 2, 3, // Right
                2, 0, 3, // Left
            ],
            normals: None,
            uvs: None,
            diagnostics: None,
        };

        let (open_edges, non_manifold_edges, diagnostics) =
            extract_mesh_diagnostics_edges(&tetra);

        // Tetrahedron is closed, so no open edges
        assert_eq!(open_edges.len(), 0);
        assert_eq!(non_manifold_edges.len(), 0);

        // Since there are no issues, diagnostics might be None
        // (or have zeros for open/non-manifold)
        if let Some(diag) = diagnostics {
            assert_eq!(diag.open_edge_count, 0);
            assert_eq!(diag.non_manifold_edge_count, 0);
        }
    }

    #[test]
    fn test_coerce_boolean_or_default() {
        assert!(coerce_boolean_or_default(&Value::Boolean(true), false));
        assert!(!coerce_boolean_or_default(&Value::Boolean(false), true));
        assert!(coerce_boolean_or_default(&Value::Number(1.0), false));
        assert!(!coerce_boolean_or_default(&Value::Number(0.0), false));
        assert!(coerce_boolean_or_default(&Value::Null, true));
        assert!(!coerce_boolean_or_default(&Value::Text("x".into()), false));
    }
}
