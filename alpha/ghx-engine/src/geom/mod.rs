mod analysis;
mod boolean;
mod bvh;
mod cache;
mod core;
mod curve;
mod curve_ops;
mod deformation;
mod diagnostics;
mod displacement;
mod extrusion;
mod fillet_chamfer;
mod loft;
mod metrics;
mod mesh;
mod offset;
mod patch;
mod pipe;
mod revolve;
mod simplify;
mod solid;
mod subdivision;
mod surface_fit;
mod sweep;
mod surface;
mod tessellation;
mod trim;
mod triangulation;

pub use analysis::{
    ClosedEdgesResult, EdgesFromDirectionsResult, EdgesFromPointsResult, LegacyBrepData,
    LegacyBrepEdge, LegacyBrepFace, SurfaceFrame, SurfaceFramesResult, closed_edges,
    edges_by_length, edges_from_directions, edges_from_faces, edges_from_points, surface_frames,
};
pub use boolean::{
    BooleanDiagnostics, BooleanError, BooleanOp, BooleanResult,
    PointContainment, Segment3, TaggedTriangle, Triangle3, TriangleContainment, TriangleSource,
    TriTriIntersection,
    boolean_meshes, classify_mesh_triangles, classify_point_in_mesh, tag_mesh_triangles,
    triangle_mesh_intersection_segments, triangle_triangle_intersection,
};
pub use cache::{GeomCache, GeomCacheStats};
pub use core::{BBox, Point3, Tolerance, Transform, Vec3};
pub use curve::{
    Arc3, Circle3, CubicBezier3, Curve3, Ellipse3, Line3, NurbsCurve3, Polyline3,
    QuadraticBezier3, tessellate_curve_uniform,
    // Curve division and sampling utilities
    CurveDivisionResult, CurveFrame, CurveSample, SubCurve,
    curve_arc_length, curve_frames, curve_plane_intersections,
    divide_curve_by_count, divide_curve_by_distance,
    extract_subcurve, frenet_frame_at, frenet_frames,
    horizontal_frame_at, horizontal_frames,
    parallel_frame_at, perp_frames,
    sample_curve_at, shatter_curve,
    // Curve analysis utilities
    CurvatureAnalysis, SegmentLengthAnalysis,
    analyze_curvature_at, analyze_polyline_segments,
    curve_angle_at, curve_curvature_center_at, curve_curvature_vector_at,
    curve_length_at, curve_parameter_at_length,
    curve_third_derivative_at, curve_torsion_at,
};
pub use curve_ops::{
    // Offset polyline
    OffsetPolylineOptions, OffsetPolylineError, OffsetPolylineDiagnostics,
    offset_polyline,
    // Join polylines
    JoinPolylinesOptions, JoinPolylinesDiagnostics,
    join_polylines,
    // Flip polyline
    FlipPolylineOptions, FlipPolylineDiagnostics,
    flip_polyline, flip_polyline_simple,
    // Extend polyline
    ExtendPolylineOptions, ExtendPolylineDiagnostics,
    extend_polyline,
    // Smooth polyline
    SmoothPolylineOptions, SmoothPolylineDiagnostics,
    smooth_polyline,
    // Simplify polyline (RDP)
    SimplifyPolylineOptions, SimplifyPolylineDiagnostics,
    simplify_polyline,
    // Resample polyline
    ResamplePolylineOptions, ResamplePolylineDiagnostics,
    resample_polyline,
    // Remesh polyline
    RemeshPolylineOptions, RemeshPolylineDiagnostics,
    remesh_polyline,
    // Collapse polyline
    CollapsePolylineOptions, CollapsePolylineDiagnostics,
    collapse_polyline,
    // Rotate seam
    RotateSeamOptions, RotateSeamDiagnostics,
    rotate_polyline_seam,
    // Project polyline
    ProjectPolylineOptions, ProjectPolylineDiagnostics,
    project_polyline,
    // Sample polyline
    PolylineSample, sample_polyline_at,
    // Fillet at parameter
    FilletAtParameterOptions, FilletAtParameterDiagnostics,
    fillet_polyline_at_parameter,
    // Perpendicular frames
    PolylineFrame, PerpFramesOptions, PerpFramesDiagnostics,
    compute_perp_frames,
};
pub use diagnostics::GeomMeshDiagnostics;
pub use deformation::{
    BendOptions, DeformationDiagnostics, DeformationError, MorphOptions, TaperOptions,
    TwistOptions, bend_mesh, bend_mesh_z, morph_mesh, taper_mesh, taper_mesh_z,
    twist_mesh, twist_mesh_z,
};
pub use displacement::{
    DisplacementDiagnostics, DisplacementError, DisplacementOptions, DisplacementSource,
    displace_mesh, displace_mesh_heightfield, displace_mesh_noise,
    displace_mesh_per_vertex, displace_mesh_uniform,
};
pub use extrusion::{
    ExtrusionCaps, ExtrusionError, extrude_angled_polyline, extrude_angled_polyline_with_tolerance,
    extrude_polyline, extrude_polyline_with_tolerance, extrude_to_point,
    extrude_to_point_with_tolerance,
};
pub use fillet_chamfer::{
    FilletChamferError, FilletEdgeOptions, FilletMeshEdgeDiagnostics, FilletPolylineDiagnostics,
    TriangleMeshEdge, fillet_legacy_triangle_mesh_edges, fillet_polyline_points,
    fillet_triangle_mesh_edges, list_triangle_mesh_edges,
};
pub use loft::{
    LoftDiagnostics, LoftError, LoftOptions, LoftType, MeshQuality,
    control_point_loft_mesh, fit_loft_mesh, loft_mesh, loft_mesh_with_context,
    loft_mesh_with_tolerance,
};
pub use revolve::{
    FrenetFrame, RevolveCaps, RevolveError, RevolveOptions,
    rail_revolve_polyline, rail_revolve_polyline_with_tolerance,
    revolve_polyline, revolve_polyline_with_options, revolve_polyline_with_tolerance,
};
pub use sweep::{
    SweepCaps, SweepError, SweepOptions,
    sweep1_polyline, sweep1_polyline_with_tolerance,
    sweep2_polyline, sweep2_polyline_with_tolerance,
};
pub use metrics::{GeomMetrics, GeomTimingReport, TimingBucket};
pub use mesh::{
    GeomContext, GeomMesh, SurfaceBuilderQuality,
    mesh_surface, mesh_surface_adaptive, mesh_surface_adaptive_with_context,
    mesh_surface_with_context,
    mesh_four_point_surface, mesh_four_point_surface_from_points,
    mesh_ruled_surface, mesh_edge_surface, mesh_edge_surface_from_edges,
    mesh_sum_surface, mesh_network_surface, mesh_network_surface_from_grid,
    weld_mesh_vertices,
};
pub use patch::{
    PatchError, boundary_surface_mesh, boundary_surface_mesh_with_tolerance,
    fragment_patch_meshes, fragment_patch_meshes_with_tolerance,
    patch_mesh, patch_mesh_with_tolerance,
};
pub use offset::{
    OffsetDiagnostics, OffsetDirection, OffsetError, OffsetOptions,
    offset_mesh, offset_mesh_inside, offset_mesh_outside, thicken_mesh,
};
pub use pipe::{
    PipeCaps, PipeError, PipeOptions,
    pipe_polyline, pipe_polyline_with_tolerance,
    pipe_variable_polyline, pipe_variable_polyline_with_tolerance,
};
pub use surface::{
    ConeSurface, CylinderSurface, NurbsSurface, PlaneSurface, SphereSurface, Surface,
    SurfaceCacheKey, TorusSurface, tessellate_surface_grid,
    ClosedSurfaceSampling, DivideSurfaceOptions, DivideSurfaceResult, divide_surface,
    IsotrimDiagnostics, IsotrimSurface, isotrim_surface,
    FlippedSurface, SurfaceFlipDiagnostics, SurfaceFlipGuide, flip_surface_orientation,
    // Surface builders
    FourPointSurface, RuledSurface, EdgeSurface, SumSurface, NetworkSurface,
};
pub use surface_fit::{
    SurfaceFitDiagnostics, SurfaceFitError, SurfaceFitOptions,
    mesh_from_grid, mesh_from_grid_with_context,
    mesh_from_scattered_points, mesh_from_scattered_points_with_context,
    surface_from_grid, surface_from_scattered_points,
};
pub use tessellation::{
    CurveTessellationOptions, SurfaceTessellationOptions, choose_surface_grid_counts,
    tessellate_curve_adaptive_points,
};
pub use triangulation::{
    TriangulationDiagnostics, TriangulationOptions, TriangulationResult, triangulate_grid,
    triangulate_grid_wrapped, triangulate_trim_region, triangulate_trim_region_with_options,
};
pub use trim::{
    TrimDiagnostics, TrimError, TrimLoop, TrimRegion, UvDomain, UvPoint,
    copy_trim_bounds, copy_trim_loops, copy_trim_region,
    retrim_bounds, retrim_loops,
    trim_loop_from_curve_uv,
    untrim_bounds, untrim_to_domain,
};
pub use subdivision::{
    EdgeTag, SubdDiagnostics, SubdEdge, SubdError, SubdFace, SubdMesh, SubdOptions, SubdVertex,
    VertexTag,
};
pub use simplify::{
    SimplifyDiagnostics, SimplifyError, SimplifyOptions, SimplifyResult, SimplifyTarget,
    simplify_by_ratio, simplify_mesh, simplify_mesh_with_tolerance, simplify_to_count,
};
pub use solid::{
    BrepJoinDiagnostics, BrepJoinResult, CapHolesDiagnostics, CapHolesExOptions, CapHolesResult,
    LegacySurfaceMesh, MergeFacesDiagnostics, MergeFacesResult,
    brep_join_legacy, cap_holes_ex_legacy, cap_holes_legacy, legacy_surface_is_closed,
    merge_faces_legacy,
};

#[cfg(test)]
mod tests;
