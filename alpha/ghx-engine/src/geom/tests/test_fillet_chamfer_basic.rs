use crate::geom::{
    FilletEdgeOptions, GeomMesh, Point3, Tolerance, fillet_polyline_points,
    fillet_triangle_mesh_edges,
};

#[test]
fn fillet_polyline_right_angle_inserts_arc_points() {
    let polyline = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
    ];

    let tol = Tolerance::default_geom();
    let (rounded, diag) = fillet_polyline_points(&polyline, 0.2, 4, false, tol).unwrap();

    assert_eq!(diag.corner_count, 1);
    assert_eq!(diag.filleted_corner_count, 1);
    assert_eq!(rounded.len(), 7);

    assert_eq!(rounded[0], polyline[0]);
    assert_eq!(*rounded.last().unwrap(), polyline[2]);

    // Tangent points for a 90-degree corner are at distance r from the corner along each segment.
    assert!((rounded[1].x - 0.8).abs() < 1e-6);
    assert!(rounded[1].y.abs() < 1e-6);
    assert!((rounded[5].x - 1.0).abs() < 1e-6);
    assert!((rounded[5].y - 0.2).abs() < 1e-6);

    let center = Point3::new(0.8, 0.2, 0.0);
    for point in &rounded[1..=5] {
        let dist = point.distance_to(center);
        assert!((dist - 0.2).abs() < 1e-6);
    }
}

#[test]
fn fillet_triangle_mesh_hinge_edge_adds_strip_triangles() {
    // Two triangles sharing edge (0,1), forming a simple hinge.
    let mesh = GeomMesh {
        positions: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
        ],
        indices: vec![0, 1, 2, 1, 0, 3],
        uvs: None,
        normals: None,
        tangents: None,
    };

    let tol = Tolerance::default_geom();
    let options = FilletEdgeOptions::new(0.1, 3);
    let (out, mesh_diag, fillet_diag) =
        fillet_triangle_mesh_edges(&mesh, &[(0, 1)], options, tol).unwrap();

    assert_eq!(fillet_diag.processed_edge_count, 1);
    assert!(out.triangle_count() >= 2 + options.segments * 2);
    assert_eq!(mesh_diag.non_manifold_edge_count, 0);
}

