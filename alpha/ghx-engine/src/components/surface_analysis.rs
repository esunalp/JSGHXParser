//! Implementaties van Grasshopper "Surface → Analysis" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_CURVES: &str = "C";
const PIN_OUTPUT_BOX_PLANE: &str = "Pl";
const PIN_OUTPUT_BOX_POINT: &str = "Pt";
const PIN_OUTPUT_BOX_INCLUDE: &str = "I";
const PIN_OUTPUT_NAKED_EDGES: &str = "En";
const PIN_OUTPUT_INTERIOR_EDGES: &str = "Ei";
const PIN_OUTPUT_NON_MANIFOLD_EDGES: &str = "Em";
const PIN_OUTPUT_POINTS: &str = "P";
const PIN_OUTPUT_WEIGHTS: &str = "W";
const PIN_OUTPUT_GREVILLE: &str = "G";
const PIN_OUTPUT_U_COUNT: &str = "U";
const PIN_OUTPUT_V_COUNT: &str = "V";
const PIN_OUTPUT_AREA: &str = "A";
const PIN_OUTPUT_VOLUME: &str = "V";
const PIN_OUTPUT_CENTROID: &str = "C";
const PIN_OUTPUT_INERTIA: &str = "I";
const PIN_OUTPUT_INERTIA_ERROR: &str = "I±";
const PIN_OUTPUT_SECONDARY: &str = "S";
const PIN_OUTPUT_SECONDARY_ERROR: &str = "S±";
const PIN_OUTPUT_GYRATION: &str = "G";
const PIN_OUTPUT_RELATION: &str = "R";
const PIN_OUTPUT_FRAME: &str = "F";
const PIN_OUTPUT_NORMAL: &str = "N";
const PIN_OUTPUT_DISTANCE: &str = "D";
const PIN_OUTPUT_U_DIRECTION: &str = "U";
const PIN_OUTPUT_V_DIRECTION: &str = "V";
const PIN_OUTPUT_UV_POINT: &str = "uvP";
const PIN_OUTPUT_CIRCLE_ONE: &str = "C1";
const PIN_OUTPUT_CIRCLE_TWO: &str = "C2";
const PIN_OUTPUT_WIREFRAME: &str = "W";
const PIN_OUTPUT_PLANAR: &str = "F";
const PIN_OUTPUT_PLANE: &str = "P";
const PIN_OUTPUT_X_SIZE: &str = "X";
const PIN_OUTPUT_Y_SIZE: &str = "Y";
const PIN_OUTPUT_Z_SIZE: &str = "Z";
const PIN_OUTPUT_INSIDE: &str = "I";
const PIN_OUTPUT_INDEX: &str = "i";
const PIN_OUTPUT_FACES: &str = "F";
const PIN_OUTPUT_EDGES: &str = "E";
const PIN_OUTPUT_VERTICES: &str = "V";
const PIN_OUTPUT_FACE_FACE: &str = "FF";
const PIN_OUTPUT_FACE_EDGE: &str = "FE";
const PIN_OUTPUT_EDGE_FACE: &str = "EF";

const EPSILON: f32 = 1e-9;

/// Beschikbare componentvarianten binnen Surface → Analysis.
#[derive(Debug, Default, Clone, Copy)]
pub enum ComponentKind {
    #[default]
    SurfaceInflection,
    EvaluateBox,
    BrepEdges,
    SurfacePoints,
    AreaMoments,
    Volume,
    ShapeInBrep,
    Area,
    VolumeMoments,
    EvaluateSurface,
    PrincipalCurvature,
    SurfaceCurvature,
    SurfaceClosestPoint,
    BrepClosestPointWithNormal,
    BrepClosestPoint,
    BrepAreaMoments,
    PointInBreps,
    BrepTopology,
    DeconstructBrep,
    BoxCorners,
    BrepArea,
    BrepWireframe,
    BoxProperties,
    OsculatingCircles,
    BrepVolume,
    IsPlanar,
    DeconstructBox,
    PointInBrep,
    Dimensions,
    PointInTrim,
    BrepVolumeMoments,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor Surface → Analysis.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{0148a65d-6f42-414a-9db7-9a9b2eb78437}"],
        names: &["Brep Edges", "Edges"],
        kind: ComponentKind::BrepEdges,
    },
    Registration {
        guids: &["{0efd7f0c-f63d-446d-970e-9fb0e636ea41}"],
        names: &["Surface Inflection", "SInf"],
        kind: ComponentKind::SurfaceInflection,
    },
    Registration {
        guids: &["{13b40e9c-3aed-4669-b2e8-60bd02091421}"],
        names: &["Evaluate Box", "Box"],
        kind: ComponentKind::EvaluateBox,
    },
    Registration {
        guids: &["{15128198-399d-4d6c-9586-1f65db3ce7bf}"],
        names: &["Surface Points", "SrfPt"],
        kind: ComponentKind::SurfacePoints,
    },
    Registration {
        guids: &["{1eb7b856-ec7d-40b6-a76c-f216a11df37c}", "{c98c1666-5f29-4bb8-aafd-bb5a708e8a95}"],
        names: &["Area Moments", "AMoments"],
        kind: ComponentKind::AreaMoments,
    },
    Registration {
        guids: &["{224f7648-5956-4b26-80d9-8d771f3dfd5d}", "{7c0523e8-79c9-45a2-8777-cf0d46bc5432}"],
        names: &["Volume"],
        kind: ComponentKind::Volume,
    },
    Registration {
        guids: &["{2ba64356-be21-4c12-bbd4-ced54f04c8ef}"],
        names: &["Shape In Brep", "ShapeIn"],
        kind: ComponentKind::ShapeInBrep,
    },
    Registration {
        guids: &["{2e205f24-9279-47b2-b414-d06dcd0b21a7}", "{86b28a7e-94d9-4791-8306-e13e10d5f8d5}", "{ab766b01-a3f5-4257-831a-fc84d7b288b4}"],
        names: &["Area"],
        kind: ComponentKind::Area,
    },
    Registration {
        guids: &["{2e685fd9-7b8f-461b-b330-44857b099937}", "{4b5f79e1-c2b3-4b9c-b97d-470145a3ca74}", "{ffdfcfc5-3933-4c38-b680-8bb530e243ff}"],
        names: &["Volume Moments", "VMoments"],
        kind: ComponentKind::VolumeMoments,
    },
    Registration {
        guids: &["{353b206e-bde5-4f02-a913-b3b8a977d4b9}", "{aa1dc107-70de-473e-9636-836030160fc3}"],
        names: &["Evaluate Surface", "EvalSrf"],
        kind: ComponentKind::EvaluateSurface,
    },
    Registration {
        guids: &["{404f75ac-5594-4c48-ad8a-7d0f472bbf8a}"],
        names: &["Principal Curvature", "Curvature"],
        kind: ComponentKind::PrincipalCurvature,
    },
    Registration {
        guids: &["{4139f3a3-cf93-4fc0-b5e0-18a3acd0b003}"],
        names: &["Surface Curvature", "Curvature"],
        kind: ComponentKind::SurfaceCurvature,
    },
    Registration {
        guids: &["{4a9e9a8e-0943-4438-b360-129c30f2bb0f}"],
        names: &["Surface Closest Point", "Srf CP"],
        kind: ComponentKind::SurfaceClosestPoint,
    },
    Registration {
        guids: &["{4beead95-8aa2-4613-8bb9-24758a0f5c4c}"],
        names: &["Brep Closest Point", "Brep CP"],
        kind: ComponentKind::BrepClosestPointWithNormal,
    },
    Registration {
        guids: &["{5d2fb801-2905-4a55-9d48-bbb22c73ad13}"],
        names: &["Brep Area Moments", "AMoments"],
        kind: ComponentKind::BrepAreaMoments,
    },
    Registration {
        guids: &["{859daa86-3ab7-49cb-9eda-f2811c984070}"],
        names: &["Point In Breps", "BrepsInc"],
        kind: ComponentKind::PointInBreps,
    },
    Registration {
        guids: &["{866ee39d-9ebf-4e1d-b209-324c56825605}"],
        names: &["Brep Topology", "Topology"],
        kind: ComponentKind::BrepTopology,
    },
    Registration {
        guids: &["{8d372bdc-9800-45e9-8a26-6e33c5253e21}"],
        names: &["Deconstruct Brep", "DeBrep"],
        kind: ComponentKind::DeconstructBrep,
    },
    Registration {
        guids: &["{a10e8cdf-7c7a-4aac-aa70-ddb7010ab231}"],
        names: &["Box Corners"],
        kind: ComponentKind::BoxCorners,
    },
    Registration {
        guids: &["{ac750e41-2450-4f98-9658-98fef97b01b2}"],
        names: &["Brep Wireframe", "Wires"],
        kind: ComponentKind::BrepWireframe,
    },
    Registration {
        guids: &["{af9cdb9d-9617-4827-bb3c-9efd88c76a70}"],
        names: &["Box Properties", "BoxProp"],
        kind: ComponentKind::BoxProperties,
    },
    Registration {
        guids: &["{b799b7c0-76df-4bdb-b3cc-401b1d021aa5}"],
        names: &["Osculating Circles", "Osc"],
        kind: ComponentKind::OsculatingCircles,
    },
    Registration {
        guids: &["{c72d0184-bb99-4af4-a629-4662e1c3d428}"],
        names: &["Brep Volume", "Volume"],
        kind: ComponentKind::BrepVolume,
    },
    Registration {
        guids: &["{cdd5d441-3bad-4f19-a370-6cf180b6f0fa}"],
        names: &["Brep Closest Point", "Brep CP"],
        kind: ComponentKind::BrepClosestPoint,
    },
    Registration {
        guids: &["{d4bc9653-c770-4bee-a31d-d120cbb75b39}"],
        names: &["Is Planar", "Planar"],
        kind: ComponentKind::IsPlanar,
    },
    Registration {
        guids: &["{db7d83b1-2898-4ef9-9be5-4e94b4e2048d}"],
        names: &["Deconstruct Box", "DeBox"],
        kind: ComponentKind::DeconstructBox,
    },
    Registration {
        guids: &["{e03561f8-0e66-41d3-afde-62049f152443}"],
        names: &["Point In Brep", "BrepInc"],
        kind: ComponentKind::PointInBrep,
    },
    Registration {
        guids: &["{f241e42e-8983-4ed3-b869-621c07630b00}"],
        names: &["Dimensions", "Dim"],
        kind: ComponentKind::Dimensions,
    },
    Registration {
        guids: &["{f881810b-96de-4668-a95a-f9a6d683e65c}"],
        names: &["Point In Trim", "TrimInc"],
        kind: ComponentKind::PointInTrim,
    },
    Registration {
        guids: &["{ffdfcfc5-3933-4c38-b680-8bb530e243ff}"],
        names: &["Brep Volume Moments", "VMoments"],
        kind: ComponentKind::BrepVolumeMoments,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::SurfaceInflection => evaluate_surface_inflection(),
            Self::EvaluateBox => evaluate_box(inputs),
            Self::BrepEdges => evaluate_brep_edges(inputs),
            Self::SurfacePoints => evaluate_surface_points(inputs),
            Self::AreaMoments => evaluate_area_moments(inputs, "Area Moments"),
            Self::Volume => evaluate_volume(inputs, "Volume"),
            Self::ShapeInBrep => evaluate_shape_in_brep(inputs),
            Self::Area => evaluate_area(inputs, "Area"),
            Self::VolumeMoments => evaluate_volume_moments(inputs, "Volume Moments"),
            Self::EvaluateSurface => evaluate_surface_sample_component(inputs),
            Self::PrincipalCurvature => evaluate_principal_curvature(inputs),
            Self::SurfaceCurvature => evaluate_surface_curvature(inputs),
            Self::SurfaceClosestPoint => evaluate_surface_closest_point(inputs),
            Self::BrepClosestPointWithNormal => {
                evaluate_brep_closest_point(inputs, true)
            }
            Self::BrepClosestPoint => evaluate_brep_closest_point(inputs, false),
            Self::BrepAreaMoments => evaluate_area_moments(inputs, "Brep Area Moments"),
            Self::PointInBreps => evaluate_point_in_breps(inputs),
            Self::BrepTopology => evaluate_brep_topology(),
            Self::DeconstructBrep => evaluate_deconstruct_brep(inputs),
            Self::BoxCorners => evaluate_box_corners(inputs),
            Self::BrepArea => evaluate_area(inputs, "Brep Area"),
            Self::BrepWireframe => evaluate_brep_wireframe(inputs),
            Self::BoxProperties => evaluate_box_properties(inputs),
            Self::OsculatingCircles => evaluate_osculating_circles(inputs),
            Self::BrepVolume => evaluate_volume(inputs, "Brep Volume"),
            Self::IsPlanar => evaluate_is_planar(inputs),
            Self::DeconstructBox => evaluate_deconstruct_box(inputs),
            Self::PointInBrep => evaluate_point_in_brep(inputs),
            Self::Dimensions => evaluate_dimensions(inputs),
            Self::PointInTrim => evaluate_point_in_trim(inputs),
            Self::BrepVolumeMoments => evaluate_volume_moments(inputs, "Brep Volume Moments"),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::SurfaceInflection => "Surface Inflection",
            Self::EvaluateBox => "Evaluate Box",
            Self::BrepEdges => "Brep Edges",
            Self::SurfacePoints => "Surface Points",
            Self::AreaMoments => "Area Moments",
            Self::Volume => "Volume",
            Self::ShapeInBrep => "Shape In Brep",
            Self::Area => "Area",
            Self::VolumeMoments => "Volume Moments",
            Self::EvaluateSurface => "Evaluate Surface",
            Self::PrincipalCurvature => "Principal Curvature",
            Self::SurfaceCurvature => "Surface Curvature",
            Self::SurfaceClosestPoint => "Surface Closest Point",
            Self::BrepClosestPointWithNormal | Self::BrepClosestPoint => "Brep Closest Point",
            Self::BrepAreaMoments => "Brep Area Moments",
            Self::PointInBreps => "Point In Breps",
            Self::BrepTopology => "Brep Topology",
            Self::DeconstructBrep => "Deconstruct Brep",
            Self::BoxCorners => "Box Corners",
            Self::BrepArea => "Brep Area",
            Self::BrepWireframe => "Brep Wireframe",
            Self::BoxProperties => "Box Properties",
            Self::OsculatingCircles => "Osculating Circles",
            Self::BrepVolume => "Brep Volume",
            Self::IsPlanar => "Is Planar",
            Self::DeconstructBox => "Deconstruct Box",
            Self::PointInBrep => "Point In Brep",
            Self::Dimensions => "Dimensions",
            Self::PointInTrim => "Point In Trim",
            Self::BrepVolumeMoments => "Brep Volume Moments",
        }
    }
}

fn evaluate_surface_inflection() -> ComponentResult {
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CURVES.to_owned(), Value::List(Vec::new()));
    Ok(outputs)
}

fn evaluate_box(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Evaluate Box verwacht een box en drie parameters",
        ));
    }

    let points = match inputs[0] {
        Value::List(ref values) if values.len() >= 8 => values
            .iter()
            .filter_map(|value| match value {
                Value::Point(point) => Some(*point),
                _ => None,
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };

    if points.len() < 8 {
        return Err(ComponentError::new(
            "Evaluate Box vereist acht hoekpunten",
        ));
    }

    let u = coerce_number(inputs.get(1), "Evaluate Box U")?.clamp(0.0, 1.0);
    let v = coerce_number(inputs.get(2), "Evaluate Box V")?.clamp(0.0, 1.0);
    let w = coerce_number(inputs.get(3), "Evaluate Box W")?.clamp(0.0, 1.0);

    let (min, max) = bounding_box(&points);
    let location = [
        min[0] + (max[0] - min[0]) * u,
        min[1] + (max[1] - min[1]) * v,
        min[2] + (max[2] - min[2]) * w,
    ];

    let plane = Value::List(vec![
        Value::Point(location),
        Value::Point([location[0] + 1.0, location[1], location[2]]),
        Value::Point([location[0], location[1] + 1.0, location[2]]),
    ]);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_BOX_PLANE.to_owned(), plane);
    outputs.insert(PIN_OUTPUT_BOX_POINT.to_owned(), Value::Point(location));
    outputs.insert(PIN_OUTPUT_BOX_INCLUDE.to_owned(), Value::Boolean(true));
    Ok(outputs)
}

fn evaluate_brep_edges(inputs: &[Value]) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Brep Edges vereist een brep"))?;
    let wireframe = create_wireframe(&metrics)
        .into_iter()
        .map(|(p1, p2)| Value::CurveLine { p1, p2 })
        .collect();
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_NAKED_EDGES.to_owned(), Value::List(wireframe));
    outputs.insert(PIN_OUTPUT_INTERIOR_EDGES.to_owned(), Value::List(Vec::new()));
    outputs.insert(PIN_OUTPUT_NON_MANIFOLD_EDGES.to_owned(), Value::List(Vec::new()));
    Ok(outputs)
}

fn evaluate_surface_points(inputs: &[Value]) -> ComponentResult {
    let surface = inputs.get(0).ok_or_else(|| {
        ComponentError::new("Surface Points vereist een surface invoer")
    })?;

    if let Some(grid) = collect_point_grid(Some(surface)) {
        let v_count = grid.len();
        let u_count = grid.iter().map(|row| row.len()).max().unwrap_or(0);
        let mut point_values = Vec::new();
        let mut weights = Vec::new();
        let mut greville = Vec::new();
        for (v_index, row) in grid.iter().enumerate() {
            for (u_index, point) in row.iter().enumerate() {
                point_values.push(Value::Point(*point));
                weights.push(Value::Number(1.0));
                let u = if u_count > 1 {
                    u_index as f32 / (u_count - 1) as f32
                } else {
                    0.0
                };
                let v = if v_count > 1 {
                    v_index as f32 / (v_count - 1) as f32
                } else {
                    0.0
                };
                greville.push(Value::Point([u, v, 0.0]));
            }
        }
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(point_values));
        outputs.insert(PIN_OUTPUT_WEIGHTS.to_owned(), Value::List(weights));
        outputs.insert(PIN_OUTPUT_GREVILLE.to_owned(), Value::List(greville));
        outputs.insert(
            PIN_OUTPUT_U_COUNT.to_owned(),
            Value::Number(u_count as f32),
        );
        outputs.insert(
            PIN_OUTPUT_V_COUNT.to_owned(),
            Value::Number(v_count as f32),
        );
        Ok(outputs)
    } else {
        let metrics = ShapeMetrics::from_inputs(Some(surface))
            .ok_or_else(|| ComponentError::new("Surface Points kon geen punten vinden"))?;
        let points = metrics
            .points
            .iter()
            .map(|point| Value::Point(*point))
            .collect();
        let weight_list = (0..metrics.points.len())
            .map(|_| Value::Number(1.0))
            .collect();
        let greville = (0..metrics.points.len())
            .map(|index| {
                let u = if metrics.points.len() > 1 {
                    index as f32 / (metrics.points.len() - 1) as f32
                } else {
                    0.0
                };
                Value::Point([u, 0.0, 0.0])
            })
            .collect();
        let mut outputs = BTreeMap::new();
        outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(points));
        outputs.insert(PIN_OUTPUT_WEIGHTS.to_owned(), Value::List(weight_list));
        outputs.insert(PIN_OUTPUT_GREVILLE.to_owned(), Value::List(greville));
        outputs.insert(PIN_OUTPUT_U_COUNT.to_owned(), Value::Number(metrics.points.len() as f32));
        outputs.insert(PIN_OUTPUT_V_COUNT.to_owned(), Value::Number(1.0));
        Ok(outputs)
    }
}

fn evaluate_area_moments(inputs: &[Value], context: &str) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0)).ok_or_else(|| {
        ComponentError::new(format!("{} vereist geometrische invoer", context))
    })?;
    let area = metrics.area();
    let centroid = Value::Point(metrics.center());
    let inertia = simple_inertia(metrics.size(), area);
    let secondary = simple_secondary(metrics.size(), area);
    let gyration = simple_gyration(inertia, area);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_AREA.to_owned(), Value::Number(area));
    outputs.insert(PIN_OUTPUT_CENTROID.to_owned(), centroid);
    outputs.insert(PIN_OUTPUT_INERTIA.to_owned(), to_number_list(&inertia));
    outputs.insert(PIN_OUTPUT_INERTIA_ERROR.to_owned(), to_number_list(&[0.0; 3]));
    outputs.insert(PIN_OUTPUT_SECONDARY.to_owned(), to_number_list(&secondary));
    outputs.insert(
        PIN_OUTPUT_SECONDARY_ERROR.to_owned(),
        to_number_list(&[0.0; 3]),
    );
    outputs.insert(PIN_OUTPUT_GYRATION.to_owned(), to_number_list(&gyration));
    Ok(outputs)
}

fn evaluate_volume(inputs: &[Value], context: &str) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0)).ok_or_else(|| {
        ComponentError::new(format!("{} vereist geometrische invoer", context))
    })?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_VOLUME.to_owned(), Value::Number(metrics.volume()));
    outputs.insert(PIN_OUTPUT_CENTROID.to_owned(), Value::Point(metrics.center()));
    Ok(outputs)
}

fn evaluate_shape_in_brep(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Shape In Brep vereist een brep en een vorm",
        ));
    }
    let brep = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Shape In Brep vereist een brep"))?;
    let shape = ShapeMetrics::from_inputs(inputs.get(1))
        .ok_or_else(|| ComponentError::new("Shape In Brep vereist een vorm"))?;

    let shape_corners = create_box_corners_points(&shape);
    let inside = shape_corners
        .iter()
        .all(|point| point_in_metrics(&brep, *point, false));
    let relation = if inside {
        0
    } else if boxes_overlap(&brep, &shape) {
        1
    } else {
        2
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_RELATION.to_owned(), Value::Number(relation as f32));
    Ok(outputs)
}

fn evaluate_area(inputs: &[Value], context: &str) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0)).ok_or_else(|| {
        ComponentError::new(format!("{} vereist geometrische invoer", context))
    })?;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_AREA.to_owned(), Value::Number(metrics.area()));
    outputs.insert(PIN_OUTPUT_CENTROID.to_owned(), Value::Point(metrics.center()));
    Ok(outputs)
}

fn evaluate_volume_moments(inputs: &[Value], context: &str) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0)).ok_or_else(|| {
        ComponentError::new(format!("{} vereist geometrische invoer", context))
    })?;
    let volume = metrics.volume();
    let inertia = simple_inertia(metrics.size(), volume);
    let secondary = simple_secondary(metrics.size(), volume);
    let gyration = simple_gyration(inertia, volume);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_VOLUME.to_owned(), Value::Number(volume));
    outputs.insert(PIN_OUTPUT_CENTROID.to_owned(), Value::Point(metrics.center()));
    outputs.insert(PIN_OUTPUT_INERTIA.to_owned(), to_number_list(&inertia));
    outputs.insert(PIN_OUTPUT_INERTIA_ERROR.to_owned(), to_number_list(&[0.0; 3]));
    outputs.insert(PIN_OUTPUT_SECONDARY.to_owned(), to_number_list(&secondary));
    outputs.insert(
        PIN_OUTPUT_SECONDARY_ERROR.to_owned(),
        to_number_list(&[0.0; 3]),
    );
    outputs.insert(PIN_OUTPUT_GYRATION.to_owned(), to_number_list(&gyration));
    Ok(outputs)
}

fn evaluate_surface_sample_component(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Evaluate Surface vereist minimaal een surface",
        ));
    }
    let surface = ShapeMetrics::from_inputs(inputs.get(0)).ok_or_else(|| {
        ComponentError::new("Evaluate Surface kon de surface niet lezen")
    })?;
    let uv = coerce_uv(inputs.get(1)).unwrap_or((0.5, 0.5));
    let point = surface.sample_point(uv);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::Point(point));
    outputs.insert(PIN_OUTPUT_NORMAL.to_owned(), Value::Vector([0.0, 0.0, 1.0]));
    outputs.insert(PIN_OUTPUT_U_DIRECTION.to_owned(), Value::Vector([1.0, 0.0, 0.0]));
    outputs.insert(PIN_OUTPUT_V_DIRECTION.to_owned(), Value::Vector([0.0, 1.0, 0.0]));
    outputs.insert(PIN_OUTPUT_FRAME.to_owned(), plane_from_point(point));
    Ok(outputs)
}

fn evaluate_principal_curvature(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 1 {
        return Err(ComponentError::new(
            "Principal Curvature vereist een surface",
        ));
    }
    let surface = ShapeMetrics::from_inputs(inputs.get(0)).ok_or_else(|| {
        ComponentError::new("Principal Curvature kon de surface niet lezen")
    })?;
    let uv = coerce_uv(inputs.get(1)).unwrap_or((0.5, 0.5));
    let point = surface.sample_point(uv);
    let size = surface.size();
    let max_curvature = if size[0].abs() <= EPSILON {
        0.0
    } else {
        2.0 / size[0].abs()
    };
    let min_curvature = if size[1].abs() <= EPSILON {
        0.0
    } else {
        2.0 / size[1].abs()
    };

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FRAME.to_owned(), plane_from_point(point));
    outputs.insert(
        "Maximum".to_owned(),
        Value::Number(max_curvature.clamp(-1e6, 1e6)),
    );
    outputs.insert(
        "Minimum".to_owned(),
        Value::Number(min_curvature.clamp(-1e6, 1e6)),
    );
    outputs.insert("K¹".to_owned(), Value::Vector([1.0, 0.0, 0.0]));
    outputs.insert("K²".to_owned(), Value::Vector([0.0, 1.0, 0.0]));
    Ok(outputs)
}

fn evaluate_surface_curvature(inputs: &[Value]) -> ComponentResult {
    if inputs.is_empty() {
        return Err(ComponentError::new(
            "Surface Curvature vereist een surface",
        ));
    }
    let surface = ShapeMetrics::from_inputs(inputs.get(0)).ok_or_else(|| {
        ComponentError::new("Surface Curvature kon de surface niet lezen")
    })?;
    let uv = coerce_uv(inputs.get(1)).unwrap_or((0.5, 0.5));
    let point = surface.sample_point(uv);
    let size = surface.size();
    let curvature_u = if size[0].abs() <= EPSILON {
        0.0
    } else {
        1.0 / size[0].abs()
    };
    let curvature_v = if size[1].abs() <= EPSILON {
        0.0
    } else {
        1.0 / size[1].abs()
    };
    let gaussian = curvature_u * curvature_v;
    let mean = (curvature_u + curvature_v) * 0.5;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FRAME.to_owned(), plane_from_point(point));
    outputs.insert("Gaussian".to_owned(), Value::Number(gaussian));
    outputs.insert("Mean".to_owned(), Value::Number(mean));
    Ok(outputs)
}

fn evaluate_surface_closest_point(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Surface Closest Point vereist een surface en een uv-waarde",
        ));
    }
    let target = coerce_point(inputs.get(0), "Surface Closest Point punt")?;
    let surface = ShapeMetrics::from_inputs(inputs.get(1))
        .ok_or_else(|| ComponentError::new("Surface Closest Point vereist een surface"))?;
    let closest = clamp_to_metrics(&surface, target);
    let uv = uv_from_point(&surface, closest);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::Point(closest));
    outputs.insert(PIN_OUTPUT_UV_POINT.to_owned(), Value::Point([uv.0, uv.1, 0.0]));
    outputs.insert(
        PIN_OUTPUT_DISTANCE.to_owned(),
        Value::Number(distance(&target, &closest)),
    );
    Ok(outputs)
}

fn evaluate_brep_closest_point(inputs: &[Value], include_normal: bool) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Brep Closest Point vereist een punt en een brep",
        ));
    }
    let target = coerce_point(inputs.get(0), "Brep Closest Point punt")?;
    let brep = ShapeMetrics::from_inputs(inputs.get(1))
        .ok_or_else(|| ComponentError::new("Brep Closest Point vereist een brep"))?;
    let closest = clamp_to_metrics(&brep, target);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::Point(closest));
    if include_normal {
        outputs.insert(PIN_OUTPUT_NORMAL.to_owned(), Value::Vector([0.0, 0.0, 1.0]));
    }
    outputs.insert(
        PIN_OUTPUT_DISTANCE.to_owned(),
        Value::Number(distance(&target, &closest)),
    );
    Ok(outputs)
}

fn evaluate_point_in_breps(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Point In Breps vereist een lijst met breps en een punt",
        ));
    }
    let list = match &inputs[0] {
        Value::List(values) => values,
        other => {
            return Err(ComponentError::new(format!(
                "Point In Breps verwacht een lijst, kreeg {}",
                other.kind()
            )))
        }
    };
    let target = coerce_point(inputs.get(1), "Point In Breps punt")?;
    let strict = coerce_boolean(inputs.get(2), false)?;

    let mut inside_index = -1;
    for (index, entry) in list.iter().enumerate() {
        if let Some(metrics) = ShapeMetrics::from_inputs(Some(entry)) {
            if point_in_metrics(&metrics, target, strict) {
                inside_index = index as i32;
                break;
            }
        }
    }
    let inside = inside_index >= 0;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_INSIDE.to_owned(), Value::Boolean(inside));
    outputs.insert(PIN_OUTPUT_INDEX.to_owned(), Value::Number(inside_index as f32));
    Ok(outputs)
}

fn evaluate_brep_topology() -> ComponentResult {
    let face_face = (0..6)
        .map(|index| {
            Value::List(vec![Value::Number(((index + 1) % 6) as f32)])
        })
        .collect();
    let face_edge = (0..6)
        .map(|index| Value::List(vec![Value::Number(index as f32)]))
        .collect();
    let edge_face = (0..12)
        .map(|index| Value::List(vec![Value::Number((index % 6) as f32)]))
        .collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FACE_FACE.to_owned(), Value::List(face_face));
    outputs.insert(PIN_OUTPUT_FACE_EDGE.to_owned(), Value::List(face_edge));
    outputs.insert(PIN_OUTPUT_EDGE_FACE.to_owned(), Value::List(edge_face));
    Ok(outputs)
}

fn evaluate_deconstruct_brep(inputs: &[Value]) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Deconstruct Brep vereist een brep"))?;
    let corners = create_box_corners_points(&metrics);
    let faces = create_box_faces(&corners)
        .into_iter()
        .map(|face| Value::List(face.into_iter().map(Value::Point).collect()))
        .collect();
    let edges = create_wireframe(&metrics)
        .into_iter()
        .map(|(p1, p2)| Value::CurveLine { p1, p2 })
        .collect();
    let vertices = corners.into_iter().map(Value::Point).collect();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_FACES.to_owned(), Value::List(faces));
    outputs.insert(PIN_OUTPUT_EDGES.to_owned(), Value::List(edges));
    outputs.insert(PIN_OUTPUT_VERTICES.to_owned(), Value::List(vertices));
    Ok(outputs)
}

fn evaluate_box_corners(inputs: &[Value]) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Box Corners vereist een box"))?;
    let corners = create_box_corners_points(&metrics)
        .into_iter()
        .map(Value::Point)
        .collect();
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::List(corners));
    Ok(outputs)
}

fn evaluate_brep_wireframe(inputs: &[Value]) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Brep Wireframe vereist een brep"))?;
    let wireframe = create_wireframe(&metrics)
        .into_iter()
        .map(|(p1, p2)| Value::CurveLine { p1, p2 })
        .collect();
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_WIREFRAME.to_owned(), Value::List(wireframe));
    Ok(outputs)
}

fn evaluate_box_properties(inputs: &[Value]) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Box Properties vereist een box"))?;
    let size = metrics.size();
    let diagonal = [size[0], size[1], size[2]];
    let degeneracy = diagonal
        .iter()
        .filter(|value| value.abs() <= EPSILON)
        .count();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_CENTROID.to_owned(), Value::Point(metrics.center()));
    outputs.insert(PIN_OUTPUT_DISTANCE.to_owned(), Value::Vector(diagonal));
    outputs.insert(PIN_OUTPUT_AREA.to_owned(), Value::Number(metrics.area()));
    outputs.insert(PIN_OUTPUT_VOLUME.to_owned(), Value::Number(metrics.volume()));
    outputs.insert(
        "d".to_owned(),
        Value::Number(degeneracy as f32),
    );
    Ok(outputs)
}

fn evaluate_osculating_circles(inputs: &[Value]) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Osculating Circles vereist een surface"))?;
    let uv = coerce_uv(inputs.get(1)).unwrap_or((0.5, 0.5));
    let point = metrics.sample_point(uv);
    let size = metrics.size();
    let radius_u = if size[0].abs() <= EPSILON {
        0.0
    } else {
        size[0].abs() * 0.5
    };
    let radius_v = if size[1].abs() <= EPSILON {
        0.0
    } else {
        size[1].abs() * 0.5
    };
    let circle_u = Value::List(vec![
        Value::Point(point),
        Value::Point([point[0] + radius_u, point[1], point[2]]),
    ]);
    let circle_v = Value::List(vec![
        Value::Point(point),
        Value::Point([point[0], point[1] + radius_v, point[2]]),
    ]);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_POINTS.to_owned(), Value::Point(point));
    outputs.insert(PIN_OUTPUT_CIRCLE_ONE.to_owned(), circle_u);
    outputs.insert(PIN_OUTPUT_CIRCLE_TWO.to_owned(), circle_v);
    Ok(outputs)
}

fn evaluate_is_planar(inputs: &[Value]) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Is Planar vereist een surface"))?;
    let size = metrics.size();
    let planar = size[2].abs() <= EPSILON;
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_PLANAR.to_owned(), Value::Boolean(planar));
    outputs.insert(PIN_OUTPUT_PLANE.to_owned(), plane_from_point(metrics.center()));
    Ok(outputs)
}

fn evaluate_deconstruct_box(inputs: &[Value]) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Deconstruct Box vereist een box"))?;
    let size = metrics.size();
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_PLANE.to_owned(), plane_from_point(metrics.center()));
    outputs.insert(PIN_OUTPUT_X_SIZE.to_owned(), Value::Number(size[0].abs()));
    outputs.insert(PIN_OUTPUT_Y_SIZE.to_owned(), Value::Number(size[1].abs()));
    outputs.insert(PIN_OUTPUT_Z_SIZE.to_owned(), Value::Number(size[2].abs()));
    Ok(outputs)
}

fn evaluate_point_in_brep(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Point In Brep vereist een brep en een punt",
        ));
    }
    let brep = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Point In Brep vereist een brep"))?;
    let point = coerce_point(inputs.get(1), "Point In Brep punt")?;
    let strict = coerce_boolean(inputs.get(2), false)?;
    let inside = point_in_metrics(&brep, point, strict);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_INSIDE.to_owned(), Value::Boolean(inside));
    Ok(outputs)
}

fn evaluate_dimensions(inputs: &[Value]) -> ComponentResult {
    let metrics = ShapeMetrics::from_inputs(inputs.get(0))
        .ok_or_else(|| ComponentError::new("Dimensions vereist een surface"))?;
    let size = metrics.size();
    let mut outputs = BTreeMap::new();
    outputs.insert("U".to_owned(), Value::Number(size[0].abs()));
    outputs.insert("V".to_owned(), Value::Number(size[1].abs()));
    Ok(outputs)
}

fn evaluate_point_in_trim(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Point In Trim vereist een surface en een uv-punt",
        ));
    }
    let uv = coerce_uv(inputs.get(1)).unwrap_or((0.5, 0.5));
    let inside = (0.0..=1.0).contains(&uv.0) && (0.0..=1.0).contains(&uv.1);
    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_INSIDE.to_owned(), Value::Boolean(inside));
    Ok(outputs)
}

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f32, ComponentError> {
    match value {
        Some(Value::Number(number)) => Ok(*number),
        Some(Value::Boolean(flag)) => Ok(if *flag { 1.0 } else { 0.0 }),
        Some(Value::List(values)) if values.len() == 1 => coerce_number(values.get(0), context),
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een getal, kreeg {}",
            context,
            other.kind()
        ))),
        None => Err(ComponentError::new(format!(
            "{} vereist een numerieke invoer",
            context
        ))),
    }
}

fn coerce_boolean(value: Option<&Value>, default: bool) -> Result<bool, ComponentError> {
    match value {
        None => Ok(default),
        Some(Value::Boolean(flag)) => Ok(*flag),
        Some(Value::Number(number)) => Ok(*number != 0.0),
        Some(Value::List(values)) if values.len() == 1 => coerce_boolean(values.get(0), default),
        Some(Value::Text(text)) => {
            let normalized = text.trim().to_ascii_lowercase();
            if ["true", "yes", "1", "on"].contains(&normalized.as_str()) {
                Ok(true)
            } else if ["false", "no", "0", "off"].contains(&normalized.as_str()) {
                Ok(false)
            } else {
                Err(ComponentError::new(format!(
                    "Kon boolean niet afleiden uit '{}'",
                    text
                )))
            }
        }
        Some(other) => Err(ComponentError::new(format!(
            "Kon boolean niet afleiden uit {}",
            other.kind()
        ))),
    }
}

fn coerce_point(value: Option<&Value>, context: &str) -> Result<[f32; 3], ComponentError> {
    match value {
        Some(Value::Point(point)) | Some(Value::Vector(point)) => Ok(*point),
        Some(Value::List(values)) if values.len() >= 3 => {
            let x = coerce_number(values.get(0), context)?;
            let y = coerce_number(values.get(1), context)?;
            let z = coerce_number(values.get(2), context)?;
            Ok([x, y, z])
        }
        Some(Value::List(values)) if !values.is_empty() => coerce_point(values.get(0), context),
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een punt, kreeg {}",
            context,
            other.kind()
        ))),
        None => Err(ComponentError::new(format!(
            "{} vereist een punt",
            context
        ))),
    }
}

fn coerce_uv(value: Option<&Value>) -> Option<(f32, f32)> {
    match value {
        Some(Value::Point([u, v, _])) => Some((*u, *v)),
        Some(Value::Vector([u, v, _])) => Some((*u, *v)),
        Some(Value::Number(number)) => Some((*number, *number)),
        Some(Value::List(values)) if values.len() >= 2 => {
            let u = coerce_number(values.get(0), "uv").ok()?;
            let v = coerce_number(values.get(1), "uv").ok()?;
            Some((u, v))
        }
        Some(Value::List(values)) if !values.is_empty() => coerce_uv(values.get(0)),
        _ => None,
    }
}

fn to_number_list(values: &[f32; 3]) -> Value {
    Value::List(values.iter().copied().map(Value::Number).collect())
}

fn distance(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[derive(Debug, Clone)]
struct ShapeMetrics {
    points: Vec<[f32; 3]>,
    min: [f32; 3],
    max: [f32; 3],
}

impl ShapeMetrics {
    fn from_inputs(value: Option<&Value>) -> Option<Self> {
        let points = collect_points(value);
        if points.is_empty() {
            return None;
        }
        let (min, max) = bounding_box(&points);
        Some(Self { points, min, max })
    }

    fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    fn size(&self) -> [f32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    fn volume(&self) -> f32 {
        let size = self.size();
        size[0].abs() * size[1].abs() * size[2].abs()
    }

    fn area(&self) -> f32 {
        let size = self.size();
        let xy = size[0].abs() * size[1].abs();
        let yz = size[1].abs() * size[2].abs();
        let zx = size[0].abs() * size[2].abs();
        if yz <= EPSILON && zx <= EPSILON {
            xy
        } else {
            2.0 * (xy + yz + zx)
        }
    }

    fn sample_point(&self, uv: (f32, f32)) -> [f32; 3] {
        [
            self.min[0] + self.size()[0] * uv.0,
            self.min[1] + self.size()[1] * uv.1,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }
}

fn simple_inertia(size: [f32; 3], mass: f32) -> [f32; 3] {
    if mass.abs() <= EPSILON {
        return [0.0; 3];
    }
    [
        mass * (size[1].powi(2) + size[2].powi(2)) / 12.0,
        mass * (size[0].powi(2) + size[2].powi(2)) / 12.0,
        mass * (size[0].powi(2) + size[1].powi(2)) / 12.0,
    ]
}

fn simple_secondary(size: [f32; 3], mass: f32) -> [f32; 3] {
    if mass.abs() <= EPSILON {
        return [0.0; 3];
    }
    [
        mass * size[0].abs() * size[1].abs() / 12.0,
        mass * size[1].abs() * size[2].abs() / 12.0,
        mass * size[0].abs() * size[2].abs() / 12.0,
    ]
}

fn simple_gyration(inertia: [f32; 3], mass: f32) -> [f32; 3] {
    if mass.abs() <= EPSILON {
        return [0.0; 3];
    }
    [
        (inertia[0] / mass).abs().sqrt(),
        (inertia[1] / mass).abs().sqrt(),
        (inertia[2] / mass).abs().sqrt(),
    ]
}

fn plane_from_point(origin: [f32; 3]) -> Value {
    Value::List(vec![
        Value::Point(origin),
        Value::Point([origin[0] + 1.0, origin[1], origin[2]]),
        Value::Point([origin[0], origin[1] + 1.0, origin[2]]),
    ])
}

fn clamp_to_metrics(metrics: &ShapeMetrics, target: [f32; 3]) -> [f32; 3] {
    [
        target[0].clamp(metrics.min[0], metrics.max[0]),
        target[1].clamp(metrics.min[1], metrics.max[1]),
        target[2].clamp(metrics.min[2], metrics.max[2]),
    ]
}

fn uv_from_point(metrics: &ShapeMetrics, point: [f32; 3]) -> (f32, f32) {
    let size = metrics.size();
    let u = if size[0].abs() <= EPSILON {
        0.0
    } else {
        (point[0] - metrics.min[0]) / size[0]
    };
    let v = if size[1].abs() <= EPSILON {
        0.0
    } else {
        (point[1] - metrics.min[1]) / size[1]
    };
    (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
}

fn point_in_metrics(metrics: &ShapeMetrics, point: [f32; 3], strict: bool) -> bool {
    let tolerance = if strict { EPSILON } else { -EPSILON };
    point[0] >= metrics.min[0] - tolerance
        && point[0] <= metrics.max[0] + tolerance
        && point[1] >= metrics.min[1] - tolerance
        && point[1] <= metrics.max[1] + tolerance
        && point[2] >= metrics.min[2] - tolerance
        && point[2] <= metrics.max[2] + tolerance
}

fn boxes_overlap(a: &ShapeMetrics, b: &ShapeMetrics) -> bool {
    !(a.max[0] < b.min[0]
        || a.min[0] > b.max[0]
        || a.max[1] < b.min[1]
        || a.min[1] > b.max[1]
        || a.max[2] < b.min[2]
        || a.min[2] > b.max[2])
}

fn collect_point_grid(value: Option<&Value>) -> Option<Vec<Vec<[f32; 3]>>> {
    match value {
        Some(Value::List(rows)) if rows.iter().all(|row| matches!(row, Value::List(_))) => {
            let mut result = Vec::new();
            for row in rows {
                if let Value::List(entries) = row {
                    let mut parsed_row = Vec::new();
                    for entry in entries {
                        if let Some(point) = try_point(entry) {
                            parsed_row.push(point);
                        }
                    }
                    if !parsed_row.is_empty() {
                        result.push(parsed_row);
                    }
                }
            }
            if result.is_empty() { None } else { Some(result) }
        }
        _ => None,
    }
}

fn collect_points(value: Option<&Value>) -> Vec<[f32; 3]> {
    match value {
        Some(Value::Point(point)) | Some(Value::Vector(point)) => vec![*point],
        Some(Value::CurveLine { p1, p2 }) => vec![*p1, *p2],
        Some(Value::Surface { vertices, .. }) => vertices.clone(),
        Some(Value::List(values)) => values
            .iter()
            .flat_map(|value| collect_points(Some(value)))
            .collect(),
        _ => Vec::new(),
    }
}

fn try_point(value: &Value) -> Option<[f32; 3]> {
    match value {
        Value::Point(point) | Value::Vector(point) => Some(*point),
        Value::List(values) if values.len() >= 3 => {
            let x = coerce_number(Some(&values[0]), "punt").ok()?;
            let y = coerce_number(Some(&values[1]), "punt").ok()?;
            let z = coerce_number(Some(&values[2]), "punt").ok()?;
            Some([x, y, z])
        }
        Value::List(values) if !values.is_empty() => try_point(&values[0]),
        _ => None,
    }
}

fn create_wireframe(metrics: &ShapeMetrics) -> Vec<([f32; 3], [f32; 3])> {
    let corners = create_box_corners_points(metrics);
    let pairs = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    pairs
        .iter()
        .map(|(a, b)| (corners[*a], corners[*b]))
        .collect()
}

fn create_box_corners_points(metrics: &ShapeMetrics) -> Vec<[f32; 3]> {
    let mut corners = Vec::with_capacity(8);
    for &z in &[metrics.min[2], metrics.max[2]] {
        for &y in &[metrics.min[1], metrics.max[1]] {
            for &x in &[metrics.min[0], metrics.max[0]] {
                corners.push([x, y, z]);
            }
        }
    }
    corners
}

fn create_box_faces(corners: &[[f32; 3]]) -> Vec<Vec<[f32; 3]>> {
    vec![
        vec![corners[0], corners[1], corners[2], corners[3]],
        vec![corners[4], corners[5], corners[6], corners[7]],
        vec![corners[0], corners[1], corners[5], corners[4]],
        vec![corners[2], corners[3], corners[7], corners[6]],
        vec![corners[1], corners[2], corners[6], corners[5]],
        vec![corners[0], corners[3], corners[7], corners[4]],
    ]
}

fn bounding_box(points: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for point in points {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    (min, max)
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentError, ComponentKind};
    use crate::graph::node::MetaMap;
    use crate::graph::value::Value;

    fn cube_points() -> Value {
        Value::List(vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([1.0, 1.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
            Value::Point([0.0, 0.0, 1.0]),
            Value::Point([1.0, 0.0, 1.0]),
            Value::Point([1.0, 1.0, 1.0]),
            Value::Point([0.0, 1.0, 1.0]),
        ])
    }

    #[test]
    fn brep_edges_returns_wireframe_curves() {
        let component = ComponentKind::BrepEdges;
        let outputs = component
            .evaluate(&[cube_points()], &MetaMap::new())
            .expect("wireframe result");
        let edges = outputs
            .get(super::PIN_OUTPUT_NAKED_EDGES)
            .and_then(|value| value.expect_list().ok())
            .expect("edge list");
        assert_eq!(edges.len(), 12);
    }

    #[test]
    fn area_moments_produces_area_and_centroid() {
        let component = ComponentKind::AreaMoments;
        let outputs = component
            .evaluate(&[cube_points()], &MetaMap::new())
            .expect("moments");
        let area_value = outputs
            .get(super::PIN_OUTPUT_AREA)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        assert!(area_value > 0.0);
        let centroid = outputs
            .get(super::PIN_OUTPUT_CENTROID)
            .and_then(|value| value.expect_point().ok())
            .unwrap();
        assert_eq!(centroid, [0.5, 0.5, 0.5]);
    }

    #[test]
    fn surface_closest_point_clamps_target() {
        let component = ComponentKind::SurfaceClosestPoint;
        let point = Value::Point([3.0, 3.0, 3.0]);
        let outputs = component
            .evaluate(&[point, cube_points()], &MetaMap::new())
            .expect("closest point");
        let closest = outputs
            .get(super::PIN_OUTPUT_POINTS)
            .and_then(|value| value.expect_point().ok())
            .unwrap();
        assert_eq!(closest, [1.0, 1.0, 1.0]);
        let distance = outputs
            .get(super::PIN_OUTPUT_DISTANCE)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        assert!(distance > 0.0);
    }

    #[test]
    fn point_in_brep_detects_inclusion() {
        let component = ComponentKind::PointInBrep;
        let inside = component
            .evaluate(
                &[
                    cube_points(),
                    Value::Point([0.5, 0.5, 0.5]),
                    Value::Boolean(false),
                ],
                &MetaMap::new(),
            )
            .expect("inclusion");
        let flag = inside
            .get(super::PIN_OUTPUT_INSIDE)
            .and_then(|value| value.expect_boolean().ok())
            .unwrap();
        assert!(flag);

        let outside = component
            .evaluate(
                &[
                    cube_points(),
                    Value::Point([2.0, 2.0, 2.0]),
                    Value::Boolean(false),
                ],
                &MetaMap::new(),
            )
            .expect("exclusion");
        let flag = outside
            .get(super::PIN_OUTPUT_INSIDE)
            .and_then(|value| value.expect_boolean().ok())
            .unwrap();
        assert!(!flag);
    }

    #[test]
    fn deconstruct_box_reports_dimensions() {
        let component = ComponentKind::DeconstructBox;
        let outputs = component
            .evaluate(&[cube_points()], &MetaMap::new())
            .expect("deconstruct box");
        let x = outputs
            .get(super::PIN_OUTPUT_X_SIZE)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        let y = outputs
            .get(super::PIN_OUTPUT_Y_SIZE)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        let z = outputs
            .get(super::PIN_OUTPUT_Z_SIZE)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        assert_eq!((x, y, z), (1.0, 1.0, 1.0));
    }

    #[test]
    fn point_in_breps_reports_index() {
        let component = ComponentKind::PointInBreps;
        let outputs = component
            .evaluate(
                &[
                    Value::List(vec![cube_points(), cube_points()]),
                    Value::Point([0.2, 0.2, 0.2]),
                    Value::Boolean(false),
                ],
                &MetaMap::new(),
            )
            .expect("point in breps");
        let index = outputs
            .get(super::PIN_OUTPUT_INDEX)
            .and_then(|value| value.expect_number().ok())
            .unwrap();
        assert_eq!(index, 0.0);
    }

    #[test]
    fn osculating_circles_return_two_curves() {
        let component = ComponentKind::OsculatingCircles;
        let outputs = component
            .evaluate(&[cube_points(), Value::Point([0.5, 0.5, 0.0])], &MetaMap::new())
            .expect("osc circles");
        let c1 = outputs
            .get(super::PIN_OUTPUT_CIRCLE_ONE)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        let c2 = outputs
            .get(super::PIN_OUTPUT_CIRCLE_TWO)
            .and_then(|value| value.expect_list().ok())
            .unwrap();
        assert_eq!(c1.len(), 2);
        assert_eq!(c2.len(), 2);
    }

    #[test]
    fn evaluate_box_requires_box_input() {
        let component = ComponentKind::EvaluateBox;
        let err = component
            .evaluate(&[Value::List(Vec::new())], &MetaMap::new())
            .unwrap_err();
        assert!(matches!(err, ComponentError::Message(_)));
    }
}
