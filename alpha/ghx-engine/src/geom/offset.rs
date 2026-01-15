//! Offset, thickening, and shelling operations for surfaces and meshes.
//!
//! This module provides functionality to:
//! - Offset mesh surfaces by a given distance (inside or outside)
//! - Create thickened/shelled meshes from thin surfaces
//! - Handle edge cases like self-intersections and degenerate geometry
//!
//! # Example
//!
//! ```ignore
//! use ghx_engine::geom::{offset_mesh, OffsetOptions, OffsetDirection};
//!
//! let (mesh, diag) = some_mesh_source();
//! let options = OffsetOptions::new(0.1).direction(OffsetDirection::Outside);
//! let (offset_result, offset_diag) = offset_mesh(&mesh, options, Tolerance::default_geom())?;
//! ```

use super::mesh::{finalize_mesh, GeomMesh};
use super::{Point3, Tolerance, Vec3};

/// Direction for offset operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OffsetDirection {
    /// Offset outward (along positive normal direction).
    #[default]
    Outside,
    /// Offset inward (along negative normal direction).
    Inside,
    /// Offset in both directions, creating a thickened shell.
    Both,
}

/// Options for offset operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OffsetOptions {
    /// The offset distance. Must be positive.
    pub distance: f64,
    /// The direction to offset.
    pub direction: OffsetDirection,
    /// Whether to cap open edges after offsetting.
    pub cap_open_edges: bool,
    /// Whether to create a solid shell (connecting inner/outer surfaces).
    pub create_shell: bool,
}

impl OffsetOptions {
    /// Create new offset options with the given distance.
    #[must_use]
    pub fn new(distance: f64) -> Self {
        Self {
            distance,
            direction: OffsetDirection::Outside,
            cap_open_edges: false,
            create_shell: false,
        }
    }

    /// Set the offset direction.
    #[must_use]
    pub const fn direction(mut self, direction: OffsetDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Enable capping of open edges.
    #[must_use]
    pub const fn cap_open_edges(mut self, cap: bool) -> Self {
        self.cap_open_edges = cap;
        self
    }

    /// Create a solid shell by connecting inner and outer surfaces.
    #[must_use]
    pub const fn create_shell(mut self, shell: bool) -> Self {
        self.create_shell = shell;
        self
    }
}

/// Errors that can occur during offset operations.
#[derive(Debug, thiserror::Error)]
pub enum OffsetError {
    /// The offset distance is not valid (not finite or negative).
    #[error("offset distance must be finite and non-negative: {distance}")]
    InvalidDistance { distance: f64 },

    /// The input mesh has no triangles.
    #[error("input mesh has no triangles")]
    EmptyMesh,

    /// The input mesh has invalid geometry (NaN/Inf values).
    #[error("input mesh contains invalid geometry (NaN/Inf values)")]
    InvalidGeometry,

    /// Unable to compute normals for the mesh.
    #[error("failed to compute vertex normals for offset")]
    NormalComputationFailed,

    /// The offset would create self-intersecting geometry.
    #[error("offset distance ({distance}) may cause self-intersection (min feature size: {min_feature_size})")]
    PotentialSelfIntersection { distance: f64, min_feature_size: f64 },
}

/// Diagnostics specific to offset operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OffsetDiagnostics {
    /// Number of vertices in the original mesh.
    pub original_vertex_count: usize,
    /// Number of triangles in the original mesh.
    pub original_triangle_count: usize,
    /// Number of vertices in the offset result.
    pub result_vertex_count: usize,
    /// Number of triangles in the offset result.
    pub result_triangle_count: usize,
    /// Number of open edges detected in the input.
    pub open_edge_count: usize,
    /// Number of rim triangles added (for shell/capping).
    pub rim_triangle_count: usize,
    /// Whether the offset may have caused self-intersections (heuristic).
    pub potential_self_intersection: bool,
    /// Warnings generated during the operation.
    pub warnings: Vec<String>,
}

/// Offset a mesh by moving vertices along their normals.
///
/// # Arguments
/// * `mesh` - The input mesh to offset.
/// * `options` - Options controlling the offset operation.
/// * `tol` - Tolerance for geometry operations.
///
/// # Returns
/// A tuple of the offset mesh and diagnostics.
///
/// # Errors
/// Returns an error if the mesh is empty, contains invalid geometry,
/// or if the offset would create self-intersecting geometry.
#[must_use]
pub fn offset_mesh(
    mesh: &GeomMesh,
    options: OffsetOptions,
    tol: Tolerance,
) -> Result<(GeomMesh, OffsetDiagnostics), OffsetError> {
    // Validate inputs
    if !options.distance.is_finite() || options.distance < 0.0 {
        return Err(OffsetError::InvalidDistance {
            distance: options.distance,
        });
    }

    if mesh.indices.is_empty() || mesh.positions.is_empty() {
        return Err(OffsetError::EmptyMesh);
    }

    // Validate geometry
    for pos in &mesh.positions {
        if !pos[0].is_finite() || !pos[1].is_finite() || !pos[2].is_finite() {
            return Err(OffsetError::InvalidGeometry);
        }
    }

    let original_vertex_count = mesh.positions.len();
    let original_triangle_count = mesh.triangle_count();

    // Compute or use existing normals
    let normals = compute_vertex_normals(mesh)?;

    // Detect open edges for diagnostics
    let open_edges = find_open_edges(&mesh.indices);
    let open_edge_count = open_edges.len();

    // Note: We allow creating shells from open meshes - the open edges will be connected
    // between inner and outer surfaces to form a solid shell.

    // Estimate minimum feature size for self-intersection warning
    let min_feature_size = estimate_min_feature_size(mesh, &normals);
    let mut potential_self_intersection = false;
    let mut warnings = Vec::new();

    if options.distance > min_feature_size * 0.5 {
        potential_self_intersection = true;
        warnings.push(format!(
            "offset distance ({:.4}) exceeds half of min feature size ({:.4}); may cause self-intersection",
            options.distance, min_feature_size
        ));
    }

    // Build offset mesh based on direction
    let (positions, indices, rim_triangle_count) = match options.direction {
        OffsetDirection::Outside => {
            let offset_positions = offset_positions(&mesh.positions, &normals, options.distance);
            if options.cap_open_edges && !open_edges.is_empty() {
                let (capped_pos, capped_idx, cap_count) =
                    add_edge_caps(&offset_positions, &mesh.indices, &open_edges, tol);
                return finalize_offset_result(
                    capped_pos,
                    capped_idx,
                    OffsetDiagnostics {
                        original_vertex_count,
                        original_triangle_count,
                        result_vertex_count: 0, // Will be set by finalize
                        result_triangle_count: 0,
                        open_edge_count,
                        rim_triangle_count: cap_count,
                        potential_self_intersection,
                        warnings,
                    },
                    tol,
                );
            }
            (offset_positions, mesh.indices.clone(), 0)
        }
        OffsetDirection::Inside => {
            let offset_positions =
                offset_positions(&mesh.positions, &normals, -options.distance);
            // Flip triangle winding for inside offset
            let flipped_indices = flip_indices(&mesh.indices);
            (offset_positions, flipped_indices, 0)
        }
        OffsetDirection::Both => {
            // Create both inner and outer surfaces (shell/thickening)
            let (shell_positions, shell_indices, rim_count) = create_shell(
                &mesh.positions,
                &mesh.indices,
                &normals,
                options.distance,
                &open_edges,
            );
            (shell_positions, shell_indices, rim_count)
        }
    };

    finalize_offset_result(
        positions,
        indices,
        OffsetDiagnostics {
            original_vertex_count,
            original_triangle_count,
            result_vertex_count: 0, // Will be set by finalize
            result_triangle_count: 0,
            open_edge_count,
            rim_triangle_count,
            potential_self_intersection,
            warnings,
        },
        tol,
    )
}

/// Offset a mesh outward by the given distance.
///
/// Convenience function that calls `offset_mesh` with `OffsetDirection::Outside`.
#[must_use]
pub fn offset_mesh_outside(
    mesh: &GeomMesh,
    distance: f64,
    tol: Tolerance,
) -> Result<(GeomMesh, OffsetDiagnostics), OffsetError> {
    offset_mesh(mesh, OffsetOptions::new(distance), tol)
}

/// Offset a mesh inward by the given distance.
///
/// Convenience function that calls `offset_mesh` with `OffsetDirection::Inside`.
#[must_use]
pub fn offset_mesh_inside(
    mesh: &GeomMesh,
    distance: f64,
    tol: Tolerance,
) -> Result<(GeomMesh, OffsetDiagnostics), OffsetError> {
    offset_mesh(
        mesh,
        OffsetOptions::new(distance).direction(OffsetDirection::Inside),
        tol,
    )
}

/// Create a thickened shell from a surface mesh.
///
/// This creates both an inner and outer offset surface and connects them
/// at the edges to form a solid shell.
///
/// # Arguments
/// * `mesh` - The input surface mesh.
/// * `thickness` - The total thickness of the shell.
/// * `tol` - Tolerance for geometry operations.
///
/// # Returns
/// A tuple of the shell mesh and diagnostics.
#[must_use]
pub fn thicken_mesh(
    mesh: &GeomMesh,
    thickness: f64,
    tol: Tolerance,
) -> Result<(GeomMesh, OffsetDiagnostics), OffsetError> {
    offset_mesh(
        mesh,
        OffsetOptions::new(thickness / 2.0)
            .direction(OffsetDirection::Both)
            .create_shell(true),
        tol,
    )
}

// ============================================================================
// Internal helper functions
// ============================================================================

/// Compute vertex normals for a mesh by averaging face normals.
fn compute_vertex_normals(mesh: &GeomMesh) -> Result<Vec<Vec3>, OffsetError> {
    // If the mesh already has normals, use them
    if let Some(ref normals) = mesh.normals {
        return Ok(normals
            .iter()
            .map(|n| Vec3::new(n[0], n[1], n[2]))
            .collect());
    }

    let n_verts = mesh.positions.len();
    let mut normals = vec![Vec3::new(0.0, 0.0, 0.0); n_verts];

    for tri in mesh.indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let (Some(p0), Some(p1), Some(p2)) = (
            mesh.positions.get(i0),
            mesh.positions.get(i1),
            mesh.positions.get(i2),
        ) else {
            continue;
        };

        let a = Point3::new(p0[0], p0[1], p0[2]);
        let b = Point3::new(p1[0], p1[1], p1[2]);
        let c = Point3::new(p2[0], p2[1], p2[2]);

        let ab = b.sub_point(a);
        let ac = c.sub_point(a);
        let face_normal = ab.cross(ac);

        // Weight by face area (cross product magnitude)
        normals[i0] = normals[i0].add(face_normal);
        normals[i1] = normals[i1].add(face_normal);
        normals[i2] = normals[i2].add(face_normal);
    }

    // Normalize all vertex normals
    for n in &mut normals {
        if let Some(normalized) = n.normalized() {
            *n = normalized;
        } else {
            // Degenerate normal - use a fallback
            *n = Vec3::new(0.0, 0.0, 1.0);
        }
    }

    Ok(normals)
}

/// Find open (boundary) edges in a mesh.
/// Returns a list of edge pairs (vertex index a, vertex index b).
fn find_open_edges(indices: &[u32]) -> Vec<(u32, u32)> {
    use std::collections::HashMap;

    let mut edge_counts: HashMap<(u32, u32), u32> = HashMap::new();

    for tri in indices.chunks_exact(3) {
        let i0 = tri[0];
        let i1 = tri[1];
        let i2 = tri[2];

        for (a, b) in [(i0, i1), (i1, i2), (i2, i0)] {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            *edge_counts.entry((lo, hi)).or_insert(0) += 1;
        }
    }

    edge_counts
        .into_iter()
        .filter_map(|(edge, count)| if count == 1 { Some(edge) } else { None })
        .collect()
}

/// Estimate the minimum feature size of a mesh based on edge lengths
/// and normal variation. Used for self-intersection warning.
fn estimate_min_feature_size(mesh: &GeomMesh, normals: &[Vec3]) -> f64 {
    let mut min_edge_length = f64::MAX;

    for tri in mesh.indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let (Some(p0), Some(p1), Some(p2)) = (
            mesh.positions.get(i0),
            mesh.positions.get(i1),
            mesh.positions.get(i2),
        ) else {
            continue;
        };

        let a = Point3::new(p0[0], p0[1], p0[2]);
        let b = Point3::new(p1[0], p1[1], p1[2]);
        let c = Point3::new(p2[0], p2[1], p2[2]);

        let len_ab = a.sub_point(b).length();
        let len_bc = b.sub_point(c).length();
        let len_ca = c.sub_point(a).length();

        if len_ab.is_finite() && len_ab > 0.0 {
            min_edge_length = min_edge_length.min(len_ab);
        }
        if len_bc.is_finite() && len_bc > 0.0 {
            min_edge_length = min_edge_length.min(len_bc);
        }
        if len_ca.is_finite() && len_ca > 0.0 {
            min_edge_length = min_edge_length.min(len_ca);
        }
    }

    // Also consider curvature via normal variation (check all 3 edges per triangle)
    let mut min_curvature_radius = f64::MAX;
    for tri in mesh.indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        // Check all 3 edges of the triangle
        for (ia, ib) in [(i0, i1), (i1, i2), (i2, i0)] {
            let (Some(na), Some(nb)) = (normals.get(ia), normals.get(ib)) else {
                continue;
            };

            let (Some(pa), Some(pb)) = (mesh.positions.get(ia), mesh.positions.get(ib)) else {
                continue;
            };

            let edge_len = {
                let dx = pb[0] - pa[0];
                let dy = pb[1] - pa[1];
                let dz = pb[2] - pa[2];
                (dx * dx + dy * dy + dz * dz).sqrt()
            };

            if edge_len <= 0.0 || !edge_len.is_finite() {
                continue;
            }

            // Angle between normals
            let dot = na.dot(*nb).clamp(-1.0, 1.0);
            let angle = dot.acos();
            if angle > 1e-6 && angle.is_finite() {
                // Approximate radius of curvature: r ≈ edge_len / angle
                let radius = edge_len / angle;
                if radius.is_finite() && radius > 0.0 {
                    min_curvature_radius = min_curvature_radius.min(radius);
                }
            }
        }
    }

    // Return the smaller of edge length and curvature radius
    if min_curvature_radius < f64::MAX {
        min_edge_length.min(min_curvature_radius)
    } else if min_edge_length < f64::MAX {
        min_edge_length
    } else {
        1.0 // Fallback
    }
}

/// Offset positions by moving them along normals.
fn offset_positions(positions: &[[f64; 3]], normals: &[Vec3], distance: f64) -> Vec<Point3> {
    positions
        .iter()
        .zip(normals.iter())
        .map(|(pos, normal)| {
            let offset = normal.mul_scalar(distance);
            Point3::new(pos[0] + offset.x, pos[1] + offset.y, pos[2] + offset.z)
        })
        .collect()
}

/// Flip triangle indices to reverse winding.
fn flip_indices(indices: &[u32]) -> Vec<u32> {
    let mut flipped = Vec::with_capacity(indices.len());
    for tri in indices.chunks_exact(3) {
        flipped.push(tri[0]);
        flipped.push(tri[2]);
        flipped.push(tri[1]);
    }
    flipped
}

/// Add cap triangles at open edges.
///
/// This function identifies connected boundary loops from the open edges,
/// projects each loop to a best-fit plane, triangulates using ear-clipping,
/// and adds the cap triangles with correct winding.
///
/// # Arguments
/// * `positions` - The mesh vertex positions.
/// * `indices` - The mesh triangle indices.
/// * `open_edges` - List of boundary edges (vertex index pairs).
/// * `tol` - Tolerance for triangulation.
///
/// # Returns
/// A tuple of (positions, indices, cap_triangle_count) with the cap triangles added.
fn add_edge_caps(
    positions: &[Point3],
    indices: &[u32],
    open_edges: &[(u32, u32)],
    tol: Tolerance,
) -> (Vec<Point3>, Vec<u32>, usize) {
    if open_edges.is_empty() {
        return (positions.to_vec(), indices.to_vec(), 0);
    }

    // Build adjacency map for boundary edges
    let loops = collect_boundary_loops(open_edges);
    if loops.is_empty() {
        return (positions.to_vec(), indices.to_vec(), 0);
    }

    let result_positions = positions.to_vec();
    let mut result_indices = indices.to_vec();
    let mut cap_triangle_count = 0;

    for loop_indices in &loops {
        if loop_indices.len() < 3 {
            continue;
        }

        // Get the 3D points for this loop
        let loop_points: Vec<Point3> = loop_indices
            .iter()
            .filter_map(|&idx| positions.get(idx as usize).copied())
            .collect();

        if loop_points.len() < 3 {
            continue;
        }

        // Compute best-fit plane (centroid + normal via Newell's method)
        let centroid = compute_loop_centroid(&loop_points);
        let normal = compute_loop_normal(&loop_points);

        // Skip if normal is degenerate
        let Some(normal) = normal.normalized() else {
            continue;
        };

        // Compute local 2D basis on the plane
        let (u_axis, v_axis) = compute_plane_basis(normal);

        // Project loop points to 2D
        let uv_points: Vec<super::UvPoint> = loop_points
            .iter()
            .map(|p| {
                let rel = p.sub_point(centroid);
                super::UvPoint::new(rel.dot(u_axis), rel.dot(v_axis))
            })
            .collect();

        // Triangulate the 2D polygon using ear-clipping
        let Ok(tri_indices) = triangulate_simple_polygon(&uv_points, tol) else {
            continue;
        };

        // Convert back to 3D indices and add triangles
        // The triangulation indices are relative to the loop, so we need to map them
        for tri in tri_indices.chunks_exact(3) {
            let i0 = loop_indices[tri[0] as usize];
            let i1 = loop_indices[tri[1] as usize];
            let i2 = loop_indices[tri[2] as usize];

            // Determine winding: cap should face opposite to the loop normal
            // (the loop normal points "outward" from the mesh, so the cap must face "inward"
            // to close the hole properly, acting as the "back" face)
            if let (Some(&p0), Some(&p1), Some(&p2)) = (
                positions.get(i0 as usize),
                positions.get(i1 as usize),
                positions.get(i2 as usize),
            ) {
                let v0 = p1.sub_point(p0);
                let v1 = p2.sub_point(p0);
                let tri_normal = v0.cross(v1);

                // If triangle normal points same direction as loop normal, flip winding
                // to make the cap face the opposite direction
                if tri_normal.dot(normal) > 0.0 {
                    result_indices.extend_from_slice(&[i0, i2, i1]);
                } else {
                    result_indices.extend_from_slice(&[i0, i1, i2]);
                }
                cap_triangle_count += 1;
            }
        }
    }

    (result_positions, result_indices, cap_triangle_count)
}

/// Collect connected boundary loops from a set of open edges.
///
/// Returns a list of loops, where each loop is an ordered list of vertex indices
/// forming a closed boundary.
fn collect_boundary_loops(open_edges: &[(u32, u32)]) -> Vec<Vec<u32>> {
    use std::collections::{HashMap, HashSet};

    if open_edges.is_empty() {
        return Vec::new();
    }

    // Build adjacency map: vertex -> list of connected vertices
    let mut adjacency: HashMap<u32, Vec<u32>> = HashMap::new();
    for &(a, b) in open_edges {
        adjacency.entry(a).or_default().push(b);
        adjacency.entry(b).or_default().push(a);
    }

    let mut visited_edges: HashSet<(u32, u32)> = HashSet::new();
    let mut loops = Vec::new();

    for &(start_a, start_b) in open_edges {
        // Normalize edge for visited check
        let edge_key = if start_a <= start_b {
            (start_a, start_b)
        } else {
            (start_b, start_a)
        };
        if visited_edges.contains(&edge_key) {
            continue;
        }

        // Try to trace a loop starting from this edge
        let mut loop_vertices = vec![start_a];
        let mut prev = start_a;
        let mut current = start_b;
        let mut guard = 0;
        let max_iterations = open_edges.len() * 2 + 1;

        while current != start_a && guard < max_iterations {
            guard += 1;

            // Mark edge as visited
            let edge_key = if prev <= current {
                (prev, current)
            } else {
                (current, prev)
            };
            visited_edges.insert(edge_key);
            loop_vertices.push(current);

            // Find next vertex (not the one we came from)
            let neighbors = adjacency.get(&current);
            let next = neighbors.and_then(|ns| {
                ns.iter()
                    .copied()
                    .find(|&n| {
                        if n == prev {
                            return false;
                        }
                        let edge = if current <= n { (current, n) } else { (n, current) };
                        !visited_edges.contains(&edge)
                    })
            });

            match next {
                Some(n) => {
                    prev = current;
                    current = n;
                }
                None => break, // Dead end, not a closed loop
            }
        }

        // Check if we closed the loop
        if current == start_a && loop_vertices.len() >= 3 {
            // Mark the closing edge as visited
            let closing_edge = if prev <= start_a {
                (prev, start_a)
            } else {
                (start_a, prev)
            };
            visited_edges.insert(closing_edge);
            loops.push(loop_vertices);
        }
    }

    loops
}

/// Compute the centroid of a loop of 3D points.
fn compute_loop_centroid(points: &[Point3]) -> Point3 {
    if points.is_empty() {
        return Point3::new(0.0, 0.0, 0.0);
    }

    let mut sx = 0.0;
    let mut sy = 0.0;
    let mut sz = 0.0;
    for p in points {
        sx += p.x;
        sy += p.y;
        sz += p.z;
    }
    let inv = 1.0 / points.len() as f64;
    Point3::new(sx * inv, sy * inv, sz * inv)
}

/// Compute the normal of a polygon using Newell's method.
fn compute_loop_normal(points: &[Point3]) -> Vec3 {
    if points.len() < 3 {
        return Vec3::new(0.0, 0.0, 0.0);
    }

    let mut n = Vec3::new(0.0, 0.0, 0.0);
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        n.x += (a.y - b.y) * (a.z + b.z);
        n.y += (a.z - b.z) * (a.x + b.x);
        n.z += (a.x - b.x) * (a.y + b.y);
    }
    n
}

/// Compute an orthonormal basis (u_axis, v_axis) for a plane with the given normal.
fn compute_plane_basis(normal: Vec3) -> (Vec3, Vec3) {
    // Choose a reference vector that's not parallel to the normal
    let reference = if normal.x.abs() < 0.9 {
        Vec3::X
    } else {
        Vec3::Y
    };

    // u_axis = reference x normal, normalized
    let u_axis = reference.cross(normal);
    let u_axis = u_axis.normalized().unwrap_or(Vec3::X);

    // v_axis = normal x u_axis
    let v_axis = normal.cross(u_axis);
    let v_axis = v_axis.normalized().unwrap_or(Vec3::Y);

    (u_axis, v_axis)
}

/// Triangulate a simple polygon (no holes) using ear-clipping.
///
/// Returns triangle indices as a flat list [i0, i1, i2, i0', i1', i2', ...].
fn triangulate_simple_polygon(points: &[super::UvPoint], tol: Tolerance) -> Result<Vec<u32>, String> {
    use super::trim::TrimLoop;
    use super::triangulation::triangulate_trim_region;

    if points.len() < 3 {
        return Err("polygon needs at least 3 points".to_string());
    }

    // Create a TrimLoop from the points
    let trim_loop = TrimLoop::new(points.to_vec(), tol).map_err(|e| e.to_string())?;
    let region = super::trim::TrimRegion::from_loops(vec![trim_loop], tol).map_err(|e| e.to_string())?;
    let result = triangulate_trim_region(&region, tol)?;

    Ok(result.indices)
}

/// Create a shell by creating both inner and outer surfaces and connecting them.
fn create_shell(
    positions: &[[f64; 3]],
    indices: &[u32],
    normals: &[Vec3],
    distance: f64,
    open_edges: &[(u32, u32)],
) -> (Vec<Point3>, Vec<u32>, usize) {
    let n_verts = positions.len();

    // Create outer surface (offset outward)
    let outer_positions = offset_positions(positions, normals, distance);

    // Create inner surface (offset inward)
    let inner_positions = offset_positions(positions, normals, -distance);

    // Combine positions: outer first, then inner
    let mut all_positions = Vec::with_capacity(n_verts * 2);
    all_positions.extend(outer_positions);
    all_positions.extend(inner_positions);

    // Outer surface triangles (original winding)
    let mut all_indices = Vec::with_capacity(indices.len() * 2);
    all_indices.extend_from_slice(indices);

    // Inner surface triangles (flipped winding, offset indices)
    let inner_offset = n_verts as u32;
    for tri in indices.chunks_exact(3) {
        all_indices.push(inner_offset + tri[0]);
        all_indices.push(inner_offset + tri[2]);
        all_indices.push(inner_offset + tri[1]);
    }

    // Connect outer and inner surfaces at open edges
    let mut rim_triangle_count = 0;
    for &(a, b) in open_edges {
        // Create a quad connecting outer edge (a,b) to inner edge (a',b')
        let outer_a = a;
        let outer_b = b;
        let inner_a = inner_offset + a;
        let inner_b = inner_offset + b;

        // Two triangles for the quad
        all_indices.extend_from_slice(&[outer_a, outer_b, inner_b]);
        all_indices.extend_from_slice(&[outer_a, inner_b, inner_a]);
        rim_triangle_count += 2;
    }

    (all_positions, all_indices, rim_triangle_count)
}

/// Finalize the offset result by welding, repairing, and computing diagnostics.
fn finalize_offset_result(
    positions: Vec<Point3>,
    indices: Vec<u32>,
    mut diag: OffsetDiagnostics,
    tol: Tolerance,
) -> Result<(GeomMesh, OffsetDiagnostics), OffsetError> {
    let (mesh, mesh_diag) = finalize_mesh(positions, None, indices, tol);

    diag.result_vertex_count = mesh.positions.len();
    diag.result_triangle_count = mesh.triangle_count();

    // Merge warnings
    diag.warnings.extend(mesh_diag.warnings.iter().cloned());

    Ok((mesh, diag))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_quad_mesh() -> GeomMesh {
        // A simple quad (two triangles) in the XY plane
        GeomMesh {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            uvs: None,
            normals: Some(vec![
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ]),
            tangents: None,
        }
    }

    #[test]
    fn test_offset_outside() {
        let mesh = simple_quad_mesh();
        let result = offset_mesh_outside(&mesh, 0.1, Tolerance::default_geom());
        assert!(result.is_ok());

        let (offset_mesh, diag) = result.unwrap();
        // Offset should preserve triangle count
        assert_eq!(diag.original_triangle_count, 2);
        assert_eq!(offset_mesh.triangle_count(), 2);

        // Verify positions are offset in Z direction
        for pos in &offset_mesh.positions {
            assert!((pos[2] - 0.1).abs() < 1e-6, "Z should be ~0.1 after offset");
        }
    }

    #[test]
    fn test_offset_inside() {
        let mesh = simple_quad_mesh();
        let result = offset_mesh_inside(&mesh, 0.1, Tolerance::default_geom());
        assert!(result.is_ok());

        let (offset_mesh, _diag) = result.unwrap();

        // Verify positions are offset in negative Z direction
        for pos in &offset_mesh.positions {
            assert!(
                (pos[2] - (-0.1)).abs() < 1e-6,
                "Z should be ~-0.1 after inside offset"
            );
        }
    }

    #[test]
    fn test_thicken_mesh() {
        let mesh = simple_quad_mesh();
        let result = thicken_mesh(&mesh, 0.2, Tolerance::default_geom());
        assert!(result.is_ok());

        let (shell_mesh, diag) = result.unwrap();
        // Shell should have more triangles (outer + inner + rim)
        assert!(shell_mesh.triangle_count() > 2);
        assert!(diag.rim_triangle_count > 0);
    }

    #[test]
    fn test_invalid_distance() {
        let mesh = simple_quad_mesh();

        // Negative distance
        let result = offset_mesh(&mesh, OffsetOptions::new(-1.0), Tolerance::default_geom());
        assert!(matches!(result, Err(OffsetError::InvalidDistance { .. })));

        // NaN distance
        let result = offset_mesh(&mesh, OffsetOptions::new(f64::NAN), Tolerance::default_geom());
        assert!(matches!(result, Err(OffsetError::InvalidDistance { .. })));

        // Infinity distance
        let result = offset_mesh(
            &mesh,
            OffsetOptions::new(f64::INFINITY),
            Tolerance::default_geom(),
        );
        assert!(matches!(result, Err(OffsetError::InvalidDistance { .. })));
    }

    #[test]
    fn test_empty_mesh() {
        let empty_mesh = GeomMesh {
            positions: vec![],
            indices: vec![],
            uvs: None,
            normals: None,
            tangents: None,
        };

        let result = offset_mesh_outside(&empty_mesh, 0.1, Tolerance::default_geom());
        assert!(matches!(result, Err(OffsetError::EmptyMesh)));
    }

    #[test]
    fn test_compute_vertex_normals() {
        let mesh = simple_quad_mesh();
        let normals = compute_vertex_normals(&mesh).unwrap();

        // All normals should point in +Z
        for n in normals {
            assert!((n.x).abs() < 1e-6);
            assert!((n.y).abs() < 1e-6);
            assert!((n.z - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_find_open_edges() {
        // A single triangle has 3 open edges
        let indices = vec![0, 1, 2];
        let open = find_open_edges(&indices);
        assert_eq!(open.len(), 3);

        // A closed box (simplified) should have no open edges
        // Two triangles sharing an edge
        let indices = vec![0, 1, 2, 0, 2, 3];
        let open = find_open_edges(&indices);
        // Still has open edges (it's a quad, not a closed surface)
        assert!(!open.is_empty());
    }

    #[test]
    fn test_flip_indices() {
        let indices = vec![0, 1, 2, 3, 4, 5];
        let flipped = flip_indices(&indices);
        assert_eq!(flipped, vec![0, 2, 1, 3, 5, 4]);
    }

    // ========================================================================
    // Regression tests for add_edge_caps (cap_open_edges feature)
    // ========================================================================

    /// A single triangle (open mesh) - has 3 boundary edges forming one loop.
    fn single_triangle_mesh() -> GeomMesh {
        GeomMesh {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            indices: vec![0, 1, 2],
            uvs: None,
            normals: Some(vec![
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ]),
            tangents: None,
        }
    }

    #[test]
    fn test_collect_boundary_loops_single_triangle() {
        let indices = vec![0u32, 1, 2];
        let open_edges = find_open_edges(&indices);
        assert_eq!(open_edges.len(), 3);

        let loops = collect_boundary_loops(&open_edges);
        assert_eq!(loops.len(), 1, "single triangle should have one boundary loop");
        assert_eq!(loops[0].len(), 3, "boundary loop should have 3 vertices");
    }

    #[test]
    fn test_collect_boundary_loops_quad() {
        // A quad (two triangles sharing an edge) - has 4 boundary edges
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        let open_edges = find_open_edges(&indices);
        // Edges: (0,1), (1,2), (2,3), (3,0) = 4 open edges
        assert_eq!(open_edges.len(), 4);

        let loops = collect_boundary_loops(&open_edges);
        assert_eq!(loops.len(), 1, "quad should have one boundary loop");
        assert_eq!(loops[0].len(), 4, "boundary loop should have 4 vertices");
    }

    #[test]
    fn test_add_edge_caps_empty_edges() {
        let positions: Vec<Point3> = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
        ];
        let indices = vec![0u32, 1, 2];
        let open_edges: Vec<(u32, u32)> = vec![];

        let (result_pos, result_idx, cap_count) =
            add_edge_caps(&positions, &indices, &open_edges, Tolerance::default_geom());

        assert_eq!(result_pos.len(), positions.len());
        assert_eq!(result_idx.len(), indices.len());
        assert_eq!(cap_count, 0);
    }

    #[test]
    fn test_add_edge_caps_single_triangle() {
        let positions: Vec<Point3> = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 1.0, 0.0),
        ];
        let indices = vec![0u32, 1, 2];
        let open_edges = find_open_edges(&indices);

        let (result_pos, result_idx, cap_count) =
            add_edge_caps(&positions, &indices, &open_edges, Tolerance::default_geom());

        // A single triangle's boundary loop is the triangle itself,
        // so capping would create one triangle (the "back" face).
        assert_eq!(result_pos.len(), 3);
        // Original 3 indices + 3 cap indices = 6
        assert_eq!(result_idx.len(), 6);
        assert_eq!(cap_count, 1);

        // The cap triangle should have opposite winding to the original
        let orig_tri = &result_idx[0..3];
        let cap_tri = &result_idx[3..6];
        // Check that all vertices are reused (no new vertices added)
        assert!(cap_tri.iter().all(|&i| i < 3));
        // Check that cap uses same vertices but different order
        assert_ne!(orig_tri, cap_tri);
    }

    #[test]
    fn test_add_edge_caps_quad() {
        // A quad in XY plane (z=0)
        let positions: Vec<Point3> = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        let open_edges = find_open_edges(&indices);

        let (result_pos, result_idx, cap_count) =
            add_edge_caps(&positions, &indices, &open_edges, Tolerance::default_geom());

        assert_eq!(result_pos.len(), 4);
        // Original 6 indices + 2 cap triangles (6 indices) = 12
        // The boundary loop has 4 vertices, which triangulates into 2 triangles
        assert_eq!(result_idx.len(), 12);
        assert_eq!(cap_count, 2);
    }

    #[test]
    fn test_offset_with_cap_open_edges() {
        let mesh = simple_quad_mesh();
        let options = OffsetOptions::new(0.1).cap_open_edges(true);
        let result = offset_mesh(&mesh, options, Tolerance::default_geom());
        assert!(result.is_ok());

        let (offset_mesh, diag) = result.unwrap();

        // With capping, we should have more triangles than the original
        assert!(offset_mesh.triangle_count() > diag.original_triangle_count);
        // Rim triangle count should be > 0
        assert!(diag.rim_triangle_count > 0);
    }

    #[test]
    fn test_offset_single_triangle_with_cap() {
        let mesh = single_triangle_mesh();
        let options = OffsetOptions::new(0.1).cap_open_edges(true);
        let result = offset_mesh(&mesh, options, Tolerance::default_geom());
        assert!(result.is_ok());

        let (offset_mesh, diag) = result.unwrap();

        // Original: 1 triangle, after capping: 2 triangles (original + back face)
        assert_eq!(diag.original_triangle_count, 1);
        // The offset mesh should have at least 2 triangles (original + cap)
        assert!(offset_mesh.triangle_count() >= 2);
        assert!(diag.rim_triangle_count >= 1);
    }

    #[test]
    fn test_compute_loop_centroid() {
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(2.0, 2.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
        ];
        let centroid = compute_loop_centroid(&points);
        assert!((centroid.x - 1.0).abs() < 1e-10);
        assert!((centroid.y - 1.0).abs() < 1e-10);
        assert!((centroid.z - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_compute_loop_normal() {
        // A square in XY plane, CCW when viewed from +Z
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];
        let normal = compute_loop_normal(&points);
        let normalized = normal.normalized().unwrap();
        // Normal should point in +Z direction
        assert!((normalized.x).abs() < 1e-10);
        assert!((normalized.y).abs() < 1e-10);
        assert!((normalized.z - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_compute_plane_basis() {
        let normal = Vec3::Z;
        let (u_axis, v_axis) = compute_plane_basis(normal);

        // u_axis and v_axis should be orthogonal to normal
        assert!(u_axis.dot(normal).abs() < 1e-10);
        assert!(v_axis.dot(normal).abs() < 1e-10);

        // u_axis and v_axis should be orthogonal to each other
        assert!(u_axis.dot(v_axis).abs() < 1e-10);

        // Both should be unit vectors
        assert!((u_axis.length() - 1.0).abs() < 1e-10);
        assert!((v_axis.length() - 1.0).abs() < 1e-10);
    }
}
