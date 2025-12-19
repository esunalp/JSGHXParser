use crate::geom::Tolerance;
use crate::geom::solid::{
    CapHolesExOptions, LegacySurfaceMesh, brep_join_legacy, cap_holes_ex_legacy,
    cap_holes_legacy, legacy_surface_is_closed, merge_faces_legacy,
};

fn open_box() -> LegacySurfaceMesh {
    LegacySurfaceMesh {
        vertices: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ],
        faces: vec![
            vec![0, 1, 2, 3], // bottom
            vec![0, 1, 5, 4],
            vec![1, 2, 6, 5],
            vec![2, 3, 7, 6],
            vec![3, 0, 4, 7],
        ],
    }
}

fn closed_box() -> LegacySurfaceMesh {
    let mut mesh = open_box();
    mesh.faces.push(vec![4, 5, 6, 7]); // top
    mesh
}

/// Creates two open boxes that share a face, suitable for testing brep_join.
fn two_adjacent_open_boxes() -> (LegacySurfaceMesh, LegacySurfaceMesh) {
    // Box 1: at origin, missing the +X face
    let box1 = LegacySurfaceMesh {
        vertices: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ],
        faces: vec![
            vec![0, 3, 2, 1], // bottom (CCW from below)
            vec![4, 5, 6, 7], // top
            vec![0, 4, 7, 3], // -X face
            vec![0, 1, 5, 4], // -Y face
            vec![3, 7, 6, 2], // +Y face
            // Missing: +X face (1, 2, 6, 5)
        ],
    };

    // Box 2: at x=1, missing the -X face
    let box2 = LegacySurfaceMesh {
        vertices: vec![
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 1.0],
            [2.0, 0.0, 1.0],
            [2.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
        ],
        faces: vec![
            vec![0, 3, 2, 1], // bottom
            vec![4, 5, 6, 7], // top
            vec![1, 5, 6, 2], // +X face
            vec![0, 1, 5, 4], // -Y face
            vec![3, 7, 6, 2], // +Y face
            // Missing: -X face (0, 4, 7, 3)
        ],
    };

    (box1, box2)
}

#[test]
fn cap_holes_closes_open_box() {
    let tol = Tolerance::default_geom();
    let mesh = open_box();
    assert!(!legacy_surface_is_closed(&mesh, tol));

    let result = cap_holes_legacy(mesh, tol);
    assert_eq!(result.diagnostics.holes_found, 1);
    assert_eq!(result.diagnostics.caps_added, 1);
    assert_eq!(result.diagnostics.added_face_count, 2);
    assert_eq!(result.diagnostics.open_edge_count_before, 4);
    assert_eq!(result.diagnostics.open_edge_count_after, 0);
    assert!(result.is_solid);
}

#[test]
fn cap_holes_no_op_on_closed_box() {
    let tol = Tolerance::default_geom();
    let mesh = closed_box();
    assert!(legacy_surface_is_closed(&mesh, tol));

    let result = cap_holes_legacy(mesh.clone(), tol);
    assert_eq!(result.diagnostics.holes_found, 0);
    assert_eq!(result.diagnostics.caps_added, 0);
    assert_eq!(result.diagnostics.added_face_count, 0);
    assert_eq!(result.diagnostics.open_edge_count_before, 0);
    assert_eq!(result.diagnostics.open_edge_count_after, 0);
    assert!(result.is_solid);
    assert_eq!(result.brep.faces.len(), mesh.faces.len());
}

#[test]
fn cap_holes_ex_respects_max_loop_vertices() {
    let tol = Tolerance::default_geom();
    let mesh = open_box();

    // With max_loop_vertices = 3, the 4-vertex hole should be skipped
    let options = CapHolesExOptions {
        max_loop_vertices: 3,
        ..Default::default()
    };

    let result = cap_holes_ex_legacy(mesh, tol, options);
    assert_eq!(result.diagnostics.holes_found, 1);
    assert_eq!(result.diagnostics.caps_added, 0); // Skipped due to vertex limit
    assert!(!result.is_solid);
    assert!(result.diagnostics.errors.iter().any(|e| e.contains("exceeds max_loop_vertices")));
}

#[test]
fn cap_holes_ex_tracks_planarity_deviation() {
    let tol = Tolerance::default_geom();
    let mesh = open_box(); // Planar hole

    let result = cap_holes_ex_legacy(mesh, tol, CapHolesExOptions::default());
    assert!(result.is_solid);
    // Planar hole should have near-zero deviation
    assert!(result.diagnostics.max_planarity_deviation < 1e-6);
}

#[test]
fn brep_join_reports_closedness() {
    let tol = Tolerance::default_geom();
    let result = brep_join_legacy(vec![open_box(), closed_box()], tol);
    // These two boxes don't share edges, so they remain separate
    assert_eq!(result.breps.len(), 2);
    // Check that we have one open and one closed, order may vary due to HashMap iteration
    assert_eq!(result.closed.iter().filter(|&&c| c).count(), 1);
    assert_eq!(result.closed.iter().filter(|&&c| !c).count(), 1);
    assert_eq!(result.diagnostics.closed_count, 1);
    assert_eq!(result.diagnostics.open_count, 1);
}

#[test]
fn brep_join_welds_adjacent_boxes() {
    let tol = Tolerance::default_geom();
    let (box1, box2) = two_adjacent_open_boxes();

    // Each box is open
    assert!(!legacy_surface_is_closed(&box1, tol));
    assert!(!legacy_surface_is_closed(&box2, tol));

    let result = brep_join_legacy(vec![box1, box2], tol);

    // The boxes share edges and should be merged
    assert!(result.diagnostics.welded_edge_count > 0 || result.diagnostics.merged_vertex_count > 0);
}

#[test]
fn brep_join_empty_input() {
    let tol = Tolerance::default_geom();
    let result = brep_join_legacy(Vec::new(), tol);
    assert!(result.breps.is_empty());
    assert!(result.closed.is_empty());
    assert_eq!(result.diagnostics.input_count, 0);
}

#[test]
fn brep_join_single_input() {
    let tol = Tolerance::default_geom();
    let mesh = closed_box();
    let result = brep_join_legacy(vec![mesh.clone()], tol);
    assert_eq!(result.breps.len(), 1);
    assert_eq!(result.closed, vec![true]);
    assert_eq!(result.diagnostics.input_count, 1);
    assert_eq!(result.diagnostics.output_count, 1);
}

#[test]
fn merge_faces_combines_coplanar_faces() {
    let tol = Tolerance::default_geom();

    // Two coplanar triangles that share an edge
    let mesh = LegacySurfaceMesh {
        vertices: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        faces: vec![
            vec![0, 1, 2], // triangle 1
            vec![0, 2, 3], // triangle 2
        ],
    };

    let result = merge_faces_legacy(&[mesh], tol)
        .expect("merge_faces_legacy should succeed");

    assert_eq!(result.diagnostics.before, 2);
    // Coplanar faces should be merged into one
    assert!(result.diagnostics.merged_count > 0 || result.diagnostics.after == 1);
}

#[test]
fn merge_faces_empty_input() {
    let tol = Tolerance::default_geom();
    let result = merge_faces_legacy(&[], tol);
    assert!(result.is_none());
}

#[test]
fn merge_faces_preserves_non_coplanar() {
    let tol = Tolerance::default_geom();

    // Two triangles that are NOT coplanar
    let mesh = LegacySurfaceMesh {
        vertices: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],  // triangle 1 on Z=0
            [0.5, 0.5, 1.0],  // triangle 2 rises to Z=1
        ],
        faces: vec![
            vec![0, 1, 2], // triangle 1
            vec![0, 1, 3], // triangle 2 (shares edge but different plane)
        ],
    };

    let result = merge_faces_legacy(&[mesh], tol)
        .expect("merge_faces_legacy should succeed");

    // Non-coplanar faces should NOT be merged
    assert_eq!(result.diagnostics.after, 2);
}

#[test]
fn legacy_surface_mesh_default() {
    let mesh = LegacySurfaceMesh::default();
    assert!(mesh.is_empty());
    assert!(mesh.vertices.is_empty());
    assert!(mesh.faces.is_empty());
}

#[test]
fn legacy_surface_mesh_with_capacity() {
    let mesh = LegacySurfaceMesh::with_capacity(100, 50);
    assert!(mesh.is_empty());
    assert!(mesh.vertices.capacity() >= 100);
    assert!(mesh.faces.capacity() >= 50);
}

