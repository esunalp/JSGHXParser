//! Tests for the subdivision module.

use crate::geom::subdivision::{
    EdgeTag, SubdDiagnostics, SubdMesh, SubdOptions, VertexTag,
};
use crate::geom::mesh::GeomMesh;

// ============================================================================
// Box creation tests
// ============================================================================

#[test]
fn test_box_from_bounds_basic() {
    let mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    
    // Unit box should have 8 vertices, 6 quad faces, 12 edges
    assert_eq!(mesh.vertices.len(), 8, "Box should have 8 vertices");
    assert_eq!(mesh.faces.len(), 6, "Box should have 6 faces");
    assert_eq!(mesh.edges.len(), 12, "Box should have 12 edges");
    
    // All faces should be quads
    for face in &mesh.faces {
        assert_eq!(face.vertex_count(), 4, "Box faces should be quads");
    }
    
    // Box should be closed (no boundary edges)
    assert!(mesh.is_closed(), "Box should be a closed mesh");
}

#[test]
fn test_box_from_bounds_inverted() {
    // Inverted bounds should still produce a valid box
    let mesh = SubdMesh::box_from_bounds([1.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
    
    assert_eq!(mesh.vertices.len(), 8);
    assert_eq!(mesh.faces.len(), 6);
    assert!(mesh.is_closed());
}

#[test]
fn test_box_from_bounds_degenerate() {
    // Degenerate bounds (zero size in one dimension) should be padded
    let mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 0.0]);
    
    assert_eq!(mesh.vertices.len(), 8);
    // Should have non-zero z extent after padding
    let bbox = mesh.bounding_box().unwrap();
    assert!(bbox.max.z - bbox.min.z > 0.0, "Degenerate box should be padded");
}

#[test]
fn test_box_bounding_box() {
    let mesh = SubdMesh::box_from_bounds([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
    let bbox = mesh.bounding_box().unwrap();
    
    assert!((bbox.min.x - 1.0).abs() < 1e-10);
    assert!((bbox.min.y - 2.0).abs() < 1e-10);
    assert!((bbox.min.z - 3.0).abs() < 1e-10);
    assert!((bbox.max.x - 4.0).abs() < 1e-10);
    assert!((bbox.max.y - 5.0).abs() < 1e-10);
    assert!((bbox.max.z - 6.0).abs() < 1e-10);
}

// ============================================================================
// From mesh tests
// ============================================================================

#[test]
fn test_from_triangle_mesh_tetrahedron() {
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
    
    assert_eq!(subd.vertices.len(), 4, "Tetrahedron should have 4 vertices");
    assert_eq!(subd.faces.len(), 4, "Tetrahedron should have 4 faces");
    
    // All faces should be triangles
    for face in &subd.faces {
        assert!(face.is_triangle(), "Tetrahedron faces should be triangles");
    }
}

#[test]
fn test_from_vertices_faces_quad() {
    let mesh = SubdMesh::from_vertices_faces(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![vec![0, 1, 2, 3]],
    );
    
    assert_eq!(mesh.vertices.len(), 4);
    assert_eq!(mesh.faces.len(), 1);
    assert_eq!(mesh.edges.len(), 4);
    
    // Single quad face means all edges are boundary
    assert!(!mesh.is_closed(), "Single quad should be open");
    assert_eq!(mesh.boundary_vertices().len(), 4);
}

#[test]
fn test_from_vertices_faces_degenerate() {
    // Faces with fewer than 3 vertices should be filtered
    let mesh = SubdMesh::from_vertices_faces(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ],
        vec![
            vec![0, 1, 2],    // valid triangle
            vec![0, 1],       // degenerate (2 vertices)
            vec![0],          // degenerate (1 vertex)
        ],
    );
    
    assert_eq!(mesh.faces.len(), 1, "Degenerate faces should be filtered");
}

// ============================================================================
// Triangulation tests
// ============================================================================

#[test]
fn test_triangulate_box() {
    let mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    let (tri_mesh, diag) = mesh.to_triangle_mesh(SubdOptions::default());
    
    // 6 quad faces * 4 triangles per quad (fan from centroid) = 24 triangles
    assert_eq!(diag.triangle_count, 24);
    assert_eq!(tri_mesh.indices.len(), 72); // 24 * 3
    
    // 8 original vertices + 6 centroids = 14
    assert_eq!(tri_mesh.positions.len(), 14);
}

#[test]
fn test_triangulate_triangle_face() {
    // A mesh with triangle faces should triangulate directly
    let mesh = SubdMesh::from_vertices_faces(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ],
        vec![vec![0, 1, 2]],
    );
    
    let (tri_mesh, diag) = mesh.to_control_mesh();
    
    assert_eq!(diag.triangle_count, 1);
    assert_eq!(tri_mesh.positions.len(), 3); // No centroid added for triangles
}

#[test]
fn test_triangulate_with_density() {
    let mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    
    let (mesh_d1, _) = mesh.to_triangle_mesh(SubdOptions::with_density(1));
    let (mesh_d2, _) = mesh.to_triangle_mesh(SubdOptions::with_density(2));
    
    // Higher density should produce different vertex positions (smoothed)
    // The structure stays the same, but positions change
    assert_eq!(mesh_d1.positions.len(), mesh_d2.positions.len());
}

// ============================================================================
// Smoothing tests
// ============================================================================

#[test]
fn test_smooth_moves_vertices() {
    let mut mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    let original_positions: Vec<_> = mesh.vertices.iter().map(|v| v.position).collect();
    
    mesh.smooth(1);
    
    // At least some vertices should have moved
    let mut any_moved = false;
    for (orig, v) in original_positions.iter().zip(&mesh.vertices) {
        let dx = (orig[0] - v.position[0]).abs();
        let dy = (orig[1] - v.position[1]).abs();
        let dz = (orig[2] - v.position[2]).abs();
        if dx > 1e-10 || dy > 1e-10 || dz > 1e-10 {
            any_moved = true;
            break;
        }
    }
    assert!(any_moved, "Smoothing should move vertices");
}

#[test]
fn test_smooth_preserves_corners() {
    let mut mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    
    // Mark vertex 0 as a corner
    mesh.apply_vertex_tag(&[0], VertexTag::Corner);
    let original_pos = mesh.vertices[0].position;
    
    mesh.smooth(3);
    
    // Corner vertex should not move
    assert_eq!(mesh.vertices[0].position, original_pos, "Corner vertex should not move");
}

#[test]
fn test_smoothed_returns_copy() {
    let mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    let original_pos = mesh.vertices[0].position;
    
    let smoothed = mesh.smoothed(2);
    
    // Original should be unchanged
    assert_eq!(mesh.vertices[0].position, original_pos);
    
    // Smoothed should be different
    assert_ne!(smoothed.vertices[0].position, original_pos);
}

// ============================================================================
// Tagging tests
// ============================================================================

#[test]
fn test_edge_tag_application() {
    let mut mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    
    // All edges start smooth
    assert!(mesh.edges.iter().all(|e| e.tag == EdgeTag::Smooth));
    
    // Mark some edges as creases
    mesh.apply_edge_tag(&[0, 1, 2], EdgeTag::Crease);
    
    let crease_count = mesh.edges.iter().filter(|e| e.tag == EdgeTag::Crease).count();
    assert_eq!(crease_count, 3);
}

#[test]
fn test_vertex_tag_application() {
    let mut mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    
    // All vertices start smooth
    assert!(mesh.vertices.iter().all(|v| v.tag == VertexTag::Smooth));
    
    // Mark some vertices as corners
    mesh.apply_vertex_tag(&[0, 1], VertexTag::Corner);
    
    let corner_count = mesh.vertices.iter().filter(|v| v.tag == VertexTag::Corner).count();
    assert_eq!(corner_count, 2);
}

#[test]
fn test_crease_boundaries() {
    let mesh = SubdMesh::from_vertices_faces(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![vec![0, 1, 2, 3]],
    );
    
    let mut mesh = mesh;
    mesh.crease_boundaries();
    
    // All 4 boundary edges should be creases
    assert!(mesh.edges.iter().all(|e| e.tag == EdgeTag::Crease));
}

#[test]
fn test_edge_tag_parsing() {
    assert_eq!(EdgeTag::from_descriptor("smooth"), EdgeTag::Smooth);
    assert_eq!(EdgeTag::from_descriptor("s"), EdgeTag::Smooth);
    assert_eq!(EdgeTag::from_descriptor("crease"), EdgeTag::Crease);
    assert_eq!(EdgeTag::from_descriptor("c"), EdgeTag::Crease);
    assert_eq!(EdgeTag::from_descriptor("sharp"), EdgeTag::Crease);
    assert!(matches!(EdgeTag::from_descriptor("custom"), EdgeTag::Custom(_)));
    
    assert_eq!(EdgeTag::from_int(0), EdgeTag::Smooth);
    assert_eq!(EdgeTag::from_int(1), EdgeTag::Crease);
    assert_eq!(EdgeTag::from_int(99), EdgeTag::Smooth);
}

#[test]
fn test_vertex_tag_parsing() {
    assert_eq!(VertexTag::from_descriptor("smooth"), VertexTag::Smooth);
    assert_eq!(VertexTag::from_descriptor("corner"), VertexTag::Corner);
    assert_eq!(VertexTag::from_descriptor("l"), VertexTag::Corner);
    assert_eq!(VertexTag::from_descriptor("dart"), VertexTag::Dart);
    assert_eq!(VertexTag::from_descriptor("d"), VertexTag::Dart);
    
    assert_eq!(VertexTag::from_int(0), VertexTag::Smooth);
    assert_eq!(VertexTag::from_int(1), VertexTag::Crease);
    assert_eq!(VertexTag::from_int(2), VertexTag::Corner);
    assert_eq!(VertexTag::from_int(3), VertexTag::Dart);
    assert_eq!(VertexTag::from_int(99), VertexTag::Smooth);
}

// ============================================================================
// Combine / Fuse tests
// ============================================================================

#[test]
fn test_combine_two_boxes() {
    let box1 = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    let box2 = SubdMesh::box_from_bounds([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);
    
    let combined = box1.combine(box2);
    
    assert_eq!(combined.vertices.len(), 16, "Combined should have 16 vertices");
    assert_eq!(combined.faces.len(), 12, "Combined should have 12 faces");
    assert_eq!(combined.edges.len(), 24, "Combined should have 24 edges");
}

#[test]
fn test_fuse_union() {
    let box1 = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    let box2 = SubdMesh::box_from_bounds([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);
    
    let fused = SubdMesh::fuse_union(Some(box1), Some(box2));
    assert_eq!(fused.vertices.len(), 16);
    
    // Union with None should return the other
    let box3 = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    let fused2 = SubdMesh::fuse_union(Some(box3), None);
    assert_eq!(fused2.vertices.len(), 8);
    
    // Both None should return empty
    let fused3 = SubdMesh::fuse_union(None, None);
    assert!(fused3.vertices.is_empty());
}

#[test]
fn test_fuse_intersection_overlapping() {
    let box1 = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
    let box2 = SubdMesh::box_from_bounds([1.0, 1.0, 1.0], [3.0, 3.0, 3.0]);
    
    let fused = SubdMesh::fuse_intersection(Some(box1), Some(box2));
    
    // Should produce a box from [1,1,1] to [2,2,2]
    let bbox = fused.bounding_box().unwrap();
    assert!((bbox.min.x - 1.0).abs() < 0.01);
    assert!((bbox.min.y - 1.0).abs() < 0.01);
    assert!((bbox.min.z - 1.0).abs() < 0.01);
    assert!((bbox.max.x - 2.0).abs() < 0.01);
    assert!((bbox.max.y - 2.0).abs() < 0.01);
    assert!((bbox.max.z - 2.0).abs() < 0.01);
}

#[test]
fn test_fuse_intersection_non_overlapping() {
    let box1 = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    let box2 = SubdMesh::box_from_bounds([5.0, 5.0, 5.0], [6.0, 6.0, 6.0]);
    
    let fused = SubdMesh::fuse_intersection(Some(box1), Some(box2));
    
    // No intersection should return empty
    assert!(fused.vertices.is_empty());
}

// ============================================================================
// MultiPipe tests
// ============================================================================

#[test]
fn test_multi_pipe_basic() {
    let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
    let mesh = SubdMesh::multi_pipe(&points, 0.5, false).unwrap();
    
    // Should produce a box enclosing the points with padding
    assert_eq!(mesh.vertices.len(), 8);
    assert_eq!(mesh.faces.len(), 6);
    
    // Verify bounding box contains all points with padding
    let bbox = mesh.bounding_box().unwrap();
    assert!(bbox.min.x < 0.0); // Should have padding
    assert!(bbox.max.x > 1.0);
}

#[test]
fn test_multi_pipe_with_caps() {
    let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
    let mesh = SubdMesh::multi_pipe(&points, 0.5, true).unwrap();
    
    // With caps, boundary edges should be creased
    // (Since it's a closed box, there are no boundary edges, but crease_boundaries was called)
    // This verifies the cap flag path works
    assert!(!mesh.vertices.is_empty());
}

#[test]
fn test_multi_pipe_empty_input() {
    let result = SubdMesh::multi_pipe(&[], 0.5, false);
    assert!(result.is_err());
}

// ============================================================================
// Boundary and topology tests
// ============================================================================

#[test]
fn test_boundary_vertices_open_surface() {
    // Single quad face = 4 boundary edges = 4 boundary vertices
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
}

#[test]
fn test_boundary_vertices_closed_mesh() {
    // A closed box has no boundary vertices
    let mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    
    let boundary = mesh.boundary_vertices();
    assert!(boundary.is_empty());
}

#[test]
fn test_is_closed() {
    let box_mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    assert!(box_mesh.is_closed());
    
    let open_mesh = SubdMesh::from_vertices_faces(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![vec![0, 1, 2, 3]],
    );
    assert!(!open_mesh.is_closed());
}

// ============================================================================
// Diagnostics tests
// ============================================================================

#[test]
fn test_diagnostics_structure() {
    let mesh = SubdMesh::box_from_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    let (_, diag) = mesh.to_triangle_mesh(SubdOptions::default());
    
    assert_eq!(diag.face_count, 6);
    assert_eq!(diag.edge_count, 12);
    assert_eq!(diag.degenerate_faces, 0);
    assert!(diag.warnings.is_empty());
}

#[test]
fn test_diagnostics_to_mesh_diagnostics() {
    let mut diag = SubdDiagnostics::default();
    diag.triangle_count = 24;
    diag.degenerate_faces = 2;
    diag.warnings.push("test warning".into());
    
    let mesh_diag = diag.to_mesh_diagnostics();
    assert_eq!(mesh_diag.triangle_count, 24);
    assert_eq!(mesh_diag.degenerate_triangle_count, 2);
    assert_eq!(mesh_diag.warnings.len(), 1);
}

// ============================================================================
// Empty mesh tests
// ============================================================================

#[test]
fn test_empty_mesh() {
    let mesh = SubdMesh::empty();
    
    assert!(mesh.vertices.is_empty());
    assert!(mesh.edges.is_empty());
    assert!(mesh.faces.is_empty());
    assert!(mesh.bounding_box().is_none());
    assert!(mesh.is_closed()); // Vacuously true
}

#[test]
fn test_empty_triangulation() {
    let mesh = SubdMesh::empty();
    let (tri_mesh, diag) = mesh.to_triangle_mesh(SubdOptions::default());
    
    assert!(tri_mesh.positions.is_empty());
    assert!(tri_mesh.indices.is_empty());
    assert_eq!(diag.triangle_count, 0);
}
