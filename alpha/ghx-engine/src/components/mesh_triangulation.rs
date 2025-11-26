//! Grasshopper components for mesh triangulation, delaunay, and voroi operations.

use std::collections::BTreeMap;

use crate::components::coerce;
use crate::graph::node::MetaMap;
use crate::graph::value::Value;

use super::{Component, ComponentError, ComponentResult};

#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    DelaunayMesh,
    FacetDome,
    QuadRemesh,
    Substrate,
    Proximity2D,
    VoronoiCell,
    QuadTree,
    TriRemesh,
    ConvexHull,
    VoronoiGroups,
    Voronoi,
    OcTree,
    Voronoi3D,
    MetaBallTCustom,
    MetaBallT,
    DelaunayEdges,
    MetaBall,
    Proximity3D,
}

impl ComponentKind {
    pub fn evaluate(self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::DelaunayMesh => DelaunayMesh.evaluate(inputs, meta),
            Self::FacetDome => FacetDome.evaluate(inputs, meta),
            Self::QuadRemesh => QuadRemesh.evaluate(inputs, meta),
            Self::Substrate => Substrate.evaluate(inputs, meta),
            Self::Proximity2D => Proximity2D.evaluate(inputs, meta),
            Self::VoronoiCell => VoronoiCell.evaluate(inputs, meta),
            Self::QuadTree => QuadTree.evaluate(inputs, meta),
            Self::TriRemesh => TriRemesh.evaluate(inputs, meta),
            Self::ConvexHull => ConvexHull.evaluate(inputs, meta),
            Self::VoronoiGroups => VoronoiGroups.evaluate(inputs, meta),
            Self::Voronoi => Voronoi.evaluate(inputs, meta),
            Self::OcTree => OcTree.evaluate(inputs, meta),
            Self::Voronoi3D => Voronoi3D.evaluate(inputs, meta),
            Self::MetaBallTCustom => MetaBallTCustom.evaluate(inputs, meta),
            Self::MetaBallT => MetaBallT.evaluate(inputs, meta),
            Self::DelaunayEdges => DelaunayEdges.evaluate(inputs, meta),
            Self::MetaBall => MetaBall.evaluate(inputs, meta),
            Self::Proximity3D => Proximity3D.evaluate(inputs, meta),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::DelaunayMesh => "Delaunay Mesh",
            Self::FacetDome => "Facet Dome",
            Self::QuadRemesh => "Quad Remesh",
            Self::Substrate => "Substrate",
            Self::Proximity2D => "Proximity 2D",
            Self::VoronoiCell => "Voronoi Cell",
            Self::QuadTree => "QuadTree",
            Self::TriRemesh => "TriRemesh",
            Self::ConvexHull => "Convex Hull",
            Self::VoronoiGroups => "Voronoi Groups",
            Self::Voronoi => "Voronoi",
            Self::OcTree => "OcTree",
            Self::Voronoi3D => "Voronoi 3D",
            Self::MetaBallTCustom => "MetaBall(t) Custom",
            Self::MetaBallT => "MetaBall(t)",
            Self::DelaunayEdges => "Delaunay Edges",
            Self::MetaBall => "MetaBall",
            Self::Proximity3D => "Proximity 3D",
        }
    }
}

pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["1eb4f6ff-3547-4184-bead-1b01e7cfd668"],
        names: &["Delaunay Mesh", "Del"],
        kind: ComponentKind::DelaunayMesh,
    },
    Registration {
        guids: &["190c0070-8cbf-4347-94c2-d84bbb488d55"],
        names: &["Facet Dome", "Facet"],
        kind: ComponentKind::FacetDome,
    },
    Registration {
        guids: &["1a17d3f0-c8f8-4ee9-8dab-ea1c29db6a49"],
        names: &["Quad Remesh", "QRMesh"],
        kind: ComponentKind::QuadRemesh,
    },
    Registration {
        guids: &["415750fd-c0ec-4411-84d0-01f28ab23066"],
        names: &["Substrate"],
        kind: ComponentKind::Substrate,
    },
    Registration {
        guids: &["458ed0e0-19a3-419b-8ead-f524925b8a35"],
        names: &["Proximity 2D", "Prox"],
        kind: ComponentKind::Proximity2D,
    },
    Registration {
        guids: &["7b181be1-30e7-4a97-915a-1b461741aef8"],
        names: &["Voronoi Cell", "VCell"],
        kind: ComponentKind::VoronoiCell,
    },
    Registration {
        guids: &["8102032b-9699-4949-ab12-3017a31d1062"],
        names: &["QuadTree", "QT"],
        kind: ComponentKind::QuadTree,
    },
    Registration {
        guids: &["866222ee-6093-4af8-8944-2f9264885385"],
        names: &["TriRemesh"],
        kind: ComponentKind::TriRemesh,
    },
    Registration {
        guids: &["9d0c5284-ea24-4f9f-a183-ef57fc48b5b8"],
        names: &["Convex Hull", "Hull"],
        kind: ComponentKind::ConvexHull,
    },
    Registration {
        guids: &[
            "9d4854fe-70db-4863-967b-4120d0b6d2e4",
            "ab454a50-debf-46d1-9bd1-82648416a802",
        ],
        names: &["Voronoi Groups", "VorGroup"],
        kind: ComponentKind::VoronoiGroups,
    },
    Registration {
        guids: &[
            "a4011be0-1c91-45bd-8280-17dd3a9f46f1",
            "ee9261ab-75a4-478f-b483-a50b755b07fd",
        ],
        names: &["Voronoi"],
        kind: ComponentKind::Voronoi,
    },
    Registration {
        guids: &["a59a68ad-fdd6-41dd-88f0-d7a6fb8d2e16"],
        names: &["OcTree", "OcT"],
        kind: ComponentKind::OcTree,
    },
    Registration {
        guids: &["ba9bb57a-61cf-4207-a1c4-994e371ba4f9"],
        names: &["Voronoi 3D", "Voronoi³"],
        kind: ComponentKind::Voronoi3D,
    },
    Registration {
        guids: &["c4373505-a4cf-4992-8db1-fd6e6bb5850d"],
        names: &["MetaBall(t) Custom"],
        kind: ComponentKind::MetaBallTCustom,
    },
    Registration {
        guids: &["c48cf4d4-432c-41b6-b77a-77650479a31f"],
        names: &["MetaBall(t)"],
        kind: ComponentKind::MetaBallT,
    },
    Registration {
        guids: &["db2a4d25-23fa-4887-8983-ee5293cc82c0"],
        names: &["Delaunay Edges", "Con"],
        kind: ComponentKind::DelaunayEdges,
    },
    Registration {
        guids: &["dc934310-67eb-4d1d-8607-7cc62a501dd9"],
        names: &["MetaBall"],
        kind: ComponentKind::MetaBall,
    },
    Registration {
        guids: &["e504d619-4467-437a-92fa-c6822d16b066"],
        names: &["Proximity 3D", "Prox"],
        kind: ComponentKind::Proximity3D,
    },
];

macro_rules! placeholder_component {
    ($name:ident, $component_name:expr) => {
        #[derive(Debug, Default, Clone, Copy)]
        pub struct $name;

        impl Component for $name {
            fn evaluate(&self, _inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
                Err(ComponentError::NotYetImplemented(
                    $component_name.to_string(),
                ))
            }
        }
    };
}

const OUTPUT_PATTERN: &str = "P";
const OUTPUT_DOME: &str = "D";

/// Component for creating a facetted dome.
#[derive(Debug, Default, Clone, Copy)]
pub struct FacetDome;

impl Component for FacetDome {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input `P` is missing"));
        }

        let points = match &inputs[0] {
            Value::List(list) => list
                .iter()
                .map(coerce::coerce_point)
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(ComponentError::new("Input `P` is not a list")),
        };

        if points.len() < 3 {
            return Err(ComponentError::new("Not enough points for triangulation"));
        }

        let delaunator_points: Vec<delaunator::Point> = points
            .iter()
            .map(|p| delaunator::Point { x: p[0], y: p[1] })
            .collect();

        let triangulation = delaunator::triangulate(&delaunator_points);

        // Create the "Dome" (Delaunay Mesh)
        let faces: Vec<Vec<u32>> = triangulation
            .triangles
            .chunks_exact(3)
            .map(|tri| vec![tri[0] as u32, tri[1] as u32, tri[2] as u32])
            .collect();
        let dome = Value::Surface {
            vertices: points.clone(),
            faces,
        };

        // Create the "Pattern" (Edges of the triangulation)
        let mut pattern_edges = Vec::new();
        for i in 0..triangulation.triangles.len() {
            let endpoint = triangulation.halfedges[i];
            if i < endpoint {
                let p1_idx = triangulation.triangles[i];
                let p2_idx = triangulation.triangles[delaunator::next_halfedge(i)];
                pattern_edges.push(Value::CurveLine {
                    p1: points[p1_idx],
                    p2: points[p2_idx],
                });
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_PATTERN.to_owned(), Value::List(pattern_edges));
        outputs.insert(OUTPUT_DOME.to_owned(), dome);

        Ok(outputs)
    }
}

placeholder_component!(QuadRemesh, "Quad Remesh");
placeholder_component!(Substrate, "Substrate");

const OUTPUT_LINKS: &str = "L";
const OUTPUT_TOPOLOGY: &str = "T";

/// Component for finding 2D proximity within a point list.
#[derive(Debug, Default, Clone, Copy)]
pub struct Proximity2D;

impl Component for Proximity2D {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 1 {
            return Err(ComponentError::new("Input `P` is missing"));
        }

        let points = match &inputs[0] {
            Value::List(list) => list
                .iter()
                .map(coerce::coerce_point)
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(ComponentError::new("Input `P` is not a list")),
        };
        // inputs[1] is Plane, which is ignored for now.
        let group = if inputs.len() > 2 && !matches!(&inputs[2], Value::Null) {
            coerce::coerce_integer(&inputs[2])? as usize
        } else {
            usize::MAX
        };
        let min_radius_sq = if inputs.len() > 3 && !matches!(&inputs[3], Value::Null) {
            let r = coerce::coerce_number(&inputs[3], None)?;
            r * r
        } else {
            0.0
        };
        let max_radius_sq = if inputs.len() > 4 && !matches!(&inputs[4], Value::Null) {
            let r = coerce::coerce_number(&inputs[4], None)?;
            r * r
        } else {
            f64::INFINITY
        };

        if points.is_empty() {
            return Ok(BTreeMap::new());
        }

        let mut links_tree = Vec::new();
        let mut topology_tree = Vec::new();

        for (i, p1) in points.iter().enumerate() {
            let mut neighbors = Vec::new();
            for (j, p2) in points.iter().enumerate() {
                if i == j {
                    continue;
                }

                let dist_sq = (p1[0] - p2[0]).powi(2) + (p1[1] - p2[1]).powi(2);

                if dist_sq >= min_radius_sq && dist_sq <= max_radius_sq {
                    neighbors.push((dist_sq, j, p2));
                }
            }

            neighbors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            let mut links_branch = Vec::new();
            let mut topology_branch = Vec::new();

            for (_, j, p2) in neighbors.iter().take(group) {
                links_branch.push(Value::CurveLine { p1: *p1, p2: **p2 });
                topology_branch.push(Value::Text(format!("{{{};{}}}", i, j)));
            }

            links_tree.push(Value::List(links_branch));
            topology_tree.push(Value::List(topology_branch));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_LINKS.to_owned(), Value::List(links_tree));
        outputs.insert(OUTPUT_TOPOLOGY.to_owned(), Value::List(topology_tree));

        Ok(outputs)
    }
}

placeholder_component!(VoronoiCell, "Voronoi Cell");
placeholder_component!(QuadTree, "QuadTree");
placeholder_component!(TriRemesh, "TriRemesh");

const OUTPUT_HULL: &str = "H";
const OUTPUT_HULL_Z: &str = "Hz";
const OUTPUT_INDICES: &str = "I";

/// Component for creating a convex hull from a set of points.
#[derive(Debug, Default, Clone, Copy)]
pub struct ConvexHull;

impl Component for ConvexHull {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input `P` is missing"));
        }

        let points_value = &inputs[0];
        let points_list = match points_value {
            Value::List(list) => list,
            _ => return Err(ComponentError::new("Input `P` is not a list")),
        };

        let mut vertices = Vec::with_capacity(points_list.len());
        let mut delaunator_points = Vec::with_capacity(points_list.len());

        for value in points_list {
            let point = coerce::coerce_point(value)?;
            vertices.push(point);
            delaunator_points.push(delaunator::Point {
                x: point[0],
                y: point[1],
            });
        }

        if vertices.len() < 3 {
            return Err(ComponentError::new("Not enough points for triangulation"));
        }

        let triangulation = delaunator::triangulate(&delaunator_points);
        let hull_indices: Vec<Value> = triangulation
            .hull
            .iter()
            .map(|&i| Value::Number(i as f64))
            .collect();

        let mut hull_lines = Vec::new();
        if triangulation.hull.len() > 1 {
            for i in 0..triangulation.hull.len() {
                let p1_idx = triangulation.hull[i];
                let p2_idx = triangulation.hull[(i + 1) % triangulation.hull.len()];
                hull_lines.push(Value::CurveLine {
                    p1: vertices[p1_idx],
                    p2: vertices[p2_idx],
                });
            }
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_HULL.to_owned(), Value::List(hull_lines.clone()));
        outputs.insert(OUTPUT_HULL_Z.to_owned(), Value::List(hull_lines));
        outputs.insert(OUTPUT_INDICES.to_owned(), Value::List(hull_indices));

        Ok(outputs)
    }
}
placeholder_component!(VoronoiGroups, "Voronoi Groups");

const OUTPUT_CELLS: &str = "C";

/// Component for creating a Voronoi diagram.
#[derive(Debug, Default, Clone, Copy)]
pub struct Voronoi;

impl Component for Voronoi {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input `P` is missing"));
        }

        let sites = match &inputs[0] {
            Value::List(list) => list
                .iter()
                .map(|v| {
                    let p = coerce::coerce_point(v)?;
                    Ok(voronoice::Point { x: p[0], y: p[1] })
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(ComponentError::new("Input `P` is not a list")),
        };

        let mut builder = voronoice::VoronoiBuilder::default();
        builder = builder.set_sites(sites);

        if inputs.len() > 2 && !matches!(&inputs[2], Value::Null) {
            if let Value::CurveLine { p1, p2 } = &inputs[2] {
                let width = (p2[0] - p1[0]).abs();
                let height = (p2[1] - p1[1]).abs();
                let center_x = p1[0] + width / 2.0;
                let center_y = p1[1] + height / 2.0;

                let bounding_box = voronoice::BoundingBox::new(
                    voronoice::Point {
                        x: center_x,
                        y: center_y,
                    },
                    width,
                    height,
                );
                builder = builder.set_bounding_box(bounding_box);
            }
        }

        if let Some(voronoi) = builder.build() {
            let cells = voronoi
                .iter_cells()
                .map(|cell| {
                    let mut points = cell
                        .iter_vertices()
                        .map(|p| Value::Point([p.x, p.y, 0.0]))
                        .collect::<Vec<_>>();
                    if !points.is_empty() {
                        points.push(points[0].clone()); // Close the polyline
                    }
                    Value::List(points)
                })
                .collect();

            let mut outputs = BTreeMap::new();
            outputs.insert(OUTPUT_CELLS.to_owned(), Value::List(cells));
            Ok(outputs)
        } else {
            Err(ComponentError::new("Failed to build Voronoi diagram"))
        }
    }
}

placeholder_component!(OcTree, "OcTree");
placeholder_component!(Voronoi3D, "Voronoi 3D");
placeholder_component!(MetaBallTCustom, "MetaBall(t) Custom");
placeholder_component!(MetaBallT, "MetaBall(t)");
const OUTPUT_CONNECTIVITY: &str = "C";
const OUTPUT_EDGES: &str = "E";

/// Component for creating Delaunay edges from a set of points.
#[derive(Debug, Default, Clone, Copy)]
pub struct DelaunayEdges;

impl Component for DelaunayEdges {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.is_empty() {
            return Err(ComponentError::new("Input `P` is missing"));
        }

        let points_value = &inputs[0];
        let points_list = match points_value {
            Value::List(list) => list,
            _ => return Err(ComponentError::new("Input `P` is not a list")),
        };

        let mut vertices = Vec::with_capacity(points_list.len());
        let mut delaunator_points = Vec::with_capacity(points_list.len());

        for value in points_list {
            let point = coerce::coerce_point(value)?;
            vertices.push(point);
            delaunator_points.push(delaunator::Point {
                x: point[0],
                y: point[1],
            });
        }

        if vertices.len() < 3 {
            return Err(ComponentError::new("Not enough points for triangulation"));
        }

        let triangulation = delaunator::triangulate(&delaunator_points);
        let mut edges = Vec::new();
        let mut connectivity: BTreeMap<usize, Vec<Value>> = BTreeMap::new();

        for i in 0..triangulation.triangles.len() {
            let endpoint = triangulation.halfedges[i];
            if i < endpoint {
                let p1_idx = triangulation.triangles[i];
                let p2_idx = triangulation.triangles[delaunator::next_halfedge(i)];

                let p1 = vertices[p1_idx];
                let p2 = vertices[p2_idx];
                edges.push(Value::CurveLine { p1, p2 });

                connectivity
                    .entry(p1_idx)
                    .or_default()
                    .push(Value::Number(p2_idx as f64));
                connectivity
                    .entry(p2_idx)
                    .or_default()
                    .push(Value::Number(p1_idx as f64));
            }
        }

        let connectivity_tree = Value::List(
            (0..vertices.len())
                .map(|i| Value::List(connectivity.remove(&i).unwrap_or_default()))
                .collect(),
        );

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_CONNECTIVITY.to_owned(), connectivity_tree);
        outputs.insert(OUTPUT_EDGES.to_owned(), Value::List(edges));

        Ok(outputs)
    }
}
placeholder_component!(MetaBall, "MetaBall");

/// Component for finding 3D proximity within a point list.
#[derive(Debug, Default, Clone, Copy)]
pub struct Proximity3D;

impl Component for Proximity3D {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 1 {
            return Err(ComponentError::new("Input `P` is missing"));
        }

        let points = match &inputs[0] {
            Value::List(list) => list
                .iter()
                .map(coerce::coerce_point)
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(ComponentError::new("Input `P` is not a list")),
        };

        let group = if inputs.len() > 1 && !matches!(&inputs[1], Value::Null) {
            coerce::coerce_integer(&inputs[1])? as usize
        } else {
            usize::MAX
        };
        let min_radius_sq = if inputs.len() > 2 && !matches!(&inputs[2], Value::Null) {
            let r = coerce::coerce_number(&inputs[2], None)?;
            r * r
        } else {
            0.0
        };
        let max_radius_sq = if inputs.len() > 3 && !matches!(&inputs[3], Value::Null) {
            let r = coerce::coerce_number(&inputs[3], None)?;
            r * r
        } else {
            f64::INFINITY
        };

        if points.is_empty() {
            return Ok(BTreeMap::new());
        }

        let mut links_tree = Vec::new();
        let mut topology_tree = Vec::new();

        for (i, p1) in points.iter().enumerate() {
            let mut neighbors = Vec::new();
            for (j, p2) in points.iter().enumerate() {
                if i == j {
                    continue;
                }

                let dist_sq =
                    (p1[0] - p2[0]).powi(2) + (p1[1] - p2[1]).powi(2) + (p1[2] - p2[2]).powi(2);

                if dist_sq >= min_radius_sq && dist_sq <= max_radius_sq {
                    neighbors.push((dist_sq, j, p2));
                }
            }

            neighbors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            let mut links_branch = Vec::new();
            let mut topology_branch = Vec::new();

            for (_, j, p2) in neighbors.iter().take(group) {
                links_branch.push(Value::CurveLine { p1: *p1, p2: **p2 });
                topology_branch.push(Value::Text(format!("{{{};{}}}", i, j)));
            }

            links_tree.push(Value::List(links_branch));
            topology_tree.push(Value::List(topology_branch));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_LINKS.to_owned(), Value::List(links_tree));
        outputs.insert(OUTPUT_TOPOLOGY.to_owned(), Value::List(topology_tree));

        Ok(outputs)
    }
}

/// Output pin for the mesh result.
const OUTPUT_MESH: &str = "M";

/// Component for creating a Delaunay mesh from a set of points.
#[derive(Debug, Default, Clone, Copy)]
pub struct DelaunayMesh;

impl Component for DelaunayMesh {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        if inputs.len() < 1 {
            return Err(ComponentError::new("Input `P` is missing"));
        }

        let points_value = &inputs[0];
        let points_list = match points_value {
            Value::List(list) => list,
            _ => return Err(ComponentError::new("Input `P` is not a list")),
        };

        let mut vertices = Vec::with_capacity(points_list.len());
        let mut delaunator_points = Vec::with_capacity(points_list.len());

        for value in points_list {
            let point = coerce::coerce_point(value)?;
            vertices.push(point);
            delaunator_points.push(delaunator::Point {
                x: point[0],
                y: point[1],
            });
        }

        if vertices.len() < 3 {
            return Err(ComponentError::new("Not enough points for triangulation"));
        }

        let triangulation = delaunator::triangulate(&delaunator_points);

        let faces: Vec<Vec<u32>> = triangulation
            .triangles
            .chunks_exact(3)
            .map(|tri| vec![tri[0] as u32, tri[1] as u32, tri[2] as u32])
            .collect();

        let mesh = Value::Surface { vertices, faces };

        let mut outputs = BTreeMap::new();
        outputs.insert(OUTPUT_MESH.to_owned(), mesh);

        Ok(outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Component, ConvexHull, DelaunayEdges, DelaunayMesh, FacetDome, OUTPUT_CELLS,
        OUTPUT_CONNECTIVITY, OUTPUT_DOME, OUTPUT_EDGES, OUTPUT_HULL, OUTPUT_HULL_Z, OUTPUT_INDICES,
        OUTPUT_LINKS, OUTPUT_MESH, OUTPUT_PATTERN, OUTPUT_TOPOLOGY, Proximity2D, Proximity3D,
        QuadRemesh, Voronoi,
    };
    use crate::graph::{node::MetaMap, value::Value};
    #[test]
    fn test_voronoi() {
        let component = Voronoi;
        let points = vec![
            Value::Point([0.25, 0.25, 0.0]),
            Value::Point([0.75, 0.25, 0.0]),
            Value::Point([0.25, 0.75, 0.0]),
            Value::Point([0.75, 0.75, 0.0]),
        ];
        let boundary = Value::CurveLine {
            p1: [0.0, 0.0, 0.0],
            p2: [1.0, 1.0, 0.0],
        };
        let inputs = vec![Value::List(points), Value::Null, boundary];

        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();

        let cells = outputs.get(OUTPUT_CELLS).unwrap();
        if let Value::List(cell_list) = cells {
            assert_eq!(cell_list.len(), 4);
            // Each cell should be a valid closed polyline
            for cell in cell_list {
                if let Value::List(points) = cell {
                    assert!(
                        points.len() >= 4,
                        "A closed cell should have at least 4 vertices (3 + 1 to close)"
                    );
                } else {
                    panic!("Expected a list of points for each cell");
                }
            }
        } else {
            panic!("Expected a List for Cells output");
        }
    }

    #[test]
    fn test_facet_dome() {
        let component = FacetDome;
        let points = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
            Value::Point([1.0, 1.0, 0.0]),
        ];
        let inputs = vec![Value::List(points)];

        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();

        let pattern = outputs.get(OUTPUT_PATTERN).unwrap();
        if let Value::List(edges) = pattern {
            // A triangulation of 4 points should have 5 edges
            assert_eq!(edges.len(), 5);
        } else {
            panic!("Expected a List for Pattern output");
        }

        let dome = outputs.get(OUTPUT_DOME).unwrap();
        if let Value::Surface { vertices, faces } = dome {
            assert_eq!(vertices.len(), 4);
            assert_eq!(faces.len(), 2);
        } else {
            panic!("Expected a Surface for Dome output");
        }
    }

    #[test]
    fn test_proximity_3d() {
        let component = Proximity3D;
        let points = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
            Value::Point([0.0, 0.0, 1.0]),
        ];
        let inputs = vec![Value::List(points.clone()), Value::Number(2.0)];

        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let links = outputs.get(OUTPUT_LINKS).unwrap();
        if let Value::List(links_tree) = links {
            assert_eq!(links_tree.len(), 4);
            // Each point is equidistant to the others, so it should connect to 2 of them.
            for branch in links_tree {
                if let Value::List(b) = branch {
                    assert_eq!(b.len(), 2);
                }
            }
        }
    }

    #[test]
    fn test_proximity_2d_basic() {
        let component = Proximity2D;
        let points = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
            Value::Point([5.0, 5.0, 0.0]),
        ];
        let inputs = vec![Value::List(points)];

        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();

        let links = outputs.get(OUTPUT_LINKS).unwrap();
        if let Value::List(links_tree) = links {
            assert_eq!(links_tree.len(), 4);
            // Point 0 should connect to 1 and 2
            assert_eq!(
                if let Value::List(branch) = &links_tree[0] {
                    branch.len()
                } else {
                    0
                },
                3
            );
            // Point 3 is far away, should only connect to the closest one (0, 1 or 2)
            assert_eq!(
                if let Value::List(branch) = &links_tree[3] {
                    branch.len()
                } else {
                    0
                },
                3
            );
        } else {
            panic!("Expected a List for Links output");
        }
    }

    #[test]
    fn test_proximity_2d_with_group() {
        let component = Proximity2D;
        let points = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
            Value::Point([0.5, 0.5, 0.0]),
        ];
        let inputs = vec![Value::List(points), Value::Null, Value::Number(2.0)];

        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let links = outputs.get(OUTPUT_LINKS).unwrap();
        if let Value::List(links_tree) = links {
            assert_eq!(links_tree.len(), 4);
            // Every point should have at most 2 links
            for branch in links_tree {
                if let Value::List(b) = branch {
                    assert!(b.len() <= 2);
                }
            }
        }
    }

    #[test]
    fn test_proximity_2d_with_radius() {
        let component = Proximity2D;
        let points = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([3.0, 0.0, 0.0]),
        ];
        // min_radius = 0.5, max_radius = 1.5
        let inputs = vec![
            Value::List(points),
            Value::Null,
            Value::Null,
            Value::Number(0.5),
            Value::Number(1.5),
        ];

        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let links = outputs.get(OUTPUT_LINKS).unwrap();
        if let Value::List(links_tree) = links {
            assert_eq!(links_tree.len(), 3);
            // P0 connects only to P1
            assert_eq!(
                if let Value::List(b) = &links_tree[0] {
                    b.len()
                } else {
                    0
                },
                1
            );
            // P1 connects only to P0
            assert_eq!(
                if let Value::List(b) = &links_tree[1] {
                    b.len()
                } else {
                    0
                },
                1
            );
            // P2 is too far from P0 and P1 with this radius
            assert_eq!(
                if let Value::List(b) = &links_tree[2] {
                    b.len()
                } else {
                    0
                },
                0
            );
        }
    }

    #[test]
    fn test_delaunay_mesh() {
        let component = DelaunayMesh;
        let points = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
            Value::Point([1.0, 1.0, 0.0]),
        ];
        let inputs = vec![Value::List(points)];

        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        let mesh = outputs.get(OUTPUT_MESH).unwrap();

        if let Value::Surface { vertices, faces } = mesh {
            assert_eq!(vertices.len(), 4);
            assert_eq!(faces.len(), 2);

            let mut sorted_faces: Vec<Vec<u32>> = faces
                .iter()
                .map(|face| {
                    let mut sorted_face = face.clone();
                    sorted_face.sort_unstable();
                    sorted_face
                })
                .collect();
            sorted_faces.sort_unstable();

            let expected1 = {
                let mut f = vec![vec![0, 1, 3], vec![0, 2, 3]];
                f.iter_mut().for_each(|face| face.sort_unstable());
                f.sort_unstable();
                f
            };
            let expected2 = {
                let mut f = vec![vec![0, 1, 2], vec![1, 2, 3]];
                f.iter_mut().for_each(|face| face.sort_unstable());
                f.sort_unstable();
                f
            };

            assert!(
                sorted_faces == expected1 || sorted_faces == expected2,
                "Generated faces do not match any expected triangulation. Got: {:?}, Expected: {:?} or {:?}",
                sorted_faces,
                expected1,
                expected2
            );
        } else {
            panic!("Expected a Surface output");
        }
    }

    #[test]
    fn test_delaunay_mesh_not_enough_points() {
        let component = DelaunayMesh;
        let points = vec![Value::Point([0.0, 0.0, 0.0]), Value::Point([1.0, 0.0, 0.0])];
        let inputs = vec![Value::List(points)];

        let err = component.evaluate(&inputs, &MetaMap::new()).unwrap_err();
        assert!(err.message().contains("Not enough points"));
    }

    #[test]
    fn test_placeholder_components() {
        let quad_remesh = QuadRemesh;

        assert!(matches!(
            quad_remesh.evaluate(&[], &MetaMap::new()),
            Err(super::ComponentError::NotYetImplemented(_))
        ));
    }

    #[test]
    fn test_delaunay_edges() {
        let component = DelaunayEdges;
        let points = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
            Value::Point([1.0, 1.0, 0.0]),
        ];
        let inputs = vec![Value::List(points)];

        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();

        let edges = outputs.get(OUTPUT_EDGES).unwrap();
        if let Value::List(edge_list) = edges {
            assert_eq!(edge_list.len(), 5);
        } else {
            panic!("Expected a List for Edges output");
        }

        let connectivity = outputs.get(OUTPUT_CONNECTIVITY).unwrap();
        if let Value::List(connectivity_list) = connectivity {
            assert_eq!(connectivity_list.len(), 4);
            let counts: Vec<usize> = connectivity_list
                .iter()
                .map(|v| match v {
                    Value::List(l) => l.len(),
                    _ => 0,
                })
                .collect();
            // In a triangulation of 4 points, there will be 2 nodes with 3 edges
            // and 2 nodes with 2 edges, or all nodes have 3 edges (if the diagonal connects both ways)
            // The delaunator implementation gives 2x3 and 2x2.
            let mut three_count = 0;
            let mut two_count = 0;
            for c in counts {
                if c == 3 {
                    three_count += 1;
                } else if c == 2 {
                    two_count += 1;
                }
            }
            assert_eq!(three_count, 2, "Expected 2 points with 3 connections");
            assert_eq!(two_count, 2, "Expected 2 points with 2 connections");
        } else {
            panic!("Expected a List for Connectivity output");
        }
    }

    #[test]
    fn test_convex_hull() {
        let component = ConvexHull;
        let points = vec![
            Value::Point([0.0, 0.0, 0.0]),
            Value::Point([1.0, 0.0, 0.0]),
            Value::Point([0.0, 1.0, 0.0]),
            Value::Point([1.0, 1.0, 0.0]),
            Value::Point([0.5, 0.5, 0.0]), // Interior point
        ];
        let inputs = vec![Value::List(points)];

        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();

        // Check Hull lines
        let hull = outputs.get(OUTPUT_HULL).unwrap();
        if let Value::List(hull_lines) = hull {
            assert_eq!(hull_lines.len(), 4);
        } else {
            panic!("Expected a List for Hull output");
        }

        // Check Hull(z) lines (should be identical)
        let hull_z = outputs.get(OUTPUT_HULL_Z).unwrap();
        if let Value::List(hull_z_lines) = hull_z {
            assert_eq!(hull_z_lines.len(), 4);
        } else {
            panic!("Expected a List for Hull(z) output");
        }

        // Check Indices
        let indices = outputs.get(OUTPUT_INDICES).unwrap();
        if let Value::List(index_list) = indices {
            assert_eq!(index_list.len(), 4);
            let mut actual_indices: Vec<usize> = index_list
                .iter()
                .map(|v| match v {
                    Value::Number(n) => *n as usize,
                    _ => panic!("Expected a number in the index list"),
                })
                .collect();
            actual_indices.sort_unstable();
            assert_eq!(actual_indices, vec![0, 1, 2, 3]);
        } else {
            panic!("Expected a List for Indices output");
        }
    }

    #[test]
    fn test_convex_hull_not_enough_points() {
        let component = ConvexHull;
        let points = vec![Value::Point([0.0, 0.0, 0.0]), Value::Point([1.0, 0.0, 0.0])];
        let inputs = vec![Value::List(points)];

        let err = component.evaluate(&inputs, &MetaMap::new()).unwrap_err();
        assert!(err.message().contains("Not enough points"));
    }
}
