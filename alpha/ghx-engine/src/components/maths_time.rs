//! Implementaties van Grasshopper "Maths → Time" componenten.

use std::collections::BTreeMap;
use std::convert::TryFrom;

use time::macros::{date, datetime, time};
use time::{Date, Duration, Month, PrimitiveDateTime, Time};

use crate::graph::node::MetaMap;
use crate::graph::value::{DateTimeValue, Value};

use super::{Component, ComponentError, ComponentResult};

const PIN_YEAR: &str = "Y";
const PIN_MONTH: &str = "M";
const PIN_DAY: &str = "D";
const PIN_HOUR_LOWER: &str = "h";
const PIN_MINUTE_LOWER: &str = "m";
const PIN_SECOND_LOWER: &str = "s";
const PIN_DATE: &str = "D";
const PIN_TIME: &str = "T";
const PIN_RESULT: &str = "R";
const PIN_RANGE: &str = "R";

const BASE_DATE: Date = date!(1970 - 01 - 01);
const BASE_TIME: Time = time!(0:00:00);
const BASE_DATETIME: PrimitiveDateTime = datetime!(1970-01-01 0:00:00);

/// Beschikbare componenten binnen deze module.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    ConstructDate,
    CombineDateTime,
    DateRange,
    InterpolateDate,
    ConstructTime,
    DeconstructDate,
    ConstructExoticDate,
    ConstructSmoothTime,
}

/// Metadata voor registraties in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Volledige lijst van componentregistraties voor de maths-time componenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{0c2f0932-5ddc-4ece-bd84-a3a059d3df7a}"],
        names: &["Construct Date", "Date"],
        kind: ComponentKind::ConstructDate,
    },
    Registration {
        guids: &["{31534405-6573-4be6-8bf8-262e55847a3a}"],
        names: &["Combine Date & Time", "CDate"],
        kind: ComponentKind::CombineDateTime,
    },
    Registration {
        guids: &["{38a4e722-ad5a-4229-a170-e27ae1345538}"],
        names: &["Date Range", "RDate"],
        kind: ComponentKind::DateRange,
    },
    Registration {
        guids: &["{4083802b-3dd9-4b13-9756-bf5441213e70}"],
        names: &["Interpolate Date", "IntDate"],
        kind: ComponentKind::InterpolateDate,
    },
    Registration {
        guids: &["{595aded2-8916-402d-87a3-a825244bbe3d}"],
        names: &["Construct Time", "Time"],
        kind: ComponentKind::ConstructTime,
    },
    Registration {
        guids: &["{d5e28df8-495b-4892-bca8-60748743d955}"],
        names: &["Deconstruct Date", "DDate"],
        kind: ComponentKind::DeconstructDate,
    },
    Registration {
        guids: &["{e5ff52c5-40df-4f43-ac3b-d2418d05ae32}"],
        names: &["Construct Exotic Date", "DateEx"],
        kind: ComponentKind::ConstructExoticDate,
    },
    Registration {
        guids: &["{f151b0b9-cef8-4809-96fc-9b14f1c3a7b9}"],
        names: &["Construct Smooth Time", "SmTime"],
        kind: ComponentKind::ConstructSmoothTime,
    },
];

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::ConstructDate => evaluate_construct_date(inputs),
            Self::CombineDateTime => evaluate_combine_date_time(inputs),
            Self::DateRange => evaluate_date_range(inputs),
            Self::InterpolateDate => evaluate_interpolate_date(inputs),
            Self::ConstructTime => evaluate_construct_time(inputs),
            Self::DeconstructDate => evaluate_deconstruct_date(inputs),
            Self::ConstructExoticDate => evaluate_construct_exotic_date(inputs),
            Self::ConstructSmoothTime => evaluate_construct_smooth_time(inputs),
        }
    }
}

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::ConstructDate => "Construct Date",
            Self::CombineDateTime => "Combine Date & Time",
            Self::DateRange => "Date Range",
            Self::InterpolateDate => "Interpolate Date",
            Self::ConstructTime => "Construct Time",
            Self::DeconstructDate => "Deconstruct Date",
            Self::ConstructExoticDate => "Construct Exotic Date",
            Self::ConstructSmoothTime => "Construct Smooth Time",
        }
    }
}

fn evaluate_construct_date(inputs: &[Value]) -> ComponentResult {
    let year = coerce_required_integer(inputs.get(0), 1, 9999, "Year")?;
    let month = coerce_required_integer(inputs.get(1), 1, 12, "Month")?;
    let day = coerce_required_integer(inputs.get(2), 1, 31, "Day")?;
    let hour = coerce_integer_with_default(inputs.get(3), 0, 0, 23, "Hour")?;
    let minute = coerce_integer_with_default(inputs.get(4), 0, 0, 59, "Minute")?;
    let second = coerce_second_with_default(inputs.get(5), 0.0, "Second")?;

    let month_enum = Month::try_from(month as u8).map_err(|_| {
        ComponentError::new(format!("Month moet tussen 1 en 12 liggen, kreeg {month}"))
    })?;
    let date = Date::from_calendar_date(year, month_enum, day as u8)
        .map_err(|err| ComponentError::new(format!("Ongeldige datum: {err}")))?;
    let time = build_time(hour, minute, second)?;

    map_with_datetime(PIN_DATE, PrimitiveDateTime::new(date, time))
}

fn evaluate_combine_date_time(inputs: &[Value]) -> ComponentResult {
    let date = coerce_date_time(inputs.get(0), "Date")?;
    let time = coerce_date_time(inputs.get(1), "Time")?;
    let combined = PrimitiveDateTime::new(date.date(), time.time());
    map_with_datetime(PIN_RESULT, combined)
}

fn evaluate_date_range(inputs: &[Value]) -> ComponentResult {
    let start = coerce_date_time(inputs.get(0), "Time A")?;
    let end = coerce_date_time(inputs.get(1), "Time B")?;
    let count = coerce_required_integer(inputs.get(2), 2, 10_000, "Count")? as usize;

    if count < 2 {
        return Err(ComponentError::new("Count moet minimaal 2 zijn"));
    }

    let diff = end - start;
    let diff_nanos = diff.whole_nanoseconds();
    let mut values = Vec::with_capacity(count);
    let step_divisor = (count - 1) as i128;

    for index in 0..count {
        let datetime = if index == count - 1 {
            end
        } else {
            let offset_nanos = if step_divisor == 0 {
                0
            } else {
                diff_nanos.saturating_mul(index as i128) / step_divisor
            };
            let offset = duration_from_nanoseconds(offset_nanos)?;
            checked_add_duration(start, offset)?
        };
        values.push(Value::DateTime(DateTimeValue::from_primitive(datetime)));
    }

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_RANGE.to_owned(), Value::List(values));
    Ok(outputs)
}

fn evaluate_interpolate_date(inputs: &[Value]) -> ComponentResult {
    let start = coerce_date_time(inputs.get(0), "Date A")?;
    let end = coerce_date_time(inputs.get(1), "Date B")?;
    let factor = coerce_number(inputs.get(2), "Interpolation")?;

    let diff = end - start;
    let nanos = (diff.whole_nanoseconds() as f32 * factor).round();
    if !nanos.is_finite() || nanos < i128::MIN as f32 || nanos > i128::MAX as f32 {
        return Err(ComponentError::new(
            "Interpolation resulteert in een waarde buiten het bereik",
        ));
    }
    let offset = duration_from_nanoseconds(nanos as i128)?;
    let result = checked_add_duration(start, offset)?;
    map_with_datetime(PIN_DATE, result)
}

fn evaluate_construct_time(inputs: &[Value]) -> ComponentResult {
    let hour = coerce_required_integer(inputs.get(0), 0, 23, "Hour")?;
    let minute = coerce_integer_with_default(inputs.get(1), 0, 0, 59, "Minute")?;
    let second = coerce_second_with_default(inputs.get(2), 0.0, "Second")?;

    let time = build_time(hour, minute, second)?;
    let datetime = PrimitiveDateTime::new(BASE_DATE, time);
    map_with_datetime(PIN_TIME, datetime)
}

fn evaluate_deconstruct_date(inputs: &[Value]) -> ComponentResult {
    let datetime = coerce_date_time(inputs.get(0), "Date")?;
    let date = datetime.date();
    let time = datetime.time();

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_YEAR.to_owned(), Value::Number(date.year() as f32));
    outputs.insert(
        PIN_MONTH.to_owned(),
        Value::Number((date.month() as u8) as f32),
    );
    outputs.insert(PIN_DAY.to_owned(), Value::Number(date.day() as f32));
    outputs.insert(PIN_HOUR_LOWER.to_owned(), Value::Number(time.hour() as f32));
    outputs.insert(
        PIN_MINUTE_LOWER.to_owned(),
        Value::Number(time.minute() as f32),
    );
    let seconds = time.second() as f32 + time.nanosecond() as f32 / 1_000_000_000.0;
    outputs.insert(PIN_SECOND_LOWER.to_owned(), Value::Number(seconds));
    Ok(outputs)
}

fn evaluate_construct_exotic_date(inputs: &[Value]) -> ComponentResult {
    let year = coerce_required_integer(inputs.get(0), 1, 9999, "Year")?;
    let month = coerce_required_integer(inputs.get(1), 1, 12, "Month")?;
    let day = coerce_required_integer(inputs.get(2), 1, 31, "Day")?;

    let month_enum = Month::try_from(month as u8).map_err(|_| {
        ComponentError::new(format!("Month moet tussen 1 en 12 liggen, kreeg {month}"))
    })?;
    let date = Date::from_calendar_date(year, month_enum, day as u8)
        .map_err(|err| ComponentError::new(format!("Ongeldige datum: {err}")))?;
    let datetime = PrimitiveDateTime::new(date, BASE_TIME);
    map_with_datetime(PIN_TIME, datetime)
}

fn evaluate_construct_smooth_time(inputs: &[Value]) -> ComponentResult {
    let days = coerce_number_with_default(inputs.get(0), 0.0, "Days")?;
    let hours = coerce_number_with_default(inputs.get(1), 0.0, "Hours")?;
    let minutes = coerce_number_with_default(inputs.get(2), 0.0, "Minutes")?;
    let seconds = coerce_number_with_default(inputs.get(3), 0.0, "Seconds")?;

    let total_seconds = days * 86_400.0 + hours * 3_600.0 + minutes * 60.0 + seconds;
    if !total_seconds.is_finite()
        || total_seconds < (i128::MIN as f32) / 1_000_000_000.0
        || total_seconds > (i128::MAX as f32) / 1_000_000_000.0
    {
        return Err(ComponentError::new(
            "De ingevoerde waarden leveren een tijd buiten het bereik op",
        ));
    }
    let nanos = (total_seconds * 1_000_000_000.0).round();
    let duration = duration_from_nanoseconds(nanos as i128)?;
    let datetime = checked_add_duration(BASE_DATETIME, duration)?;
    map_with_datetime(PIN_TIME, datetime)
}

fn coerce_required_integer(
    value: Option<&Value>,
    min: i32,
    max: i32,
    label: &str,
) -> Result<i32, ComponentError> {
    let value = value.ok_or_else(|| ComponentError::new(format!("{label} vereist een invoer")))?;
    let integer = integer_from_value(value, label)?;
    if integer < min || integer > max {
        return Err(ComponentError::new(format!(
            "{label} moet tussen {min} en {max} liggen, kreeg {integer}"
        )));
    }
    Ok(integer)
}

fn coerce_integer_with_default(
    value: Option<&Value>,
    default: i32,
    min: i32,
    max: i32,
    label: &str,
) -> Result<i32, ComponentError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let integer = integer_from_value(value, label)?;
    if integer < min || integer > max {
        return Err(ComponentError::new(format!(
            "{label} moet tussen {min} en {max} liggen, kreeg {integer}"
        )));
    }
    Ok(integer)
}

fn coerce_second_with_default(
    value: Option<&Value>,
    default: f32,
    label: &str,
) -> Result<f32, ComponentError> {
    let seconds = match value {
        Some(value) => number_from_value(value, label)?,
        None => return Ok(default),
    };
    if !(0.0..60.0).contains(&seconds) {
        return Err(ComponentError::new(format!(
            "{label} moet tussen 0 en 60 liggen, kreeg {seconds}"
        )));
    }
    Ok(seconds)
}

fn coerce_number(value: Option<&Value>, label: &str) -> Result<f32, ComponentError> {
    let value = value.ok_or_else(|| ComponentError::new(format!("{label} vereist een invoer")))?;
    number_from_value(value, label)
}

fn coerce_number_with_default(
    value: Option<&Value>,
    default: f32,
    label: &str,
) -> Result<f32, ComponentError> {
    match value {
        Some(value) => number_from_value(value, label),
        None => Ok(default),
    }
}

fn coerce_date_time(
    value: Option<&Value>,
    label: &str,
) -> Result<PrimitiveDateTime, ComponentError> {
    let value = value.ok_or_else(|| ComponentError::new(format!("{label} vereist een invoer")))?;
    date_time_from_value(value, label)
}

fn integer_from_value(value: &Value, label: &str) -> Result<i32, ComponentError> {
    match value {
        Value::Number(number) => {
            if !number.is_finite() {
                return Err(ComponentError::new(format!(
                    "{label} moet een eindig getal zijn"
                )));
            }
            let rounded = number.round();
            if (rounded - number).abs() > 1e-6 {
                return Err(ComponentError::new(format!(
                    "{label} verwacht een geheel getal, kreeg {number}"
                )));
            }
            if rounded < i32::MIN as f32 || rounded > i32::MAX as f32 {
                return Err(ComponentError::new(format!(
                    "{label} valt buiten het ondersteunde bereik"
                )));
            }
            Ok(rounded as i32)
        }
        Value::List(values) if values.len() == 1 => integer_from_value(&values[0], label),
        other => Err(ComponentError::new(format!(
            "{label} verwacht een Number, kreeg {}",
            other.kind()
        ))),
    }
}

fn number_from_value(value: &Value, label: &str) -> Result<f32, ComponentError> {
    match value {
        Value::Number(number) => {
            if !number.is_finite() {
                return Err(ComponentError::new(format!(
                    "{label} moet een eindig getal zijn"
                )));
            }
            Ok(*number)
        }
        Value::List(values) if values.len() == 1 => number_from_value(&values[0], label),
        other => Err(ComponentError::new(format!(
            "{label} verwacht een Number, kreeg {}",
            other.kind()
        ))),
    }
}

fn date_time_from_value(value: &Value, label: &str) -> Result<PrimitiveDateTime, ComponentError> {
    match value {
        Value::DateTime(date_time) => Ok(date_time.primitive()),
        Value::List(values) if values.len() == 1 => date_time_from_value(&values[0], label),
        other => Err(ComponentError::new(format!(
            "{label} verwacht een DateTime, kreeg {}",
            other.kind()
        ))),
    }
}

fn build_time(hour: i32, minute: i32, seconds: f32) -> Result<Time, ComponentError> {
    if !(0.0..60.0).contains(&seconds) {
        return Err(ComponentError::new(format!(
            "Second moet tussen 0 en 60 liggen, kreeg {seconds}"
        )));
    }
    let whole = seconds.floor();
    let nanos = ((seconds - whole) * 1_000_000_000.0).round();
    let nanos = nanos.clamp(0.0, 999_999_999.0) as u32;
    Time::from_hms_nano(hour as u8, minute as u8, whole as u8, nanos)
        .map_err(|err| ComponentError::new(format!("Ongeldige tijd: {err}")))
}

fn map_with_datetime(pin: &str, datetime: PrimitiveDateTime) -> ComponentResult {
    let mut outputs = BTreeMap::new();
    outputs.insert(
        pin.to_owned(),
        Value::DateTime(DateTimeValue::from_primitive(datetime)),
    );
    Ok(outputs)
}

fn checked_add_duration(
    datetime: PrimitiveDateTime,
    duration: Duration,
) -> Result<PrimitiveDateTime, ComponentError> {
    datetime.checked_add(duration).ok_or_else(|| {
        ComponentError::new("Datum-tijd berekening valt buiten het ondersteunde bereik")
    })
}

fn duration_from_nanoseconds(nanos: i128) -> Result<Duration, ComponentError> {
    let value = i64::try_from(nanos).map_err(|_| {
        ComponentError::new("Datum-tijd berekening valt buiten het ondersteunde bereik")
    })?;
    Ok(Duration::nanoseconds(value))
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentKind, PIN_DATE, PIN_RANGE, PIN_RESULT, PIN_TIME};
    use crate::graph::node::MetaMap;
    use crate::graph::value::{DateTimeValue, Value};
    use time::PrimitiveDateTime;
    use time::macros::datetime;

    #[test]
    fn construct_date_includes_time_components() {
        let component = ComponentKind::ConstructDate;
        let outputs = component
            .evaluate(
                &[
                    Value::Number(2024.0),
                    Value::Number(6.0),
                    Value::Number(15.0),
                    Value::Number(14.0),
                    Value::Number(30.0),
                    Value::Number(45.5),
                ],
                &MetaMap::new(),
            )
            .expect("construct date succeeds");

        let Value::DateTime(result) = outputs.get(PIN_DATE).expect("date output").clone() else {
            panic!("expected datetime value");
        };
        assert_eq!(result.primitive(), datetime!(2024-06-15 14:30:45.5));
    }

    #[test]
    fn combine_date_and_time_uses_components() {
        let component = ComponentKind::CombineDateTime;
        let date = Value::DateTime(DateTimeValue::from_primitive(
            datetime!(2023-01-01 01:02:03),
        ));
        let time = Value::DateTime(DateTimeValue::from_primitive(
            datetime!(1970-01-01 23:59:59),
        ));
        let outputs = component
            .evaluate(&[date, time], &MetaMap::new())
            .expect("combine date");
        let Value::DateTime(result) = outputs.get(PIN_RESULT).expect("result").clone() else {
            panic!("expected datetime value");
        };
        assert_eq!(result.primitive(), datetime!(2023-01-01 23:59:59));
    }

    #[test]
    fn date_range_creates_sequence() {
        let component = ComponentKind::DateRange;
        let outputs = component
            .evaluate(
                &[
                    Value::DateTime(DateTimeValue::from_primitive(datetime!(2023-01-01 00:00))),
                    Value::DateTime(DateTimeValue::from_primitive(datetime!(2023-01-01 03:00))),
                    Value::Number(3.0),
                ],
                &MetaMap::new(),
            )
            .expect("date range");

        let Value::List(values) = outputs.get(PIN_RANGE).expect("range").clone() else {
            panic!("expected list");
        };
        assert_eq!(values.len(), 3);
        let first = extract_datetime(&values[0]);
        let second = extract_datetime(&values[1]);
        let third = extract_datetime(&values[2]);
        assert_eq!(first, datetime!(2023-01-01 00:00));
        assert_eq!(second, datetime!(2023-01-01 01:30));
        assert_eq!(third, datetime!(2023-01-01 03:00));
    }

    #[test]
    fn interpolate_date_returns_midpoint() {
        let component = ComponentKind::InterpolateDate;
        let outputs = component
            .evaluate(
                &[
                    Value::DateTime(DateTimeValue::from_primitive(datetime!(2023-01-01 00:00))),
                    Value::DateTime(DateTimeValue::from_primitive(datetime!(2023-01-02 00:00))),
                    Value::Number(0.5),
                ],
                &MetaMap::new(),
            )
            .expect("interpolate");
        let Value::DateTime(result) = outputs.get(PIN_DATE).expect("date").clone() else {
            panic!("expected datetime value");
        };
        assert_eq!(result.primitive(), datetime!(2023-01-01 12:00));
    }

    #[test]
    fn construct_time_uses_base_date() {
        let component = ComponentKind::ConstructTime;
        let outputs = component
            .evaluate(
                &[Value::Number(5.0), Value::Number(15.0), Value::Number(30.0)],
                &MetaMap::new(),
            )
            .expect("construct time");
        let Value::DateTime(result) = outputs.get(PIN_TIME).expect("time").clone() else {
            panic!("expected datetime value");
        };
        assert_eq!(result.primitive(), datetime!(1970-01-01 05:15:30));
    }

    #[test]
    fn construct_smooth_time_accumulates_components() {
        let component = ComponentKind::ConstructSmoothTime;
        let outputs = component
            .evaluate(
                &[
                    Value::Number(1.0),
                    Value::Number(2.0),
                    Value::Number(0.5),
                    Value::Number(30.0),
                ],
                &MetaMap::new(),
            )
            .expect("smooth time");
        let Value::DateTime(result) = outputs.get(PIN_TIME).expect("time").clone() else {
            panic!("expected datetime value");
        };
        assert_eq!(result.primitive(), datetime!(1970-01-02 02:01:00));
    }

    #[test]
    fn construct_date_validates_inputs() {
        let component = ComponentKind::ConstructDate;
        let err = component
            .evaluate(
                &[
                    Value::Number(2023.0),
                    Value::Number(13.0),
                    Value::Number(1.0),
                ],
                &MetaMap::new(),
            )
            .unwrap_err();
        assert!(err.message().contains("Month"));
    }

    fn extract_datetime(value: &Value) -> PrimitiveDateTime {
        let Value::DateTime(date_time) = value.clone() else {
            panic!("expected datetime value");
        };
        date_time.primitive()
    }
}
