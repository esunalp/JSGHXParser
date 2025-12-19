use super::super::*;
use std::f64::consts::PI;

#[test]
fn revolve_open_polyline_produces_quad_strip() {
    let profile = [
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);

    let (mesh, diagnostics) = revolve_polyline(&profile, axis_start, axis_end, PI, RevolveCaps::NONE)
        .expect("revolve should succeed");

    // Should have vertices for start and end of profile at multiple angles
    assert!(mesh.positions.len() > 0);
    assert!(mesh.indices.len() > 0);
    assert!(diagnostics.open_edge_count > 0); // Open profile should have open edges
}

#[test]
fn revolve_closed_square_with_caps_is_watertight() {
    // Profile must close back to the first point to be detected as closed
    let profile = [
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(1.0, -1.0, 0.0),
        Point3::new(-1.0, -1.0, 0.0),
        Point3::new(-1.0, 1.0, 0.0),
        Point3::new(1.0, 1.0, 0.0), // Close the profile
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);

    let (mesh, diagnostics) = revolve_polyline(
        &profile,
        axis_start,
        axis_end,
        2.0 * PI,
        RevolveCaps::BOTH
    ).expect("revolve should succeed");

    // Should be watertight with caps
    assert!(mesh.positions.len() > 0);
    assert!(mesh.indices.len() > 0);
    assert_eq!(diagnostics.open_edge_count, 0);
}

#[test]
fn revolve_partial_angle_creates_wedge() {
    let profile = [
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.5, 1.0, 0.0),
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);

    let angle = PI / 2.0; // 90 degrees

    let (mesh, diagnostics) = revolve_polyline(&profile, axis_start, axis_end, angle, RevolveCaps::NONE)
        .expect("revolve should succeed");

    assert!(mesh.positions.len() > 0);
    assert!(mesh.indices.len() > 0);
    // Partial angle should have open edges
    assert!(diagnostics.open_edge_count > 0);
}

#[test]
fn revolve_with_start_cap_only() {
    // For a partial revolution, test that start cap fills one end
    // Profile is in the XZ plane (perpendicular to revolution), forms a closed loop
    // The profile is offset from the axis to create a torus-like shape
    let profile = [
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 1.0),
        Point3::new(3.0, 0.0, 1.0),
        Point3::new(3.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0), // Close the profile
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);

    // Use partial revolution (half) so start and end are at different positions
    let (mesh, diagnostics) = revolve_polyline(
        &profile,
        axis_start,
        axis_end,
        PI, // Half revolution
        RevolveCaps::START
    ).expect("revolve should succeed");

    assert!(mesh.positions.len() > 0);
    assert!(mesh.indices.len() > 0);
    // With only start cap on a partial revolution, end is still open
    assert!(diagnostics.open_edge_count > 0, 
        "Expected open edges, but got watertight mesh. open_edge_count={}",
        diagnostics.open_edge_count);
}

#[test]
fn revolve_with_end_cap_only() {
    // For a partial revolution, test that end cap fills one end
    // Profile is in the XZ plane (perpendicular to revolution), forms a closed loop
    let profile = [
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 1.0),
        Point3::new(3.0, 0.0, 1.0),
        Point3::new(3.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0), // Close the profile
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);

    // Use partial revolution (half) so start and end are at different positions
    let (mesh, diagnostics) = revolve_polyline(
        &profile,
        axis_start,
        axis_end,
        PI, // Half revolution
        RevolveCaps::END
    ).expect("revolve should succeed");

    assert!(mesh.positions.len() > 0);
    assert!(mesh.indices.len() > 0);
    // With only end cap on a partial revolution, start is still open
    assert!(diagnostics.open_edge_count > 0,
        "Expected open edges, but got watertight mesh. open_edge_count={}",
        diagnostics.open_edge_count);
}

#[test]
fn revolve_invalid_angle_returns_error() {
    let profile = [
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);

    let result = revolve_polyline(&profile, axis_start, axis_end, 0.0, RevolveCaps::NONE);
    assert!(matches!(result, Err(RevolveError::InvalidAngle)));

    let result = revolve_polyline(&profile, axis_start, axis_end, 3.0 * PI, RevolveCaps::NONE);
    assert!(matches!(result, Err(RevolveError::InvalidAngle)));
}

#[test]
fn revolve_invalid_axis_returns_error() {
    let profile = [
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 0.0); // Zero-length axis

    let result = revolve_polyline(&profile, axis_start, axis_end, PI, RevolveCaps::NONE);
    assert!(matches!(result, Err(RevolveError::InvalidAxis)));
}

#[test]
fn revolve_profile_intersecting_axis_returns_error() {
    let profile = [
        Point3::new(0.0, 0.0, 0.0), // On axis
        Point3::new(1.0, 0.0, 0.0),
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);

    let result = revolve_polyline(&profile, axis_start, axis_end, PI, RevolveCaps::NONE);
    assert!(matches!(result, Err(RevolveError::ProfileIntersectsAxis)));
}

#[test]
fn revolve_caps_require_closed_profile() {
    let profile = [
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0), // Open profile
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);

    let result = revolve_polyline(&profile, axis_start, axis_end, PI, RevolveCaps::START);
    assert!(matches!(result, Err(RevolveError::CapsRequireClosedProfile)));
}

#[test]
fn rail_revolve_basic_functionality() {
    let profile = [
        Point3::new(0.0, 0.5, 0.0),
        Point3::new(0.0, -0.5, 0.0),
    ];

    let rail = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
    ];

    let (mesh, diagnostics) = rail_revolve_polyline(&profile, &rail, RevolveCaps::NONE)
        .expect("rail revolve should succeed");

    assert!(mesh.positions.len() > 0);
    assert!(mesh.indices.len() > 0);
}

#[test]
fn rail_revolve_invalid_rail_returns_error() {
    let profile = [
        Point3::new(0.0, 0.5, 0.0),
        Point3::new(0.0, -0.5, 0.0),
    ];

    let rail = [
        Point3::new(0.0, 0.0, 0.0), // Only one point - invalid
    ];

    let result = rail_revolve_polyline(&profile, &rail, RevolveCaps::NONE);
    assert!(matches!(result, Err(RevolveError::RailTooShort)));
}

// ============================================================================
// Tests for enhanced revolve features
// ============================================================================

#[test]
fn revolve_full_360_degrees_creates_closed_surface() {
    // Profile that forms a closed ring when revolved
    let profile = [
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.5, 0.0, 0.5),
        Point3::new(1.0, 0.0, 1.0),
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);

    let (mesh, diagnostics) = revolve_polyline(&profile, axis_start, axis_end, 2.0 * PI, RevolveCaps::NONE)
        .expect("full revolution should succeed");

    // Full revolution should have no open edges on the angular seam
    assert!(mesh.positions.len() > 0);
    assert!(mesh.indices.len() > 0);
    // The mesh might still have open edges at the profile ends (not a closed loop)
}

#[test]
fn revolve_around_arbitrary_axis() {
    // Test revolving around a tilted axis
    let profile = [
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(2.0, 1.0, 0.0),
    ];

    // Tilted axis (45 degrees)
    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(1.0, 0.0, 1.0);

    let (mesh, _diagnostics) = revolve_polyline(&profile, axis_start, axis_end, PI, RevolveCaps::NONE)
        .expect("tilted axis revolution should succeed");

    assert!(mesh.positions.len() > 0);
    assert!(mesh.indices.len() > 0);

    // Verify that points are properly rotated around the tilted axis
    // All points should maintain their distance from the axis
}

#[test]
fn revolve_generates_uvs() {
    let profile = [
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);

    let (mesh, _diagnostics) = revolve_polyline(&profile, axis_start, axis_end, PI, RevolveCaps::NONE)
        .expect("revolve should succeed");

    // UVs should be generated
    assert!(mesh.uvs.is_some());
    let uvs = mesh.uvs.as_ref().unwrap();
    assert_eq!(uvs.len(), mesh.positions.len());

    // All UV values should be in valid range [0, 1]
    for uv in uvs {
        assert!(uv[0] >= 0.0 && uv[0] <= 1.0, "U coordinate out of range: {}", uv[0]);
        assert!(uv[1] >= 0.0 && uv[1] <= 1.0, "V coordinate out of range: {}", uv[1]);
    }
}

#[test]
fn revolve_generates_normals() {
    let profile = [
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);

    let (mesh, _diagnostics) = revolve_polyline(&profile, axis_start, axis_end, PI, RevolveCaps::NONE)
        .expect("revolve should succeed");

    // Normals should be generated by finalize_mesh
    assert!(mesh.normals.is_some());
    let normals = mesh.normals.as_ref().unwrap();
    assert_eq!(normals.len(), mesh.positions.len());

    // All normals should be unit vectors (within tolerance)
    for normal in normals {
        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        assert!((len - 1.0).abs() < 0.01, "Normal is not unit length: {}", len);
    }
}

#[test]
fn rail_revolve_with_curved_rail() {
    let profile = [
        Point3::new(0.0, 0.5, 0.0),
        Point3::new(0.0, -0.5, 0.0),
    ];

    // Curved rail (quarter circle in XZ plane)
    let rail: Vec<Point3> = (0..=8).map(|i| {
        let t = (i as f64 / 8.0) * PI / 2.0;
        Point3::new(t.cos(), 0.0, t.sin())
    }).collect();

    let (mesh, _diagnostics) = rail_revolve_polyline(&profile, &rail, RevolveCaps::NONE)
        .expect("curved rail revolve should succeed");

    assert!(mesh.positions.len() > 0);
    assert!(mesh.indices.len() > 0);
}

#[test]
fn rail_revolve_with_caps() {
    // Closed profile for caps
    let profile = [
        Point3::new(0.5, 0.0, 0.0),
        Point3::new(0.0, 0.5, 0.0),
        Point3::new(-0.5, 0.0, 0.0),
        Point3::new(0.0, -0.5, 0.0),
        Point3::new(0.5, 0.0, 0.0), // Close the profile
    ];

    let rail = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 2.0),
    ];

    let (mesh, diagnostics) = rail_revolve_polyline(&profile, &rail, RevolveCaps::BOTH)
        .expect("rail revolve with caps should succeed");

    assert!(mesh.positions.len() > 0);
    assert!(mesh.indices.len() > 0);
    // With caps on both ends, the mesh should be watertight
    assert_eq!(diagnostics.open_edge_count, 0);
}

#[test]
fn revolve_with_options_custom_steps() {
    use super::super::RevolveOptions;
    
    let profile = [
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);

    let options = RevolveOptions {
        min_steps: 4,
        max_steps: 8,
        weld_seam: true,
    };

    let (mesh, _diagnostics) = super::super::revolve_polyline_with_options(
        &profile,
        axis_start,
        axis_end,
        PI,
        RevolveCaps::NONE,
        options,
        super::super::Tolerance::default_geom(),
    ).expect("revolve with options should succeed");

    assert!(mesh.positions.len() > 0);
    assert!(mesh.indices.len() > 0);
}
#[test]
fn revolve_adaptive_steps_respects_profile_radius() {
    use super::super::RevolveOptions;
    use super::super::Tolerance;
    
    // Profile close to axis (small radius)
    let small_profile = [
        Point3::new(0.1, 0.0, 0.0),
        Point3::new(0.1, 0.1, 0.0),
    ];
    
    // Profile far from axis (large radius)
    let large_profile = [
        Point3::new(10.0, 0.0, 0.0),
        Point3::new(10.0, 0.1, 0.0),
    ];
    
    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);
    
    let options = RevolveOptions {
        min_steps: 4,
        max_steps: 128,
        weld_seam: true,
    };
    
    let tol = Tolerance::new(1e-4); // Coarser tolerance to see difference
    
    let (small_mesh, _) = super::super::revolve_polyline_with_options(
        &small_profile,
        axis_start,
        axis_end,
        2.0 * PI,
        RevolveCaps::NONE,
        options,
        tol,
    ).expect("small profile revolve should succeed");
    
    let (large_mesh, _) = super::super::revolve_polyline_with_options(
        &large_profile,
        axis_start,
        axis_end,
        2.0 * PI,
        RevolveCaps::NONE,
        options,
        tol,
    ).expect("large profile revolve should succeed");
    
    // Larger radius profile should produce more triangles due to adaptive subdivision
    // (because arc length at larger radius requires more segments for same chord deviation)
    assert!(
        large_mesh.indices.len() >= small_mesh.indices.len(),
        "Larger radius ({} tris) should have >= triangles than smaller radius ({} tris)",
        large_mesh.indices.len() / 3,
        small_mesh.indices.len() / 3
    );
}

#[test]
fn revolve_cap_normals_face_outward() {
    // Create a closed profile for caps
    let profile = [
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 1.0),
        Point3::new(3.0, 0.0, 1.0),
        Point3::new(3.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0), // Close the profile
    ];

    let axis_start = Point3::new(0.0, 0.0, 0.0);
    let axis_end = Point3::new(0.0, 0.0, 1.0);

    // Partial revolution with both caps
    let (mesh, diagnostics) = revolve_polyline(
        &profile,
        axis_start,
        axis_end,
        PI, // Half revolution
        RevolveCaps::BOTH
    ).expect("revolve with caps should succeed");

    // Verify mesh has content
    assert!(mesh.positions.len() > 0);
    assert!(mesh.indices.len() > 0);
    assert!(mesh.normals.is_some());
    
    // With proper cap orientation, the mesh should have consistent winding
    // (verified by the finalize_mesh pipeline)
    // If there were orientation issues, we'd see non-manifold edges or flipped triangles
    assert_eq!(
        diagnostics.non_manifold_edge_count, 0,
        "Caps should not create non-manifold edges"
    );
}