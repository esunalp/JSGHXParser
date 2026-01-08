//! Implementaties van Grasshopper "Transform → Affine" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

const PIN_OUTPUT_GEOMETRY: &str = "G";
const PIN_OUTPUT_TRANSFORM: &str = "X";

const EPSILON: f64 = 1e-9;

/// Beschikbare componenten binnen Transform → Affine.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    ProjectAlong,
    OrientDirection,
    OrientDirectionGeometry,
    RectangleMapping,
    Project,
    ProjectGeometry,
    ScaleNonUniform,
    Shear,
    ShearWithTransform,
    CameraObscura,
    Scale,
    ScaleGeometry,
    TriangleMapping,
    ScaleNonUniformVariant,
    ShearAngle,
    ShearAngleWithTransform,
    BoxMapping,
    ProjectVariant,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Registraties van alle Transform → Affine componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{06d7bc4a-ba3e-4445-8ab5-079613b52f28}"],
        names: &["Project Along", "ProjectA"],
        kind: ComponentKind::ProjectAlong,
    },
    Registration {
        guids: &["{1602b2cc-007c-4b79-8926-0067c6184e44}"],
        names: &["Orient Direction", "Orient"],
        kind: ComponentKind::OrientDirection,
    },
    Registration {
        guids: &["{4041be93-6746-4cdb-aa95-929bff544fb0}"],
        names: &["Orient Direction", "Orient"],
        kind: ComponentKind::OrientDirectionGeometry,
    },
    Registration {
        guids: &["{17d40004-489e-42d9-ad10-857f7b436801}"],
        names: &["Rectangle Mapping", "RecMap"],
        kind: ComponentKind::RectangleMapping,
    },
    Registration {
        guids: &["{23285717-156c-468f-a691-b242488c06a6}"],
        names: &["Project", "Project"],
        kind: ComponentKind::Project,
    },
    Registration {
        guids: &["{24e913c9-7530-436d-b81d-bc3aa27296a4}"],
        names: &["Project", "Project"],
        kind: ComponentKind::ProjectGeometry,
    },
    Registration {
        guids: &["{290f418a-65ee-406a-a9d0-35699815b512}"],
        names: &["Scale NU", "Scale NU"],
        kind: ComponentKind::ScaleNonUniform,
    },
    Registration {
        guids: &["{7753fb03-c1f1-4dbe-8557-f01e23aa3b20}"],
        names: &["Scale NU", "Scale NU"],
        kind: ComponentKind::ScaleNonUniformVariant,
    },
    Registration {
        guids: &["{3ae3a462-38fb-4d49-9f86-7558dfed7c3e}"],
        names: &["Shear", "Shear"],
        kind: ComponentKind::Shear,
    },
    Registration {
        guids: &["{5a27203a-e05f-4eea-b80f-a5f29a00fdf2}"],
        names: &["Shear", "Shear"],
        kind: ComponentKind::ShearWithTransform,
    },
    Registration {
        guids: &["{407e35c6-7c40-4652-bd80-fde1eb7ec034}"],
        names: &["Camera Obscura", "CO"],
        kind: ComponentKind::CameraObscura,
    },
    Registration {
        guids: &["{4d2a06bd-4b0f-4c65-9ee0-4220e4c01703}"],
        names: &["Scale", "Scale"],
        kind: ComponentKind::Scale,
    },
    Registration {
        guids: &["{4f0dfac8-6c61-40ef-ad41-aad84533f382}"],
        names: &["Scale", "Scale"],
        kind: ComponentKind::ScaleGeometry,
    },
    Registration {
        guids: &["{61d81100-c4d3-462d-8b51-d951c0ae32db}"],
        names: &["Triangle Mapping", "TriMap"],
        kind: ComponentKind::TriangleMapping,
    },
    Registration {
        guids: &["{77bfb6a1-0305-4645-b309-cd6dbf1205d7}"],
        names: &["Shear Angle", "Shear"],
        kind: ComponentKind::ShearAngle,
    },
    Registration {
        guids: &["{f19ee36c-f21f-4e25-be4c-4ca4b30eda0d}"],
        names: &["Shear Angle", "Shear"],
        kind: ComponentKind::ShearAngleWithTransform,
    },
    Registration {
        guids: &["{8465bcce-9e0a-4cf4-bbda-1a7ce5681e10}"],
        names: &["Box Mapping", "BoxMap"],
        kind: ComponentKind::BoxMapping,
    },
    Registration {
        guids: &["{9025f4ca-159f-4c54-958b-0aad379dae77}"],
        names: &["Project", "Project"],
        kind: ComponentKind::ProjectVariant,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::ProjectAlong => evaluate_project_along(inputs),
            Self::OrientDirection => evaluate_orient_direction(inputs, true),
            Self::OrientDirectionGeometry => evaluate_orient_direction(inputs, false),
            Self::RectangleMapping => evaluate_rectangle_mapping(inputs),
            Self::Project | Self::ProjectVariant => evaluate_project(inputs, true),
            Self::ProjectGeometry => evaluate_project(inputs, false),
            Self::ScaleNonUniform | Self::ScaleNonUniformVariant => {
                evaluate_scale_non_uniform(inputs, true)
            }
            Self::Shear => evaluate_shear(inputs, false),
            Self::ShearWithTransform => evaluate_shear(inputs, true),
            Self::CameraObscura => evaluate_camera_obscura(inputs),
            Self::Scale => evaluate_scale(inputs, true),
            Self::ScaleGeometry => evaluate_scale(inputs, false),
            Self::TriangleMapping => evaluate_triangle_mapping(inputs),
            Self::ShearAngle => evaluate_shear_angle(inputs, false),
            Self::ShearAngleWithTransform => evaluate_shear_angle(inputs, true),
            Self::BoxMapping => evaluate_box_mapping(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::ProjectAlong => "Project Along",
            Self::OrientDirection | Self::OrientDirectionGeometry => "Orient Direction",
            Self::RectangleMapping => "Rectangle Mapping",
            Self::Project | Self::ProjectGeometry | Self::ProjectVariant => "Project",
            Self::ScaleNonUniform | Self::ScaleNonUniformVariant => "Scale NU",
            Self::Shear | Self::ShearWithTransform => "Shear",
            Self::CameraObscura => "Camera Obscura",
            Self::Scale | Self::ScaleGeometry => "Scale",
            Self::TriangleMapping => "Triangle Mapping",
            Self::ShearAngle | Self::ShearAngleWithTransform => "Shear Angle",
            Self::BoxMapping => "Box Mapping",
        }
    }
}

fn evaluate_project_along(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Project Along vereist geometrie, een vlak en een richting",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Project Along vereist geometrie"))?;
    let plane = coerce_plane(inputs.get(1), "Project Along vlak")?;
    let direction = coerce_direction(inputs.get(2), "Project Along richting")?;

    let mut point_fn = |point: [f64; 3]| plane.project_along(point, direction);
    let mut vector_fn = |vector: [f64; 3]| {
        let along = dot(vector, plane.normal());
        subtract(vector, scale(direction, along))
    };
    let transformed = map_geometry(&geometry, &mut point_fn, &mut vector_fn);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);
    outputs.insert(
        PIN_OUTPUT_TRANSFORM.to_owned(),
        Value::List(vec![
            Value::Text("Project Along".into()),
            Value::Point(plane.origin),
            Value::Vector(direction),
        ]),
    );
    Ok(outputs)
}

fn evaluate_orient_direction(inputs: &[Value], include_transform: bool) -> ComponentResult {
    if inputs.len() < 5 {
        return Err(ComponentError::new(
            "Orient Direction vereist geometrie, punten en richtingen",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Orient Direction vereist geometrie"))?;
    let point_a = coerce_point(inputs.get(1), "Orient Direction punt A")?;
    let dir_a = coerce_direction(inputs.get(2), "Orient Direction richting A")?;
    let point_b = coerce_point(inputs.get(3), "Orient Direction punt B")?;
    let dir_b = coerce_direction(inputs.get(4), "Orient Direction richting B")?;

    let cross_dirs = cross(dir_a, dir_b);
    let (axis, angle) = if length_squared(cross_dirs) < EPSILON {
        let dot_dirs = clamp(dot(dir_a, dir_b), -1.0, 1.0);
        if dot_dirs < -0.999_999 {
            let fallback = orthogonal_vector(dir_a);
            (fallback, std::f64::consts::PI)
        } else {
            ([0.0, 0.0, 1.0], 0.0)
        }
    } else {
        let axis = normalize(cross_dirs);
        let dot_dirs = clamp(dot(dir_a, dir_b), -1.0, 1.0);
        (axis, dot_dirs.acos())
    };

    let mut point_fn = |point: [f64; 3]| {
        let relative = subtract(point, point_a);
        let rotated = rotate_vector(relative, axis, angle);
        add(point_b, rotated)
    };
    let mut vector_fn = |vector: [f64; 3]| rotate_vector(vector, axis, angle);
    let transformed = map_geometry(&geometry, &mut point_fn, &mut vector_fn);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);
    if include_transform {
        outputs.insert(
            PIN_OUTPUT_TRANSFORM.to_owned(),
            Value::List(vec![
                Value::Text("Orient Direction".into()),
                Value::Point(point_a),
                Value::Point(point_b),
                Value::Vector(axis),
                Value::Number(angle),
            ]),
        );
    }
    Ok(outputs)
}

fn evaluate_rectangle_mapping(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Rectangle Mapping vereist geometrie en twee rechthoeken",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Rectangle Mapping vereist geometrie"))?;
    let source_points = collect_points(inputs.get(1));
    let target_points = collect_points(inputs.get(2));
    if source_points.len() < 2 || target_points.len() < 2 {
        return Err(ComponentError::new(
            "Rectangle Mapping verwacht minstens twee punten per rechthoek",
        ));
    }

    let source_bounds = Bounds::from_points(&source_points);
    let target_bounds = Bounds::from_points(&target_points);
    let ratios = target_bounds.ratios(&source_bounds);

    let mut point_fn = |point: [f64; 3]| map_between_bounds(point, &source_bounds, &target_bounds);
    let mut vector_fn = |vector: [f64; 3]| scale_components(vector, ratios);
    let transformed = map_geometry(&geometry, &mut point_fn, &mut vector_fn);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);
    outputs.insert(
        PIN_OUTPUT_TRANSFORM.to_owned(),
        Value::List(vec![
            Value::Text("Rectangle Mapping".into()),
            Value::Point(source_bounds.min),
            Value::Point(target_bounds.min),
        ]),
    );
    Ok(outputs)
}

fn evaluate_project(inputs: &[Value], include_transform: bool) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new("Project vereist geometrie en een vlak"));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Project vereist geometrie"))?;
    let plane = coerce_plane(inputs.get(1), "Project vlak")?;

    let mut point_fn = |point: [f64; 3]| plane.project(point);
    let mut vector_fn = |vector: [f64; 3]| {
        let normal_component = scale(plane.normal(), dot(vector, plane.normal()));
        subtract(vector, normal_component)
    };
    let transformed = map_geometry(&geometry, &mut point_fn, &mut vector_fn);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);
    if include_transform {
        outputs.insert(
            PIN_OUTPUT_TRANSFORM.to_owned(),
            Value::List(vec![
                Value::Text("Project".into()),
                Value::Point(plane.origin),
                Value::Vector(plane.normal()),
            ]),
        );
    }
    Ok(outputs)
}

fn evaluate_scale_non_uniform(inputs: &[Value], include_transform: bool) -> ComponentResult {
    if inputs.len() < 5 {
        return Err(ComponentError::new(
            "Scale NU vereist geometrie, vlak en schalen",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Scale NU vereist geometrie"))?;
    let plane = coerce_plane(inputs.get(1), "Scale NU vlak")?;
    let scale_x = coerce_number(inputs.get(2), "Scale NU X")?;
    let scale_y = coerce_number(inputs.get(3), "Scale NU Y")?;
    let scale_z = coerce_number(inputs.get(4), "Scale NU Z")?;

    let mut point_fn = |point: [f64; 3]| {
        let local = plane.to_local(point);
        plane.from_local([local[0] * scale_x, local[1] * scale_y, local[2] * scale_z])
    };
    let mut vector_fn = |vector: [f64; 3]| {
        let local = plane.vector_to_local(vector);
        plane.vector_from_local([local[0] * scale_x, local[1] * scale_y, local[2] * scale_z])
    };
    let transformed = map_geometry(&geometry, &mut point_fn, &mut vector_fn);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);
    if include_transform {
        outputs.insert(
            PIN_OUTPUT_TRANSFORM.to_owned(),
            Value::List(vec![
                Value::Text("Scale NU".into()),
                Value::Point(plane.origin),
                Value::Vector([scale_x, scale_y, scale_z]),
            ]),
        );
    }
    Ok(outputs)
}

fn evaluate_shear(inputs: &[Value], include_transform: bool) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Shear vereist geometrie, vlak, grip en target",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Shear vereist geometrie"))?;
    let plane = coerce_plane(inputs.get(1), "Shear vlak")?;
    let grip = coerce_point(inputs.get(2), "Shear grip")?;
    let target = coerce_point(inputs.get(3), "Shear target")?;

    let grip_local = plane.to_local(grip);
    let delta_local = subtract(plane.to_local(target), grip_local);
    let reference_height = grip_local[2].abs().max(1.0);

    let mut point_fn = |point: [f64; 3]| {
        let mut local = plane.to_local(point);
        let factor = (local[2] / reference_height).clamp(-2.0, 2.0);
        local[0] += delta_local[0] * factor;
        local[1] += delta_local[1] * factor;
        plane.from_local(local)
    };
    let mut vector_fn = |vector: [f64; 3]| {
        let mut local = plane.vector_to_local(vector);
        let factor = (local[2] / reference_height).clamp(-2.0, 2.0);
        local[0] += delta_local[0] * factor;
        local[1] += delta_local[1] * factor;
        plane.vector_from_local(local)
    };
    let transformed = map_geometry(&geometry, &mut point_fn, &mut vector_fn);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);
    if include_transform {
        outputs.insert(
            PIN_OUTPUT_TRANSFORM.to_owned(),
            Value::List(vec![
                Value::Text("Shear".into()),
                Value::Point(grip),
                Value::Point(target),
            ]),
        );
    }
    Ok(outputs)
}

fn evaluate_camera_obscura(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Camera Obscura vereist geometrie, punt en factor",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Camera Obscura vereist geometrie"))?;
    let focus = coerce_point(inputs.get(1), "Camera Obscura punt")?;
    let factor = coerce_number(inputs.get(2), "Camera Obscura factor")?;

    let mut point_fn = |point: [f64; 3]| add(focus, scale(subtract(point, focus), factor));
    let mut vector_fn = |vector: [f64; 3]| scale(vector, factor);
    let transformed = map_geometry(&geometry, &mut point_fn, &mut vector_fn);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);
    outputs.insert(
        PIN_OUTPUT_TRANSFORM.to_owned(),
        Value::List(vec![
            Value::Text("Camera Obscura".into()),
            Value::Point(focus),
            Value::Number(factor),
        ]),
    );
    Ok(outputs)
}

fn evaluate_scale(inputs: &[Value], include_transform: bool) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Scale vereist geometrie, centrum en factor",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Scale vereist geometrie"))?;
    let center = coerce_point(inputs.get(1), "Scale centrum")?;
    let factor = coerce_number(inputs.get(2), "Scale factor")?;

    let mut point_fn = |point: [f64; 3]| add(center, scale(subtract(point, center), factor));
    let mut vector_fn = |vector: [f64; 3]| scale(vector, factor);
    let transformed = map_geometry(&geometry, &mut point_fn, &mut vector_fn);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);
    if include_transform {
        outputs.insert(
            PIN_OUTPUT_TRANSFORM.to_owned(),
            Value::List(vec![
                Value::Text("Scale".into()),
                Value::Point(center),
                Value::Number(factor),
            ]),
        );
    }
    Ok(outputs)
}

fn evaluate_triangle_mapping(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Triangle Mapping vereist geometrie en twee driehoeken",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Triangle Mapping vereist geometrie"))?;
    let source_triangle = coerce_triangle(inputs.get(1), "Triangle Mapping bron")?;
    let target_triangle = coerce_triangle(inputs.get(2), "Triangle Mapping doel")?;
    let mapper = TriangleMap::new(source_triangle, target_triangle);

    let mut point_fn = |point: [f64; 3]| mapper.map_point(point);
    let mut vector_fn = |vector: [f64; 3]| mapper.map_vector(vector);
    let transformed = map_geometry(&geometry, &mut point_fn, &mut vector_fn);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);
    outputs.insert(
        PIN_OUTPUT_TRANSFORM.to_owned(),
        Value::List(vec![
            Value::Text("Triangle Mapping".into()),
            Value::Point(source_triangle[0]),
            Value::Point(target_triangle[0]),
        ]),
    );
    Ok(outputs)
}

fn evaluate_shear_angle(inputs: &[Value], include_transform: bool) -> ComponentResult {
    if inputs.len() < 4 {
        return Err(ComponentError::new(
            "Shear Angle vereist geometrie, vlak en twee hoeken",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Shear Angle vereist geometrie"))?;
    let plane = coerce_plane(inputs.get(1), "Shear Angle vlak")?;
    let angle_x = coerce_number(inputs.get(2), "Shear Angle Ax")?;
    let angle_y = coerce_number(inputs.get(3), "Shear Angle Ay")?;
    let shear_x = angle_x.tan();
    let shear_y = angle_y.tan();

    let mut point_fn = |point: [f64; 3]| {
        let mut local = plane.to_local(point);
        local[0] += local[2] * shear_x;
        local[1] += local[2] * shear_y;
        plane.from_local(local)
    };
    let mut vector_fn = |vector: [f64; 3]| {
        let mut local = plane.vector_to_local(vector);
        local[0] += local[2] * shear_x;
        local[1] += local[2] * shear_y;
        plane.vector_from_local(local)
    };
    let transformed = map_geometry(&geometry, &mut point_fn, &mut vector_fn);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);
    if include_transform {
        outputs.insert(
            PIN_OUTPUT_TRANSFORM.to_owned(),
            Value::List(vec![
                Value::Text("Shear Angle".into()),
                Value::Number(angle_x),
                Value::Number(angle_y),
            ]),
        );
    }
    Ok(outputs)
}

fn evaluate_box_mapping(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 3 {
        return Err(ComponentError::new(
            "Box Mapping vereist geometrie en twee boxen",
        ));
    }

    let geometry = inputs
        .get(0)
        .cloned()
        .ok_or_else(|| ComponentError::new("Box Mapping vereist geometrie"))?;
    let source_points = collect_points(inputs.get(1));
    let target_points = collect_points(inputs.get(2));
    if source_points.len() < 2 || target_points.len() < 2 {
        return Err(ComponentError::new(
            "Box Mapping verwacht minstens twee punten per box",
        ));
    }

    let source_bounds = Bounds::from_points(&source_points);
    let target_bounds = Bounds::from_points(&target_points);
    let ratios = target_bounds.ratios(&source_bounds);

    let mut point_fn = |point: [f64; 3]| map_between_bounds(point, &source_bounds, &target_bounds);
    let mut vector_fn = |vector: [f64; 3]| scale_components(vector, ratios);
    let transformed = map_geometry(&geometry, &mut point_fn, &mut vector_fn);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_OUTPUT_GEOMETRY.to_owned(), transformed);
    outputs.insert(
        PIN_OUTPUT_TRANSFORM.to_owned(),
        Value::List(vec![
            Value::Text("Box Mapping".into()),
            Value::Point(source_bounds.min),
            Value::Point(target_bounds.min),
        ]),
    );
    Ok(outputs)
}

fn map_geometry<FPoint, FVector>(
    value: &Value,
    point_fn: &mut FPoint,
    vector_fn: &mut FVector,
) -> Value
where
    FPoint: FnMut([f64; 3]) -> [f64; 3],
    FVector: FnMut([f64; 3]) -> [f64; 3],
{
    match value {
        Value::Point(point) => Value::Point(point_fn(*point)),
        Value::Vector(vector) => Value::Vector(vector_fn(*vector)),
        Value::CurveLine { p1, p2 } => Value::CurveLine {
            p1: point_fn(*p1),
            p2: point_fn(*p2),
        },
        Value::Surface { vertices, faces } => Value::Surface {
            vertices: vertices.iter().map(|v| point_fn(*v)).collect(),
            faces: faces.clone(),
        },
        Value::Mesh {
            vertices,
            indices,
            normals,
            uvs,
            diagnostics,
        } => {
            // Transform vertex positions using the point function
            let transformed_vertices: Vec<[f64; 3]> =
                vertices.iter().map(|v| point_fn(*v)).collect();

            // Transform normals using the vector function, then re-normalize
            // to handle non-uniform scaling correctly.
            // The vector_fn already handles the directional transformation,
            // but after non-uniform scaling, normals may no longer be unit length.
            let transformed_normals = normals.as_ref().map(|norms| {
                norms
                    .iter()
                    .map(|n| {
                        let transformed = vector_fn(*n);
                        // Re-normalize to ensure unit length after transformation.
                        // For non-uniform scaling, the normal direction changes and
                        // must be normalized. If the normal becomes degenerate (zero
                        // length after transform), preserve the original direction.
                        let len_sq = transformed[0] * transformed[0]
                            + transformed[1] * transformed[1]
                            + transformed[2] * transformed[2];
                        if len_sq > EPSILON * EPSILON {
                            let len = len_sq.sqrt();
                            [transformed[0] / len, transformed[1] / len, transformed[2] / len]
                        } else {
                            // Degenerate case: normal collapsed to zero.
                            // Keep original normal as fallback.
                            *n
                        }
                    })
                    .collect()
            });

            Value::Mesh {
                vertices: transformed_vertices,
                indices: indices.clone(),
                normals: transformed_normals,
                // UVs are texture coordinates and remain unchanged by spatial transforms
                uvs: uvs.clone(),
                // Diagnostics remain unchanged as they describe the original mesh quality
                diagnostics: diagnostics.clone(),
            }
        }
        Value::List(values) => {
            let mut mapped = Vec::with_capacity(values.len());
            for value in values {
                mapped.push(map_geometry(value, point_fn, vector_fn));
            }
            Value::List(mapped)
        }
        _ => value.clone(),
    }
}

#[derive(Debug, Clone, Copy)]
struct Bounds {
    min: [f64; 3],
    max: [f64; 3],
}

impl Bounds {
    fn from_points(points: &[[f64; 3]]) -> Self {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for point in points {
            for axis in 0..3 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        }
        Self { min, max }
    }

    fn size(&self) -> [f64; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    fn ratios(&self, other: &Self) -> [f64; 3] {
        let self_size = self.size();
        let other_size = other.size();
        [
            scale_ratio(self_size[0], other_size[0]),
            scale_ratio(self_size[1], other_size[1]),
            scale_ratio(self_size[2], other_size[2]),
        ]
    }
}

fn scale_ratio(target: f64, source: f64) -> f64 {
    if source.abs() < EPSILON {
        1.0
    } else {
        target / source
    }
}

fn map_between_bounds(point: [f64; 3], source: &Bounds, target: &Bounds) -> [f64; 3] {
    let source_size = source.size();
    let target_size = target.size();
    [
        target.min[0]
            + normalized_coordinate(point[0], source.min[0], source_size[0]) * target_size[0],
        target.min[1]
            + normalized_coordinate(point[1], source.min[1], source_size[1]) * target_size[1],
        target.min[2]
            + normalized_coordinate(point[2], source.min[2], source_size[2]) * target_size[2],
    ]
}

fn normalized_coordinate(value: f64, min: f64, size: f64) -> f64 {
    if size.abs() < EPSILON {
        0.0
    } else {
        (value - min) / size
    }
}

fn scale_components(vector: [f64; 3], ratios: [f64; 3]) -> [f64; 3] {
    [
        vector[0] * ratios[0],
        vector[1] * ratios[1],
        vector[2] * ratios[2],
    ]
}

#[derive(Debug, Clone, Copy)]
struct TriangleMap {
    source: [[f64; 3]; 3],
    target: [[f64; 3]; 3],
    source_normal: [f64; 3],
    target_normal: [f64; 3],
}

impl TriangleMap {
    fn new(source: [[f64; 3]; 3], target: [[f64; 3]; 3]) -> Self {
        let source_normal = cross(
            subtract(source[1], source[0]),
            subtract(source[2], source[0]),
        );
        let target_normal = cross(
            subtract(target[1], target[0]),
            subtract(target[2], target[0]),
        );
        Self {
            source,
            target,
            source_normal,
            target_normal,
        }
    }

    fn map_point(&self, point: [f64; 3]) -> [f64; 3] {
        let bary = barycentric(point, self.source);
        add(
            add(
                scale(self.target[0], bary[0]),
                scale(self.target[1], bary[1]),
            ),
            scale(self.target[2], bary[2]),
        )
    }

    fn map_vector(&self, vector: [f64; 3]) -> [f64; 3] {
        let a = subtract(self.source[1], self.source[0]);
        let b = subtract(self.source[2], self.source[0]);
        let c = self.source_normal;
        let Some(coeffs) = solve_basis(a, b, c, vector) else {
            return vector;
        };
        let ta = subtract(self.target[1], self.target[0]);
        let tb = subtract(self.target[2], self.target[0]);
        let tc = self.target_normal;
        add(
            add(scale(ta, coeffs[0]), scale(tb, coeffs[1])),
            scale(
                tc,
                coeffs[2] * normal_ratio(self.source_normal, self.target_normal),
            ),
        )
    }
}

fn barycentric(point: [f64; 3], triangle: [[f64; 3]; 3]) -> [f64; 3] {
    let a = triangle[0];
    let b = triangle[1];
    let c = triangle[2];
    let v0 = subtract(b, a);
    let v1 = subtract(c, a);
    let v2 = subtract(point, a);
    let d00 = dot(v0, v0);
    let d01 = dot(v0, v1);
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);
    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < EPSILON {
        return [1.0, 0.0, 0.0];
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    [u, v, w]
}

fn solve_basis(a: [f64; 3], b: [f64; 3], c: [f64; 3], vector: [f64; 3]) -> Option<[f64; 3]> {
    let denom = dot(a, cross(b, c));
    if denom.abs() < EPSILON {
        return None;
    }
    let x = dot(vector, cross(b, c)) / denom;
    let y = dot(vector, cross(c, a)) / denom;
    let z = dot(vector, cross(a, b)) / denom;
    Some([x, y, z])
}

fn normal_ratio(source: [f64; 3], target: [f64; 3]) -> f64 {
    let source_len = length(source).max(EPSILON);
    let target_len = length(target).max(EPSILON);
    target_len / source_len
}

#[derive(Debug, Clone, Copy)]
struct Plane {
    origin: [f64; 3],
    x_axis: [f64; 3],
    y_axis: [f64; 3],
    z_axis: [f64; 3],
}

impl Plane {
    fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            z_axis: [0.0, 0.0, 1.0],
        }
    }

    fn from_points(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Self {
        let x_axis = safe_normalize(subtract(b, a))
            .map(|(axis, _)| axis)
            .unwrap_or([1.0, 0.0, 0.0]);
        let raw_y = subtract(c, a);
        let y_projection = subtract(raw_y, scale(x_axis, dot(raw_y, x_axis)));
        let y_axis = safe_normalize(y_projection)
            .map(|(axis, _)| axis)
            .unwrap_or([0.0, 1.0, 0.0]);
        let z_axis = normalize(cross(x_axis, y_axis));
        Self::normalize_axes(a, x_axis, y_axis, z_axis)
    }

    fn from_origin(origin: [f64; 3]) -> Self {
        Self {
            origin,
            ..Self::default()
        }
    }

    fn normalize_axes(
        origin: [f64; 3],
        x_axis: [f64; 3],
        y_axis: [f64; 3],
        z_axis: [f64; 3],
    ) -> Self {
        let x_axis = normalize(x_axis);
        let mut y_axis = subtract(y_axis, scale(x_axis, dot(y_axis, x_axis)));
        if length_squared(y_axis) < EPSILON {
            y_axis = orthogonal_vector(x_axis);
        }
        let y_axis = normalize(y_axis);
        let z_axis = normalize(z_axis);
        Self {
            origin,
            x_axis,
            y_axis,
            z_axis,
        }
    }

    fn normal(&self) -> [f64; 3] {
        self.z_axis
    }

    fn project(&self, point: [f64; 3]) -> [f64; 3] {
        let distance = dot(subtract(point, self.origin), self.z_axis);
        subtract(point, scale(self.z_axis, distance))
    }

    fn project_along(&self, point: [f64; 3], direction: [f64; 3]) -> [f64; 3] {
        let numerator = dot(subtract(self.origin, point), self.z_axis);
        let denominator = dot(direction, self.z_axis);
        if denominator.abs() < EPSILON {
            return self.project(point);
        }
        add(point, scale(direction, numerator / denominator))
    }

    fn to_local(&self, point: [f64; 3]) -> [f64; 3] {
        let delta = subtract(point, self.origin);
        [
            dot(delta, self.x_axis),
            dot(delta, self.y_axis),
            dot(delta, self.z_axis),
        ]
    }

    fn from_local(&self, coords: [f64; 3]) -> [f64; 3] {
        add(
            add(
                add(self.origin, scale(self.x_axis, coords[0])),
                scale(self.y_axis, coords[1]),
            ),
            scale(self.z_axis, coords[2]),
        )
    }

    fn vector_to_local(&self, vector: [f64; 3]) -> [f64; 3] {
        [
            dot(vector, self.x_axis),
            dot(vector, self.y_axis),
            dot(vector, self.z_axis),
        ]
    }

    fn vector_from_local(&self, coords: [f64; 3]) -> [f64; 3] {
        add(
            add(scale(self.x_axis, coords[0]), scale(self.y_axis, coords[1])),
            scale(self.z_axis, coords[2]),
        )
    }
}

fn coerce_triangle(value: Option<&Value>, context: &str) -> Result<[[f64; 3]; 3], ComponentError> {
    let points = collect_points(value);
    if points.len() < 3 {
        return Err(ComponentError::new(format!(
            "{} verwacht minstens drie punten",
            context
        )));
    }
    Ok([points[0], points[1], points[2]])
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
            if length_squared(direction) < EPSILON {
                Ok(Plane::from_origin(origin))
            } else {
                let x_axis = normalize(direction);
                let y_axis = orthogonal_vector(x_axis);
                let z_axis = normalize(cross(x_axis, y_axis));
                Ok(Plane::normalize_axes(origin, x_axis, y_axis, z_axis))
            }
        }
        Some(Value::List(values)) if values.len() == 1 => coerce_plane(values.get(0), context),
        Some(Value::Point(point)) => Ok(Plane::from_origin(*point)),
        Some(Value::Vector(vector)) => {
            let normal = if length_squared(*vector) < EPSILON {
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

fn coerce_direction(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    let vector = coerce_vector(value, context)?;
    safe_normalize(vector)
        .map(|(dir, _)| dir)
        .ok_or_else(|| ComponentError::new(format!("{} verwacht een niet-nul vector", context)))
}

fn coerce_point(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Some(Value::Point(point)) => Ok(*point),
        Some(Value::Vector(vector)) => Ok(*vector),
        Some(Value::List(values)) if values.len() == 1 => coerce_point(values.get(0), context),
        Some(Value::List(values)) if values.len() >= 3 => {
            let x = coerce_number(values.get(0), context)?;
            let y = coerce_number(values.get(1), context)?;
            let z = coerce_number(values.get(2), context)?;
            Ok([x, y, z])
        }
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een punt, kreeg {}",
            context,
            other.kind()
        ))),
        None => Err(ComponentError::new(format!("{} vereist een punt", context))),
    }
}

fn coerce_vector(value: Option<&Value>, context: &str) -> Result<[f64; 3], ComponentError> {
    match value {
        Some(Value::Vector(vector)) => Ok(*vector),
        Some(Value::Point(point)) => Ok(*point),
        Some(Value::List(values)) if values.len() == 1 => coerce_vector(values.get(0), context),
        Some(Value::List(values)) if values.len() >= 3 => {
            let x = coerce_number(values.get(0), context)?;
            let y = coerce_number(values.get(1), context)?;
            let z = coerce_number(values.get(2), context)?;
            Ok([x, y, z])
        }
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een vector, kreeg {}",
            context,
            other.kind()
        ))),
        None => Err(ComponentError::new(format!(
            "{} vereist een vector",
            context
        ))),
    }
}

fn coerce_number(value: Option<&Value>, context: &str) -> Result<f64, ComponentError> {
    match value {
        Some(Value::Number(number)) => Ok(*number),
        Some(Value::Boolean(flag)) => Ok(if *flag { 1.0 } else { 0.0 }),
        Some(Value::List(values)) if !values.is_empty() => coerce_number(values.get(0), context),
        Some(other) => Err(ComponentError::new(format!(
            "{} verwacht een numerieke waarde, kreeg {}",
            context,
            other.kind()
        ))),
        None => Err(ComponentError::new(format!(
            "{} vereist een numerieke waarde",
            context
        ))),
    }
}

fn collect_points(value: Option<&Value>) -> Vec<[f64; 3]> {
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

fn rotate_vector(vector: [f64; 3], axis: [f64; 3], angle: f64) -> [f64; 3] {
    if angle.abs() < EPSILON {
        return vector;
    }
    let axis = normalize(axis);
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();
    add(
        add(
            scale(vector, cos_angle),
            scale(cross(axis, vector), sin_angle),
        ),
        scale(axis, dot(axis, vector) * (1.0 - cos_angle)),
    )
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(vector: [f64; 3], factor: f64) -> [f64; 3] {
    [vector[0] * factor, vector[1] * factor, vector[2] * factor]
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

fn length(vector: [f64; 3]) -> f64 {
    length_squared(vector).sqrt()
}

fn length_squared(vector: [f64; 3]) -> f64 {
    dot(vector, vector)
}

fn normalize(vector: [f64; 3]) -> [f64; 3] {
    safe_normalize(vector)
        .map(|(unit, _)| unit)
        .unwrap_or([1.0, 0.0, 0.0])
}

fn safe_normalize(vector: [f64; 3]) -> Option<([f64; 3], f64)> {
    let length = length(vector);
    if length < EPSILON {
        None
    } else {
        Some((
            [vector[0] / length, vector[1] / length, vector[2] / length],
            length,
        ))
    }
}

fn orthogonal_vector(vector: [f64; 3]) -> [f64; 3] {
    if vector[0].abs() < vector[1].abs() && vector[0].abs() < vector[2].abs() {
        normalize([0.0, -vector[2], vector[1]])
    } else if vector[1].abs() < vector[2].abs() {
        normalize([-vector[2], 0.0, vector[0]])
    } else {
        normalize([-vector[1], vector[0], 0.0])
    }
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}