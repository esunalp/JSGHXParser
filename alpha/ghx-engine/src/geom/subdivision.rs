//! Subdivision surface (SubD) geometry support for the geom mesh engine.
//!
//! This module provides a standalone SubD representation that can be:
//! - Created from primitives (box, mesh)
//! - Manipulated (fuse, smooth, tag edges/vertices)
//! - Converted to triangle meshes for rendering
//!
//! # Phase 3 Integration
//! This module is now integrated with the component layer. Use the bridge methods
//! (`from_value`, `to_value`, `to_mesh_value`) to convert between `SubdMesh` and
//! `Value` types.
//!
//! # Example
//! ```ignore
//! use ghx_engine::geom::subdivision::{SubdMesh, SubdOptions};
//!
//! // Create a box SubD
//! let subd = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
//!
//! // Smooth it
//! let smoothed = subd.smoothed(2);
//!
//! // Convert to triangle mesh
//! let (mesh, diag) = smoothed.to_triangle_mesh(SubdOptions::default());
//! ```

use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::core::{BBox, Point3};
use super::diagnostics::GeomMeshDiagnostics;
use super::mesh::GeomMesh;
use crate::graph::value::Value;

// ============================================================================
// Error types
// ============================================================================

/// Errors that can occur during SubD operations.
#[derive(Debug, Clone)]
pub enum SubdError {
    /// No input provided.
    EmptyInput(String),
    /// Invalid mesh topology.
    InvalidTopology(String),
    /// Operation failed.
    OperationFailed(String),
}

impl std::fmt::Display for SubdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput(msg) => write!(f, "Empty input: {msg}"),
            Self::InvalidTopology(msg) => write!(f, "Invalid topology: {msg}"),
            Self::OperationFailed(msg) => write!(f, "Operation failed: {msg}"),
        }
    }
}

impl std::error::Error for SubdError {}

// ============================================================================
// Tags for edge and vertex sharpness
// ============================================================================

/// Edge tag controlling subdivision behavior at an edge.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EdgeTag {
    /// Smooth subdivision (default).
    Smooth,
    /// Sharp crease edge.
    Crease,
    /// Custom tag with a name.
    Custom(String),
}

impl Default for EdgeTag {
    fn default() -> Self {
        Self::Smooth
    }
}

impl EdgeTag {
    /// Parse an edge tag from a string descriptor.
    #[must_use]
    pub fn from_descriptor(descriptor: &str) -> Self {
        match descriptor.trim().to_lowercase().as_str() {
            "s" | "smooth" => Self::Smooth,
            "c" | "crease" | "sharp" => Self::Crease,
            other => Self::Custom(other.to_owned()),
        }
    }

    /// Parse an edge tag from an integer (0 = smooth, 1 = crease).
    #[must_use]
    pub fn from_int(value: i32) -> Self {
        match value {
            1 => Self::Crease,
            _ => Self::Smooth,
        }
    }

    /// Parse an edge tag from a `Value`.
    ///
    /// Accepts:
    /// - `Value::Text` - parsed as descriptor
    /// - `Value::Number` - 1 = crease, other = smooth
    /// - `Value::List` - uses first element
    ///
    /// Returns `None` for incompatible values.
    #[must_use]
    pub fn parse(value: Option<&Value>) -> Option<Self> {
        match value {
            Some(Value::Text(text)) => Some(Self::from_descriptor(text)),
            Some(Value::Number(number)) => {
                if number.round() as i32 == 1 {
                    Some(Self::Crease)
                } else {
                    Some(Self::Smooth)
                }
            }
            Some(Value::List(values)) if !values.is_empty() => Self::parse(values.first()),
            None => Some(Self::default()),
            _ => None,
        }
    }

    /// Parse an edge tag from a `Value`, returning an error on failure.
    ///
    /// # Arguments
    /// * `value` - The value to parse
    /// * `context` - Context string for error messages
    pub fn parse_or_err(value: Option<&Value>, context: &str) -> Result<Self, SubdError> {
        Self::parse(value).ok_or_else(|| {
            SubdError::OperationFailed(format!("{context} requires a tag value (text or number)"))
        })
    }

    /// Convert to a string representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Smooth => "smooth",
            Self::Crease => "crease",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Check if this edge should remain sharp during subdivision.
    #[must_use]
    pub fn is_sharp(&self) -> bool {
        matches!(self, Self::Crease)
    }
}

/// Vertex tag controlling subdivision behavior at a vertex.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VertexTag {
    /// Smooth vertex (default).
    Smooth,
    /// Crease vertex (semi-sharp).
    Crease,
    /// Corner vertex (fully sharp).
    Corner,
    /// Dart vertex (single sharp edge).
    Dart,
    /// Custom tag with a name.
    Custom(String),
}

impl Default for VertexTag {
    fn default() -> Self {
        Self::Smooth
    }
}

impl VertexTag {
    /// Parse a vertex tag from a string descriptor.
    #[must_use]
    pub fn from_descriptor(descriptor: &str) -> Self {
        match descriptor.trim().to_lowercase().as_str() {
            "s" | "smooth" => Self::Smooth,
            "c" | "crease" => Self::Crease,
            "l" | "corner" => Self::Corner,
            "d" | "dart" => Self::Dart,
            other => Self::Custom(other.to_owned()),
        }
    }

    /// Parse a vertex tag from an integer (0 = smooth, 1 = crease, 2 = corner, 3 = dart).
    #[must_use]
    pub fn from_int(value: i32) -> Self {
        match value {
            1 => Self::Crease,
            2 => Self::Corner,
            3 => Self::Dart,
            _ => Self::Smooth,
        }
    }

    /// Parse a vertex tag from a `Value`.
    ///
    /// Accepts:
    /// - `Value::Text` - parsed as descriptor
    /// - `Value::Number` - 0=smooth, 1=crease, 2=corner, 3=dart
    /// - `Value::List` - uses first element
    ///
    /// Returns `None` for incompatible values.
    #[must_use]
    pub fn parse(value: Option<&Value>) -> Option<Self> {
        match value {
            Some(Value::Text(text)) => Some(Self::from_descriptor(text)),
            Some(Value::Number(number)) => Some(Self::from_int(number.round() as i32)),
            Some(Value::List(values)) if !values.is_empty() => Self::parse(values.first()),
            None => Some(Self::default()),
            _ => None,
        }
    }

    /// Parse a vertex tag from a `Value`, returning an error on failure.
    ///
    /// # Arguments
    /// * `value` - The value to parse
    /// * `context` - Context string for error messages
    pub fn parse_or_err(value: Option<&Value>, context: &str) -> Result<Self, SubdError> {
        Self::parse(value).ok_or_else(|| {
            SubdError::OperationFailed(format!("{context} requires a tag value (text or number)"))
        })
    }

    /// Convert to a string representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Smooth => "smooth",
            Self::Crease => "crease",
            Self::Corner => "corner",
            Self::Dart => "dart",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Check if this vertex should remain sharp during subdivision.
    #[must_use]
    pub fn is_sharp(&self) -> bool {
        matches!(self, Self::Corner | Self::Crease)
    }
}

// ============================================================================
// SubD mesh elements
// ============================================================================

/// A vertex in a SubD mesh.
#[derive(Debug, Clone)]
pub struct SubdVertex {
    /// Unique vertex ID.
    pub id: usize,
    /// 3D position.
    pub position: [f64; 3],
    /// Vertex sharpness tag.
    pub tag: VertexTag,
}

impl SubdVertex {
    /// Create a new vertex with default (smooth) tag.
    #[must_use]
    pub fn new(id: usize, position: [f64; 3]) -> Self {
        Self {
            id,
            position,
            tag: VertexTag::default(),
        }
    }
}

/// An edge in a SubD mesh.
#[derive(Debug, Clone)]
pub struct SubdEdge {
    /// Unique edge ID.
    pub id: usize,
    /// Vertex indices (start, end).
    pub vertices: (usize, usize),
    /// Edge sharpness tag.
    pub tag: EdgeTag,
    /// Face IDs sharing this edge.
    pub faces: Vec<usize>,
}

impl SubdEdge {
    /// Create a new edge with default (smooth) tag.
    #[must_use]
    pub fn new(id: usize, v0: usize, v1: usize) -> Self {
        Self {
            id,
            vertices: normalized_edge_pair(v0, v1),
            tag: EdgeTag::default(),
            faces: Vec::new(),
        }
    }

    /// Check if this is a boundary edge (only one adjacent face).
    #[must_use]
    pub fn is_boundary(&self) -> bool {
        self.faces.len() <= 1
    }
}

/// A face in a SubD mesh (n-sided polygon).
#[derive(Debug, Clone)]
pub struct SubdFace {
    /// Unique face ID.
    pub id: usize,
    /// Vertex indices forming the face (in order).
    pub vertices: Vec<usize>,
    /// Edge IDs forming the face boundary.
    pub edges: Vec<usize>,
}

impl SubdFace {
    /// Create a new face.
    #[must_use]
    pub fn new(id: usize, vertices: Vec<usize>) -> Self {
        Self {
            id,
            vertices,
            edges: Vec::new(),
        }
    }

    /// Get the number of vertices/edges in this face.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Check if this is a quad face.
    #[must_use]
    pub fn is_quad(&self) -> bool {
        self.vertices.len() == 4
    }

    /// Check if this is a triangle face.
    #[must_use]
    pub fn is_triangle(&self) -> bool {
        self.vertices.len() == 3
    }
}

// ============================================================================
// Main SubD mesh structure
// ============================================================================

/// A subdivision surface mesh.
///
/// This is a polygonal mesh (typically quads) that can be subdivided
/// using Catmull-Clark or similar schemes. It tracks topology explicitly
/// for efficient subdivision operations.
///
/// # Vertex ID Invariant
/// Vertex IDs always match their index in the `vertices` Vec after any
/// topology-modifying operation (construction, combine, rebuild_topology).
#[derive(Debug, Clone)]
pub struct SubdMesh {
    /// All vertices in the mesh.
    pub vertices: Vec<SubdVertex>,
    /// All edges in the mesh.
    pub edges: Vec<SubdEdge>,
    /// All faces in the mesh.
    pub faces: Vec<SubdFace>,
}

impl Default for SubdMesh {
    fn default() -> Self {
        Self::empty()
    }
}

impl SubdMesh {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create an empty SubD mesh.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
        }
    }

    /// Create a SubD mesh from raw vertices and face indices.
    ///
    /// # Arguments
    /// * `positions` - Vertex positions.
    /// * `face_indices` - Indices into positions for each face (can be quads, triangles, or n-gons).
    #[must_use]
    pub fn from_vertices_faces(positions: Vec<[f64; 3]>, face_indices: Vec<Vec<usize>>) -> Self {
        let vertices = positions
            .into_iter()
            .enumerate()
            .map(|(id, pos)| SubdVertex::new(id, pos))
            .collect();

        let faces = face_indices
            .into_iter()
            .enumerate()
            .filter_map(|(id, verts)| {
                if verts.len() < 3 {
                    None
                } else {
                    Some(SubdFace::new(id, verts))
                }
            })
            .collect();

        let mut mesh = Self {
            vertices,
            edges: Vec::new(),
            faces,
        };
        mesh.rebuild_topology();
        mesh
    }

    /// Create a SubD mesh from a triangle mesh (GeomMesh).
    ///
    /// Converts triangles to faces directly. For better SubD results,
    /// consider quad-remeshing first.
    #[must_use]
    pub fn from_triangle_mesh(mesh: &GeomMesh) -> Self {
        let positions = mesh.positions.clone();
        let mut face_indices = Vec::new();

        for chunk in mesh.indices.chunks(3) {
            if chunk.len() == 3 {
                face_indices.push(vec![chunk[0] as usize, chunk[1] as usize, chunk[2] as usize]);
            }
        }

        Self::from_vertices_faces(positions, face_indices)
    }

    /// Create a box SubD from bounding box corners.
    ///
    /// This creates a 6-faced quad box that subdivides cleanly.
    #[must_use]
    pub fn box_from_bounds(min: [f64; 3], max: [f64; 3]) -> Self {
        // Ensure min < max
        let (min, max) = ensure_valid_bounds(min, max);

        // 8 vertices of a box
        let positions = vec![
            [min[0], min[1], min[2]], // 0: bottom-back-left
            [max[0], min[1], min[2]], // 1: bottom-back-right
            [max[0], max[1], min[2]], // 2: bottom-front-right
            [min[0], max[1], min[2]], // 3: bottom-front-left
            [min[0], min[1], max[2]], // 4: top-back-left
            [max[0], min[1], max[2]], // 5: top-back-right
            [max[0], max[1], max[2]], // 6: top-front-right
            [min[0], max[1], max[2]], // 7: top-front-left
        ];

        // 6 quad faces (counter-clockwise winding for outward normals)
        let face_indices = vec![
            vec![0, 3, 2, 1], // bottom (z = min)
            vec![4, 5, 6, 7], // top (z = max)
            vec![0, 1, 5, 4], // back (y = min)
            vec![1, 2, 6, 5], // right (x = max)
            vec![2, 3, 7, 6], // front (y = max)
            vec![3, 0, 4, 7], // left (x = min)
        ];

        Self::from_vertices_faces(positions, face_indices)
    }

    /// Create a box SubD from a BBox.
    #[must_use]
    pub fn box_from_bbox(bbox: BBox) -> Self {
        Self::box_from_bounds(bbox.min.to_array(), bbox.max.to_array())
    }

    // ========================================================================
    // Topology operations
    // ========================================================================

    /// Rebuild the edge and face-edge topology from face vertex lists.
    ///
    /// This should be called after modifying vertex indices in faces.
    pub fn rebuild_topology(&mut self) {
        // Re-index vertices and faces
        for (i, v) in self.vertices.iter_mut().enumerate() {
            v.id = i;
        }
        for (i, f) in self.faces.iter_mut().enumerate() {
            f.id = i;
        }

        // Preserve existing edge tags
        let mut tag_lookup: HashMap<(usize, usize), EdgeTag> = HashMap::new();
        for edge in &self.edges {
            let key = normalized_edge_pair(edge.vertices.0, edge.vertices.1);
            tag_lookup.entry(key).or_insert_with(|| edge.tag.clone());
        }

        // Build edges from face boundaries
        let mut edge_map: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        let mut edges = Vec::new();

        for face in &mut self.faces {
            face.edges.clear();
            let n = face.vertices.len();
            if n < 2 {
                continue;
            }

            for i in 0..n {
                let v0 = face.vertices[i];
                let v1 = face.vertices[(i + 1) % n];
                let key = normalized_edge_pair(v0, v1);

                let edge_id = *edge_map.entry(key).or_insert_with(|| {
                    let id = edges.len();
                    let tag = tag_lookup.get(&key).cloned().unwrap_or_default();
                    edges.push(SubdEdge {
                        id,
                        vertices: key,
                        tag,
                        faces: Vec::new(),
                    });
                    id
                });

                let edge = &mut edges[edge_id];
                if !edge.faces.contains(&face.id) {
                    edge.faces.push(face.id);
                }
                face.edges.push(edge_id);
            }
        }

        self.edges = edges;
    }

    /// Get a vertex by ID.
    #[must_use]
    pub fn vertex(&self, id: usize) -> Option<&SubdVertex> {
        self.vertices.get(id)
    }

    /// Get an edge by ID.
    #[must_use]
    pub fn edge(&self, id: usize) -> Option<&SubdEdge> {
        self.edges.get(id)
    }

    /// Get a face by ID.
    #[must_use]
    pub fn face(&self, id: usize) -> Option<&SubdFace> {
        self.faces.get(id)
    }

    /// Find boundary vertices (vertices on boundary edges).
    #[must_use]
    pub fn boundary_vertices(&self) -> Vec<usize> {
        let mut boundary = BTreeSet::new();
        for edge in &self.edges {
            if edge.is_boundary() {
                boundary.insert(edge.vertices.0);
                boundary.insert(edge.vertices.1);
            }
        }
        boundary.into_iter().collect()
    }

    /// Check if the mesh is closed (no boundary edges).
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.edges.iter().all(|e| e.faces.len() >= 2)
    }

    /// Compute bounding box.
    #[must_use]
    pub fn bounding_box(&self) -> Option<BBox> {
        if self.vertices.is_empty() {
            return None;
        }

        let first = &self.vertices[0].position;
        let mut min = Point3::new(first[0], first[1], first[2]);
        let mut max = min;

        for v in &self.vertices {
            min.x = min.x.min(v.position[0]);
            min.y = min.y.min(v.position[1]);
            min.z = min.z.min(v.position[2]);
            max.x = max.x.max(v.position[0]);
            max.y = max.y.max(v.position[1]);
            max.z = max.z.max(v.position[2]);
        }

        Some(BBox::new(min, max))
    }

    // ========================================================================
    // Tagging operations
    // ========================================================================

    /// Apply an edge tag to specified edges.
    pub fn apply_edge_tag(&mut self, edge_ids: &[usize], tag: EdgeTag) {
        let lookup: BTreeSet<_> = edge_ids.iter().copied().collect();
        for edge in &mut self.edges {
            if lookup.contains(&edge.id) {
                edge.tag = tag.clone();
            }
        }
    }

    /// Apply a vertex tag to specified vertices.
    pub fn apply_vertex_tag(&mut self, vertex_ids: &[usize], tag: VertexTag) {
        let lookup: BTreeSet<_> = vertex_ids.iter().copied().collect();
        for vertex in &mut self.vertices {
            if lookup.contains(&vertex.id) {
                vertex.tag = tag.clone();
            }
        }
    }

    /// Mark all boundary edges as creases.
    pub fn crease_boundaries(&mut self) {
        for edge in &mut self.edges {
            if edge.is_boundary() {
                edge.tag = EdgeTag::Crease;
            }
        }
    }

    /// Mark all boundary vertices as corners.
    pub fn corner_boundary_vertices(&mut self) {
        let boundary = self.boundary_vertices();
        self.apply_vertex_tag(&boundary, VertexTag::Corner);
    }

    // ========================================================================
    // Subdivision / smoothing
    // ========================================================================

    /// Apply Catmull-Clark-like subdivision smoothing (in-place).
    ///
    /// This is a simplified vertex averaging that approximates subdivision.
    /// Each iteration moves vertices toward the average of their neighbors.
    ///
    /// # Crease/Corner Behavior
    /// - Vertices tagged as `Corner` or `Crease` remain fixed.
    /// - Edges tagged as `Crease` are skipped when averaging neighbors.
    /// - Vertices connected only via crease edges will not move.
    ///
    /// # Arguments
    /// * `iterations` - Number of smoothing passes (0 = no-op).
    ///
    /// # See Also
    /// Use [`smoothed`](Self::smoothed) to get a smoothed copy without mutation.
    pub fn smooth(&mut self, iterations: usize) {
        if iterations == 0 || self.vertices.is_empty() {
            return;
        }

        self.rebuild_topology();

        for _ in 0..iterations {
            let n = self.vertices.len();
            let mut sums = vec![[0.0, 0.0, 0.0]; n];
            let mut counts = vec![0usize; n];

            // Accumulate neighbor positions via edges
            for edge in &self.edges {
                let (a, b) = edge.vertices;
                if a < n && b < n {
                    let pa = self.vertices[a].position;
                    let pb = self.vertices[b].position;

                    // Skip sharp edges for crease preservation
                    if !edge.tag.is_sharp() {
                        sums[a][0] += pb[0];
                        sums[a][1] += pb[1];
                        sums[a][2] += pb[2];
                        counts[a] += 1;

                        sums[b][0] += pa[0];
                        sums[b][1] += pa[1];
                        sums[b][2] += pa[2];
                        counts[b] += 1;
                    }
                }
            }

            // Update vertex positions
            for (i, vertex) in self.vertices.iter_mut().enumerate() {
                // Skip sharp vertices
                if vertex.tag.is_sharp() {
                    continue;
                }

                if counts[i] == 0 {
                    continue;
                }

                let avg = [
                    sums[i][0] / counts[i] as f64,
                    sums[i][1] / counts[i] as f64,
                    sums[i][2] / counts[i] as f64,
                ];

                // Blend original with average (0.5 = midpoint)
                vertex.position = [
                    (vertex.position[0] + avg[0]) * 0.5,
                    (vertex.position[1] + avg[1]) * 0.5,
                    (vertex.position[2] + avg[2]) * 0.5,
                ];
            }
        }
    }

    /// Return a smoothed copy without modifying self.
    #[must_use]
    pub fn smoothed(&self, iterations: usize) -> Self {
        let mut copy = self.clone();
        copy.smooth(iterations);
        copy
    }

    // ========================================================================
    // Combine / Fuse operations
    // ========================================================================

    /// Combine two SubD meshes (union without boolean).
    ///
    /// Simply merges vertices and faces; does not resolve intersections.
    /// Edge tags from both meshes are preserved.
    #[must_use]
    pub fn combine(mut self, other: Self) -> Self {
        let vertex_offset = self.vertices.len();
        let face_offset = self.faces.len();

        // Add vertices with offset IDs
        for (i, mut v) in other.vertices.into_iter().enumerate() {
            v.id = vertex_offset + i;
            self.vertices.push(v);
        }

        // Preserve edge tags from the other mesh before rebuild
        // Store them with offset vertex indices so they can be restored
        for edge in &other.edges {
            let offset_key = normalized_edge_pair(
                edge.vertices.0 + vertex_offset,
                edge.vertices.1 + vertex_offset,
            );
            // Temporarily add edges so rebuild_topology can preserve their tags
            self.edges.push(SubdEdge {
                id: self.edges.len(),
                vertices: offset_key,
                tag: edge.tag.clone(),
                faces: Vec::new(), // Will be rebuilt
            });
        }

        // Add faces with offset vertex indices
        for (i, mut face) in other.faces.into_iter().enumerate() {
            face.id = face_offset + i;
            face.vertices = face.vertices.into_iter().map(|v| v + vertex_offset).collect();
            face.edges.clear();
            self.faces.push(face);
        }

        self.rebuild_topology();
        self
    }

    /// Fuse two SubD meshes with union semantics.
    ///
    /// This is a simple combine; true boolean operations would require
    /// intersection handling.
    #[must_use]
    pub fn fuse_union(a: Option<Self>, b: Option<Self>) -> Self {
        match (a, b) {
            (Some(left), Some(right)) => left.combine(right),
            (Some(left), None) => left,
            (None, Some(right)) => right,
            (None, None) => Self::empty(),
        }
    }

    /// Fuse two SubD meshes with intersection semantics.
    ///
    /// This is a simplified version that creates a box from the bbox intersection.
    #[must_use]
    pub fn fuse_intersection(a: Option<Self>, b: Option<Self>) -> Self {
        let a = match a {
            Some(mesh) => mesh,
            None => return b.unwrap_or_else(Self::empty),
        };
        let b = match b {
            Some(mesh) => mesh,
            None => return a,
        };

        let bbox_a = match a.bounding_box() {
            Some(b) => b,
            None => return Self::empty(),
        };
        let bbox_b = match b.bounding_box() {
            Some(b) => b,
            None => return Self::empty(),
        };

        // Intersect bounding boxes
        let min = [
            bbox_a.min.x.max(bbox_b.min.x),
            bbox_a.min.y.max(bbox_b.min.y),
            bbox_a.min.z.max(bbox_b.min.z),
        ];
        let max = [
            bbox_a.max.x.min(bbox_b.max.x),
            bbox_a.max.y.min(bbox_b.max.y),
            bbox_a.max.z.min(bbox_b.max.z),
        ];

        // Check for valid intersection
        if min[0] > max[0] || min[1] > max[1] || min[2] > max[2] {
            return Self::empty();
        }

        Self::box_from_bounds(min, max)
    }

    // ========================================================================
    // MultiPipe support
    // ========================================================================

    /// Create a simple SubD representation for a pipe network.
    ///
    /// # Current Limitations
    /// This is a **placeholder implementation** that creates a bounding box
    /// enclosing the input points with padding equal to the radius.
    /// A full implementation would create actual tubular geometry along
    /// the point path with proper junction handling.
    ///
    /// # Arguments
    /// * `points` - Points along the pipe network (at least one required).
    /// * `radius` - Pipe radius (minimum 0.25 used if smaller).
    /// * `cap` - Whether to mark boundary edges as creases.
    ///
    /// # Errors
    /// Returns `SubdError::EmptyInput` if `points` is empty.
    #[must_use]
    pub fn multi_pipe(points: &[[f64; 3]], radius: f64, cap: bool) -> Result<Self, SubdError> {
        if points.is_empty() {
            return Err(SubdError::EmptyInput("MultiPipe requires at least one point".into()));
        }

        // Compute bounding box
        let mut min = points[0];
        let mut max = points[0];
        for p in points.iter().skip(1) {
            for i in 0..3 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }

        // Expand by radius (with minimum to avoid degenerate geometry)
        let padding = radius.abs().max(MIN_PIPE_RADIUS);
        for i in 0..3 {
            min[i] -= padding;
            max[i] += padding;
            // Ensure non-degenerate box
            if (max[i] - min[i]).abs() < MIN_BOX_DIMENSION {
                min[i] -= DEGENERATE_BOX_PADDING;
                max[i] += DEGENERATE_BOX_PADDING;
            }
        }

        let mut mesh = Self::box_from_bounds(min, max);

        if cap {
            mesh.crease_boundaries();
        }

        Ok(mesh)
    }

    // ========================================================================
    // Conversion to triangle mesh
    // ========================================================================

    /// Convert the SubD control mesh to a triangle mesh.
    ///
    /// This triangulates each face by fanning from the centroid.
    #[must_use]
    pub fn to_triangle_mesh(&self, options: SubdOptions) -> (GeomMesh, SubdDiagnostics) {
        let smoothed = if options.density > 1 {
            self.smoothed(options.density - 1)
        } else {
            self.clone()
        };

        smoothed.triangulate()
    }

    /// Triangulate the current mesh state.
    fn triangulate(&self) -> (GeomMesh, SubdDiagnostics) {
        let mut positions: Vec<[f64; 3]> = self.vertices.iter().map(|v| v.position).collect();
        let mut indices: Vec<u32> = Vec::new();
        let mut diagnostics = SubdDiagnostics::default();

        for face in &self.faces {
            if face.vertices.len() < 3 {
                diagnostics.degenerate_faces += 1;
                continue;
            }

            if face.is_triangle() {
                // Direct triangle
                indices.push(face.vertices[0] as u32);
                indices.push(face.vertices[1] as u32);
                indices.push(face.vertices[2] as u32);
            } else {
                // Fan triangulation from centroid
                let mut centroid = [0.0, 0.0, 0.0];
                let mut count = 0usize;

                for &vid in &face.vertices {
                    if let Some(v) = self.vertex(vid) {
                        centroid[0] += v.position[0];
                        centroid[1] += v.position[1];
                        centroid[2] += v.position[2];
                        count += 1;
                    }
                }

                if count < 3 {
                    diagnostics.degenerate_faces += 1;
                    continue;
                }

                centroid[0] /= count as f64;
                centroid[1] /= count as f64;
                centroid[2] /= count as f64;

                let centroid_idx = positions.len() as u32;
                positions.push(centroid);

                let n = face.vertices.len();
                for i in 0..n {
                    let v0 = face.vertices[i] as u32;
                    let v1 = face.vertices[(i + 1) % n] as u32;
                    indices.push(v0);
                    indices.push(v1);
                    indices.push(centroid_idx);
                }
            }
        }

        diagnostics.vertex_count = positions.len();
        diagnostics.triangle_count = indices.len() / 3;
        diagnostics.face_count = self.faces.len();
        diagnostics.edge_count = self.edges.len();

        let mesh = GeomMesh {
            positions,
            indices,
            uvs: None,
            normals: None,
            tangents: None,
        };

        (mesh, diagnostics)
    }

    /// Convert the SubD control mesh directly (without smoothing).
    ///
    /// This is equivalent to the "Control Polygon" operation.
    #[must_use]
    pub fn to_control_mesh(&self) -> (GeomMesh, SubdDiagnostics) {
        self.triangulate()
    }

    // ========================================================================
    // Value bridge methods (Phase 3 integration)
    // ========================================================================

    /// Attempt to parse a `SubdMesh` from a `Value`.
    ///
    /// Accepts either:
    /// - A serialized SubD: `List["subd", vertices, edges, faces]`
    /// - A `Value::Surface` or `Value::Mesh` (converted to SubD faces)
    ///
    /// Returns `None` if the value cannot be parsed.
    #[must_use]
    pub fn from_value(value: &Value) -> Option<Self> {
        // Try parsing as serialized SubD format
        if let Some(subd) = Self::try_parse_subd_value(value) {
            return Some(subd);
        }
        // Try parsing as surface/mesh
        Self::from_surface_value(value)
    }

    /// Try to parse a `Value::Surface` or `Value::Mesh` as a `SubdMesh`.
    #[must_use]
    pub fn from_surface_value(value: &Value) -> Option<Self> {
        match value {
            Value::Surface { vertices, faces } => {
                let face_indices: Vec<Vec<usize>> = faces
                    .iter()
                    .filter(|face| face.len() >= 3)
                    .map(|face| face.iter().map(|idx| *idx as usize).collect())
                    .collect();
                Some(Self::from_vertices_faces(vertices.clone(), face_indices))
            }
            Value::Mesh {
                vertices, indices, ..
            } => {
                // Convert triangle indices to faces
                let face_indices: Vec<Vec<usize>> = indices
                    .chunks(3)
                    .filter(|chunk| chunk.len() == 3)
                    .map(|chunk| vec![chunk[0] as usize, chunk[1] as usize, chunk[2] as usize])
                    .collect();
                Some(Self::from_vertices_faces(vertices.clone(), face_indices))
            }
            Value::List(values) => {
                // Try each element in the list
                for entry in values {
                    if let Some(subd) = Self::from_surface_value(entry) {
                        return Some(subd);
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Try to parse a serialized SubD value (format: `["subd", vertices, edges, faces]`).
    fn try_parse_subd_value(value: &Value) -> Option<Self> {
        let Value::List(items) = value else {
            return None;
        };
        if items.len() != 4 {
            return None;
        }
        let Value::Text(label) = &items[0] else {
            return None;
        };
        if !label.eq_ignore_ascii_case("subd") {
            return None;
        }
        let vertices = Self::parse_vertex_list(&items[1])?;
        let edges = Self::parse_edge_list(&items[2]).unwrap_or_default();
        let faces = Self::parse_face_list(&items[3])?;

        let mut subd = Self {
            vertices,
            edges,
            faces,
        };
        subd.rebuild_topology();
        Some(subd)
    }

    /// Parse a list of vertices from a Value.
    fn parse_vertex_list(value: &Value) -> Option<Vec<SubdVertex>> {
        let Value::List(entries) = value else {
            return None;
        };
        let mut vertices = Vec::with_capacity(entries.len());
        for (index, entry) in entries.iter().enumerate() {
            let Value::List(items) = entry else {
                return None;
            };
            let mut id = index;
            let mut position = None;
            let mut tag = VertexTag::default();
            for item in items {
                match item {
                    Value::Number(number) => {
                        if number.is_finite() {
                            id = number.round().max(0.0) as usize;
                        }
                    }
                    Value::Point(value) | Value::Vector(value) => {
                        position = Some(*value);
                    }
                    Value::List(values) => {
                        if position.is_none() {
                            if let Some(parsed) = Self::list_to_point(values) {
                                position = Some(parsed);
                            }
                        }
                    }
                    Value::Text(text) => {
                        tag = VertexTag::from_descriptor(text);
                    }
                    _ => {}
                }
            }
            let position = position?;
            vertices.push(SubdVertex { id, position, tag });
        }
        Some(vertices)
    }

    /// Parse a list of edges from a Value.
    fn parse_edge_list(value: &Value) -> Option<Vec<SubdEdge>> {
        let Value::List(entries) = value else {
            return None;
        };
        let mut edges = Vec::with_capacity(entries.len());
        for (index, entry) in entries.iter().enumerate() {
            let Value::List(items) = entry else {
                return None;
            };
            let mut iter = items.iter();
            let mut id = index;
            if let Some(Value::Number(number)) = iter.next() {
                if number.is_finite() {
                    id = number.round().max(0.0) as usize;
                }
            }
            let vertex_indices = Self::collect_indices_from_value(iter.next());
            let vertices = if vertex_indices.len() >= 2 {
                normalized_edge_pair(vertex_indices[0], vertex_indices[1])
            } else {
                (0, 0)
            };
            let mut tag = EdgeTag::default();
            let mut faces = Vec::new();
            if let Some(v) = iter.next() {
                if let Value::Text(text) = v {
                    tag = EdgeTag::from_descriptor(text);
                } else {
                    faces = Self::collect_indices_from_value(Some(v));
                }
            }
            if faces.is_empty() {
                faces = Self::collect_indices_from_value(iter.next());
            }
            edges.push(SubdEdge {
                id,
                vertices,
                tag,
                faces,
            });
        }
        Some(edges)
    }

    /// Parse a list of faces from a Value.
    fn parse_face_list(value: &Value) -> Option<Vec<SubdFace>> {
        let Value::List(entries) = value else {
            return None;
        };
        let mut faces = Vec::with_capacity(entries.len());
        for (index, entry) in entries.iter().enumerate() {
            let Value::List(items) = entry else {
                return None;
            };
            let mut iter = items.iter();
            let mut id = index;
            if let Some(Value::Number(number)) = iter.next() {
                if number.is_finite() {
                    id = number.round().max(0.0) as usize;
                }
            }
            let vertices = iter
                .next()
                .map(|v| Self::collect_indices_from_value(Some(v)))
                .unwrap_or_default();
            let edge_list = iter
                .next()
                .map(|v| Self::collect_indices_from_value(Some(v)))
                .unwrap_or_default();
            faces.push(SubdFace {
                id,
                vertices,
                edges: edge_list,
            });
        }
        Some(faces)
    }

    /// Helper to convert a list of numbers to a point.
    fn list_to_point(values: &[Value]) -> Option<[f64; 3]> {
        if values.len() < 3 {
            return None;
        }
        let x = match &values[0] {
            Value::Number(v) => *v,
            _ => return None,
        };
        let y = match &values[1] {
            Value::Number(v) => *v,
            _ => return None,
        };
        let z = match &values[2] {
            Value::Number(v) => *v,
            _ => return None,
        };
        Some([x, y, z])
    }

    /// Helper to collect indices from a Value.
    fn collect_indices_from_value(value: Option<&Value>) -> Vec<usize> {
        match value {
            Some(Value::Number(number)) if number.is_finite() => {
                if *number < 0.0 {
                    Vec::new()
                } else {
                    vec![number.round() as usize]
                }
            }
            Some(Value::Text(text)) => text
                .split(|c| c == ',' || c == ';' || c == ' ')
                .filter_map(|part| part.trim().parse::<usize>().ok())
                .collect(),
            Some(Value::List(values)) => values
                .iter()
                .flat_map(|v| Self::collect_indices_from_value(Some(v)))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Serialize this `SubdMesh` to a `Value` in SubD format.
    ///
    /// Format: `["subd", [vertices...], [edges...], [faces...]]`
    ///
    /// This format preserves all topology information including edge/vertex tags.
    #[must_use]
    pub fn to_value(&self) -> Value {
        let mut clone = self.clone();
        clone.rebuild_topology();

        let vertices = clone
            .vertices
            .iter()
            .map(|vertex| {
                Value::List(vec![
                    Value::Number(vertex.id as f64),
                    Value::Point(vertex.position),
                    Value::Text(vertex.tag.as_str().to_owned()),
                ])
            })
            .collect();

        let edges = clone
            .edges
            .iter()
            .map(|edge| {
                let mut entry = vec![
                    Value::Number(edge.id as f64),
                    Value::List(vec![
                        Value::Number(edge.vertices.0 as f64),
                        Value::Number(edge.vertices.1 as f64),
                    ]),
                    Value::Text(edge.tag.as_str().to_owned()),
                ];
                if !edge.faces.is_empty() {
                    entry.push(Value::List(
                        edge.faces
                            .iter()
                            .map(|id| Value::Number(*id as f64))
                            .collect(),
                    ));
                }
                Value::List(entry)
            })
            .collect();

        let faces = clone
            .faces
            .iter()
            .map(|face| {
                let mut entry = vec![
                    Value::Number(face.id as f64),
                    Value::List(
                        face.vertices
                            .iter()
                            .map(|id| Value::Number(*id as f64))
                            .collect(),
                    ),
                ];
                if !face.edges.is_empty() {
                    entry.push(Value::List(
                        face.edges
                            .iter()
                            .map(|id| Value::Number(*id as f64))
                            .collect(),
                    ));
                }
                Value::List(entry)
            })
            .collect();

        Value::List(vec![
            Value::Text("subd".to_owned()),
            Value::List(vertices),
            Value::List(edges),
            Value::List(faces),
        ])
    }

    /// Convert to a legacy `Value::Surface`.
    ///
    /// This creates a polygon mesh (not triangulated) from the SubD control mesh.
    #[must_use]
    pub fn to_surface_value(&self) -> Value {
        let mut clone = self.clone();
        clone.rebuild_topology();
        let vertices = clone.vertices.iter().map(|v| v.position).collect();
        let faces = clone
            .faces
            .iter()
            .filter(|face| face.vertices.len() >= 3)
            .map(|face| face.vertices.iter().map(|idx| *idx as u32).collect())
            .collect();
        Value::Surface { vertices, faces }
    }

    /// Convert to a `Value::Surface` with subdivision/smoothing applied.
    ///
    /// # Arguments
    /// * `density` - Subdivision density (1 = control mesh, 2+ = smoothed).
    #[must_use]
    pub fn to_surface_with_density(&self, density: usize) -> Value {
        if density <= 1 {
            return self.to_surface_value();
        }
        let mut clone = self.clone();
        let steps = density.saturating_sub(1);
        if steps > 0 {
            clone.smooth(steps);
        }
        clone.rebuild_topology();

        let mut vertices: Vec<[f64; 3]> = clone.vertices.iter().map(|v| v.position).collect();
        let mut faces: Vec<Vec<u32>> = Vec::new();

        for face in &clone.faces {
            if face.vertices.len() < 3 {
                continue;
            }
            let mut centroid = [0.0, 0.0, 0.0];
            let mut count = 0usize;
            for id in &face.vertices {
                if let Some(vertex) = clone.vertex(*id) {
                    centroid[0] += vertex.position[0];
                    centroid[1] += vertex.position[1];
                    centroid[2] += vertex.position[2];
                    count += 1;
                }
            }
            if count < 3 {
                continue;
            }
            centroid[0] /= count as f64;
            centroid[1] /= count as f64;
            centroid[2] /= count as f64;
            let centroid_index = vertices.len();
            vertices.push(centroid);
            for index in 0..face.vertices.len() {
                let a = face.vertices[index] as u32;
                let b = face.vertices[(index + 1) % face.vertices.len()] as u32;
                faces.push(vec![a, b, centroid_index as u32]);
            }
        }

        if faces.is_empty() {
            return clone.to_surface_value();
        }
        Value::Surface { vertices, faces }
    }

    /// Convert to a `Value::Mesh` (triangle mesh).
    ///
    /// # Arguments
    /// * `options` - Subdivision options (density, interpolation).
    #[must_use]
    pub fn to_mesh_value(&self, options: SubdOptions) -> Value {
        let (mesh, diag) = self.to_triangle_mesh(options);
        Value::Mesh {
            vertices: mesh.positions,
            indices: mesh.indices,
            normals: mesh.normals,
            uvs: mesh.uvs,
            diagnostics: Some(crate::graph::value::MeshDiagnostics {
                triangle_count: diag.triangle_count,
                vertex_count: diag.vertex_count,
                degenerate_triangle_count: diag.degenerate_faces,
                open_edge_count: 0,
                non_manifold_edge_count: 0,
                welded_vertex_count: 0,
                flipped_triangle_count: 0,
                self_intersection_count: 0,
                boolean_fallback_used: false,
                warnings: diag.warnings.clone(),
            }),
        }
    }
}

// ============================================================================
// Options and diagnostics
// ============================================================================

/// Options for SubD operations.
#[derive(Debug, Clone)]
pub struct SubdOptions {
    /// Subdivision density (1 = control mesh, 2+ = smoothed).
    /// Values greater than 1 apply (density - 1) smoothing iterations.
    pub density: usize,
    /// Whether to interpolate vertices during smoothing.
    /// 
    /// **Note:** This option is reserved for future use and currently has no effect.
    pub interpolate: bool,
}

impl Default for SubdOptions {
    fn default() -> Self {
        Self {
            density: 1,
            interpolate: false,
        }
    }
}

impl SubdOptions {
    /// Create options with specified density.
    #[must_use]
    pub fn with_density(density: usize) -> Self {
        Self {
            density: density.max(1),
            ..Default::default()
        }
    }
}

/// Diagnostics from SubD operations.
#[derive(Debug, Clone, Default)]
pub struct SubdDiagnostics {
    /// Number of vertices in output.
    pub vertex_count: usize,
    /// Number of triangles in output.
    pub triangle_count: usize,
    /// Number of faces in SubD mesh.
    pub face_count: usize,
    /// Number of edges in SubD mesh.
    pub edge_count: usize,
    /// Number of degenerate faces skipped.
    pub degenerate_faces: usize,
    /// Warnings generated during processing.
    pub warnings: Vec<String>,
}

impl SubdDiagnostics {
    /// Convert to a GeomMeshDiagnostics.
    #[must_use]
    pub fn to_mesh_diagnostics(&self) -> GeomMeshDiagnostics {
        GeomMeshDiagnostics {
            triangle_count: self.triangle_count,
            degenerate_triangle_count: self.degenerate_faces,
            warnings: self.warnings.clone(),
            ..Default::default()
        }
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Normalize an edge pair so that the smaller index comes first.
#[inline]
fn normalized_edge_pair(a: usize, b: usize) -> (usize, usize) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Minimum dimension for a non-degenerate box.
const MIN_BOX_DIMENSION: f64 = 1e-10;
/// Default padding applied to degenerate box dimensions.
const DEGENERATE_BOX_PADDING: f64 = 0.5;
/// Minimum radius for multi_pipe bounding box.
const MIN_PIPE_RADIUS: f64 = 0.25;

/// Ensure bounds are valid (min <= max), with fallback padding.
fn ensure_valid_bounds(mut min: [f64; 3], mut max: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    for i in 0..3 {
        if min[i] > max[i] {
            std::mem::swap(&mut min[i], &mut max[i]);
        }
        // Ensure non-degenerate
        if (max[i] - min[i]).abs() < MIN_BOX_DIMENSION {
            min[i] -= DEGENERATE_BOX_PADDING;
            max[i] += DEGENERATE_BOX_PADDING;
        }
    }
    (min, max)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_creation() {
        let mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert_eq!(mesh.vertices.len(), 8);
        assert_eq!(mesh.faces.len(), 6);
        assert_eq!(mesh.edges.len(), 12);

        // All faces should be quads
        for face in &mesh.faces {
            assert!(face.is_quad(), "Box face should be a quad");
        }

        // Should be closed
        assert!(mesh.is_closed());
    }

    #[test]
    fn test_box_triangulation() {
        let mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let (tri_mesh, diag) = mesh.to_control_mesh();

        // 6 quad faces * 4 triangles per quad (fan from centroid) = 24 triangles
        assert_eq!(diag.triangle_count, 24);
        assert_eq!(tri_mesh.indices.len(), 72); // 24 * 3

        // Original 8 vertices + 6 centroids = 14
        assert_eq!(tri_mesh.positions.len(), 14);
    }

    #[test]
    fn test_smooth() {
        let mut mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let original_positions: Vec<_> = mesh.vertices.iter().map(|v| v.position).collect();

        mesh.smooth(1);

        // Vertices should have moved
        let mut any_moved = false;
        for (orig, v) in original_positions.iter().zip(&mesh.vertices) {
            if orig != &v.position {
                any_moved = true;
                break;
            }
        }
        assert!(any_moved, "Smoothing should move vertices");
    }

    #[test]
    fn test_edge_tags() {
        let mut mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(mesh.edges.iter().all(|e| e.tag == EdgeTag::Smooth));

        mesh.apply_edge_tag(&[0, 1, 2], EdgeTag::Crease);
        let crease_count = mesh.edges.iter().filter(|e| e.tag == EdgeTag::Crease).count();
        assert_eq!(crease_count, 3);
    }

    #[test]
    fn test_vertex_tags() {
        let mut mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(mesh.vertices.iter().all(|v| v.tag == VertexTag::Smooth));

        mesh.apply_vertex_tag(&[0, 1], VertexTag::Corner);
        let corner_count = mesh.vertices.iter().filter(|v| v.tag == VertexTag::Corner).count();
        assert_eq!(corner_count, 2);
    }

    #[test]
    fn test_combine() {
        let box1 = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let box2 = SubdMesh::box_from_bounds([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);

        let combined = box1.combine(box2);
        assert_eq!(combined.vertices.len(), 16);
        assert_eq!(combined.faces.len(), 12);
    }

    #[test]
    fn test_combine_preserves_edge_tags() {
        let mut box1 = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let mut box2 = SubdMesh::box_from_bounds([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);

        // Mark some edges as creases in both boxes
        box1.apply_edge_tag(&[0, 1, 2], EdgeTag::Crease);
        box2.apply_edge_tag(&[0, 1], EdgeTag::Crease);

        let crease_count_box1 = box1.edges.iter().filter(|e| e.tag == EdgeTag::Crease).count();
        let crease_count_box2 = box2.edges.iter().filter(|e| e.tag == EdgeTag::Crease).count();
        assert_eq!(crease_count_box1, 3);
        assert_eq!(crease_count_box2, 2);

        let combined = box1.combine(box2);
        let total_creases = combined.edges.iter().filter(|e| e.tag == EdgeTag::Crease).count();
        
        // Edge tags from both meshes should be preserved
        assert_eq!(total_creases, 5, "Edge tags should be preserved when combining meshes");
    }

    #[test]
    fn test_fuse_union() {
        let box1 = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let box2 = SubdMesh::box_from_bounds([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);

        let fused = SubdMesh::fuse_union(Some(box1), Some(box2));
        assert_eq!(fused.vertices.len(), 16);
    }

    #[test]
    fn test_fuse_intersection() {
        let box1 = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let box2 = SubdMesh::box_from_bounds([1.0, 1.0, 1.0], [3.0, 3.0, 3.0]);

        let fused = SubdMesh::fuse_intersection(Some(box1), Some(box2));
        // Should get a box from [1,1,1] to [2,2,2]
        assert!(!fused.vertices.is_empty());

        let bbox = fused.bounding_box().unwrap();
        assert!((bbox.min.x - 1.0).abs() < 0.01);
        assert!((bbox.max.x - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_multi_pipe() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        let mesh = SubdMesh::multi_pipe(&points, 0.5, false).unwrap();
        assert!(!mesh.vertices.is_empty());
    }

    #[test]
    fn test_from_triangle_mesh() {
        // Create a simple tetrahedron
        let tri_mesh = GeomMesh {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            indices: vec![
                0, 1, 2, // base
                0, 1, 3, // side 1
                1, 2, 3, // side 2
                2, 0, 3, // side 3
            ],
            uvs: None,
            normals: None,
            tangents: None,
        };

        let subd = SubdMesh::from_triangle_mesh(&tri_mesh);
        assert_eq!(subd.vertices.len(), 4);
        assert_eq!(subd.faces.len(), 4);
    }

    #[test]
    fn test_boundary_vertices() {
        // Create a simple quad (open surface with 4 boundary edges)
        let mesh = SubdMesh::from_vertices_faces(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![vec![0, 1, 2, 3]],
        );

        let boundary = mesh.boundary_vertices();
        assert_eq!(boundary.len(), 4);
        assert!(!mesh.is_closed());
    }

    #[test]
    fn test_edge_tag_parsing() {
        assert_eq!(EdgeTag::from_descriptor("smooth"), EdgeTag::Smooth);
        assert_eq!(EdgeTag::from_descriptor("crease"), EdgeTag::Crease);
        assert_eq!(EdgeTag::from_descriptor("sharp"), EdgeTag::Crease);
        assert_eq!(EdgeTag::from_descriptor("custom"), EdgeTag::Custom("custom".into()));
        assert_eq!(EdgeTag::from_int(0), EdgeTag::Smooth);
        assert_eq!(EdgeTag::from_int(1), EdgeTag::Crease);
    }

    #[test]
    fn test_vertex_tag_parsing() {
        assert_eq!(VertexTag::from_descriptor("smooth"), VertexTag::Smooth);
        assert_eq!(VertexTag::from_descriptor("corner"), VertexTag::Corner);
        assert_eq!(VertexTag::from_descriptor("dart"), VertexTag::Dart);
        assert_eq!(VertexTag::from_int(0), VertexTag::Smooth);
        assert_eq!(VertexTag::from_int(2), VertexTag::Corner);
        assert_eq!(VertexTag::from_int(3), VertexTag::Dart);
    }
}
