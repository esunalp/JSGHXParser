use crate::geom::{
    BooleanOp, GeomMesh, Point3, Tolerance, Triangle3, Vec3, boolean_meshes,
    classify_point_in_mesh, extrude_polyline, triangle_triangle_intersection, ExtrusionCaps,
    PointContainment, TriTriIntersection,
};

fn transform_mesh_z(mesh: &GeomMesh, rotate_z_radians: f64, translate: Vec3) -> GeomMesh {
    let c = rotate_z_radians.cos();
    let s = rotate_z_radians.sin();
    let positions = mesh
        .positions
        .iter()
        .copied()
        .map(|p| {
            let x = p[0];
            let y = p[1];
            let z = p[2];
            [
                c * x - s * y + translate.x,
                s * x + c * y + translate.y,
                z + translate.z,
            ]
        })
        .collect();

    GeomMesh {
        positions,
        indices: mesh.indices.clone(),
        uvs: None,
        normals: None,
        tangents: None,
    }
}

#[test]
fn triangle_triangle_intersection_returns_segment() {
    let tri_a = Triangle3::new(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    );
    let tri_b = Triangle3::new(
        Point3::new(0.25, 0.5, -1.0),
        Point3::new(0.75, 0.5, -1.0),
        Point3::new(0.5, 0.5, 1.0),
    );

    let tol = Tolerance::default_geom();
    let hit = triangle_triangle_intersection(tri_a, tri_b, tol).expect("expected intersection");
    let TriTriIntersection::Segment(seg) = hit else {
        panic!("expected segment intersection, got {hit:?}");
    };

    for p in [seg.a, seg.b] {
        assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
        assert!((p.y - 0.5).abs() < 1e-6);
        assert!(p.z.abs() < 1e-6);
    }
}

#[test]
fn classify_point_in_mesh_cube_inside_outside() {
    let square = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let (cube, _) = extrude_polyline(&square, Vec3::new(0.0, 0.0, 1.0), ExtrusionCaps::BOTH)
        .expect("extrude cube");

    let tol = Tolerance::default_geom();
    assert_eq!(
        classify_point_in_mesh(Point3::new(0.5, 0.5, 0.5), &cube, tol),
        PointContainment::Inside
    );
    assert_eq!(
        classify_point_in_mesh(Point3::new(2.0, 0.5, 0.5), &cube, tol),
        PointContainment::Outside
    );
}

#[test]
fn boolean_union_returns_non_empty_mesh() {
    let square = [
        Point3::new(-0.5, -0.5, 0.0),
        Point3::new(0.5, -0.5, 0.0),
        Point3::new(0.5, 0.5, 0.0),
        Point3::new(-0.5, 0.5, 0.0),
    ];
    let (a, _) = extrude_polyline(&square, Vec3::new(0.0, 0.0, 1.0), ExtrusionCaps::BOTH)
        .expect("extrude mesh A");
    let (b_raw, _) =
        extrude_polyline(&square, Vec3::new(0.0, 0.0, 1.0), ExtrusionCaps::BOTH)
            .expect("extrude mesh B");
    let b = transform_mesh_z(&b_raw, 0.2, Vec3::new(0.3, 0.0, 0.2));

    let tol = Tolerance::default_geom();
    let result = boolean_meshes(&a, &b, BooleanOp::Union, tol).expect("boolean union");

    assert!(!result.mesh.positions.is_empty());
    assert!(!result.mesh.indices.is_empty());
    assert_eq!(result.mesh.indices.len() % 3, 0);
    assert!(result
        .mesh
        .indices
        .iter()
        .all(|&i| (i as usize) < result.mesh.positions.len()));

    if result.diagnostics.tolerance_relaxed || result.diagnostics.voxel_fallback_used {
        assert!(result.mesh_diagnostics.boolean_fallback_used);
    }
}
