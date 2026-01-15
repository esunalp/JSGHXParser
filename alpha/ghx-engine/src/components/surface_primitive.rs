//! Implementaties van Grasshopper "Surface → Primitive" componenten.
//!
//! This module uses `geom::surface` primitives (CylinderSurface, ConeSurface,
//! SphereSurface, PlaneSurface) and the shared mesh pipeline when the
//! `mesh_engine_next` feature is enabled. Components remain thin wrappers that
//! coerce inputs, build geom surfaces, mesh them, and return outputs.

use std::collections::BTreeMap;
use std::f64::consts::TAU;

use crate::graph::node::MetaMap;
use crate::graph::value::{Domain, MeshDiagnostics, Value};

use super::{Component, ComponentError, ComponentResult};

// Import geom types for surface primitives and meshing
use crate::geom::{
    CylinderSurface as GeomCylinderSurface,
    ConeSurface as GeomConeSurface,
    SphereSurface as GeomSphereSurface,
    PlaneSurface as GeomPlaneSurface,
    Point3 as GeomPoint3,
    Vec3 as GeomVec3,
    GeomMesh, GeomMeshDiagnostics,
    mesh_surface,
    // Cube-sphere (QuadSphere) mesh generation
    CubeSphereOptions, mesh_cube_sphere,
};

const PIN_OUTPUT_CYLINDER: &str = "C";
const PIN_OUTPUT_CONE: &str = "C";
const PIN_OUTPUT_TIP: &str = "T";
const PIN_OUTPUT_WORLD_BOXES: &str = "B";
const PIN_OUTPUT_PLANE_BOXES: &str = "Plane";
const PIN_OUTPUT_BOX: &str = "B";
const PIN_OUTPUT_SURFACE: &str = "S";
const PIN_OUTPUT_PLANE: &str = "P";
const PIN_OUTPUT_CENTER: &str = "C";
const PIN_OUTPUT_RADIUS: &str = "R";

// Append-only mesh output pin (consistent with surface_freeform.rs pattern)
// Provides Value::Mesh with normals, UVs, and diagnostics alongside legacy Value::Surface
const PIN_OUTPUT_MESH: &str = "M";

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Cylinder,
    Cone,
    ConeObsolete,
    BoundingBoxPlane,
    CenterBox,
    BoxTwoPoint,
    QuadSphere,
    PlaneSurface,
    BoundingBoxUnion,
    DomainBox,
    BoundingBoxPlaneUnion,
    BoxTwoPointObsolete,
    BoundingBoxLegacy,
    SphereFourPoint,
    BoxRectangle,
    PlaneThroughShape,
    Sphere,
    SphereFit,
    PlaneThroughBox,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de surface-primitive componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{0373008a-80ee-45be-887d-ab5a244afc29}"],
        names: &["Cylinder", "Cyl"],
        kind: ComponentKind::Cylinder,
    },
    Registration {
        guids: &["{03e331ed-c4d1-4a23-afa2-f57b87d2043c}"],
        names: &["Cone"],
        kind: ComponentKind::Cone,
    },
    Registration {
        guids: &["{22e61c07-c02f-4c53-b567-c821a164fd92}"],
        names: &["Cone [OBSOLETE]"],
        kind: ComponentKind::ConeObsolete,
    },
    Registration {
        guids: &["{0bb3d234-9097-45db-9998-621639c87d3b}"],
        names: &["Bounding Box"],
        kind: ComponentKind::BoundingBoxPlane,
    },
    Registration {
        guids: &["{28061aae-04fb-4cb5-ac45-16f3b66bc0a4}"],
        names: &["Center Box", "CntrBx"],
        kind: ComponentKind::CenterBox,
    },
    Registration {
        guids: &["{2a43ef96-8f87-4892-8b94-237a47e8d3cf}"],
        names: &["Box 2Pt", "BBox 2Pt"],
        kind: ComponentKind::BoxTwoPoint,
    },
    Registration {
        guids: &["{361790d6-9d66-4808-8c5a-8de9c218c227}"],
        names: &["Quad Sphere", "QSph"],
        kind: ComponentKind::QuadSphere,
    },
    Registration {
        guids: &["{439a55a5-2f9e-4f66-9de2-32f24fec2ef5}"],
        names: &["Plane Surface", "Pln"],
        kind: ComponentKind::PlaneSurface,
    },
    Registration {
        guids: &["{6aa8da2e-6f25-4585-8b37-aa44609beb46}"],
        names: &["Bounding Box"],
        kind: ComponentKind::BoundingBoxUnion,
    },
    Registration {
        guids: &["{79aa7f47-397c-4d3f-9761-aaf421bb7f5f}"],
        names: &["Domain Box", "DomBox"],
        kind: ComponentKind::DomainBox,
    },
    Registration {
        guids: &["{87df35c8-6e1d-4e2a-821a-7c1066714409}"],
        names: &["Bounding Box"],
        kind: ComponentKind::BoundingBoxPlaneUnion,
    },
    Registration {
        guids: &["{9aef6eb4-98c3-4b0e-b875-1a7cb1bb1038}"],
        names: &["Box 2Pt [OBSOLETE]"],
        kind: ComponentKind::BoxTwoPointObsolete,
    },
    Registration {
        guids: &["{9d375779-649d-49f1-baaf-04560a51cd3d}"],
        names: &["Bounding Box [OBSOLETE]"],
        kind: ComponentKind::BoundingBoxLegacy,
    },
    Registration {
        guids: &["{b083c06d-9a71-4f40-b354-1d80bba1e858}"],
        names: &["Sphere 4Pt", "Sph4Pt"],
        kind: ComponentKind::SphereFourPoint,
    },
    Registration {
        guids: &["{d0a56c9e-2483-45e7-ab98-a450b97f1bc0}"],
        names: &["Box Rectangle", "BBox Rect"],
        kind: ComponentKind::BoxRectangle,
    },
    Registration {
        guids: &["{d8698126-0e91-4ae7-ba05-2490258573ea}"],
        names: &["Plane Through Shape", "PxS"],
        kind: ComponentKind::PlaneThroughShape,
    },
    Registration {
        guids: &["{dabc854d-f50e-408a-b001-d043c7de151d}"],
        names: &["Sphere", "Sph"],
        kind: ComponentKind::Sphere,
    },
    Registration {
        guids: &["{e7ffb3af-2d77-4804-a260-755308bf8285}"],
        names: &["Sphere Fit", "SFit"],
        kind: ComponentKind::SphereFit,
    },
    Registration {
        guids: &["{f565fd67-5a98-4b48-9ea9-2e184a9ef0e6}"],
        names: &["Plane Through Box", "PxB"],
        kind: ComponentKind::PlaneThroughBox,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Cylinder => evaluate_cylinder(inputs),
            Self::Cone => evaluate_cone(inputs, true),
            Self::ConeObsolete => evaluate_cone(inputs, false),
            Self::BoundingBoxPlane => {
                evaluate_bounding_box(inputs, BoundingBoxMode::PerItemWithPlane)
            }
            Self::CenterBox => evaluate_center_box(inputs),
            Self::BoxTwoPoint => evaluate_box_two_point(inputs),
            Self::QuadSphere => evaluate_sphere(inputs, SphereMode::QuadSphere),
            Self::PlaneSurface => evaluate_plane_surface(inputs),
            Self::BoundingBoxUnion => evaluate_bounding_box(inputs, BoundingBoxMode::WorldUnion),
            Self::DomainBox => evaluate_domain_box(inputs),
            Self::BoundingBoxPlaneUnion => {
                evaluate_bounding_box(inputs, BoundingBoxMode::PlaneUnion)
            }
            Self::BoxTwoPointObsolete => evaluate_box_two_point_legacy(inputs),
            Self::BoundingBoxLegacy => evaluate_bounding_box(inputs, BoundingBoxMode::LegacyWorld),
            Self::SphereFourPoint => evaluate_sphere_from_points(inputs, SphereInput::FourPoints),
            Self::BoxRectangle => evaluate_box_rectangle(inputs),
            Self::PlaneThroughShape => {
                evaluate_plane_through_collection(inputs, ShapeInput::General)
            }
            Self::Sphere => evaluate_sphere(inputs, SphereMode::Standard),
            Self::SphereFit => evaluate_sphere_from_points(inputs, SphereInput::FitCollection),
            Self::PlaneThroughBox => evaluate_plane_through_collection(inputs, ShapeInput::Box),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Cylinder => "Cylinder",
            Self::Cone => "Cone",
            Self::ConeObsolete => "Cone [OBSOLETE]",
            Self::BoundingBoxPlane => "Bounding Box",
            Self::CenterBox => "Center Box",
            Self::BoxTwoPoint => "Box 2Pt",
            Self::QuadSphere => "Quad Sphere",
            Self::PlaneSurface => "Plane Surface",
            Self::BoundingBoxUnion => "Bounding Box",
            Self::DomainBox => "Domain Box",
            Self::BoundingBoxPlaneUnion => "Bounding Box",
            Self::BoxTwoPointObsolete => "Box 2Pt [OBSOLETE]",
            Self::BoundingBoxLegacy => "Bounding Box [OBSOLETE]",
            Self::SphereFourPoint => "Sphere 4Pt",
            Self::BoxRectangle => "Box Rectangle",
            Self::PlaneThroughShape => "Plane Through Shape",
            Self::Sphere => "Sphere",
            Self::SphereFit => "Sphere Fit",
            Self::PlaneThroughBox => "Plane Through Box",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum BoundingBoxMode {
    PerItemWithPlane,
    WorldUnion,
    PlaneUnion,
    LegacyWorld,
}

#[derive(Debug, Clone, Copy)]
enum SphereMode {
    Standard,
    QuadSphere,
}

#[derive(Debug, Clone, Copy)]
enum SphereInput {
    FourPoints,
    FitCollection,
}

#[derive(Debug, Clone, Copy)]
enum ShapeInput {
    General,
    Box,
}

fn evaluate_cylinder(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Cylinder component vereist een vlak, straal en hoogte",
        ));
    }

    let plane = coerce_plane(inputs.get(0), "Cylinder")?;
    let radius = coerce_positive_number(inputs.get(1), "Cylinder straal")?;
    let height = coerce_number(inputs.get(2), "Cylinder hoogte")?;
    if height.abs() < EPSILON {
        return Err(ComponentError::new(
            "Cylinder component vereist een niet-nul hoogte",
        ));
    }

    let dual_output = create_cylinder_surface(&plane, radius, height);

    let mut outputs = BTreeMap::new();
    // Legacy surface on existing "C" pin for backward compatibility
    // New mesh on append-only "M" pin with normals, UVs, and diagnostics
    dual_output.insert_into(&mut outputs, PIN_OUTPUT_CYLINDER);
    Ok(outputs)
}

fn evaluate_cone(inputs: &[Value], include_tip: bool) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Cone component vereist een vlak, straal en hoogte",
        ));
    }

    let plane = coerce_plane(inputs.get(0), "Cone")?;
    let radius = coerce_positive_number(inputs.get(1), "Cone straal")?;
    let height = coerce_number(inputs.get(2), "Cone hoogte")?;
    if height.abs() < EPSILON {
        return Err(ComponentError::new(
            "Cone component vereist een niet-nul hoogte",
        ));
    }

    let (dual_output, tip) = create_cone_surface(&plane, radius, height);

    let mut outputs = BTreeMap::new();
    // Legacy surface on existing "C" pin for backward compatibility
    // New mesh on append-only "M" pin with normals, UVs, and diagnostics
    dual_output.insert_into(&mut outputs, PIN_OUTPUT_CONE);
    if include_tip {
        outputs.insert(PIN_OUTPUT_TIP.to_owned(), Value::Point(tip));
    }
    Ok(outputs)
}

fn evaluate_center_box(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Center Box vereist een vlak en drie afmetingen",
        ));
    }

    let plane = coerce_plane(inputs.get(0), "Center Box")?;
    let size_x = coerce_number(inputs.get(1), "Center Box X")?.abs();
    let size_y = coerce_number(inputs.get(2), "Center Box Y")?.abs();
    let size_z = coerce_number(inputs.get(3), "Center Box Z")?;

    let half_x = size_x / 2.0;
    let half_y = size_y / 2.0;
    let half_z = size_z.abs() / 2.0;
    let (min_z, max_z) = if size_z >= 0.0 {
        (-half_z, half_z)
    } else {
        (half_z, -half_z)
    };

    let box_value = create_oriented_box(&plane, [-half_x, -half_y, min_z], [half_x, half_y, max_z]);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BOX.to_owned(), box_value);
    Ok(outputs)
}

fn evaluate_box_two_point(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Box 2Pt vereist twee punten"));
    }

    let plane = coerce_plane(inputs.get(2), "Box 2Pt")?;
    let point_a = coerce_point(inputs.get(0), "Box 2Pt A")?;
    let point_b = coerce_point(inputs.get(1), "Box 2Pt B")?;

    let coords_a = plane.coordinates(point_a);
    let coords_b = plane.coordinates(point_b);

    let min = [
        coords_a[0].min(coords_b[0]),
        coords_a[1].min(coords_b[1]),
        coords_a[2].min(coords_b[2]),
    ];
    let max = [
        coords_a[0].max(coords_b[0]),
        coords_a[1].max(coords_b[1]),
        coords_a[2].max(coords_b[2]),
    ];

    let box_value = create_oriented_box(&plane, min, max);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BOX.to_owned(), box_value);
    Ok(outputs)
}

fn evaluate_box_two_point_legacy(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Box 2Pt [OBSOLETE] vereist twee punten",
        ));
    }

    let point_a = coerce_point(inputs.get(0), "Box 2Pt [OBSOLETE] A")?;
    let point_b = coerce_point(inputs.get(1), "Box 2Pt [OBSOLETE] B")?;

    let box_value = create_axis_aligned_box(&[point_a, point_b]);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BOX.to_owned(), box_value);
    Ok(outputs)
}

fn evaluate_box_rectangle(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Box Rectangle vereist een rechthoek"));
    }

    let rectangle_points = collect_points(inputs.get(0), "Box Rectangle rechthoek")?;
    if rectangle_points.len() < 2 {
        return Err(ComponentError::new(
            "Box Rectangle kon onvoldoende punten uit de rechthoek lezen",
        ));
    }
    let height = coerce_number(inputs.get(1), "Box Rectangle hoogte")?;

    let plane = if rectangle_points.len() >= 3 {
        Plane::from_points(
            rectangle_points[0],
            rectangle_points[1],
            rectangle_points[2],
        )
    } else {
        Plane::default()
    };

    let mut uvs = Vec::with_capacity(rectangle_points.len());
    for point in rectangle_points {
        let coords = plane.coordinates(point);
        uvs.push([coords[0], coords[1]]);
    }

    // Remove duplicate closing point while preserving input order.
    if uvs.len() > 1 {
        let first = uvs.first().unwrap();
        let last = uvs.last().unwrap();
        if (first[0] - last[0]).abs() <= EPSILON && (first[1] - last[1]).abs() <= EPSILON {
            uvs.pop();
        }
    }

    let mut profile_loop = Vec::new();
    for uv in uvs {
        if profile_loop.last().map_or(false, |last: &[f64; 2]| {
            (last[0] - uv[0]).abs() <= EPSILON && (last[1] - uv[1]).abs() <= EPSILON
        }) {
            continue;
        }
        profile_loop.push(uv);
    }

    if profile_loop.len() < 3 {
        return Err(ComponentError::new(
            "Box Rectangle kon de rechthoek niet projecteren",
        ));
    }

    // Determine extrusion direction from polygon winding so flipping the input curve flips the box.
    let mut signed_area = 0.0;
    for i in 0..profile_loop.len() {
        let j = (i + 1) % profile_loop.len();
        signed_area += profile_loop[i][0] * profile_loop[j][1]
            - profile_loop[j][0] * profile_loop[i][1];
    }
    let direction = if signed_area >= 0.0 { 1.0 } else { -1.0 };
    let dual_output = create_box_rectangle_surface(&plane, &profile_loop, height * direction);

    let mut outputs = BTreeMap::new();
    // Legacy surface on existing "B" pin for backward compatibility
    // New mesh on append-only "M" pin with normals, UVs, and diagnostics
    dual_output.insert_into(&mut outputs, PIN_OUTPUT_BOX);
    Ok(outputs)
}

fn evaluate_plane_surface(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Plane Surface vereist een vlak en twee afmetingen",
        ));
    }

    let plane = coerce_plane(inputs.get(0), "Plane Surface")?;
    let size_x = coerce_number(inputs.get(1), "Plane Surface X")?;
    let size_y = coerce_number(inputs.get(2), "Plane Surface Y")?;

    let dual_output = create_plane_surface(&plane, size_x, size_y);

    let mut outputs = BTreeMap::new();
    // Legacy surface on existing "P" pin for backward compatibility
    // New mesh on append-only "M" pin with normals, UVs, and diagnostics
    dual_output.insert_into(&mut outputs, PIN_OUTPUT_PLANE);
    Ok(outputs)
}

fn evaluate_bounding_box(inputs: &[Value], mode: BoundingBoxMode) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Bounding Box vereist minimaal één invoer",
        ));
    }

    let content = inputs.get(0);
    let plane = match mode {
        BoundingBoxMode::PerItemWithPlane | BoundingBoxMode::PlaneUnion => inputs
            .get(1)
            .and_then(|value| coerce_plane(Some(value), "Bounding Box vlak").ok()),
        _ => None,
    };
    let union = match mode {
        BoundingBoxMode::PerItemWithPlane | BoundingBoxMode::LegacyWorld => false,
        BoundingBoxMode::WorldUnion => coerce_boolean(inputs.get(1), false)?,
        BoundingBoxMode::PlaneUnion => coerce_boolean(inputs.get(2), false)?,
    };

    let (world_boxes, plane_boxes) = compute_bounding_boxes(content, plane.as_ref(), union, mode)?;

    let mut outputs = BTreeMap::new();
    match mode {
        BoundingBoxMode::PerItemWithPlane | BoundingBoxMode::PlaneUnion => {
            outputs.insert(
                PIN_OUTPUT_WORLD_BOXES.to_owned(),
                Value::List(world_boxes.clone()),
            );
            outputs.insert(PIN_OUTPUT_PLANE_BOXES.to_owned(), Value::List(plane_boxes));
        }
        BoundingBoxMode::WorldUnion | BoundingBoxMode::LegacyWorld => {
            outputs.insert(PIN_OUTPUT_BOX.to_owned(), Value::List(world_boxes));
        }
    }

    Ok(outputs)
}

fn evaluate_domain_box(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Domain Box vereist een vlak en drie domeinen",
        ));
    }

    let plane = coerce_plane(inputs.get(0), "Domain Box")?;
    let domain_x = coerce_domain_range(inputs.get(1), "Domain Box X", (-0.5, 0.5))?;
    let domain_y = coerce_domain_range(inputs.get(2), "Domain Box Y", (-0.5, 0.5))?;
    let domain_z = coerce_domain_range(inputs.get(3), "Domain Box Z", (-0.5, 0.5))?;

    let box_value = create_oriented_box(
        &plane,
        [domain_x.0, domain_y.0, domain_z.0],
        [domain_x.1, domain_y.1, domain_z.1],
    );

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BOX.to_owned(), box_value);
    Ok(outputs)
}

fn evaluate_sphere(inputs: &[Value], mode: SphereMode) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Sphere vereist een vlak en straal"));
    }

    let plane = coerce_plane(inputs.get(0), "Sphere")?;
    let radius = coerce_positive_number(inputs.get(1), "Sphere straal")?;

    let dual_output = match mode {
        SphereMode::Standard => {
            // Standard sphere uses UV-sphere tessellation (latitude/longitude grid)
            create_sphere_surface(&plane, radius, false)
        }
        SphereMode::QuadSphere => {
            // QuadSphere uses cube-sphere tessellation (spherified cube)
            // This produces a more uniform vertex distribution without pole compression
            create_quad_sphere_surface(&plane, radius)
        }
    };

    let mut outputs = BTreeMap::new();
    // Legacy surface on existing "S" pin for backward compatibility
    // New mesh on append-only "M" pin with normals, UVs, and diagnostics
    dual_output.insert_into(&mut outputs, PIN_OUTPUT_SURFACE);
    Ok(outputs)
}

fn evaluate_sphere_from_points(inputs: &[Value], mode: SphereInput) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new("Sphere component vereist punten"));
    }

    let points = match mode {
        SphereInput::FourPoints => {
            if inputs.len() < 4 {
                return Err(ComponentError::new("Sphere 4Pt vereist vier punten"));
            }
            let mut pts = Vec::with_capacity(4);
            for (index, value) in inputs.iter().take(4).enumerate() {
                pts.push(coerce_point(
                    Some(value),
                    &format!("Sphere 4Pt P{}", index + 1),
                )?);
            }
            pts
        }
        SphereInput::FitCollection => collect_points(inputs.get(0), "Sphere Fit punten")?,
    };

    if points.len() < 3 {
        return Err(ComponentError::new(
            "Sphere component kon onvoldoende punten verzamelen",
        ));
    }

    let (center, radius) =
        fit_sphere_to_points(&points).ok_or_else(|| ComponentError::new("Kon geen bol fitten"))?;

    let plane = if points.len() >= 3 {
        Plane::from_points(points[0], points[1], points[2])
    } else {
        Plane::default()
    };
    let oriented_plane = Plane::normalize_axes(center, plane.x_axis, plane.y_axis, plane.z_axis);
    let dual_output = create_sphere_surface(&oriented_plane, radius, true);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CENTER.to_owned(), Value::Point(center));
    outputs.insert(PIN_OUTPUT_RADIUS.to_owned(), Value::Number(radius));
    // Legacy surface on existing "S" pin for backward compatibility
    // New mesh on append-only "M" pin with normals, UVs, and diagnostics
    dual_output.insert_into(&mut outputs, PIN_OUTPUT_SURFACE);
    Ok(outputs)
}

fn evaluate_plane_through_collection(inputs: &[Value], shape: ShapeInput) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Plane Through component vereist een vlak en geometrie",
        ));
    }

    let plane = coerce_plane(inputs.get(0), "Plane Through")?;
    let inflate = coerce_number(inputs.get(2), "Plane Through inflate")
        .unwrap_or(0.0)
        .abs();

    let points = match shape {
        ShapeInput::General => collect_points(inputs.get(1), "Plane Through shape")?,
        ShapeInput::Box => collect_points(inputs.get(1), "Plane Through box")?,
    };

    if points.is_empty() {
        return Err(ComponentError::new("Plane Through kon geen punten vinden"));
    }

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for point in points {
        let coords = plane.coordinates(point);
        min_x = min_x.min(coords[0]);
        max_x = max_x.max(coords[0]);
        min_y = min_y.min(coords[1]);
        max_y = max_y.max(coords[1]);
    }

    if !min_x.is_finite() || !max_x.is_finite() || !min_y.is_finite() || !max_y.is_finite() {
        return Err(ComponentError::new(
            "Plane Through kon de punten niet projecteren",
        ));
    }

    min_x -= inflate;
    min_y -= inflate;
    max_x += inflate;
    max_y += inflate;

    let dual_output = create_planar_surface_from_bounds(&plane, min_x, max_x, min_y, max_y);

    let mut outputs = BTreeMap::new();
    // Legacy surface on existing "S" pin for backward compatibility
    // New mesh on append-only "M" pin with normals, UVs, and diagnostics
    dual_output.insert_into(&mut outputs, PIN_OUTPUT_SURFACE);
    Ok(outputs)
}

fn compute_bounding_boxes(
    content: Option<&Value>,
    plane: Option<&Plane>,
    union: bool,
    mode: BoundingBoxMode,
) -> Result<(Vec<Value>, Vec<Value>), ComponentError> {
    let mut world_boxes = Vec::new();
    let mut plane_boxes = Vec::new();
    let mut all_points = Vec::new();
    let mut all_plane_points = Vec::new();

    let items = collect_top_level_items(content);
    for item in items {
        let points = collect_points(Some(item), "Bounding Box inhoud")?;
        if points.is_empty() {
            continue;
        }
        if !union {
            world_boxes.push(create_axis_aligned_box(&points));
        }
        if let Some(plane) = plane {
            if !union {
                plane_boxes.push(create_oriented_box_from_points(plane, &points));
            }
            all_plane_points.extend(points.iter().copied());
        }
        all_points.extend(points);
    }

    if union && !all_points.is_empty() {
        world_boxes.push(create_axis_aligned_box(&all_points));
    }
    if union {
        match mode {
            BoundingBoxMode::PlaneUnion | BoundingBoxMode::PerItemWithPlane => {
                if let Some(plane) = plane {
                    if !all_plane_points.is_empty() {
                        plane_boxes.push(create_oriented_box_from_points(plane, &all_plane_points));
                    }
                }
            }
            _ => {}
        }
    }

    Ok((world_boxes, plane_boxes))
}

fn create_sphere_surface_points(
    plane: &Plane,
    radius: f64,
    lat_segments: usize,
    lon_segments: usize,
) -> (Vec<[f64; 3]>, Vec<Vec<u32>>) {
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    for lat in 0..=lat_segments {
        let v = lat as f64 / lat_segments as f64;
        let phi = TAU * 0.5 * v;
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        for lon in 0..=lon_segments {
            let u = lon as f64 / lon_segments as f64;
            let theta = TAU * u;
            let x = radius * sin_phi * theta.cos();
            let y = radius * sin_phi * theta.sin();
            let z = radius * cos_phi;
            vertices.push(plane.apply(x, y, z));
        }
    }

    let row_length = lon_segments + 1;
    for lat in 0..lat_segments {
        for lon in 0..lon_segments {
            let current = (lat * row_length + lon) as u32;
            let next = current + 1;
            let below = current + row_length as u32;
            let below_next = below + 1;

            if lat != 0 {
                faces.push(vec![current, below, next]);
            }
            if lat != lat_segments - 1 {
                faces.push(vec![next, below, below_next]);
            }
        }
    }

    (vertices, faces)
}

/// Creates a cylinder surface mesh using the geom pipeline.
///
/// This function now uses `geom::CylinderSurface` and the shared mesh pipeline
/// Creates a cylinder surface mesh using the geom pipeline.
///
/// This function now uses `geom::CylinderSurface` and the shared mesh pipeline
/// to create proper meshes with normals and UVs.
///
/// Returns a `DualMeshOutput` containing both `Value::Mesh` (primary) and
/// `Value::Surface` (legacy) for backward compatibility.
fn create_cylinder_surface(plane: &Plane, radius: f64, height: f64) -> DualMeshOutput {
    create_cylinder_mesh_geom(plane, radius, height)
}

/// Creates a cone surface mesh using the geom pipeline.
///
/// This function now uses `geom::ConeSurface` and the shared mesh pipeline
/// to create proper meshes with normals and UVs.
///
/// Returns `(DualMeshOutput, tip_point)` with both output formats.
fn create_cone_surface(plane: &Plane, radius: f64, height: f64) -> (DualMeshOutput, [f64; 3]) {
    create_cone_mesh_geom(plane, radius, height)
}

/// Creates a plane surface mesh using the geom pipeline.
///
/// This function now uses `geom::PlaneSurface` and the shared mesh pipeline
/// to create proper meshes with normals and UVs.
///
/// Returns a `DualMeshOutput` containing both `Value::Mesh` (primary) and
/// `Value::Surface` (legacy) for backward compatibility.
fn create_plane_surface(plane: &Plane, size_x: f64, size_y: f64) -> DualMeshOutput {
    create_plane_mesh_geom(plane, size_x, size_y)
}

/// Creates a planar surface mesh from bounds using the geom pipeline.
///
/// Returns a `DualMeshOutput` containing both `Value::Mesh` (primary) and
/// `Value::Surface` (legacy) for backward compatibility.
fn create_planar_surface_from_bounds(
    plane: &Plane,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
) -> DualMeshOutput {
    create_planar_mesh_from_bounds_geom(plane, min_x, max_x, min_y, max_y)
}

/// Creates a sphere surface mesh using the geom pipeline.
///
/// This function now uses `geom::SphereSurface` and the shared mesh pipeline
/// to create proper meshes with normals and UVs.
///
/// Returns a `DualMeshOutput` containing both `Value::Mesh` (primary) and
/// `Value::Surface` (legacy) for backward compatibility.
fn create_sphere_surface(plane: &Plane, radius: f64, detailed: bool) -> DualMeshOutput {
    create_sphere_mesh_geom(plane, radius, detailed)
}

/// Legacy sphere surface creation for fallback.
#[allow(dead_code)]
fn create_sphere_surface_legacy(plane: &Plane, radius: f64, detailed: bool) -> Value {
    let lat_segments = if detailed { 24 } else { 16 };
    let lon_segments = if detailed { 32 } else { 16 };
    let (vertices, faces) = create_sphere_surface_points(plane, radius, lat_segments, lon_segments);
    Value::Surface { vertices, faces }
}

// ============================================================================
// Geom-based surface creation using the shared mesh pipeline
// ============================================================================

/// Converts an array [f64; 3] to a geom Point3.
#[inline]
fn to_geom_point(p: [f64; 3]) -> GeomPoint3 {
    GeomPoint3::new(p[0], p[1], p[2])
}

/// Converts an array [f64; 3] to a geom Vec3.
#[inline]
fn to_geom_vec(v: [f64; 3]) -> GeomVec3 {
    GeomVec3::new(v[0], v[1], v[2])
}

/// Converts a geom Point3 to an array [f64; 3].
#[inline]
#[allow(dead_code)]
fn from_geom_point(p: GeomPoint3) -> [f64; 3] {
    p.to_array()
}

/// Converts a `geom::GeomMesh` to `Value::Mesh`.
fn geom_mesh_to_value_mesh(
    mesh: GeomMesh,
    diagnostics: Option<GeomMeshDiagnostics>,
) -> Value {
    Value::Mesh {
        vertices: mesh.positions,
        indices: mesh.indices,
        normals: mesh.normals,
        uvs: mesh.uvs,
        diagnostics: diagnostics.map(|d| MeshDiagnostics {
            vertex_count: d.vertex_count,
            triangle_count: d.triangle_count,
            welded_vertex_count: d.welded_vertex_count,
            flipped_triangle_count: d.flipped_triangle_count,
            degenerate_triangle_count: d.degenerate_triangle_count,
            open_edge_count: d.open_edge_count,
            non_manifold_edge_count: d.non_manifold_edge_count,
            self_intersection_count: d.self_intersection_count,
            boolean_fallback_used: d.boolean_fallback_used,
            warnings: d.warnings,
        }),
    }
}

/// Converts a `geom::GeomMesh` to `Value::Surface` (legacy format).
///
/// This is for backward compatibility with existing consumers expecting surfaces.
fn geom_mesh_to_value_surface(mesh: &GeomMesh) -> Value {
    let faces: Vec<Vec<u32>> = mesh
        .indices
        .chunks(3)
        .filter(|chunk| chunk.len() == 3)
        .map(|chunk| vec![chunk[0], chunk[1], chunk[2]])
        .collect();
    Value::Surface {
        vertices: mesh.positions.clone(),
        faces,
    }
}

/// Dual mesh output containing both `Value::Surface` (legacy, for backward compat) and `Value::Mesh` (new).
///
/// This struct is used to provide both output formats from surface primitive components.
/// Following the pattern from `surface_freeform.rs`:
/// - Legacy `Value::Surface` is emitted on existing pins (e.g., "C", "P", "S") for backward compatibility
/// - New `Value::Mesh` is emitted on append-only "M" pin with normals, UVs, and diagnostics
struct DualMeshOutput {
    /// The new mesh output with normals, UVs, and diagnostics (emitted on "M" pin).
    mesh: Value,
    /// The legacy surface output for backward compatibility (emitted on existing pins).
    surface_legacy: Value,
}

impl DualMeshOutput {
    /// Creates a dual output from a `GeomMesh` and optional diagnostics.
    ///
    /// **Note**: This derives the legacy surface from the geom mesh, which may have
    /// different vertex ordering than the original legacy algorithm. For primitives
    /// that need backward-compatible vertex ordering (cylinder, cone, sphere), use
    /// `from_geom_mesh_with_legacy_surface` instead.
    fn from_geom_mesh(mesh: GeomMesh, diagnostics: Option<GeomMeshDiagnostics>) -> Self {
        // Create legacy surface first (needs reference to positions/indices)
        let surface_legacy = geom_mesh_to_value_surface(&mesh);
        // Create the primary mesh output (consumes mesh)
        let mesh_value = geom_mesh_to_value_mesh(mesh, diagnostics);
        Self {
            mesh: mesh_value,
            surface_legacy,
        }
    }

    /// Creates a dual output from a `GeomMesh` and a separately computed legacy surface.
    ///
    /// Use this constructor when the legacy surface must preserve specific vertex ordering
    /// and face topology for backward compatibility (e.g., cylinder, cone, sphere primitives).
    /// The primary `Value::Mesh` output uses the geom pipeline (with normals, UVs, diagnostics),
    /// while the legacy `Value::Surface` output uses the pre-computed legacy representation.
    fn from_geom_mesh_with_legacy_surface(
        mesh: GeomMesh,
        diagnostics: Option<GeomMeshDiagnostics>,
        legacy_surface: Value,
    ) -> Self {
        let mesh_value = geom_mesh_to_value_mesh(mesh, diagnostics);
        Self {
            mesh: mesh_value,
            surface_legacy: legacy_surface,
        }
    }

    /// Inserts both outputs into a BTreeMap following the pattern from `surface_freeform.rs`.
    ///
    /// - `legacy_pin`: Pin name for the `Value::Surface` output (existing pin for backward compat)
    ///
    /// The `Value::Mesh` output is always emitted on the append-only "M" pin, consistent with
    /// how `surface_freeform.rs` components expose mesh outputs. This ensures:
    /// - Existing graphs that expect `Value::Surface` on standard pins continue to work
    /// - New code can use the "M" pin to access `Value::Mesh` with normals, UVs, and diagnostics
    fn insert_into(self, outputs: &mut BTreeMap<String, Value>, legacy_pin: &str) {
        outputs.insert(legacy_pin.to_owned(), self.surface_legacy);
        outputs.insert(PIN_OUTPUT_MESH.to_owned(), self.mesh);
    }
}

/// Creates a cylinder mesh using geom::CylinderSurface and the shared mesh pipeline.
///
/// Returns a `DualMeshOutput` containing both `Value::Mesh` (primary) and
/// `Value::Surface` (legacy) for backward compatibility.
///
/// The primary mesh output uses the geom pipeline with proper normals and UVs.
/// The legacy surface output uses the original algorithm to preserve vertex ordering:
/// - 32 segments around the circumference
/// - Vertices ordered as base/top pairs: `[base0, top0, base1, top1, ...]`
/// - 64 triangular faces (2 per quad segment)
fn create_cylinder_mesh_geom(plane: &Plane, radius: f64, height: f64) -> DualMeshOutput {
    // Build the geom CylinderSurface
    let base = to_geom_point(plane.origin);
    let axis = to_geom_vec(plane.z_axis).mul_scalar(height);
    let x_axis = to_geom_vec(plane.x_axis);

    let cylinder = match GeomCylinderSurface::from_base_axis_xaxis(base, axis, x_axis, radius) {
        Ok(cyl) => cyl,
        Err(_) => {
            // Fallback to legacy if geom construction fails
            return create_cylinder_dual_legacy(plane, radius, height);
        }
    };

    // Mesh the cylinder using the shared pipeline for the primary output
    // Use fixed counts for cylinder: 32 around, 2 along height (open surface, no caps)
    let u_count = 32; // Around the circumference
    let v_count = 2;  // Along height (base to top)
    let (mesh, diagnostics) = mesh_surface(&cylinder, u_count, v_count);

    // Create the legacy surface using the original algorithm for backward-compatible ordering
    let legacy_surface = create_cylinder_surface_legacy(plane, radius, height);

    DualMeshOutput::from_geom_mesh_with_legacy_surface(mesh, Some(diagnostics), legacy_surface)
}

/// Legacy fallback for cylinder that returns dual output.
fn create_cylinder_dual_legacy(plane: &Plane, radius: f64, height: f64) -> DualMeshOutput {
    let surface = create_cylinder_surface_legacy(plane, radius, height);
    // Extract mesh data from the legacy surface to create Value::Mesh
    if let Value::Surface { vertices, faces } = &surface {
        let indices: Vec<u32> = faces.iter().flat_map(|f| f.iter().copied()).collect();
        let mesh = Value::Mesh {
            vertices: vertices.clone(),
            indices,
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        DualMeshOutput {
            mesh,
            surface_legacy: surface,
        }
    } else {
        // Should never happen, but handle gracefully
        DualMeshOutput {
            mesh: surface.clone(),
            surface_legacy: surface,
        }
    }
}

/// Creates a cone mesh using geom::ConeSurface and the shared mesh pipeline.
///
/// Returns `(DualMeshOutput, tip_point)` with proper normals, UVs, and diagnostics.
///
/// The primary mesh output uses the geom pipeline with proper normals and UVs.
/// The legacy surface output uses the original algorithm to preserve vertex ordering:
/// - 32 base vertices around the circumference + 1 tip vertex (33 total)
/// - 32 triangular faces forming a fan from the tip
fn create_cone_mesh_geom(plane: &Plane, radius: f64, height: f64) -> (DualMeshOutput, [f64; 3]) {
    // Build the geom ConeSurface
    let base = to_geom_point(plane.origin);
    let axis = to_geom_vec(plane.z_axis).mul_scalar(height);
    let x_axis = to_geom_vec(plane.x_axis);

    // Cone goes from radius at base (v=0) to 0 at tip (v=1)
    let cone = match GeomConeSurface::from_base_axis_xaxis(base, axis, x_axis, radius, 0.0) {
        Ok(c) => c,
        Err(_) => {
            // Fallback to legacy if geom construction fails
            return create_cone_dual_legacy(plane, radius, height);
        }
    };

    // Calculate tip position
    let tip = plane.apply(0.0, 0.0, height);

    // Mesh the cone using the shared pipeline for the primary output
    // Use fixed counts: 32 around, higher V count since surface tapers to a pole
    let u_count = 32;
    let v_count = 16; // More segments along height for pole handling
    let (mesh, diagnostics) = mesh_surface(&cone, u_count, v_count);

    // Create the legacy surface using the original algorithm for backward-compatible ordering
    let (legacy_surface, _tip) = create_cone_surface_legacy(plane, radius, height);

    (DualMeshOutput::from_geom_mesh_with_legacy_surface(mesh, Some(diagnostics), legacy_surface), tip)
}

/// Legacy fallback for cone that returns dual output.
fn create_cone_dual_legacy(plane: &Plane, radius: f64, height: f64) -> (DualMeshOutput, [f64; 3]) {
    let (surface, tip) = create_cone_surface_legacy(plane, radius, height);
    // Extract mesh data from the legacy surface to create Value::Mesh
    if let Value::Surface { vertices, faces } = &surface {
        let indices: Vec<u32> = faces.iter().flat_map(|f| f.iter().copied()).collect();
        let mesh = Value::Mesh {
            vertices: vertices.clone(),
            indices,
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        (DualMeshOutput {
            mesh,
            surface_legacy: surface,
        }, tip)
    } else {
        // Should never happen, but handle gracefully
        (DualMeshOutput {
            mesh: surface.clone(),
            surface_legacy: surface,
        }, tip)
    }
}

/// Creates a sphere mesh using geom::SphereSurface and the shared mesh pipeline.
///
/// Returns a `DualMeshOutput` containing both `Value::Mesh` (primary) and
/// `Value::Surface` (legacy) for backward compatibility.
///
/// The primary mesh output uses the geom pipeline with proper normals and UVs.
/// The legacy surface output uses the original algorithm to preserve vertex ordering:
/// - `(lat_segments + 1) * (lon_segments + 1)` vertices in lat/lon grid order
/// - Special pole handling: no triangles at north pole row, no triangles at south pole row
/// - Standard: 16 lat x 16 lon (289 vertices, 480 faces)
/// - Detailed: 24 lat x 32 lon (833 vertices, 1472 faces)
fn create_sphere_mesh_geom(plane: &Plane, radius: f64, detailed: bool) -> DualMeshOutput {
    // Build the geom SphereSurface
    let center = to_geom_point(plane.origin);
    let x_axis = to_geom_vec(plane.x_axis);
    let normal = to_geom_vec(plane.z_axis);

    let sphere = match GeomSphereSurface::from_center_xaxis_normal(center, x_axis, normal, radius) {
        Ok(s) => s,
        Err(_) => {
            // Fallback to legacy if geom construction fails
            return create_sphere_dual_legacy(plane, radius, detailed);
        }
    };

    // Mesh the sphere using the shared pipeline for the primary output
    // Sphere has poles at v=0 and v=1, so use higher V count
    let (u_count, v_count) = if detailed {
        (32, 24) // Higher resolution for detailed view
    } else {
        (16, 16) // Standard resolution
    };
    let (mesh, diagnostics) = mesh_surface(&sphere, u_count, v_count);

    // Create the legacy surface using the original algorithm for backward-compatible ordering
    let legacy_surface = create_sphere_surface_legacy(plane, radius, detailed);

    DualMeshOutput::from_geom_mesh_with_legacy_surface(mesh, Some(diagnostics), legacy_surface)
}

/// Legacy fallback for sphere that returns dual output.
fn create_sphere_dual_legacy(plane: &Plane, radius: f64, detailed: bool) -> DualMeshOutput {
    let surface = create_sphere_surface_legacy(plane, radius, detailed);
    // Extract mesh data from the legacy surface to create Value::Mesh
    if let Value::Surface { vertices, faces } = &surface {
        let indices: Vec<u32> = faces.iter().flat_map(|f| f.iter().copied()).collect();
        let mesh = Value::Mesh {
            vertices: vertices.clone(),
            indices,
            normals: None,
            uvs: None,
            diagnostics: None,
        };
        DualMeshOutput {
            mesh,
            surface_legacy: surface,
        }
    } else {
        // Should never happen, but handle gracefully
        DualMeshOutput {
            mesh: surface.clone(),
            surface_legacy: surface,
        }
    }
}

/// Creates a cube-sphere mesh (QuadSphere) using the spherified cube algorithm.
///
/// A cube-sphere (also known as a quad-sphere or spherified cube) produces a more
/// uniform vertex distribution across the sphere surface compared to a traditional
/// UV-sphere, which suffers from vertex compression at the poles.
///
/// # Algorithm
///
/// 1. Start with a unit cube centered at the origin
/// 2. Subdivide each of the 6 faces into a grid of quads
/// 3. Project (normalize) each vertex onto the unit sphere
/// 4. Scale by the desired radius and translate to the center
///
/// # Advantages over UV-sphere
///
/// - **Uniform distribution**: Vertices are spread more evenly across the surface
/// - **No pole pinching**: UV-spheres have many triangles converging at poles
/// - **Better triangle quality**: More uniform triangle aspect ratios
/// - **Physics-friendly**: Better for collision detection and simulations
///
/// # Tessellation
///
/// Uses 8 subdivisions per cube face edge, producing approximately:
/// - ~386 vertices (after welding shared edges/corners)
/// - 768 triangles (128 per face × 6 faces)
///
/// Returns a `DualMeshOutput` containing both `Value::Mesh` (primary) and
/// `Value::Surface` (legacy) for backward compatibility.
fn create_quad_sphere_surface(plane: &Plane, radius: f64) -> DualMeshOutput {
    // Build cube-sphere options with the plane's orientation frame
    let center = to_geom_point(plane.origin);
    let x_axis = to_geom_vec(plane.x_axis);
    let y_axis = to_geom_vec(plane.y_axis);
    let z_axis = to_geom_vec(plane.z_axis);

    // Use 8 subdivisions for a good balance of quality and performance
    // This produces ~386 vertices and 768 triangles
    let subdivisions = 8;

    let options = CubeSphereOptions::new(center, radius, subdivisions)
        .with_frame(x_axis, y_axis, z_axis);

    let (mesh, diagnostics) = mesh_cube_sphere(options);

    // Convert to Value::Mesh with diagnostics
    let mesh_value = geom_mesh_to_value_mesh(mesh.clone(), Some(diagnostics));

    // Create legacy surface output for backward compatibility
    // The cube-sphere mesh is converted to the legacy format
    let legacy_surface = geom_mesh_to_value_surface(&mesh);

    DualMeshOutput {
        mesh: mesh_value,
        surface_legacy: legacy_surface,
    }
}

/// Creates a plane surface mesh using geom::PlaneSurface and the shared mesh pipeline.
///
/// Returns a `DualMeshOutput` containing both `Value::Mesh` (primary) and
/// `Value::Surface` (legacy) for backward compatibility.
fn create_plane_mesh_geom(plane: &Plane, size_x: f64, size_y: f64) -> DualMeshOutput {
    let half_x = size_x / 2.0;
    let half_y = size_y / 2.0;
    create_planar_mesh_from_bounds_geom(plane, -half_x, half_x, -half_y, half_y)
}

/// Creates a planar surface mesh from bounds using geom::PlaneSurface.
///
/// Returns a `DualMeshOutput` containing both `Value::Mesh` (primary) and
/// `Value::Surface` (legacy) for backward compatibility.
fn create_planar_mesh_from_bounds_geom(
    plane: &Plane,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
) -> DualMeshOutput {
    // Build the geom PlaneSurface
    // The plane surface uses u_axis and v_axis scaled to the bounds
    let u_axis = to_geom_vec(plane.x_axis);
    let v_axis = to_geom_vec(plane.y_axis);
    
    // Calculate corner of the plane (bottom-left in plane coordinates)
    let origin = to_geom_point(plane.apply(min_x, min_y, 0.0));
    
    // Scale axes to span the bounds
    let u_span = max_x - min_x;
    let v_span = max_y - min_y;
    
    let plane_surface = GeomPlaneSurface::new(
        origin,
        u_axis.mul_scalar(u_span),
        v_axis.mul_scalar(v_span),
    );

    // Mesh the plane - simple 2x2 grid is sufficient for a flat surface
    let u_count = 2;
    let v_count = 2;
    let (mesh, diagnostics) = mesh_surface(&plane_surface, u_count, v_count);

    DualMeshOutput::from_geom_mesh(mesh, Some(diagnostics))
}

// ============================================================================
// Legacy surface creation functions (kept for fallback compatibility)
// ============================================================================

/// Legacy cylinder surface creation (for fallback).
fn create_cylinder_surface_legacy(plane: &Plane, radius: f64, height: f64) -> Value {
    let segments = 32;
    let mut vertices = Vec::with_capacity(segments * 2);
    let mut faces = Vec::with_capacity(segments * 2);

    for i in 0..segments {
        let angle = TAU * i as f64 / segments as f64;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        let base = plane.apply(x, y, 0.0);
        let top = plane.apply(x, y, height);
        vertices.push(base);
        vertices.push(top);
    }

    for i in 0..segments {
        let next = (i + 1) % segments;
        let base_i = (2 * i) as u32;
        let top_i = base_i + 1;
        let base_next = (2 * next) as u32;
        let top_next = base_next + 1;
        faces.push(vec![base_i, base_next, top_next]);
        faces.push(vec![base_i, top_next, top_i]);
    }

    Value::Surface { vertices, faces }
}

/// Legacy cone surface creation (for fallback).
fn create_cone_surface_legacy(plane: &Plane, radius: f64, height: f64) -> (Value, [f64; 3]) {
    let segments = 32;
    let mut vertices = Vec::with_capacity(segments + 1);
    let mut faces = Vec::with_capacity(segments);

    for i in 0..segments {
        let angle = TAU * i as f64 / segments as f64;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        vertices.push(plane.apply(x, y, 0.0));
    }

    let tip = plane.apply(0.0, 0.0, height);
    vertices.push(tip);
    let tip_index = vertices.len() as u32 - 1;

    for i in 0..segments {
        let next = (i + 1) % segments;
        faces.push(vec![i as u32, next as u32, tip_index]);
    }

    (Value::Surface { vertices, faces }, tip)
}

fn create_axis_aligned_box(points: &[[f64; 3]]) -> Value {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for point in points {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    create_box_from_extents(min, max)
}

fn create_oriented_box_from_points(plane: &Plane, points: &[[f64; 3]]) -> Value {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for point in points {
        let coords = plane.coordinates(*point);
        for axis in 0..3 {
            min[axis] = min[axis].min(coords[axis]);
            max[axis] = max[axis].max(coords[axis]);
        }
    }
    create_oriented_box(plane, min, max)
}

fn create_box_from_extents(min: [f64; 3], max: [f64; 3]) -> Value {
    let corners = [
        [min[0], min[1], min[2]],
        [max[0], min[1], min[2]],
        [max[0], max[1], min[2]],
        [min[0], max[1], min[2]],
        [min[0], min[1], max[2]],
        [max[0], min[1], max[2]],
        [max[0], max[1], max[2]],
        [min[0], max[1], max[2]],
    ];
    Value::List(corners.into_iter().map(Value::Point).collect())
}

fn create_oriented_box_vertices(plane: &Plane, min: [f64; 3], max: [f64; 3]) -> Vec<[f64; 3]> {
    let mut corners = Vec::with_capacity(8);
    for &z in &[min[2], max[2]] {
        for &y in &[min[1], max[1]] {
            for &x in &[min[0], max[0]] {
                corners.push(plane.apply(x, y, z));
            }
        }
    }
    corners
}

fn create_oriented_box(plane: &Plane, min: [f64; 3], max: [f64; 3]) -> Value {
    let vertices = create_oriented_box_vertices(plane, min, max);
    Value::List(vertices.into_iter().map(Value::Point).collect())
}

fn create_box_rectangle_surface(
    plane: &Plane,
    profile: &[[f64; 2]],
    height: f64,
) -> DualMeshOutput {
    let vertex_count = profile.len();
    let mut vertices = Vec::with_capacity(vertex_count * 2);
    let base_z = 0.0;
    let top_z = height;
    for uv in profile {
        vertices.push(plane.apply(uv[0], uv[1], base_z));
    }
    for uv in profile {
        vertices.push(plane.apply(uv[0], uv[1], top_z));
    }

    let base_indices: Vec<u32> = (0..vertex_count).map(|index| index as u32).collect();
    let top_indices: Vec<u32> = (vertex_count as u32..(vertex_count * 2) as u32).collect();

    let mut faces = Vec::with_capacity((vertex_count - 2) * 2 + vertex_count * 2);

    // Triangulate base cap
    let mut base_loop = base_indices.clone();
    if height >= 0.0 {
        base_loop.reverse();
    }
    for i in 1..base_loop.len() - 1 {
        faces.push(vec![base_loop[0], base_loop[i], base_loop[i + 1]]);
    }

    // Triangulate top cap
    let mut top_loop = top_indices.clone();
    if height < 0.0 {
        top_loop.reverse();
    }
    for i in 1..top_loop.len() - 1 {
        faces.push(vec![top_loop[0], top_loop[i], top_loop[i + 1]]);
    }

    // Side faces (two triangles per edge)
    let height_positive = height >= 0.0;
    for i in 0..vertex_count {
        let next = (i + 1) % vertex_count;
        if height_positive {
            faces.push(vec![base_indices[i], base_indices[next], top_indices[next]]);
            faces.push(vec![base_indices[i], top_indices[next], top_indices[i]]);
        } else {
            faces.push(vec![base_indices[next], base_indices[i], top_indices[i]]);
            faces.push(vec![base_indices[next], top_indices[i], top_indices[next]]);
        }
    }

    // Flip normals: reverse winding of all faces.
    for face in &mut faces {
        face.reverse();
    }

    // Create the legacy Value::Surface
    let surface_legacy = Value::Surface {
        vertices: vertices.clone(),
        faces: faces.clone(),
    };

    // Create Value::Mesh with flattened indices
    let indices: Vec<u32> = faces.iter().flat_map(|f| f.iter().copied()).collect();
    let mesh = Value::Mesh {
        vertices,
        indices,
        normals: None,
        uvs: None,
        diagnostics: None,
    };

    DualMeshOutput {
        mesh,
        surface_legacy,
    }
}

fn collect_top_level_items(value: Option<&Value>) -> Vec<&Value> {
    match value {
        Some(value) => vec![value],
        None => Vec::new(),
    }
}

fn coerce_plane(value: Option<&Value>, context: &str) -> Result<Plane, ComponentError> {
    match value {
        None => Ok(Plane::default()),
        Some(Value::List(values)) if values.len() >= 3 => {
            let a = coerce_point(values.get(0), context)?;
            let b = coerce_point(values.get(1), context)?;
            let c = coerce_point(values.get(2), context)?;
            Ok(Plane::from_points(a, b, c))
        }
        Some(Value::List(values)) if values.len() == 2 => {
            let origin = coerce_point(values.get(0), context)?;
            let direction = coerce_vector(values.get(1), context)?;
            if vector_length_squared(direction) < EPSILON {
                Ok(Plane::default())
            } else {
                let x_axis = normalize(direction);
                let z_axis = orthogonal_vector(x_axis);
                let y_axis = normalize(cross(z_axis, x_axis));
                Ok(Plane::normalize_axes(origin, x_axis, y_axis, z_axis))
            }
        }
        Some(Value::List(values)) if values.len() == 1 => coerce_plane(values.get(0), context),
        Some(Value::Point(point)) => Ok(Plane::from_origin(*point)),
        Some(Value::Vector(vector)) => {
            let normal = if vector_length_squared(*vector) < EPSILON {
                [0.0, 0.0, 1.0]
            } else {
                normalize(*vector)
            };
            let x_axis = orthogonal_vector(normal);
            let y_axis = normalize(cross(normal, x_axis));
            Ok(Plane::normalize_axes(
                [0.0, 0.0, 0.0],
                x_axis,
                y_axis,
                normal,
            ))
        }
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een vlak, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!(
            "{} vereist een numerieke invoer",
            context
        )));
    };
    match value {
        Value::Number(number) => Ok(*number),
        Value::List(values) if values.len() == 1 => coerce_number(values.get(0), context),
        other => Err(ComponentError::new(format!(
            "{} verwacht een numerieke waarde, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_positive_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    let number = coerce_number(value, context)?;
    if number <= 0.0 {
        return Err(ComponentError::new(format!(
            "{} vereist een waarde groter dan nul",
            context
        )));
    }
    Ok(number)
}

fn coerce_boolean(value: Option<&Value>, default: bool) -> Result<bool, ComponentError> {
    let Some(value) = value else {
        return Ok(default);
    };
    match value {
        Value::Boolean(flag) => Ok(*flag),
        Value::Number(number) => Ok(*number >= 0.5),
        Value::List(values) if values.len() == 1 => coerce_boolean(values.get(0), default),
        Value::Text(text) => Ok(matches!(text.trim().to_lowercase().as_str(), "true" | "1")),
        other => Err(ComponentError::new(format!(
            "Kon boolean niet lezen uit {}",
            other.kind()
        ))),
    }
}

fn coerce_domain_range(
    value: Option<&Value>,
    context: &str,
    default: (f64, f64),
) -> Result<(f64, f64), ComponentError> {
    match value {
        None => Ok(default),
        Some(Value::Domain(Domain::One(domain))) => {
            Ok((domain.min.min(domain.max), domain.max.max(domain.min)))
        }
        Some(Value::List(values)) if values.len() >= 2 => {
            let min = coerce_number(values.get(0), context)?;
            let max = coerce_number(values.get(1), context)?;
            Ok((min.min(max), max.max(min)))
        }
        Some(Value::Number(number)) => {
            let extent = number.abs() / 2.0;
            Ok((-extent, extent))
        }
        Some(Value::List(values)) if values.len() == 1 => {
            coerce_domain_range(values.get(0), context, default)
        }
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een domein, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_point(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!("{} vereist een punt", context)));
    };
    match value {
        Value::Point(point) | Value::Vector(point) => Ok(*point),
        Value::List(values) if values.len() == 1 => coerce_point(values.get(0), context),
        Value::List(values) if values.len() >= 3 => {
            let x = coerce_number(values.get(0), context)?;
            let y = coerce_number(values.get(1), context)?;
            let z = coerce_number(values.get(2), context)?;
            Ok([x, y, z])
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een punt, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn coerce_vector(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    let Some(value) = value else {
        return Err(ComponentError::new(format!(
            "{} vereist een vector",
            context
        )));
    };
    match value {
        Value::Vector(vector) | Value::Point(vector) => Ok(*vector),
        Value::List(values) if values.len() == 1 => coerce_vector(values.get(0), context),
        Value::List(values) if values.len() >= 3 => {
            let x = coerce_number(values.get(0), context)?;
            let y = coerce_number(values.get(1), context)?;
            let z = coerce_number(values.get(2), context)?;
            Ok([x, y, z])
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een vector, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn collect_points(value: Option<&Value>, context: &str) -> Result<Vec<[f64; 3]>, ComponentError> {
    let mut points = Vec::new();
    if let Some(value) = value {
        collect_points_into(value, context, &mut points)?;
    }
    Ok(points)
}

fn collect_points_into(
    value: &Value,
    context: &str,
    output: &mut Vec<[f64; 3]>,
) -> Result<(), ComponentError> {
    match value {
        Value::Point(point) | Value::Vector(point) => {
            output.push(*point);
            Ok(())
        }
        Value::CurveLine { p1, p2 } => {
            output.push(*p1);
            output.push(*p2);
            Ok(())
        }
        Value::Surface { vertices, .. } => {
            output.extend(vertices.iter().copied());
            Ok(())
        }
        Value::Mesh { vertices, .. } => {
            // Support new Value::Mesh inputs (from updated surface primitives)
            output.extend(vertices.iter().copied());
            Ok(())
        }
        Value::List(values) => {
            if let Ok(point) = coerce_point(Some(value), context) {
                output.push(point);
                return Ok(());
            }
            for entry in values {
                collect_points_into(entry, context, output)?;
            }
            Ok(())
        }
        Value::Number(number) => {
            output.push([*number, 0.0, 0.0]);
            Ok(())
        }
        Value::Boolean(boolean) => {
            output.push([if *boolean { 1.0 } else { 0.0 }, 0.0, 0.0]);
            Ok(())
        }
        Value::Text(text) => {
            if let Ok(parsed) = text.trim().parse::<f64>() {
                output.push([parsed, 0.0, 0.0]);
                Ok(())
            } else {
                Err(ComponentError::new(format!(
                    "{} kon tekst '{}' niet als punt interpreteren",
                    context, text
                )))
            }
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht geometrie, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn fit_sphere_to_points(points: &[[f64; 3]]) -> Option<([f64; 3], f64)> {
    if points.len() < 3 {
        return None;
    }

    let mut ata = [[0.0_f64; 4]; 4];
    let mut atb = [0.0_f64; 4];

    for point in points {
        let row = [2.0 * point[0], 2.0 * point[1], 2.0 * point[2], 1.0];
        let rhs = point[0] * point[0] + point[1] * point[1] + point[2] * point[2];
        for i in 0..4 {
            for j in 0..4 {
                ata[i][j] += row[i] * row[j];
            }
            atb[i] += row[i] * rhs;
        }
    }

    if let Some(solution) = solve_linear_system(&mut ata, &mut atb) {
        let center = [solution[0], solution[1], solution[2]];
        let radius_sq =
            center[0] * center[0] + center[1] * center[1] + center[2] * center[2] + solution[3];
        if radius_sq > 0.0 && radius_sq.is_finite() {
            return Some((center, radius_sq.sqrt()));
        }
    }

    let mut center = [0.0; 3];
    for point in points {
        for axis in 0..3 {
            center[axis] += point[axis];
        }
    }
    for axis in 0..3 {
        center[axis] /= points.len() as f64;
    }
    let mut radius = 0.0_f64;
    for point in points {
        radius = radius.max(vector_length(subtract(*point, center)));
    }
    if radius <= EPSILON {
        None
    } else {
        Some((center, radius))
    }
}

fn solve_linear_system(matrix: &mut [[f64; 4]; 4], vector: &mut [f64; 4]) -> Option<[f64; 4]> {
    for i in 0..4 {
        let mut pivot_row = i;
        let mut pivot_value = matrix[i][i].abs();
        for row in i + 1..4 {
            let value = matrix[row][i].abs();
            if value > pivot_value {
                pivot_value = value;
                pivot_row = row;
            }
        }
        if pivot_value < EPSILON {
            return None;
        }
        if pivot_row != i {
            matrix.swap(i, pivot_row);
            vector.swap(i, pivot_row);
        }

        let pivot = matrix[i][i];
        for col in i..4 {
            matrix[i][col] /= pivot;
        }
        vector[i] /= pivot;

        for row in 0..4 {
            if row == i {
                continue;
            }
            let factor = matrix[row][i];
            for col in i..4 {
                matrix[row][col] -= factor * matrix[i][col];
            }
            vector[row] -= factor * vector[i];
        }
    }

    Some(*vector)
}

fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(v: [f64; 3], factor: f64) -> [f64; 3] {
    [v[0] * factor, v[1] * factor, v[2] * factor]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vector_length_squared(v: [f64; 3]) -> f64 {
    dot(v, v)
}

fn vector_length(v: [f64; 3]) -> f64 {
    vector_length_squared(v).sqrt()
}

fn safe_normalized(v: [f64; 3]) -> Option<([f64; 3], f64)> {
    let length = vector_length(v);
    if length < EPSILON {
        None
    } else {
        Some((scale(v, 1.0 / length), length))
    }
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    safe_normalized(v)
        .map(|(vector, _)| vector)
        .unwrap_or([0.0, 0.0, 0.0])
}

fn orthogonal_vector(reference: [f64; 3]) -> [f64; 3] {
    let mut candidate = if reference[0].abs() < reference[1].abs() {
        [0.0, -reference[2], reference[1]]
    } else {
        [-reference[2], 0.0, reference[0]]
    };
    if vector_length_squared(candidate) < EPSILON {
        candidate = [reference[1], -reference[0], 0.0];
    }
    let normalized = normalize(candidate);
    if vector_length_squared(normalized) < EPSILON {
        [1.0, 0.0, 0.0]
    } else {
        normalized
    }
}

const EPSILON: f64 = 1e-9;

#[derive(Debug, Clone, Copy)]
struct Plane {
    origin: [f64; 3],
    x_axis: [f64; 3],
    y_axis: [f64; 3],
    z_axis: [f64; 3],
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            z_axis: [0.0, 0.0, 1.0],
        }
    }
}

impl Plane {
    fn from_origin(origin: [f64; 3]) -> Self {
        Self {
            origin,
            ..Self::default()
        }
    }

    fn from_points(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Self {
        let ab = subtract(b, a);
        let ac = subtract(c, a);
        let normal = cross(ab, ac);
        if vector_length_squared(normal) < EPSILON {
            return Self::default();
        }
        let x_axis = if vector_length_squared(ab) < EPSILON {
            orthogonal_vector(normal)
        } else {
            normalize(ab)
        };
        let z_axis = normalize(normal);
        let y_axis = normalize(cross(z_axis, x_axis));
        Self::normalize_axes(a, x_axis, y_axis, z_axis)
    }

    fn normalize_axes(
        origin: [f64; 3],
        x_axis: [f64; 3],
        y_axis: [f64; 3],
        z_axis: [f64; 3],
    ) -> Self {
        let z_axis = safe_normalized(z_axis)
            .map(|(vector, _)| vector)
            .unwrap_or([0.0, 0.0, 1.0]);

        let mut x_axis = safe_normalized(x_axis)
            .map(|(vector, _)| vector)
            .unwrap_or_else(|| orthogonal_vector(z_axis));

        let mut y_axis = safe_normalized(y_axis)
            .map(|(vector, _)| vector)
            .unwrap_or_else(|| normalize(cross(z_axis, x_axis)));

        let x_cross = cross(y_axis, z_axis);
        if vector_length_squared(x_cross) < EPSILON {
            x_axis = orthogonal_vector(z_axis);
        } else {
            x_axis = normalize(x_cross);
        }

        let y_cross = cross(z_axis, x_axis);
        if vector_length_squared(y_cross) < EPSILON {
            y_axis = orthogonal_vector(x_axis);
        } else {
            y_axis = normalize(y_cross);
        }

        Self {
            origin,
            x_axis,
            y_axis,
            z_axis,
        }
    }

    fn apply(&self, u: f64, v: f64, w: f64) -> [f64; 3] {
        add(
            add(
                add(self.origin, scale(self.x_axis, u)),
                scale(self.y_axis, v),
            ),
            scale(self.z_axis, w),
        )
    }

    fn coordinates(&self, point: [f64; 3]) -> [f64; 3] {
        let relative = subtract(point, self.origin);
        [
            dot(relative, self.x_axis),
            dot(relative, self.y_axis),
            dot(relative, self.z_axis),
        ]
    }
}