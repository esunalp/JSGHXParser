//! Basis Value-enum waarin componentwaarden en -resultaten worden
//! opgeslagen.

use core::fmt;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use num_complex::Complex;
use time::PrimitiveDateTime;

/// Een complex getal, aliased van `num_complex::Complex`.
pub type ComplexValue = Complex<f64>;

/// Beschikbare waardetypes binnen de evaluator.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Een null-waarde, vergelijkbaar met `null` in andere talen.
    Null,
    /// Een enkele numerieke waarde.
    Number(f64),
    /// Een booleaanse waarde.
    Boolean(bool),
    /// Een complex getal.
    Complex(ComplexValue),
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
    /// Een datum-tijdwaarde zonder tijdzone.
    DateTime(DateTimeValue),
    /// Een lijst van waarden.
    List(Vec<Value>),
    /// Een tekstuele waarde.
    Text(String),
    /// Een tekstlabel met oriëntatie en optionele kleur.
    Tag(TextTagValue),
    /// Een RGB-kleurwaarde.
    Color(ColorValue),
    /// Een weergavemateriaal.
    Material(MaterialValue),
    /// Een weergavesymbool.
    Symbol(SymbolValue),
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Value::Null => {}
            Value::Number(n) => n.to_bits().hash(state),
            Value::Boolean(b) => b.hash(state),
            Value::Complex(c) => {
                c.re.to_bits().hash(state);
                c.im.to_bits().hash(state);
            }
            Value::Point(p) => {
                p.iter().for_each(|x| x.to_bits().hash(state));
            }
            Value::Vector(v) => {
                v.iter().for_each(|x| x.to_bits().hash(state));
            }
            Value::CurveLine { p1, p2 } => {
                p1.iter().for_each(|x| x.to_bits().hash(state));
                p2.iter().for_each(|x| x.to_bits().hash(state));
            }
            Value::List(l) => l.hash(state),
            Value::Text(s) => s.hash(state),
            Value::DateTime(dt) => dt.hash(state),
            Value::Color(c) => {
                c.r.to_bits().hash(state);
                c.g.to_bits().hash(state);
                c.b.to_bits().hash(state);
            }
            // Non-trivial hash impls below.
            // For now, these are not hashed, which is not ideal but avoids complexity.
            Value::Surface { .. } => {}
            Value::Domain(_) => {}
            Value::Matrix(_) => {}
            Value::Tag(_) => {}
            Value::Material(_) => {}
            Value::Symbol(_) => {}
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "Null"),
            Self::Number(n) => write!(f, "{}", n),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Complex(c) => write!(f, "{}", c),
            Self::Point(p) => write!(f, "{},{},{}", p[0], p[1], p[2]),
            Self::Vector(v) => write!(f, "{},{},{}", v[0], v[1], v[2]),
            Self::CurveLine { p1, p2 } => write!(
                f,
                "Line [{},{},{}] to [{},{},{}]",
                p1[0], p1[1], p1[2], p2[0], p2[1], p2[2]
            ),
            Self::Surface { vertices, faces } => {
                write!(
                    f,
                    "Surface [{} vertices, {} faces]",
                    vertices.len(),
                    faces.len()
                )
            }
            Self::Domain(d) => match d {
                Domain::One(d1) => write!(f, "Domain {} to {}", d1.start, d1.end),
                Domain::Two(d2) => write!(
                    f,
                    "Domain U({} to {}), V({} to {})",
                    d2.u.start, d2.u.end, d2.v.start, d2.v.end
                ),
            },
            Self::Matrix(m) => write!(f, "Matrix [{}x{}]", m.rows, m.columns),
            Self::DateTime(dt) => write!(f, "{}", dt.primitive()),
            Self::List(l) => write!(f, "List [{} items]", l.len()),
            Self::Text(s) => write!(f, "{}", s),
            Self::Tag(t) => write!(f, "Tag: {}", t.text),
            Self::Color(c) => write!(f, "Color [R={}, G={}, B={}]", c.r, c.g, c.b),
            Self::Material(_) => write!(f, "Material"),
            Self::Symbol(_) => write!(f, "Symbol"),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::Boolean(a), Value::Boolean(b)) => a.partial_cmp(b),
            (Value::DateTime(a), Value::DateTime(b)) => a.primitive().partial_cmp(&b.primitive()),
            (Value::Text(a), Value::Text(b)) => a.partial_cmp(b),
            (Value::Null, Value::Null) => Some(Ordering::Equal),
            _ => None,
        }
    }
}

impl Value {
    /// Geeft de variantnaam terug. Wordt gebruikt in foutmeldingen.
    #[must_use]
    pub fn kind(&self) -> ValueKind {
        match self {
            Self::Null => ValueKind::Null,
            Self::Number(_) => ValueKind::Number,
            Self::Boolean(_) => ValueKind::Boolean,
            Self::Complex(_) => ValueKind::Complex,
            Self::Point(_) => ValueKind::Point,
            Self::Vector(_) => ValueKind::Vector,
            Self::CurveLine { .. } => ValueKind::CurveLine,
            Self::Surface { .. } => ValueKind::Surface,
            Self::Domain(_) => ValueKind::Domain,
            Self::Matrix(_) => ValueKind::Matrix,
            Self::DateTime(_) => ValueKind::DateTime,
            Self::List(_) => ValueKind::List,
            Self::Text(_) => ValueKind::Text,
            Self::Tag(_) => ValueKind::Tag,
            Self::Color(_) => ValueKind::Color,
            Self::Material(_) => ValueKind::Material,
            Self::Symbol(_) => ValueKind::Symbol,
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

    /// Verwacht een `Complex` en retourneert de waarde.
    pub fn expect_complex(&self) -> Result<ComplexValue, ValueError> {
        match self {
            Self::Complex(value) => Ok(*value),
            _ => Err(ValueError::type_mismatch("Complex", self.kind())),
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

    /// Verwacht een `DateTime` en retourneert de waarde.
    pub fn expect_date_time(&self) -> Result<PrimitiveDateTime, ValueError> {
        match self {
            Self::DateTime(date_time) => Ok(date_time.primitive()),
            _ => Err(ValueError::type_mismatch("DateTime", self.kind())),
        }
    }

    /// Verwacht een `Tag` en retourneert de taggegevens.
    pub fn expect_tag(&self) -> Result<&TextTagValue, ValueError> {
        match self {
            Self::Tag(tag) => Ok(tag),
            _ => Err(ValueError::type_mismatch("Tag", self.kind())),
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
    Null,
    Number,
    Boolean,
    Point,
    Vector,
    CurveLine,
    Surface,
    Domain,
    List,
    Matrix,
    Complex,
    DateTime,
    Text,
    Tag,
    Color,
    Material,
    Symbol,
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Null => "Null",
            Self::Number => "Number",
            Self::Boolean => "Boolean",
            Self::Point => "Point",
            Self::Vector => "Vector",
            Self::CurveLine => "CurveLine",
            Self::Surface => "Surface",
            Self::Domain => "Domain",
            Self::Matrix => "Matrix",
            Self::Complex => "Complex",
            Self::DateTime => "DateTime",
            Self::List => "List",
            Self::Text => "Text",
            Self::Tag => "Tag",
            Self::Color => "Color",
            Self::Material => "Material",
            Self::Symbol => "Symbol",
        };
        f.write_str(name)
    }
}

/// Beschrijving van een weergavemateriaal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialValue {
    pub diffuse: ColorValue,
    pub specular: ColorValue,
    pub emission: ColorValue,
    pub transparency: f64,
    pub shine: f64,
}

/// Beschrijving van een weergavesymbool.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolValue {
    pub style: String,
    pub size_primary: f64,
    pub size_secondary: Option<f64>,
    pub rotation: f64,
    pub fill: ColorValue,
    pub edge: Option<ColorValue>,
    pub width: f64,
    pub adjust: bool,
}

/// Beschrijving van een vlak in de ruimte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaneValue {
    pub origin: [f64; 3],
    pub x_axis: [f64; 3],
    pub y_axis: [f64; 3],
    pub z_axis: [f64; 3],
}

impl PlaneValue {
    /// Maak een nieuw vlak met opgegeven basisvectoren.
    #[must_use]
    pub fn new(origin: [f64; 3], x_axis: [f64; 3], y_axis: [f64; 3], z_axis: [f64; 3]) -> Self {
        Self {
            origin,
            x_axis,
            y_axis,
            z_axis,
        }
    }

    /// Geeft een standaard vlak terug met assen gelijk aan de wereldassen.
    #[must_use]
    pub fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            z_axis: [0.0, 0.0, 1.0],
        }
    }
}

/// RGB kleurwaarden genormaliseerd tussen 0 en 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorValue {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl ColorValue {
    /// Maak een nieuwe kleur aan en klem componenten binnen [0, 1].
    #[must_use]
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: clamp01(r),
            g: clamp01(g),
            b: clamp01(b),
        }
    }

    /// Maak een kleur uit waarden in het bereik [0, 255].
    #[must_use]
    pub fn from_rgb255(r: f64, g: f64, b: f64) -> Self {
        Self::new(r / 255.0, g / 255.0, b / 255.0)
    }

    /// Maak een grijstint op basis van een scalar.
    #[must_use]
    pub fn grayscale(value: f64) -> Self {
        if value <= 1.0 {
            Self::new(value, value, value)
        } else {
            Self::from_rgb255(value, value, value)
        }
    }
}

fn clamp01(value: f64) -> f64 {
    if !value.is_finite() {
        return 0.0;
    }
    if value <= 0.0 {
        0.0
    } else if value >= 1.0 {
        1.0
    } else {
        value
    }
}

/// Beschrijving van een Grasshopper teksttag.
#[derive(Debug, Clone, PartialEq)]
pub struct TextTagValue {
    pub plane: PlaneValue,
    pub text: String,
    pub size: f64,
    pub color: Option<ColorValue>,
}

impl TextTagValue {
    /// Maak een nieuwe teksttag aan.
    #[must_use]
    pub fn new(
        plane: PlaneValue,
        text: impl Into<String>,
        size: f64,
        color: Option<ColorValue>,
    ) -> Self {
        Self {
            plane,
            text: text.into(),
            size,
            color,
        }
    }
}

/// Een tijdstip bestaande uit een datum en tijd zonder tijdzone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DateTimeValue {
    datetime: PrimitiveDateTime,
}

impl DateTimeValue {
    /// Maakt een nieuwe datum-tijdwaarde aan.
    #[must_use]
    pub fn from_primitive(datetime: PrimitiveDateTime) -> Self {
        Self { datetime }
    }

    /// Geeft de onderliggende `PrimitiveDateTime` terug.
    #[must_use]
    pub fn primitive(&self) -> PrimitiveDateTime {
        self.datetime
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
    use super::{ComplexValue, DateTimeValue, Value, ValueError, ValueKind};
    use num_complex::Complex;
    use time::macros::datetime;

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
    fn expect_complex_accepts_complex() {
        let value = Value::Complex(ComplexValue::new(2.0, -3.5));
        assert_eq!(value.expect_complex().unwrap(), Complex::new(2.0, -3.5));
    }

    #[test]
    fn complex_helpers_compute_properties() {
        let complex = ComplexValue::new(3.0, 4.0);
        assert_eq!(complex.norm(), 5.0);
        assert_eq!(complex.conj(), Complex::new(3.0, -4.0));
        assert!((complex.arg() - (4.0f64).atan2(3.0)).abs() < 1e-12);
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

    #[test]
    fn expect_date_time_returns_datetime() {
        let datetime = datetime!(2024-06-01 12:30:45);
        let value = Value::DateTime(DateTimeValue::from_primitive(datetime));
        assert_eq!(value.expect_date_time().unwrap(), datetime);
    }

    #[test]
    fn expect_date_time_rejects_other_types() {
        let value = Value::Number(1.0);
        let err = value.expect_date_time().unwrap_err();
        assert_eq!(err.expected(), "DateTime");
        assert_eq!(err.found(), ValueKind::Number);
    }
}
