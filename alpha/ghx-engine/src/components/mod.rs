//! Component registry en evaluatie-logica.

use std::collections::HashMap;
use std::fmt;

use crate::graph::node::MetaMap;
use crate::graph::value::Value;

pub mod add;
pub mod construct_point;
pub mod extrude;
pub mod line;
pub mod maths_domain;
pub mod maths_matrix;
pub mod maths_operators;
pub mod maths_polynomials;
pub mod maths_script;
pub mod maths_time;
pub mod number_slider;

/// Output-map van een component: pinnickname → waarde.
pub type OutputMap = std::collections::BTreeMap<String, Value>;

/// Fouttype voor component-evaluaties.
#[derive(Debug, Clone)]
pub struct ComponentError {
    message: String,
}

impl ComponentError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
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
    NumberSlider(number_slider::ComponentImpl),
    Add(add::ComponentImpl),
    ConstructPoint(construct_point::ComponentImpl),
    Line(line::ComponentImpl),
    Extrude(extrude::ComponentImpl),
    MathsOperator(maths_operators::ComponentKind),
    MathsDomain(maths_domain::ComponentKind),
    MathsPolynomial(maths_polynomials::ComponentKind),
    MathsMatrix(maths_matrix::ComponentKind),
    MathsScript(maths_script::ComponentKind),
    MathsTime(maths_time::ComponentKind),
}

impl ComponentKind {
    #[must_use]
    pub fn evaluate(&self, inputs: &[Value], meta: &MetaMap) -> ComponentResult {
        match self {
            Self::NumberSlider(component) => component.evaluate(inputs, meta),
            Self::Add(component) => component.evaluate(inputs, meta),
            Self::ConstructPoint(component) => component.evaluate(inputs, meta),
            Self::Line(component) => component.evaluate(inputs, meta),
            Self::Extrude(component) => component.evaluate(inputs, meta),
            Self::MathsOperator(component) => component.evaluate(inputs, meta),
            Self::MathsDomain(component) => component.evaluate(inputs, meta),
            Self::MathsPolynomial(component) => component.evaluate(inputs, meta),
            Self::MathsMatrix(component) => component.evaluate(inputs, meta),
            Self::MathsScript(component) => component.evaluate(inputs, meta),
            Self::MathsTime(component) => component.evaluate(inputs, meta),
        }
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::NumberSlider(_) => "Number Slider",
            Self::Add(_) => "Addition",
            Self::ConstructPoint(_) => "Construct Point",
            Self::Line(_) => "Line",
            Self::Extrude(_) => "Extrude",
            Self::MathsOperator(component) => component.name(),
            Self::MathsDomain(component) => component.name(),
            Self::MathsPolynomial(component) => component.name(),
            Self::MathsMatrix(component) => component.name(),
            Self::MathsScript(component) => component.name(),
            Self::MathsTime(component) => component.name(),
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

        let number_slider = ComponentKind::NumberSlider(number_slider::ComponentImpl);
        registry.register_guid("{5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b}", number_slider);
        registry.register_names(&["Number Slider", "Slider"], number_slider);

        let add = ComponentKind::Add(add::ComponentImpl);
        registry.register_guid("{a0d62394-a118-422d-abb3-6af115c75b25}", add);
        registry.register_guid("{d18db32b-7099-4eea-85c4-8ba675ee8ec3}", add);
        registry.register_names(&["Addition", "Add", "A+B"], add);

        let construct_point = ComponentKind::ConstructPoint(construct_point::ComponentImpl);
        registry.register_guid("{3581f42a-9592-4549-bd6b-1c0fc39d067b}", construct_point);
        registry.register_names(&["Construct Point", "Point XYZ", "Point"], construct_point);

        let line = ComponentKind::Line(line::ComponentImpl);
        registry.register_guid("{4c4e56eb-2f04-43f9-95a3-cc46a14f495a}", line);
        registry.register_names(&["Line", "Ln"], line);

        let extrude = ComponentKind::Extrude(extrude::ComponentImpl);
        registry.register_guid("{962034e9-cc27-4394-afc4-5c16e3447cf9}", extrude);
        registry.register_names(&["Extrude", "Extr"], extrude);

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
    guid.chars()
        .filter(|c| *c != '{' && *c != '}')
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::{ComponentKind, ComponentRegistry};

    #[test]
    fn lookup_by_guid_and_name() {
        let registry = ComponentRegistry::default();

        let component = registry
            .resolve(Some("{5E0B22AB-F3AA-4CC2-8329-7E548BB9A58B}"), None, None)
            .unwrap();
        assert!(matches!(component, ComponentKind::NumberSlider(_)));

        let by_name = registry.resolve(None, Some("addition"), None).unwrap();
        assert!(matches!(by_name, ComponentKind::Add(_)));

        let by_nickname = registry.resolve(None, None, Some("extr")).unwrap();
        assert!(matches!(by_nickname, ComponentKind::Extrude(_)));
    }
}
