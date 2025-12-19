use crate::geom::{GeomContext, GeomMesh, PlaneSurface, Point3, Vec3, mesh_surface_with_context};

#[test]
fn mesh_surface_has_finite_vertices_and_valid_indices() {
    let plane = PlaneSurface::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.0, 3.0, 0.0));
    let mut ctx = GeomContext::new();
    let (mesh, diag) = mesh_surface_with_context(&plane, 8, 6, &mut ctx);

    mesh.validate().expect("mesh validate");
    assert_eq!(mesh.positions_flat().len(), mesh.positions.len() * 3);
    assert_eq!(mesh.uvs_flat().unwrap().len(), mesh.positions.len() * 2);
    assert_eq!(mesh.normals_flat().unwrap().len(), mesh.positions.len() * 3);
    assert_eq!(mesh.tangents_flat().unwrap().len(), mesh.positions.len() * 3);

    assert_eq!(mesh.positions.len(), 8 * 6);
    assert_eq!(diag.vertex_count, mesh.positions.len());
    assert_eq!(diag.triangle_count, mesh.indices.len() / 3);
    assert_eq!(diag.welded_vertex_count, 0);
    assert_eq!(diag.flipped_triangle_count, 0);
    assert!(diag.open_edge_count > 0);
    assert_eq!(diag.non_manifold_edge_count, 0);

    assert_eq!(ctx.cache.stats().surface_grid_entries, 1);
    assert_eq!(ctx.cache.stats().grid_triangulation_entries, 1);

    let _ = mesh_surface_with_context(&plane, 8, 6, &mut ctx);
    assert_eq!(ctx.cache.stats().surface_grid_entries, 1);
    assert_eq!(ctx.cache.stats().grid_triangulation_entries, 1);

    for p in &mesh.positions {
        assert!(p[0].is_finite());
        assert!(p[1].is_finite());
        assert!(p[2].is_finite());
    }

    let uvs = mesh.uvs.as_ref().unwrap();
    assert_eq!(uvs.len(), mesh.positions.len());
    assert_eq!(uvs[0], [0.0, 0.0]);

    let normals = mesh.normals.as_ref().unwrap();
    assert_eq!(normals.len(), mesh.positions.len());
    for n in normals {
        assert!(n[0].is_finite());
        assert!(n[1].is_finite());
        assert!(n[2].is_finite());
        assert!(n[2] > 0.0);
    }

    assert!(mesh
        .indices
        .iter()
        .all(|i| (*i as usize) < mesh.positions.len()));
}

#[test]
fn geom_mesh_validate_rejects_bad_buffers() {
    let mesh = GeomMesh::new(vec![[0.0, 0.0, 0.0]], vec![0]);
    assert!(mesh.validate().is_err());

    let mesh = GeomMesh::new(vec![[0.0, 0.0, 0.0]], vec![0, 1, 0]);
    assert!(mesh.validate().is_err());
}
