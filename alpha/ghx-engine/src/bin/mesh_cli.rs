#[cfg(target_arch = "wasm32")]
fn main() {
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "mesh_engine_next")))]
fn main() {
    eprintln!(
        "mesh_cli is a native-only tool and requires `--features mesh_engine_next`.\n\
         Example: cargo run -p ghx-engine --bin mesh_cli --features mesh_engine_next -- list"
    );
    std::process::exit(1);
}

#[cfg(all(not(target_arch = "wasm32"), feature = "mesh_engine_next"))]
fn main() {
    if let Err(err) = native::run() {
        eprintln!("mesh_cli error: {err}");
        std::process::exit(1);
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "mesh_engine_next"))]
mod native {
    use ghx_engine::geom::{
        BooleanDiagnostics, BooleanOp, DeformationDiagnostics, ExtrusionCaps, GeomMesh,
        GeomMeshDiagnostics, LoftDiagnostics, LoftOptions, OffsetDiagnostics, PipeCaps, PipeOptions,
        Point3, RevolveCaps, RevolveOptions, SweepCaps, SweepOptions, Tolerance, Vec3,
        boolean_meshes, extrude_polyline, loft_mesh, offset_mesh_outside, patch_mesh_with_tolerance,
        pipe_polyline_with_tolerance, revolve_polyline_with_options, sweep1_polyline_with_tolerance,
        twist_mesh_z,
    };
    use std::fmt::Write as _;
    use std::fs::{self, File};
    use std::io::{BufWriter, Write};
    use std::path::{Path, PathBuf};

    const SNAPSHOT_QUANTIZE: f64 = 1e-6;
    const SNAPSHOT_DECIMALS: usize = 6;

    const USAGE: &str = r#"mesh_cli (ghx-engine)

USAGE:
  mesh_cli list
  mesh_cli run <scenario|all> [options]

SCENARIOS:
  extrude_square_prism
  loft_square_prism
  sweep_square_prism
  pipe_straight
  revolve_wedge
  boolean_union
  patch_square_with_hole
  offset_quad_outside
  deform_twist_box

OPTIONS (run):
  --out-dir <dir>    Write <scenario>.obj and/or <scenario>.snap to this dir (required for `all`)
  --obj <path>       Write OBJ (single scenario only)
  --snap <path>      Write golden-style snapshot (single scenario only)
  --no-obj           Skip OBJ when using --out-dir
  --no-snap          Skip snapshot when using --out-dir
  --overwrite        Overwrite existing output files
  -h, --help         Show this help
"#;

    pub fn run() -> Result<(), String> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut args = Args::new(args);

        let Some(command) = args.next() else {
            print_usage();
            return Ok(());
        };

        match command.as_str() {
            "list" => {
                print_scenarios();
                Ok(())
            }
            "run" => cmd_run(&mut args),
            "-h" | "--help" | "help" => {
                print_usage();
                Ok(())
            }
            other => Err(format!("unknown command `{other}`\n\n{USAGE}")),
        }
    }

    fn print_usage() {
        println!("{USAGE}");
    }

    fn print_scenarios() {
        for scenario in Scenario::ALL {
            println!("{}", scenario.name());
        }
    }

    fn cmd_run(args: &mut Args) -> Result<(), String> {
        let scenario_name = args.next().ok_or("missing scenario name")?;

        let mut out_dir: Option<PathBuf> = None;
        let mut obj_path: Option<PathBuf> = None;
        let mut snap_path: Option<PathBuf> = None;
        let mut overwrite = false;
        let mut write_obj = true;
        let mut write_snap = true;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--out-dir" => out_dir = Some(PathBuf::from(args.value("--out-dir")?)),
                "--obj" => obj_path = Some(PathBuf::from(args.value("--obj")?)),
                "--snap" => snap_path = Some(PathBuf::from(args.value("--snap")?)),
                "--overwrite" => overwrite = true,
                "--no-obj" => write_obj = false,
                "--no-snap" => write_snap = false,
                "-h" | "--help" => {
                    print_usage();
                    return Ok(());
                }
                other => return Err(format!("unknown option `{other}`\n\n{USAGE}")),
            }
        }

        if let Some(dir) = out_dir.as_ref() {
            if obj_path.is_some() || snap_path.is_some() {
                return Err("use either --out-dir or --obj/--snap (not both)".to_string());
            }
            if !write_obj && !write_snap {
                return Err("nothing to write (both --no-obj and --no-snap set)".to_string());
            }

            fs::create_dir_all(dir).map_err(|e| format!("create out dir: {e}"))?;

            if scenario_name == "all" {
                for scenario in Scenario::ALL {
                    run_one_scenario_to_dir(*scenario, dir, write_obj, write_snap, overwrite)?;
                }
                return Ok(());
            }

            let scenario = Scenario::from_str(scenario_name.as_str())
                .ok_or_else(|| unknown_scenario(&scenario_name))?;
            return run_one_scenario_to_dir(scenario, dir, write_obj, write_snap, overwrite);
        }

        if scenario_name == "all" {
            return Err("`run all` requires --out-dir".to_string());
        }

        let scenario =
            Scenario::from_str(scenario_name.as_str()).ok_or_else(|| unknown_scenario(&scenario_name))?;
        let output = run_scenario(scenario)?;

        if let Some(path) = snap_path.as_deref() {
            write_text_file(path, &output.snapshot, overwrite)?;
            eprintln!("wrote {}", path.display());
        } else {
            print!("{}", output.snapshot);
        }

        if let Some(path) = obj_path.as_deref() {
            write_obj_file(path, &output.mesh, output.name, overwrite)?;
            eprintln!("wrote {}", path.display());
        }

        if let Some(mesh_diag) = output.mesh_diag.as_ref() {
            eprintln!(
                "{}: vertices={} triangles={} | {}",
                output.name,
                output.mesh.vertex_count(),
                output.mesh.triangle_count(),
                mesh_diag.summary()
            );
        } else {
            eprintln!(
                "{}: vertices={} triangles={}",
                output.name,
                output.mesh.vertex_count(),
                output.mesh.triangle_count()
            );
        }

        Ok(())
    }

    fn run_one_scenario_to_dir(
        scenario: Scenario,
        dir: &Path,
        write_obj: bool,
        write_snap: bool,
        overwrite: bool,
    ) -> Result<(), String> {
        let output = run_scenario(scenario)?;

        if write_snap {
            let path = dir.join(format!("{}.snap", output.name));
            write_text_file(&path, &output.snapshot, overwrite)?;
            eprintln!("wrote {}", path.display());
        }

        if write_obj {
            let path = dir.join(format!("{}.obj", output.name));
            write_obj_file(&path, &output.mesh, output.name, overwrite)?;
            eprintln!("wrote {}", path.display());
        }

        if let Some(mesh_diag) = output.mesh_diag.as_ref() {
            eprintln!(
                "{}: vertices={} triangles={} | {}",
                output.name,
                output.mesh.vertex_count(),
                output.mesh.triangle_count(),
                mesh_diag.summary()
            );
        } else {
            eprintln!(
                "{}: vertices={} triangles={}",
                output.name,
                output.mesh.vertex_count(),
                output.mesh.triangle_count()
            );
        }

        Ok(())
    }

    fn unknown_scenario(name: &str) -> String {
        let mut msg = String::new();
        msg.push_str(&format!("unknown scenario `{name}`\n\navailable scenarios:\n"));
        for scenario in Scenario::ALL {
            msg.push_str(&format!("  {}\n", scenario.name()));
        }
        msg
    }

    fn write_text_file(path: &Path, text: &str, overwrite: bool) -> Result<(), String> {
        if path.exists() && !overwrite {
            return Err(format!(
                "refusing to overwrite existing file {} (use --overwrite)",
                path.display()
            ));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create dir {}: {e}", parent.display()))?;
        }
        fs::write(path, normalize_snapshot_text(text)).map_err(|e| format!("write {}: {e}", path.display()))
    }

    fn write_obj_file(path: &Path, mesh: &GeomMesh, name: &str, overwrite: bool) -> Result<(), String> {
        mesh.validate().map_err(|e| format!("mesh validation failed: {e}"))?;

        if path.exists() && !overwrite {
            return Err(format!(
                "refusing to overwrite existing file {} (use --overwrite)",
                path.display()
            ));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create dir {}: {e}", parent.display()))?;
        }

        let file = File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
        let mut w = BufWriter::new(file);

        writeln!(w, "# ghx-engine mesh_cli").map_err(|e| format!("write obj: {e}"))?;
        writeln!(w, "o {name}").map_err(|e| format!("write obj: {e}"))?;

        for p in mesh.positions.iter().copied() {
            writeln!(w, "v {} {} {}", p[0], p[1], p[2]).map_err(|e| format!("write obj: {e}"))?;
        }

        if let Some(uvs) = mesh.uvs.as_ref() {
            for uv in uvs.iter().copied() {
                writeln!(w, "vt {} {}", uv[0], uv[1]).map_err(|e| format!("write obj: {e}"))?;
            }
        }

        if let Some(normals) = mesh.normals.as_ref() {
            for n in normals.iter().copied() {
                writeln!(w, "vn {} {} {}", n[0], n[1], n[2]).map_err(|e| format!("write obj: {e}"))?;
            }
        }

        let has_uvs = mesh.uvs.is_some();
        let has_normals = mesh.normals.is_some();

        for tri in mesh.indices.chunks_exact(3) {
            let a = tri[0] + 1;
            let b = tri[1] + 1;
            let c = tri[2] + 1;

            match (has_uvs, has_normals) {
                (true, true) => writeln!(w, "f {a}/{a}/{a} {b}/{b}/{b} {c}/{c}/{c}"),
                (true, false) => writeln!(w, "f {a}/{a} {b}/{b} {c}/{c}"),
                (false, true) => writeln!(w, "f {a}//{a} {b}//{b} {c}//{c}"),
                (false, false) => writeln!(w, "f {a} {b} {c}"),
            }
            .map_err(|e| format!("write obj: {e}"))?;
        }

        w.flush().map_err(|e| format!("flush {}: {e}", path.display()))
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

    fn snapshot(op: &str, sections: impl FnOnce(&mut String)) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "# ghx-engine golden v1");
        let _ = writeln!(out, "op {op}");
        let _ = writeln!(out, "quantize {SNAPSHOT_QUANTIZE:.1e}");
        sections(&mut out);
        normalize_snapshot_text(&out)
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Scenario {
        ExtrudeSquarePrism,
        LoftSquarePrism,
        SweepSquarePrism,
        PipeStraight,
        RevolveWedge,
        BooleanUnion,
        PatchSquareWithHole,
        OffsetQuadOutside,
        DeformTwistBox,
    }

    impl Scenario {
        const ALL: &'static [Scenario] = &[
            Scenario::ExtrudeSquarePrism,
            Scenario::LoftSquarePrism,
            Scenario::SweepSquarePrism,
            Scenario::PipeStraight,
            Scenario::RevolveWedge,
            Scenario::BooleanUnion,
            Scenario::PatchSquareWithHole,
            Scenario::OffsetQuadOutside,
            Scenario::DeformTwistBox,
        ];

        fn name(self) -> &'static str {
            match self {
                Scenario::ExtrudeSquarePrism => "extrude_square_prism",
                Scenario::LoftSquarePrism => "loft_square_prism",
                Scenario::SweepSquarePrism => "sweep_square_prism",
                Scenario::PipeStraight => "pipe_straight",
                Scenario::RevolveWedge => "revolve_wedge",
                Scenario::BooleanUnion => "boolean_union",
                Scenario::PatchSquareWithHole => "patch_square_with_hole",
                Scenario::OffsetQuadOutside => "offset_quad_outside",
                Scenario::DeformTwistBox => "deform_twist_box",
            }
        }

        fn from_str(name: &str) -> Option<Self> {
            match name {
                "extrude_square_prism" => Some(Scenario::ExtrudeSquarePrism),
                "loft_square_prism" => Some(Scenario::LoftSquarePrism),
                "sweep_square_prism" => Some(Scenario::SweepSquarePrism),
                "pipe_straight" => Some(Scenario::PipeStraight),
                "revolve_wedge" => Some(Scenario::RevolveWedge),
                "boolean_union" => Some(Scenario::BooleanUnion),
                "patch_square_with_hole" => Some(Scenario::PatchSquareWithHole),
                "offset_quad_outside" => Some(Scenario::OffsetQuadOutside),
                "deform_twist_box" => Some(Scenario::DeformTwistBox),
                _ => None,
            }
        }
    }

    struct ScenarioOutput {
        name: &'static str,
        mesh: GeomMesh,
        mesh_diag: Option<GeomMeshDiagnostics>,
        snapshot: String,
    }

    fn run_scenario(scenario: Scenario) -> Result<ScenarioOutput, String> {
        match scenario {
            Scenario::ExtrudeSquarePrism => scenario_extrude_square_prism(),
            Scenario::LoftSquarePrism => scenario_loft_square_prism(),
            Scenario::SweepSquarePrism => scenario_sweep_square_prism(),
            Scenario::PipeStraight => scenario_pipe_straight(),
            Scenario::RevolveWedge => scenario_revolve_wedge(),
            Scenario::BooleanUnion => scenario_boolean_union(),
            Scenario::PatchSquareWithHole => scenario_patch_square_with_hole(),
            Scenario::OffsetQuadOutside => scenario_offset_quad_outside(),
            Scenario::DeformTwistBox => scenario_deform_twist_box(),
        }
    }

    fn scenario_extrude_square_prism() -> Result<ScenarioOutput, String> {
        let profile = [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];

        let (mesh, diag) = extrude_polyline(&profile, Vec3::new(0.0, 0.0, 1.0), ExtrusionCaps::BOTH)
            .map_err(|e| e.to_string())?;

        let snap = snapshot("extrude_square_prism", |out| {
            write_geom_mesh_diagnostics(out, &diag);
            write_mesh(out, &mesh);
        });

        Ok(ScenarioOutput {
            name: "extrude_square_prism",
            mesh,
            mesh_diag: Some(diag),
            snapshot: snap,
        })
    }

    fn scenario_loft_square_prism() -> Result<ScenarioOutput, String> {
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

        let (mesh, mesh_diag, loft_diag) = loft_mesh(&profiles, options).map_err(|e| e.to_string())?;

        let snap = snapshot("loft_square_prism", |out| {
            write_geom_mesh_diagnostics(out, &mesh_diag);
            write_loft_diagnostics(out, &loft_diag);
            write_mesh(out, &mesh);
        });

        Ok(ScenarioOutput {
            name: "loft_square_prism",
            mesh,
            mesh_diag: Some(mesh_diag),
            snapshot: snap,
        })
    }

    fn scenario_sweep_square_prism() -> Result<ScenarioOutput, String> {
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
        let (mesh, diag) =
            sweep1_polyline_with_tolerance(&profile, &rail, SweepCaps::BOTH, options, tol)
                .map_err(|e| e.to_string())?;

        let snap = snapshot("sweep_square_prism", |out| {
            write_geom_mesh_diagnostics(out, &diag);
            write_mesh(out, &mesh);
        });

        Ok(ScenarioOutput {
            name: "sweep_square_prism",
            mesh,
            mesh_diag: Some(diag),
            snapshot: snap,
        })
    }

    fn scenario_pipe_straight() -> Result<ScenarioOutput, String> {
        let rail = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 2.0)];

        let tol = Tolerance::default_geom();
        let options = PipeOptions { radial_segments: 8 };
        let (mesh, diag) =
            pipe_polyline_with_tolerance(&rail, 0.5, PipeCaps::BOTH, options, tol)
                .map_err(|e| e.to_string())?;

        let snap = snapshot("pipe_straight", |out| {
            write_geom_mesh_diagnostics(out, &diag);
            write_mesh(out, &mesh);
        });

        Ok(ScenarioOutput {
            name: "pipe_straight",
            mesh,
            mesh_diag: Some(diag),
            snapshot: snap,
        })
    }

    fn scenario_revolve_wedge() -> Result<ScenarioOutput, String> {
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
        .map_err(|e| e.to_string())?;

        let snap = snapshot("revolve_wedge", |out| {
            write_geom_mesh_diagnostics(out, &diag);
            write_mesh(out, &mesh);
        });

        Ok(ScenarioOutput {
            name: "revolve_wedge",
            mesh,
            mesh_diag: Some(diag),
            snapshot: snap,
        })
    }

    fn scenario_boolean_union() -> Result<ScenarioOutput, String> {
        let square = [
            Point3::new(-0.5, -0.5, 0.0),
            Point3::new(0.5, -0.5, 0.0),
            Point3::new(0.5, 0.5, 0.0),
            Point3::new(-0.5, 0.5, 0.0),
        ];

        let (a, _) = extrude_polyline(&square, Vec3::new(0.0, 0.0, 1.0), ExtrusionCaps::BOTH)
            .map_err(|e| e.to_string())?;
        let (b_raw, _) = extrude_polyline(&square, Vec3::new(0.0, 0.0, 1.0), ExtrusionCaps::BOTH)
            .map_err(|e| e.to_string())?;
        let b = transform_mesh_z(&b_raw, 0.0, Vec3::new(2.0, 0.0, 0.0));

        let tol = Tolerance::default_geom();
        let result = boolean_meshes(&a, &b, BooleanOp::Union, tol).map_err(|e| e.to_string())?;
        result.mesh.validate().map_err(|e| format!("mesh validation failed: {e}"))?;

        let snap = snapshot("boolean_union", |out| {
            write_geom_mesh_diagnostics(out, &result.mesh_diagnostics);
            write_boolean_diagnostics(out, &result.diagnostics);
            write_mesh(out, &result.mesh);
        });

        Ok(ScenarioOutput {
            name: "boolean_union",
            mesh: result.mesh,
            mesh_diag: Some(result.mesh_diagnostics),
            snapshot: snap,
        })
    }

    fn scenario_patch_square_with_hole() -> Result<ScenarioOutput, String> {
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
        let (mesh, diag) = patch_mesh_with_tolerance(&outer, &[hole], tol).map_err(|e| e.to_string())?;

        let snap = snapshot("patch_square_with_hole", |out| {
            write_geom_mesh_diagnostics(out, &diag);
            write_mesh(out, &mesh);
        });

        Ok(ScenarioOutput {
            name: "patch_square_with_hole",
            mesh,
            mesh_diag: Some(diag),
            snapshot: snap,
        })
    }

    fn scenario_offset_quad_outside() -> Result<ScenarioOutput, String> {
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
        let (offset_mesh, diag) = offset_mesh_outside(&mesh, 0.25, tol).map_err(|e| e.to_string())?;

        let snap = snapshot("offset_quad_outside", |out| {
            write_offset_diagnostics(out, &diag);
            write_mesh(out, &offset_mesh);
        });

        Ok(ScenarioOutput {
            name: "offset_quad_outside",
            mesh: offset_mesh,
            mesh_diag: None,
            snapshot: snap,
        })
    }

    fn scenario_deform_twist_box() -> Result<ScenarioOutput, String> {
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
                0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 5, 1, 0, 4, 5, 2, 7, 3, 2, 6, 7,
                0, 3, 7, 0, 7, 4, 1, 6, 2, 1, 5, 6,
            ],
            uvs: None,
            normals: None,
            tangents: None,
        };

        let tol = Tolerance::default_geom();
        let (twisted, diag) = twist_mesh_z(&mesh, std::f64::consts::FRAC_PI_2, tol).map_err(|e| e.to_string())?;

        let snap = snapshot("deform_twist_box", |out| {
            write_deformation_diagnostics(out, &diag);
            write_mesh(out, &twisted);
        });

        Ok(ScenarioOutput {
            name: "deform_twist_box",
            mesh: twisted,
            mesh_diag: None,
            snapshot: snap,
        })
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
                    x * c - y * s + translate.x,
                    x * s + y * c + translate.y,
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

    struct Args {
        args: Vec<String>,
        pos: usize,
    }

    impl Args {
        fn new(args: Vec<String>) -> Self {
            Self { args, pos: 0 }
        }

        fn next(&mut self) -> Option<String> {
            let arg = self.args.get(self.pos)?.clone();
            self.pos += 1;
            Some(arg)
        }

        fn value(&mut self, flag: &str) -> Result<String, String> {
            self.next()
                .ok_or_else(|| format!("missing value for {flag}"))
        }
    }
}
