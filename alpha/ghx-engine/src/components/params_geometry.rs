//! Implements parameter components for geometry types.

use std::collections::BTreeMap;

use crate::components::{Component, ComponentError, ComponentResult};
use crate::graph::node::MetaMap;
use crate::graph::value::{Value, ValueKind};

/// Defines a component's registration information.
pub struct Registration<T> {
    /// The component's kind.
    pub kind: T,
    /// A list of GUIDs that identify the component.
    pub guids: &'static [&'static str],
    /// A list of names and nicknames for the component.
    pub names: &'static [&'static str],
}

impl<T: Copy> Registration<T> {
    /// Creates a new `Registration` instance.
    pub const fn new(kind: T, guids: &'static [&'static str], names: &'static [&'static str]) -> Self {
        Self { kind, guids, names }
    }
}

// --- ComponentKind Enum ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentKind {
    Point,
    Vector,
    Line,
    Mesh,
    Surface,
    CircularArc,
    Transform,
    Field,
    Plane,
    TwistedBox,
    Location,
    SubD,
    Brep,
    Atom,
    Rectangle,
    Geometry,
    Group,
    GeometryPipeline,
    MesherSettings,
    Box,
    Circle,
    Curve,
    MeshFace,
    GeometryCache,
    MeshPoint,
}

impl ComponentKind {
    pub fn evaluate(self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Point => PointComponent.evaluate(inputs, meta),
            Self::Vector => VectorComponent.evaluate(inputs, meta),
            Self::Line => LineComponent.evaluate(inputs, meta),
            Self::Mesh => MeshComponent.evaluate(inputs, meta),
            Self::Surface => SurfaceComponent.evaluate(inputs, meta),
            // Placeholders
            _ => Err(ComponentError::NotYetImplemented(self.name().to_string())),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Point => "Point",
            Self::Vector => "Vector",
            Self::Line => "Line",
            Self::Mesh => "Mesh",
            Self::Surface => "Surface",
            Self::CircularArc => "Circular Arc",
            Self::Transform => "Transform",
            Self::Field => "Field",
            Self::Plane => "Plane",
            Self::TwistedBox => "Twisted Box",
            Self::Location => "Location",
            Self::SubD => "SubD",
            Self::Brep => "Brep",
            Self::Atom => "Atom",
            Self::Rectangle => "Rectangle",
            Self::Geometry => "Geometry",
            Self::Group => "Group",
            Self::GeometryPipeline => "Geometry Pipeline",
            Self::MesherSettings => "Mesher Settings",
            Self::Box => "Box",
            Self::Circle => "Circle",
            Self::Curve => "Curve",
            Self::MeshFace => "Mesh Face",
            Self::GeometryCache => "Geometry Cache",
            Self::MeshPoint => "Mesh Point",
        }
    }
}

// A macro to define a parameter component that passes through a specific `Value` type.
macro_rules! define_param_component {
    (
        $struct_name:ident,
        $output_pin:expr,
        $expected_kind:path
    ) => {
        #[derive(Debug, Default, Clone, Copy)]
        struct $struct_name;

        impl Component for $struct_name {
            fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
                if inputs.is_empty() {
                    let mut outputs = BTreeMap::new();
                    outputs.insert($output_pin.to_owned(), Value::Null);
                    return Ok(outputs);
                }

                let input_value = &inputs[0];

                let is_valid = match input_value {
                    Value::List(items) => items.iter().all(|item| item.kind() == $expected_kind || matches!(item, Value::Null)),
                    value => value.kind() == $expected_kind || matches!(value, Value::Null),
                };

                if !is_valid {
                    return Err(ComponentError::new(format!(
                        "Expected {} or a List of {}, but got {}.",
                        $expected_kind,
                        $expected_kind,
                        input_value.kind()
                    )));
                }

                let mut outputs = BTreeMap::new();
                outputs.insert($output_pin.to_owned(), input_value.clone());
                Ok(outputs)
            }
        }
    };
}

// --- Implemented Components ---
define_param_component!(PointComponent, "Pt", ValueKind::Point);
define_param_component!(VectorComponent, "Vec", ValueKind::Vector);
define_param_component!(LineComponent, "Line", ValueKind::CurveLine);
define_param_component!(MeshComponent, "Mesh", ValueKind::Surface);
define_param_component!(SurfaceComponent, "Srf", ValueKind::Surface);

// --- Registrations ---
pub const REGISTRATIONS: &[Registration<ComponentKind>] = &[
    Registration::new(ComponentKind::Point, &["fbac3e32-f100-4292-8692-77240a42fd1a"], &["Point", "Pt"]),
    Registration::new(ComponentKind::Vector, &["16ef3e75-e315-4899-b531-d3166b42dac9"], &["Vector", "Vec"]),
    Registration::new(ComponentKind::Line, &["8529dbdf-9b6f-42e9-8e1f-c7a2bde56a70"], &["Line"]),
    Registration::new(ComponentKind::Mesh, &["1e936df3-0eea-4246-8549-514cb8862b7a"], &["Mesh"]),
    Registration::new(ComponentKind::Surface, &["deaf8653-5528-4286-807c-3de8b8dad781"], &["Surface", "Srf"]),
    Registration::new(ComponentKind::CircularArc, &["04d3eace-deaa-475e-9e69-8f804d687998"], &["Circular Arc", "Arc"]),
    Registration::new(ComponentKind::Transform, &["28f40e48-e739-4211-91bd-f4aefa5965f8"], &["Transform"]),
    Registration::new(ComponentKind::Field, &["3175e3eb-1ae0-4d0b-9395-53fd3e8f8a28"], &["Field"]),
    Registration::new(ComponentKind::Plane, &["4f8984c4-7c7a-4d69-b0a2-183cbb330d20"], &["Plane", "Pln"]),
    Registration::new(ComponentKind::TwistedBox, &["6db039c4-cad1-4549-bd45-e31cb0f71692"], &["Twisted Box", "TBox"]),
    Registration::new(ComponentKind::Location, &["87391af3-35fe-4a40-b001-2bd4547ccd45"], &["Location", "Loc"]),
    Registration::new(ComponentKind::SubD, &["89cd1a12-0007-4581-99ba-66578665e610"], &["SubD"]),
    Registration::new(ComponentKind::Brep, &["919e146f-30ae-4aae-be34-4d72f555e7da"], &["Brep"]),
    Registration::new(ComponentKind::Atom, &["a80395af-f134-4d6a-9b89-15edf3161619"], &["Atom"]),
    Registration::new(ComponentKind::Rectangle, &["abf9c670-5462-4cd8-acb3-f1ab0256dbf3"], &["Rectangle", "Rec"]),
    Registration::new(ComponentKind::Geometry, &["ac2bc2cb-70fb-4dd5-9c78-7e1ea97fe278"], &["Geometry", "Geo"]),
    Registration::new(ComponentKind::Group, &["b0851fc0-ab55-47d8-bdda-cc6306a40176"], &["Group", "Grp"]),
    Registration::new(ComponentKind::GeometryPipeline, &["b341e2e5-c4b3-49a3-b3a4-b4e6e2054516"], &["Geometry Pipeline", "Pipeline"]),
    Registration::new(ComponentKind::MesherSettings, &["c3407fda-b505-4686-9165-38fe7a9274cf"], &["Mesher Settings"]),
    Registration::new(ComponentKind::Box, &["c9482db6-bea9-448d-98ff-fed6d69a8efc"], &["Box"]),
    Registration::new(ComponentKind::Circle, &["d1028c72-ff86-4057-9eb0-36c687a4d98c"], &["Circle"]),
    Registration::new(ComponentKind::Curve, &["d5967b9f-e8ee-436b-a8ad-29fdcecf32d5"], &["Curve", "Crv"]),
    Registration::new(ComponentKind::MeshFace, &["e02b3da5-543a-46ac-a867-0ba6b0a524de"], &["Mesh Face", "Face"]),
    Registration::new(ComponentKind::GeometryCache, &["f91778ca-2700-42fc-8ee6-74049a2292b5"], &["Geometry Cache"]),
    Registration::new(ComponentKind::MeshPoint, &["fa20fe95-5775-417b-92ff-b77c13cbf40c"], &["Mesh Point", "MPoint"]),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::MetaMap;

    #[test]
    fn test_point_param_component_pass_through() {
        let component = PointComponent;
        let point = Value::Point([1.0, 2.0, 3.0]);
        let inputs = vec![point.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Pt"), Some(&point));
    }

    #[test]
    fn test_point_param_component_wrong_type() {
        let component = PointComponent;
        let vector = Value::Vector([1.0, 2.0, 3.0]);
        let inputs = vec![vector];
        let err = component.evaluate(&inputs, &MetaMap::new()).unwrap_err();
        assert!(matches!(err, ComponentError::Message(_)));
    }

    #[test]
    fn test_point_param_component_no_input() {
        let component = PointComponent;
        let outputs = component.evaluate(&[], &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Pt"), Some(&Value::Null));
    }

    #[test]
    fn test_vector_param_component_pass_through() {
        let component = VectorComponent;
        let vector = Value::Vector([1.0, 2.0, 3.0]);
        let inputs = vec![vector.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Vec"), Some(&vector));
    }

    #[test]
    fn test_line_param_component_pass_through() {
        let component = LineComponent;
        let line = Value::CurveLine { p1: [0.0; 3], p2: [1.0; 3] };
        let inputs = vec![line.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Line"), Some(&line));
    }

    #[test]
    fn test_mesh_param_component_pass_through() {
        let component = MeshComponent;
        let mesh = Value::Surface { vertices: vec![], faces: vec![] };
        let inputs = vec![mesh.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Mesh"), Some(&mesh));
    }

    #[test]
    fn test_surface_param_component_pass_through() {
        let component = SurfaceComponent;
        let surface = Value::Surface { vertices: vec![], faces: vec![] };
        let inputs = vec![surface.clone()];
        let outputs = component.evaluate(&inputs, &MetaMap::new()).unwrap();
        assert_eq!(outputs.get("Srf"), Some(&surface));
    }

    #[test]
    fn test_placeholder_components() {
        let kind = ComponentKind::Circle;
        let err = kind.evaluate(&[], &MetaMap::new()).unwrap_err();
        assert!(matches!(err, ComponentError::NotYetImplemented(_)));
    }
}
