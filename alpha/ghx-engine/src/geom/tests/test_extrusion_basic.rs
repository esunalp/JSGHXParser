use crate::geom::{
    ExtrusionCaps, Point3, Vec3, extrude_angled_polyline, extrude_polyline, extrude_to_point,
};

#[test]
fn extrude_open_polyline_produces_a_quad_strip() {
    let profile = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
    ];

    let (mesh, diag) =
        extrude_polyline(&profile, Vec3::new(0.0, 0.0, 1.0), ExtrusionCaps::NONE).unwrap();

    assert_eq!(mesh.triangle_count(), 2);
    assert_eq!(diag.triangle_count, 2);
    assert_eq!(diag.vertex_count, 4);
    assert!(diag.open_edge_count > 0);
    assert_eq!(diag.non_manifold_edge_count, 0);
}

#[test]
fn extrude_closed_square_with_caps_is_watertight() {
    let profile = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];

    let (_mesh, diag) =
        extrude_polyline(&profile, Vec3::new(0.0, 0.0, 1.0), ExtrusionCaps::BOTH).unwrap();

    assert_eq!(diag.triangle_count, 12);
    assert_eq!(diag.vertex_count, 8);
    assert_eq!(diag.open_edge_count, 0);
    assert_eq!(diag.non_manifold_edge_count, 0);
}

#[test]
fn extrude_to_point_with_base_cap_is_watertight() {
    let profile = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let tip = Point3::new(0.5, 0.5, 1.0);

    let (_mesh, diag) = extrude_to_point(&profile, tip, true).unwrap();

    assert_eq!(diag.triangle_count, 6);
    assert_eq!(diag.vertex_count, 5);
    assert_eq!(diag.open_edge_count, 0);
    assert_eq!(diag.non_manifold_edge_count, 0);
}

#[test]
fn extrude_angled_zero_angles_matches_prism_triangle_count() {
    let polyline = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];

    let (_mesh, diag) = extrude_angled_polyline(&polyline, 0.0, 1.0, &[]).unwrap();

    assert_eq!(diag.triangle_count, 12);
    assert_eq!(diag.open_edge_count, 0);
    assert_eq!(diag.non_manifold_edge_count, 0);
}

#[test]
fn extrude_angled_nonzero_angles_is_watertight_and_offsets_top_ring() {
    let polyline = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];

    let (mesh, diag) = extrude_angled_polyline(&polyline, 0.0, 1.0, &[0.2]).unwrap();

    assert_eq!(diag.open_edge_count, 0);
    assert_eq!(diag.non_manifold_edge_count, 0);

    let mut max_x = f64::NEG_INFINITY;
    for p in &mesh.positions {
        max_x = max_x.max(p[0]);
    }
    assert!(max_x > 1.0);
}
