#![cfg(feature = "mesh_engine_next")]

use ghx_engine::geom::{
    BooleanDiagnostics, BooleanOp, DeformationDiagnostics, ExtrusionCaps, GeomMesh,
    GeomMeshDiagnostics, LoftDiagnostics, LoftOptions, OffsetDiagnostics, PipeCaps, PipeOptions,
    Point3, RevolveCaps, RevolveOptions, SweepCaps, SweepOptions, Tolerance, Vec3,
    boolean_meshes, extrude_polyline, loft_mesh, offset_mesh_outside, pipe_polyline_with_tolerance,
    revolve_polyline_with_options, sweep1_polyline_with_tolerance, twist_mesh_z,
};

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const SNAPSHOT_QUANTIZE: f64 = 1e-6;
const SNAPSHOT_DECIMALS: usize = 6;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("mesh_engine_next")
}

fn fixture_path(name: &str) -> PathBuf {
    fixtures_dir().join(name)
}

fn normalize_snapshot_text(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n");
    if normalized.ends_with('\n') {
        normalized
    } else {
        format!("{normalized}\n")
    }
}

fn quantize_f64(value: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }
    let value = if value == -0.0 { 0.0 } else { value };
    let q = (value / SNAPSHOT_QUANTIZE).round() * SNAPSHOT_QUANTIZE;
    if q == -0.0 { 0.0 } else { q }
}

fn write_f64(out: &mut String, value: f64) {
    let value = quantize_f64(value);
    let _ = write!(out, "{value:.SNAPSHOT_DECIMALS$}");
}

fn write_vec3_line(out: &mut String, prefix: &str, v: [f64; 3]) {
    let _ = write!(out, "{prefix} ");
    write_f64(out, v[0]);
    out.push(' ');
    write_f64(out, v[1]);
    out.push(' ');
    write_f64(out, v[2]);
    out.push('\n');
}

fn write_vec2_line(out: &mut String, prefix: &str, v: [f64; 2]) {
    let _ = write!(out, "{prefix} ");
    write_f64(out, v[0]);
    out.push(' ');
    write_f64(out, v[1]);
    out.push('\n');
}

fn write_geom_mesh_diagnostics(out: &mut String, diag: &GeomMeshDiagnostics) {
    let _ = writeln!(out, "mesh_diag.vertex_count {}", diag.vertex_count);
    let _ = writeln!(out, "mesh_diag.triangle_count {}", diag.triangle_count);
    let _ = writeln!(out, "mesh_diag.welded_vertex_count {}", diag.welded_vertex_count);
    let _ = writeln!(out, "mesh_diag.flipped_triangle_count {}", diag.flipped_triangle_count);
    let _ = writeln!(
        out,
        "mesh_diag.degenerate_triangle_count {}",
        diag.degenerate_triangle_count
    );
    let _ = writeln!(out, "mesh_diag.open_edge_count {}", diag.open_edge_count);
    let _ = writeln!(
        out,
        "mesh_diag.non_manifold_edge_count {}",
        diag.non_manifold_edge_count
    );
    let _ = writeln!(
        out,
        "mesh_diag.boolean_fallback_used {}",
        diag.boolean_fallback_used
    );
    let _ = writeln!(out, "mesh_diag.warning_count {}", diag.warnings.len());
    for (idx, warning) in diag.warnings.iter().enumerate() {
        let _ = writeln!(out, "mesh_diag.warning.{idx} {warning}");
    }
}

fn write_loft_diagnostics(out: &mut String, diag: &LoftDiagnostics) {
    let _ = writeln!(out, "loft_diag.profile_count {}", diag.profile_count);
    let _ = writeln!(out, "loft_diag.points_per_profile {}", diag.points_per_profile);
    let _ = writeln!(out, "loft_diag.twist_detected {}", diag.twist_detected);
    let _ = write!(out, "loft_diag.max_twist_angle ");
    write_f64(out, diag.max_twist_angle);
    out.push('\n');

    let _ = writeln!(out, "loft_diag.twist_angle_count {}", diag.twist_angles.len());
    for (idx, angle) in diag.twist_angles.iter().copied().enumerate() {
        let _ = write!(out, "loft_diag.twist_angle.{idx} ");
        write_f64(out, angle);
        out.push('\n');
    }

    let _ = writeln!(out, "loft_diag.seam_adjusted {}", diag.seam_adjusted);
    let _ = writeln!(out, "loft_diag.seam_rotation_count {}", diag.seam_rotations.len());
    for (idx, rot) in diag.seam_rotations.iter().copied().enumerate() {
        let _ = writeln!(out, "loft_diag.seam_rotation.{idx} {rot}");
    }

    let _ = writeln!(
        out,
        "loft_diag.self_intersection_detected {}",
        diag.self_intersection_detected
    );
    let _ = writeln!(
        out,
        "loft_diag.self_intersection_hint_count {}",
        diag.self_intersection_hints.len()
    );
    for (idx, (a, b)) in diag.self_intersection_hints.iter().copied().enumerate() {
        let _ = writeln!(out, "loft_diag.self_intersection_hint.{idx} {a} {b}");
    }
}

fn write_boolean_diagnostics(out: &mut String, diag: &BooleanDiagnostics) {
    let _ = writeln!(out, "boolean_diag.op {:?}", diag.op);
    let _ = writeln!(
        out,
        "boolean_diag.input_a_vertex_count {}",
        diag.input_a_vertex_count
    );
    let _ = writeln!(
        out,
        "boolean_diag.input_a_triangle_count {}",
        diag.input_a_triangle_count
    );
    let _ = writeln!(
        out,
        "boolean_diag.input_b_vertex_count {}",
        diag.input_b_vertex_count
    );
    let _ = writeln!(
        out,
        "boolean_diag.input_b_triangle_count {}",
        diag.input_b_triangle_count
    );
    let _ = writeln!(
        out,
        "boolean_diag.intersection_segment_count {}",
        diag.intersection_segment_count
    );
    let _ = writeln!(
        out,
        "boolean_diag.intersection_point_count {}",
        diag.intersection_point_count
    );
    let _ = writeln!(
        out,
        "boolean_diag.coplanar_pair_count {}",
        diag.coplanar_pair_count
    );
    let _ = writeln!(out, "boolean_diag.split_triangle_count_a {}", diag.split_triangle_count_a);
    let _ = writeln!(out, "boolean_diag.split_triangle_count_b {}", diag.split_triangle_count_b);
    let _ = writeln!(
        out,
        "boolean_diag.complex_triangle_count_a {}",
        diag.complex_triangle_count_a
    );
    let _ = writeln!(
        out,
        "boolean_diag.complex_triangle_count_b {}",
        diag.complex_triangle_count_b
    );
    let _ = writeln!(out, "boolean_diag.kept_triangle_count_a {}", diag.kept_triangle_count_a);
    let _ = writeln!(out, "boolean_diag.kept_triangle_count_b {}", diag.kept_triangle_count_b);
    let _ = writeln!(
        out,
        "boolean_diag.indeterminate_triangle_count {}",
        diag.indeterminate_triangle_count
    );
    let _ = writeln!(out, "boolean_diag.tolerance_relaxed {}", diag.tolerance_relaxed);
    let _ = writeln!(out, "boolean_diag.voxel_fallback_used {}", diag.voxel_fallback_used);
    let _ = write!(out, "boolean_diag.tolerance_used ");
    write_f64(out, diag.tolerance_used);
    out.push('\n');

    let _ = writeln!(out, "boolean_diag.warning_count {}", diag.warnings.len());
    for (idx, warning) in diag.warnings.iter().enumerate() {
        let _ = writeln!(out, "boolean_diag.warning.{idx} {warning}");
    }
}

fn write_offset_diagnostics(out: &mut String, diag: &OffsetDiagnostics) {
    let _ = writeln!(out, "offset_diag.original_vertex_count {}", diag.original_vertex_count);
    let _ = writeln!(
        out,
        "offset_diag.original_triangle_count {}",
        diag.original_triangle_count
    );
    let _ = writeln!(out, "offset_diag.result_vertex_count {}", diag.result_vertex_count);
    let _ = writeln!(out, "offset_diag.result_triangle_count {}", diag.result_triangle_count);
    let _ = writeln!(out, "offset_diag.open_edge_count {}", diag.open_edge_count);
    let _ = writeln!(out, "offset_diag.rim_triangle_count {}", diag.rim_triangle_count);
    let _ = writeln!(
        out,
        "offset_diag.potential_self_intersection {}",
        diag.potential_self_intersection
    );
    let _ = writeln!(out, "offset_diag.warning_count {}", diag.warnings.len());
    for (idx, warning) in diag.warnings.iter().enumerate() {
        let _ = writeln!(out, "offset_diag.warning.{idx} {warning}");
    }
}

fn write_deformation_diagnostics(out: &mut String, diag: &DeformationDiagnostics) {
    let _ = writeln!(
        out,
        "deform_diag.original_vertex_count {}",
        diag.original_vertex_count
    );
    let _ = writeln!(
        out,
        "deform_diag.original_triangle_count {}",
        diag.original_triangle_count
    );
    let _ = writeln!(out, "deform_diag.result_vertex_count {}", diag.result_vertex_count);
    let _ = writeln!(
        out,
        "deform_diag.result_triangle_count {}",
        diag.result_triangle_count
    );
    let _ = write!(out, "deform_diag.min_displacement ");
    write_f64(out, diag.min_displacement);
    out.push('\n');
    let _ = write!(out, "deform_diag.max_displacement ");
    write_f64(out, diag.max_displacement);
    out.push('\n');
    let _ = write!(out, "deform_diag.avg_displacement ");
    write_f64(out, diag.avg_displacement);
    out.push('\n');
    let _ = writeln!(out, "deform_diag.welded_vertex_count {}", diag.welded_vertex_count);
    let _ = writeln!(out, "deform_diag.warning_count {}", diag.warnings.len());
    for (idx, warning) in diag.warnings.iter().enumerate() {
        let _ = writeln!(out, "deform_diag.warning.{idx} {warning}");
    }
}

fn write_mesh(out: &mut String, mesh: &GeomMesh) {
    mesh.validate().expect("mesh should be internally consistent");

    let _ = writeln!(out, "mesh.vertex_count {}", mesh.positions.len());
    let _ = writeln!(out, "mesh.triangle_count {}", mesh.indices.len() / 3);
    let _ = writeln!(out, "mesh.has_uvs {}", mesh.uvs.is_some());
    let _ = writeln!(out, "mesh.has_normals {}", mesh.normals.is_some());
    let _ = writeln!(out, "mesh.has_tangents {}", mesh.tangents.is_some());

    let _ = writeln!(out, "mesh.positions {}", mesh.positions.len());
    for p in mesh.positions.iter().copied() {
        write_vec3_line(out, "p", p);
    }

    let _ = writeln!(out, "mesh.indices {}", mesh.indices.len());
    for tri in mesh.indices.chunks_exact(3) {
        let _ = writeln!(out, "i {} {} {}", tri[0], tri[1], tri[2]);
    }

    if let Some(uvs) = mesh.uvs.as_ref() {
        let _ = writeln!(out, "mesh.uvs {}", uvs.len());
        for uv in uvs.iter().copied() {
            write_vec2_line(out, "uv", uv);
        }
    } else {
        let _ = writeln!(out, "mesh.uvs none");
    }

    if let Some(normals) = mesh.normals.as_ref() {
        let _ = writeln!(out, "mesh.normals {}", normals.len());
        for n in normals.iter().copied() {
            write_vec3_line(out, "n", n);
        }
    } else {
        let _ = writeln!(out, "mesh.normals none");
    }

    if let Some(tangents) = mesh.tangents.as_ref() {
        let _ = writeln!(out, "mesh.tangents {}", tangents.len());
        for t in tangents.iter().copied() {
            write_vec3_line(out, "t", t);
        }
    } else {
        let _ = writeln!(out, "mesh.tangents none");
    }
}

fn assert_or_update_fixture(name: &str, actual: &str) {
    let path = fixture_path(name);
    let actual = normalize_snapshot_text(actual);

    if std::env::var_os("GHX_UPDATE_GOLDENS").is_some() {
        std::fs::create_dir_all(fixtures_dir()).expect("create fixtures dir");
        std::fs::write(&path, actual).expect("write golden fixture");
        return;
    }

    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("missing fixture `{}`: {err}", path.display()));
    let expected = normalize_snapshot_text(&expected);

    assert_eq!(
        actual, expected,
        "golden mismatch for `{name}` (set GHX_UPDATE_GOLDENS=1 to update)"
    );
}

fn snapshot(op: &str, sections: impl FnOnce(&mut String)) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# ghx-engine golden v1");
    let _ = writeln!(out, "op {op}");
    let _ = writeln!(out, "quantize {SNAPSHOT_QUANTIZE:.1e}");
    sections(&mut out);
    out
}

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
        uvs: mesh.uvs.clone(),
        normals: mesh.normals.clone(),
        tangents: mesh.tangents.clone(),
    }
}

#[test]
fn golden_extrude_square_prism() {
    let profile = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];

    let (mesh, diag) = extrude_polyline(&profile, Vec3::new(0.0, 0.0, 1.0), ExtrusionCaps::BOTH)
        .expect("extrude should succeed");
    assert_eq!(diag.open_edge_count, 0);
    assert_eq!(diag.non_manifold_edge_count, 0);

    let snap = snapshot("extrude_square_prism", |out| {
        write_geom_mesh_diagnostics(out, &diag);
        write_mesh(out, &mesh);
    });
    assert_or_update_fixture("extrude_square_prism.snap", &snap);
}

#[test]
fn golden_loft_square_prism() {
    let profile0 = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 0.0),
    ];
    let profile1 = vec![
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 1.0),
    ];
    let profiles: Vec<&[Point3]> = vec![&profile0, &profile1];

    let options = LoftOptions {
        rebuild: true,
        rebuild_point_count: 4,
        adjust_seams: false,
        cap_start: true,
        cap_end: true,
        ..Default::default()
    };

    let (mesh, mesh_diag, loft_diag) =
        loft_mesh(&profiles, options).expect("loft should succeed");
    assert_eq!(mesh_diag.open_edge_count, 0);
    assert_eq!(mesh_diag.non_manifold_edge_count, 0);

    let snap = snapshot("loft_square_prism", |out| {
        write_geom_mesh_diagnostics(out, &mesh_diag);
        write_loft_diagnostics(out, &loft_diag);
        write_mesh(out, &mesh);
    });
    assert_or_update_fixture("loft_square_prism.snap", &snap);
}

#[test]
fn golden_sweep_square_prism() {
    let profile = vec![
        Point3::new(-0.5, -0.5, 0.0),
        Point3::new(0.5, -0.5, 0.0),
        Point3::new(0.5, 0.5, 0.0),
        Point3::new(-0.5, 0.5, 0.0),
        Point3::new(-0.5, -0.5, 0.0),
    ];
    let rail = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 2.0)];

    let tol = Tolerance::default_geom();
    let options = SweepOptions::default();
    let (mesh, diag) = sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::BOTH, options, tol)
        .expect("sweep should succeed");
    assert_eq!(diag.open_edge_count, 0);
    assert_eq!(diag.non_manifold_edge_count, 0);

    let snap = snapshot("sweep_square_prism", |out| {
        write_geom_mesh_diagnostics(out, &diag);
        write_mesh(out, &mesh);
    });
    assert_or_update_fixture("sweep_square_prism.snap", &snap);
}

#[test]
fn golden_pipe_straight() {
    let rail = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 2.0)];

    let tol = Tolerance::default_geom();
    let options = PipeOptions { radial_segments: 8 };

    let (mesh, diag) = pipe_polyline_with_tolerance(&rail, 0.5, PipeCaps::BOTH, options, tol)
        .expect("pipe should succeed");
    assert_eq!(diag.open_edge_count, 0);
    assert_eq!(diag.non_manifold_edge_count, 0);

    let snap = snapshot("pipe_straight", |out| {
        write_geom_mesh_diagnostics(out, &diag);
        write_mesh(out, &mesh);
    });
    assert_or_update_fixture("pipe_straight.snap", &snap);
}

#[test]
fn golden_revolve_wedge() {
    let profile = [
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 1.0),
        Point3::new(3.0, 0.0, 1.0),
        Point3::new(3.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);

    let options = RevolveOptions {
        min_steps: 8,
        max_steps: 8,
        weld_seam: true,
    };
    let tol = Tolerance::default_geom();

    let (mesh, diag) = revolve_polyline_with_options(
        &profile,
        axis_start,
        axis_end,
        std::f64::consts::PI,
        RevolveCaps::BOTH,
        options,
        tol,
    )
    .expect("revolve should succeed");
    assert_eq!(diag.open_edge_count, 0);
    assert_eq!(diag.non_manifold_edge_count, 0);

    let snap = snapshot("revolve_wedge", |out| {
        write_geom_mesh_diagnostics(out, &diag);
        write_mesh(out, &mesh);
    });
    assert_or_update_fixture("revolve_wedge.snap", &snap);
}

#[test]
fn golden_boolean_union() {
    let square = [
        Point3::new(-0.5, -0.5, 0.0),
        Point3::new(0.5, -0.5, 0.0),
        Point3::new(0.5, 0.5, 0.0),
        Point3::new(-0.5, 0.5, 0.0),
    ];

    let (a, _) =
        extrude_polyline(&square, Vec3::new(0.0, 0.0, 1.0), ExtrusionCaps::BOTH).expect("A");
    let (b_raw, _) =
        extrude_polyline(&square, Vec3::new(0.0, 0.0, 1.0), ExtrusionCaps::BOTH).expect("B");

    // Disjoint meshes: exercises boolean classification + merge without
    // triggering voxel fallback (keeps the golden fixture small and deterministic).
    let b = transform_mesh_z(&b_raw, 0.0, Vec3::new(2.0, 0.0, 0.0));

    let tol = Tolerance::default_geom();
    let result = boolean_meshes(&a, &b, BooleanOp::Union, tol).expect("boolean union");
    assert!(result.mesh.validate().is_ok());
    assert!(!result.diagnostics.voxel_fallback_used);
    assert!(!result.diagnostics.tolerance_relaxed);
    assert!(!result.mesh_diagnostics.boolean_fallback_used);

    let snap = snapshot("boolean_union", |out| {
        write_geom_mesh_diagnostics(out, &result.mesh_diagnostics);
        write_boolean_diagnostics(out, &result.diagnostics);
        write_mesh(out, &result.mesh);
    });
    assert_or_update_fixture("boolean_union.snap", &snap);
}

#[test]
fn golden_patch_square_with_hole() {
    use ghx_engine::geom::patch_mesh_with_tolerance;

    let outer = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(2.0, 2.0, 0.0),
        Point3::new(0.0, 2.0, 0.0),
    ];
    let hole = vec![
        Point3::new(0.75, 0.75, 0.0),
        Point3::new(1.25, 0.75, 0.0),
        Point3::new(1.25, 1.25, 0.0),
        Point3::new(0.75, 1.25, 0.0),
    ];

    let tol = Tolerance::default_geom();
    let (mesh, diag) = patch_mesh_with_tolerance(&outer, &[hole], tol).expect("patch should succeed");
    assert!(diag.open_edge_count > 0);
    assert_eq!(diag.non_manifold_edge_count, 0);

    let snap = snapshot("patch_square_with_hole", |out| {
        write_geom_mesh_diagnostics(out, &diag);
        write_mesh(out, &mesh);
    });
    assert_or_update_fixture("patch_square_with_hole.snap", &snap);
}

#[test]
fn golden_offset_quad_outside() {
    let mesh = GeomMesh {
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
    };

    let tol = Tolerance::default_geom();
    let (offset_mesh, diag) = offset_mesh_outside(&mesh, 0.25, tol).expect("offset should succeed");
    assert_eq!(diag.original_triangle_count, 2);

    let snap = snapshot("offset_quad_outside", |out| {
        write_offset_diagnostics(out, &diag);
        write_mesh(out, &offset_mesh);
    });
    assert_or_update_fixture("offset_quad_outside.snap", &snap);
}

#[test]
fn golden_deform_twist_box() {
    let mesh = GeomMesh {
        positions: vec![
            [-0.5, -0.5, 0.0],
            [0.5, -0.5, 0.0],
            [0.5, 0.5, 0.0],
            [-0.5, 0.5, 0.0],
            [-0.5, -0.5, 2.0],
            [0.5, -0.5, 2.0],
            [0.5, 0.5, 2.0],
            [-0.5, 0.5, 2.0],
        ],
        indices: vec![
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 5, 1, 0, 4, 5, 2, 7, 3, 2, 6, 7, 0,
            3, 7, 0, 7, 4, 1, 6, 2, 1, 5, 6,
        ],
        uvs: None,
        normals: None,
        tangents: None,
    };

    let tol = Tolerance::default_geom();
    let (twisted, diag) = twist_mesh_z(&mesh, std::f64::consts::FRAC_PI_2, tol).expect("twist");
    assert!(!twisted.indices.is_empty());

    let snap = snapshot("deform_twist_box", |out| {
        write_deformation_diagnostics(out, &diag);
        write_mesh(out, &twisted);
    });
    assert_or_update_fixture("deform_twist_box.snap", &snap);
}
