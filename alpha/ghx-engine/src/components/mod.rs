//! Component registry en evaluatie-logica.

use std::collections::HashMap;
use std::fmt;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

pub mod complex;
pub mod curve_analysis;
pub mod curve_division;
pub mod curve_primitive;
pub mod curve_sampler;
pub mod curve_spline;
pub mod curve_util;
pub mod extrude;
pub mod maths_domain;
pub mod maths_matrix;
pub mod maths_operators;
pub mod mesh_analysis;
pub mod mesh_primitive;
pub mod mesh_triangulation;
pub mod maths_polynomials;
pub mod maths_script;
pub mod maths_time;
pub mod maths_trig;
pub mod maths_util;
pub mod scalar;
pub mod sets_list;
pub mod sets_sequence;
pub mod sets_sets;
pub mod sets_text;
pub mod sets_tree;
pub mod surface_analysis;
pub mod surface_freeform;
pub mod surface_primitive;
pub mod surface_subd;
pub mod surface_util;
pub mod transform_affine;
pub mod transform_array;
pub mod transform_euclidean;
pub mod transform_util;
pub mod vector_field;
pub mod vector_grid;
pub mod vector_plane;
pub mod vector_point;
pub mod vector_vector;
pub mod display_preview;
pub mod params_geometry;
pub mod params_primitive;
pub mod params_input;
pub mod params_util;
pub mod coerce;

/// Output-map van een component: pinnickname → waarde.
pub type OutputMap = std::collections::BTreeMap<String, Value>;

/// Fouttype voor component-evaluaties.
#[derive(Debug, Clone)]
pub enum ComponentError {
    /// Een generieke fout met een bericht.
    Message(String),
    /// Geeft aan dat een component nog niet geïmplementeerd is.
    NotYetImplemented(String),
}

impl ComponentError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Self::Message(s) => s,
            Self::NotYetImplemented(s) => s,
        }
    }
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(s) => f.write_str(s),
            Self::NotYetImplemented(s) => write!(f, "Component not yet implemented: {}", s),
        }
    }
}

impl std::error::Error for ComponentError {}

/// Resultaat van een component-executie.
pub type ComponentResult = Result<OutputMap, ComponentError>;

/// Trait die alle componentimplementaties dienen te implementeren.
pub trait Component {
    fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult;
}

/// Beschikbare componenttypen binnen de registry.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    Extrude(extrude::ComponentImpl),
    CurvePrimitive(curve_primitive::ComponentKind),
    CurveDivision(curve_division::ComponentKind),
    CurveAnalysis(curve_analysis::ComponentKind),
    CurveSpline(curve_spline::ComponentKind),
    CurveUtil(curve_util::ComponentKind),
    MathsOperator(maths_operators::ComponentKind),
    MathsDomain(maths_domain::ComponentKind),
    MathsPolynomial(maths_polynomials::ComponentKind),
    MathsMatrix(maths_matrix::ComponentKind),
    MathsScript(maths_script::ComponentKind),
    MathsTime(maths_time::ComponentKind),
    MathsTrig(maths_trig::ComponentKind),
    MathsUtil(maths_util::ComponentKind),
    SurfacePrimitive(surface_primitive::ComponentKind),
    SurfaceFreeform(surface_freeform::ComponentKind),
    SurfaceAnalysis(surface_analysis::ComponentKind),
    SurfaceSubd(surface_subd::ComponentKind),
    SurfaceUtil(surface_util::ComponentKind),
    TransformAffine(transform_affine::ComponentKind),
    TransformArray(transform_array::ComponentKind),
    TransformEuclidean(transform_euclidean::ComponentKind),
    TransformUtil(transform_util::ComponentKind),
    VectorVector(vector_vector::ComponentKind),
    VectorPoint(vector_point::ComponentKind),
    VectorPlane(vector_plane::ComponentKind),
    VectorGrid(vector_grid::ComponentKind),
    VectorField(vector_field::ComponentKind),
    Complex(complex::ComponentKind),
    Scalar(scalar::ComponentKind),
    SetsList(sets_list::ComponentKind),
    SetsSequence(sets_sequence::ComponentKind),
    SetsSets(sets_sets::ComponentKind),
    SetsText(sets_text::ComponentKind),
    SetsTree(sets_tree::ComponentKind),
    DisplayPreview(display_preview::ComponentKind),
    MeshPrimitive(mesh_primitive::ComponentKind),
    MeshAnalysis(mesh_analysis::ComponentKind),
    MeshTriangulation(mesh_triangulation::ComponentKind),
    ParamsGeometry(params_geometry::ComponentKind),
    ParamsPrimitive(params_primitive::ComponentKind),
    ParamsInput(params_input::ComponentKind),
    ParamsUtil(params_util::ComponentKind),
}

impl ComponentKind {
    #[must_use]
    pub fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::Extrude(component) => component.evaluate(inputs, meta),
            Self::CurvePrimitive(component) => component.evaluate(inputs, meta),
            Self::CurveDivision(component) => component.evaluate(inputs, meta),
            Self::CurveAnalysis(component) => component.evaluate(inputs, meta),
            Self::CurveSpline(component) => component.evaluate(inputs, meta),
            Self::CurveUtil(component) => component.evaluate(inputs, meta),
            Self::MathsOperator(component) => component.evaluate(inputs, meta),
            Self::MathsDomain(component) => component.evaluate(inputs, meta),
            Self::MathsPolynomial(component) => component.evaluate(inputs, meta),
            Self::MathsMatrix(component) => component.evaluate(inputs, meta),
            Self::MathsScript(component) => component.evaluate(inputs, meta),
            Self::MathsTime(component) => component.evaluate(inputs, meta),
            Self::MathsTrig(component) => component.evaluate(inputs, meta),
            Self::MathsUtil(component) => component.evaluate(inputs, meta),
            Self::SurfacePrimitive(component) => component.evaluate(inputs, meta),
            Self::SurfaceFreeform(component) => component.evaluate(inputs, meta),
            Self::SurfaceAnalysis(component) => component.evaluate(inputs, meta),
            Self::SurfaceSubd(component) => component.evaluate(inputs, meta),
            Self::SurfaceUtil(component) => component.evaluate(inputs, meta),
            Self::TransformAffine(component) => component.evaluate(inputs, meta),
            Self::TransformArray(component) => component.evaluate(inputs, meta),
            Self::TransformEuclidean(component) => component.evaluate(inputs, meta),
            Self::TransformUtil(component) => component.evaluate(inputs, meta),
            Self::VectorVector(component) => component.evaluate(inputs, meta),
            Self::VectorPoint(component) => component.evaluate(inputs, meta),
            Self::VectorPlane(component) => component.evaluate(inputs, meta),
            Self::VectorGrid(component) => component.evaluate(inputs, meta),
            Self::VectorField(component) => component.evaluate(inputs, meta),
            Self::Complex(component) => component.evaluate(inputs, meta),
            Self::Scalar(component) => component.evaluate(inputs, meta),
            Self::SetsList(component) => component.evaluate(inputs, meta),
            Self::SetsSequence(component) => component.evaluate(inputs, meta),
            Self::SetsSets(component) => component.evaluate(inputs, meta),
            Self::SetsText(component) => sets_text::ComponentKind::evaluate(component, inputs, meta),
            Self::SetsTree(component) => sets_tree::ComponentKind::evaluate(*component, inputs, meta),
            Self::DisplayPreview(component) => component.evaluate(inputs, meta),
            Self::MeshPrimitive(component) => component.evaluate(inputs, meta),
            Self::MeshAnalysis(component) => component.evaluate(inputs, meta),
            Self::MeshTriangulation(component) => component.evaluate(inputs, meta),
            Self::ParamsGeometry(component) => component.evaluate(inputs, meta),
            Self::ParamsPrimitive(component) => component.evaluate(inputs, meta),
            Self::ParamsInput(component) => component.evaluate(inputs, meta),
            Self::ParamsUtil(component) => component.evaluate(inputs, meta),
        }
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Extrude(_) => "Extrude",
            Self::CurvePrimitive(component) => component.name(),
            Self::CurveDivision(component) => component.name(),
            Self::CurveAnalysis(component) => component.name(),
            Self::CurveSpline(component) => component.name(),
            Self::CurveUtil(component) => component.name(),
            Self::MathsOperator(component) => component.name(),
            Self::MathsDomain(component) => component.name(),
            Self::MathsPolynomial(component) => component.name(),
            Self::MathsMatrix(component) => component.name(),
            Self::MathsScript(component) => component.name(),
            Self::MathsTime(component) => component.name(),
            Self::MathsTrig(component) => component.name(),
            Self::MathsUtil(component) => component.name(),
            Self::SurfacePrimitive(component) => component.name(),
            Self::SurfaceFreeform(component) => component.name(),
            Self::SurfaceAnalysis(component) => component.name(),
            Self::SurfaceSubd(component) => component.name(),
            Self::SurfaceUtil(component) => component.name(),
            Self::TransformAffine(component) => component.name(),
            Self::TransformArray(component) => component.name(),
            Self::TransformEuclidean(component) => component.name(),
            Self::TransformUtil(component) => component.name(),
            Self::VectorVector(component) => component.name(),
            Self::VectorPoint(component) => component.name(),
            Self::VectorPlane(component) => component.name(),
            Self::VectorGrid(component) => component.name(),
            Self::VectorField(component) => component.name(),
            Self::Complex(component) => component.name(),
            Self::Scalar(component) => component.name(),
            Self::SetsList(component) => component.name(),
            Self::SetsSequence(component) => component.name(),
            Self::SetsSets(component) => component.name(),
            Self::SetsText(component) => sets_text::ComponentKind::name(component),
            Self::SetsTree(component) => sets_tree::ComponentKind::name(*component),
            Self::DisplayPreview(component) => component.name(),
            Self::MeshPrimitive(component) => component.name(),
            Self::MeshAnalysis(component) => component.name(),
            Self::MeshTriangulation(component) => component.name(),
            Self::ParamsGeometry(component) => component.name(),
            Self::ParamsPrimitive(component) => component.name(),
            Self::ParamsInput(component) => component.name(),
            Self::ParamsUtil(component) => component.name(),
        }
    }

    #[must_use]
    pub fn optional_input_pins(&self) -> &'static [&'static str] {
        match self {
            Self::SurfaceUtil(component) => component.optional_input_pins(),
            _ => &[],
        }
    }
}

/// Registry die componentimplementaties opzoekt op GUID of naam.
#[derive(Debug, Clone)]
pub struct ComponentRegistry {
    by_guid: HashMap<String, ComponentKind>,
    by_name: HashMap<String, ComponentKind>,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        let extrude = ComponentKind::Extrude(extrude::ComponentImpl);
        registry.register_guid("{962034e9-cc27-4394-afc4-5c16e3447cf9}", extrude);
        registry.register_names(&["Extrude", "Extr"], extrude);

        for registration in curve_primitive::REGISTRATIONS {
            let kind = ComponentKind::CurvePrimitive(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in params_geometry::REGISTRATIONS {
            let kind = ComponentKind::ParamsGeometry(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in transform_array::REGISTRATIONS {
            let kind = ComponentKind::TransformArray(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in curve_analysis::REGISTRATIONS {
            let kind = ComponentKind::CurveAnalysis(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in curve_spline::REGISTRATIONS {
            let kind = ComponentKind::CurveSpline(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in curve_division::REGISTRATIONS {
            let kind = ComponentKind::CurveDivision(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in curve_util::REGISTRATIONS {
            let kind = ComponentKind::CurveUtil(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in maths_operators::REGISTRATIONS {
            let kind = ComponentKind::MathsOperator(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in maths_domain::REGISTRATIONS {
            let kind = ComponentKind::MathsDomain(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in maths_polynomials::REGISTRATIONS {
            let kind = ComponentKind::MathsPolynomial(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in maths_matrix::REGISTRATIONS {
            let kind = ComponentKind::MathsMatrix(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in maths_script::REGISTRATIONS {
            let kind = ComponentKind::MathsScript(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in maths_time::REGISTRATIONS {
            let kind = ComponentKind::MathsTime(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in maths_trig::REGISTRATIONS {
            let kind = ComponentKind::MathsTrig(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in maths_util::REGISTRATIONS {
            let kind = ComponentKind::MathsUtil(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in surface_primitive::REGISTRATIONS {
            let kind = ComponentKind::SurfacePrimitive(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in surface_freeform::REGISTRATIONS {
            let kind = ComponentKind::SurfaceFreeform(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in surface_analysis::REGISTRATIONS {
            let kind = ComponentKind::SurfaceAnalysis(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }
        for registration in surface_subd::REGISTRATIONS {
            let kind = ComponentKind::SurfaceSubd(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }
        for registration in surface_util::REGISTRATIONS {
            let kind = ComponentKind::SurfaceUtil(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in transform_affine::REGISTRATIONS {
            let kind = ComponentKind::TransformAffine(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in transform_euclidean::REGISTRATIONS {
            let kind = ComponentKind::TransformEuclidean(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in transform_util::REGISTRATIONS {
            let kind = ComponentKind::TransformUtil(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in vector_vector::REGISTRATIONS {
            let kind = ComponentKind::VectorVector(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in vector_point::REGISTRATIONS {
            let kind = ComponentKind::VectorPoint(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in vector_plane::REGISTRATIONS {
            let kind = ComponentKind::VectorPlane(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in vector_field::REGISTRATIONS {
            let kind = ComponentKind::VectorField(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in vector_grid::REGISTRATIONS {
            let kind = ComponentKind::VectorGrid(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in complex::REGISTRATIONS {
            let kind = ComponentKind::Complex(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in scalar::REGISTRATIONS {
            let kind = ComponentKind::Scalar(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in sets_list::REGISTRATIONS {
            let kind = ComponentKind::SetsList(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in display_preview::REGISTRATIONS {
            let kind = ComponentKind::DisplayPreview(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in sets_sequence::REGISTRATIONS {
            let kind = ComponentKind::SetsSequence(registration.kind());
            for guid in registration.guids() {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names(), kind);
        }

        for registration in sets_sets::REGISTRATIONS {
            let kind = ComponentKind::SetsSets(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in sets_sets::REGISTRATIONS {
            let kind = ComponentKind::SetsSets(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in sets_text::REGISTRATIONS {
            let kind = ComponentKind::SetsText(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in sets_tree::REGISTRATIONS {
            let kind = ComponentKind::SetsTree(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in mesh_primitive::REGISTRATIONS {
            let kind = ComponentKind::MeshPrimitive(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in mesh_analysis::REGISTRATIONS {
            let kind = ComponentKind::MeshAnalysis(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in mesh_triangulation::REGISTRATIONS {
            let kind = ComponentKind::MeshTriangulation(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in params_primitive::REGISTRATIONS {
            let kind = ComponentKind::ParamsPrimitive(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in params_input::REGISTRATIONS {
            let kind = ComponentKind::ParamsInput(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        for registration in params_util::REGISTRATIONS {
            let kind = ComponentKind::ParamsUtil(registration.kind);
            for guid in registration.guids {
                registry.register_guid(guid, kind);
            }
            registry.register_names(registration.names, kind);
        }

        registry
    }
}

impl ComponentRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_guid: HashMap::new(),
            by_name: HashMap::new(),
        }
    }

    pub fn register_guid(&mut self, guid: impl AsRef<str>, kind: ComponentKind) {
        let key = normalize_guid(guid.as_ref());
        self.by_guid.insert(key, kind);
    }

    pub fn register_names(&mut self, names: &[&str], kind: ComponentKind) {
        for name in names {
            let key = normalize_name(name);
            self.by_name.insert(key, kind);
        }
    }

    #[must_use]
    pub fn resolve(
        &self,
        guid: Option<&str>,
        name: Option<&str>,
        nickname: Option<&str>,
    ) -> Option<ComponentKind> {
        if let Some(guid) = guid {
            if let Some(component) = self.by_guid.get(&normalize_guid(guid)) {
                return Some(*component);
            }
        }

        if let Some(name) = name {
            if let Some(component) = self.by_name.get(&normalize_name(name)) {
                return Some(*component);
            }
        }

        if let Some(nickname) = nickname {
            if let Some(component) = self.by_name.get(&normalize_name(nickname)) {
                return Some(*component);
            }
        }

        None
    }
}

fn normalize_guid(guid: &str) -> String {
    guid.trim_matches(|c| c == '{' || c == '}').to_lowercase()
}

fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::{maths_operators, params_input, ComponentKind, ComponentRegistry};

    #[test]
    fn lookup_by_guid_and_name() {
        let registry = ComponentRegistry::default();

        let component = registry
            .resolve(Some("{57da07bd-ecab-415d-9d86-af36d7073abc}"), None, None)
            .unwrap();
        assert!(
            matches!(component, ComponentKind::ParamsInput(params_input::ComponentKind::NumberSlider))
        );

        let by_name = registry.resolve(None, Some("Add"), None).unwrap();
        assert!(matches!(
            by_name,
            ComponentKind::MathsOperator(maths_operators::ComponentKind::Addition)
        ));

        let by_nickname = registry.resolve(None, None, Some("extr")).unwrap();
        assert!(matches!(by_nickname, ComponentKind::Extrude(_)));
    }
}
