//! Basis Value-enum waarin componentwaarden en -resultaten worden
//! opgeslagen.

use core::fmt;

/// Beschikbare waardetypes binnen de evaluator.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Een enkele numerieke waarde.
    Number(f64),
    /// Een booleaanse waarde.
    Boolean(bool),
    /// Een 3D-punt.
    Point([f64; 3]),
    /// Een 3D-vector.
    Vector([f64; 3]),
    /// Een lijnsegment, beschreven door twee punten.
    CurveLine { p1: [f64; 3], p2: [f64; 3] },
    /// Een (prismatische) mesh representatie.
    Surface {
        vertices: Vec<[f64; 3]>,
        faces: Vec<Vec<u32>>,
    },
    /// Een numeriek domein (1D of 2D).
    Domain(Domain),
    /// Een matrix van numerieke waarden.
    Matrix(Matrix),
    /// Een lijst van waarden.
    List(Vec<Value>),
}

impl Value {
    /// Geeft de variantnaam terug. Wordt gebruikt in foutmeldingen.
    #[must_use]
    pub fn kind(&self) -> ValueKind {
        match self {
            Self::Number(_) => ValueKind::Number,
            Self::Boolean(_) => ValueKind::Boolean,
            Self::Point(_) => ValueKind::Point,
            Self::Vector(_) => ValueKind::Vector,
            Self::CurveLine { .. } => ValueKind::CurveLine,
            Self::Surface { .. } => ValueKind::Surface,
            Self::Domain(_) => ValueKind::Domain,
            Self::Matrix(_) => ValueKind::Matrix,
            Self::List(_) => ValueKind::List,
        }
    }

    /// Verwacht een `Number` en retourneert de f64-waarde.
    pub fn expect_number(&self) -> Result<f64, ValueError> {
        match self {
            Self::Number(value) => Ok(*value),
            _ => Err(ValueError::type_mismatch("Number", self.kind())),
        }
    }

    /// Verwacht een `Boolean` en retourneert de waarde.
    pub fn expect_boolean(&self) -> Result<bool, ValueError> {
        match self {
            Self::Boolean(value) => Ok(*value),
            _ => Err(ValueError::type_mismatch("Boolean", self.kind())),
        }
    }

    /// Verwacht een `Point` en retourneert de coördinaten.
    pub fn expect_point(&self) -> Result<[f64; 3], ValueError> {
        match self {
            Self::Point(point) => Ok(*point),
            _ => Err(ValueError::type_mismatch("Point", self.kind())),
        }
    }

    /// Verwacht een `Vector` en retourneert de componenten.
    pub fn expect_vector(&self) -> Result<[f64; 3], ValueError> {
        match self {
            Self::Vector(vector) => Ok(*vector),
            _ => Err(ValueError::type_mismatch("Vector", self.kind())),
        }
    }

    /// Verwacht een `CurveLine` en retourneert de eindpunten.
    pub fn expect_curve_line(&self) -> Result<([f64; 3], [f64; 3]), ValueError> {
        match self {
            Self::CurveLine { p1, p2 } => Ok((*p1, *p2)),
            _ => Err(ValueError::type_mismatch("CurveLine", self.kind())),
        }
    }

    /// Verwacht een `Surface` en retourneert de mesh-data.
    pub fn expect_surface(&self) -> Result<(&[[f64; 3]], &[Vec<u32>]), ValueError> {
        match self {
            Self::Surface { vertices, faces } => Ok((vertices, faces)),
            _ => Err(ValueError::type_mismatch("Surface", self.kind())),
        }
    }

    /// Verwacht een lijst en geeft een slice terug.
    pub fn expect_list(&self) -> Result<&[Value], ValueError> {
        match self {
            Self::List(values) => Ok(values),
            _ => Err(ValueError::type_mismatch("List", self.kind())),
        }
    }

    /// Verwacht een `Domain` en retourneert een verwijzing.
    pub fn expect_domain(&self) -> Result<&Domain, ValueError> {
        match self {
            Self::Domain(domain) => Ok(domain),
            _ => Err(ValueError::type_mismatch("Domain", self.kind())),
        }
    }

    /// Verwacht een `Matrix` en retourneert een verwijzing.
    pub fn expect_matrix(&self) -> Result<&Matrix, ValueError> {
        match self {
            Self::Matrix(matrix) => Ok(matrix),
            _ => Err(ValueError::type_mismatch("Matrix", self.kind())),
        }
    }
}

/// Typefout voor wanneer een `Value` naar het verkeerde type wordt
/// geconverteerd.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueError {
    expected: &'static str,
    found: ValueKind,
}

impl ValueError {
    #[must_use]
    pub fn type_mismatch(expected: &'static str, found: ValueKind) -> Self {
        Self { expected, found }
    }

    /// Hulptoegang voor tests en foutafhandeling.
    #[must_use]
    pub fn expected(&self) -> &'static str {
        self.expected
    }

    #[must_use]
    pub fn found(&self) -> ValueKind {
        self.found
    }
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "verwachtte type `{}` maar kreeg `{}`",
            self.expected, self.found
        )
    }
}

impl std::error::Error for ValueError {}

/// Beschrijft het soort `Value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Number,
    Boolean,
    Point,
    Vector,
    CurveLine,
    Surface,
    Domain,
    List,
    Matrix,
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Number => "Number",
            Self::Boolean => "Boolean",
            Self::Point => "Point",
            Self::Vector => "Vector",
            Self::CurveLine => "CurveLine",
            Self::Surface => "Surface",
            Self::Domain => "Domain",
            Self::Matrix => "Matrix",
            Self::List => "List",
        };
        f.write_str(name)
    }
}

/// Een eenvoudige matrixstructuur die door componenten kan worden gebruikt.
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    pub rows: usize,
    pub columns: usize,
    pub values: Vec<f64>,
}

impl Matrix {
    /// Maakt een matrix aan wanneer de afmetingen en waarden overeenkomen.
    #[must_use]
    pub fn new(rows: usize, columns: usize, values: Vec<f64>) -> Option<Self> {
        if rows == 0 || columns == 0 || values.len() != rows * columns {
            return None;
        }
        Some(Self {
            rows,
            columns,
            values,
        })
    }
}

/// Een één-dimensionaal numeriek domein.
#[derive(Debug, Clone, PartialEq)]
pub struct Domain1D {
    pub start: f64,
    pub end: f64,
    pub min: f64,
    pub max: f64,
    pub span: f64,
    pub length: f64,
    pub center: f64,
}

/// Een twee-dimensionaal domein opgebouwd uit twee 1D-domeinen.
#[derive(Debug, Clone, PartialEq)]
pub struct Domain2D {
    pub u: Domain1D,
    pub v: Domain1D,
}

/// Beschikbare domeinvarianten die opgeslagen kunnen worden in `Value::Domain`.
#[derive(Debug, Clone, PartialEq)]
pub enum Domain {
    One(Domain1D),
    Two(Domain2D),
}

#[cfg(test)]
mod tests {
    use super::{Value, ValueError, ValueKind};

    #[test]
    fn expect_number_accepts_number() {
        let value = Value::Number(42.0);
        assert_eq!(value.expect_number().unwrap(), 42.0);
    }

    #[test]
    fn expect_number_rejects_wrong_type() {
        let value = Value::Point([0.0, 0.0, 0.0]);
        let err = value.expect_number().unwrap_err();
        assert_eq!(err.expected(), "Number");
        assert_eq!(err.found(), ValueKind::Point);
    }

    #[test]
    fn expect_boolean_accepts_boolean() {
        let value = Value::Boolean(true);
        assert!(value.expect_boolean().unwrap());
    }

    #[test]
    fn expect_boolean_rejects_other_types() {
        let value = Value::Number(0.0);
        let err = value.expect_boolean().unwrap_err();
        assert_eq!(err.expected(), "Boolean");
        assert_eq!(err.found(), ValueKind::Number);
    }

    #[test]
    fn expect_surface_returns_references() {
        let vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let faces = vec![vec![0, 1, 1]];
        let value = Value::Surface {
            vertices: vertices.clone(),
            faces: faces.clone(),
        };

        let (verts, fcs) = value.expect_surface().unwrap();
        assert_eq!(verts, vertices.as_slice());
        assert_eq!(fcs, faces.as_slice());
    }

    #[test]
    fn expect_curve_line_returns_endpoints() {
        let value = Value::CurveLine {
            p1: [0.0, 0.0, 0.0],
            p2: [1.0, 2.0, 3.0],
        };
        let (p1, p2) = value.expect_curve_line().unwrap();
        assert_eq!(p1, [0.0, 0.0, 0.0]);
        assert_eq!(p2, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn list_expectation_requires_list() {
        let value = Value::List(vec![Value::Number(1.0)]);
        assert_eq!(value.expect_list().unwrap().len(), 1);

        let non_list = Value::Number(3.0);
        assert!(matches!(non_list.expect_list(), Err(ValueError { .. })));
    }
}
