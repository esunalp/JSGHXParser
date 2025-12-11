//! Implementaties van de Grasshopper "Maths → Matrix" componenten.

use std::collections::BTreeMap;

use crate::graph::node::MetaMap;
use crate::graph::value::{Matrix, Value};

use super::{Component, ComponentError, ComponentResult};

const PIN_MATRIX: &str = "M";
const PIN_ROWS: &str = "R";
const PIN_COLUMNS: &str = "C";
const PIN_VALUES: &str = "V";
const PIN_SUCCESS: &str = "S";

const DEFAULT_TOLERANCE: f64 = 1e-10;

/// Beschikbare matrixcomponenten.
#[derive(Debug, Clone, Copy)]
pub enum ComponentKind {
    ConstructMatrix,
    DeconstructMatrix,
    TransposeMatrix,
    SwapRows,
    SwapColumns,
    InvertMatrix,
}

/// Metadata voor registratie in de componentregistry.
#[derive(Debug, Clone, Copy)]
pub struct Registration {
    pub guids: &'static [&'static str],
    pub names: &'static [&'static str],
    pub kind: ComponentKind,
}

/// Lijst met registraties voor alle matrixcomponenten.
pub const REGISTRATIONS: &[Registration] = &[
    Registration {
        guids: &["{54ac80cf-74f3-43f7-834c-0e3fe94632c6}"],
        names: &["Construct Matrix", "Matrix"],
        kind: ComponentKind::ConstructMatrix,
    },
    Registration {
        guids: &["{3aa2a080-e322-4be3-8c6e-baf6c8000cf1}"],
        names: &["Deconstruct Matrix", "DeMatrix"],
        kind: ComponentKind::DeconstructMatrix,
    },
    Registration {
        guids: &["{0e90b1f3-b870-4e09-8711-4bf819675d90}"],
        names: &["Transpose Matrix", "Transpose"],
        kind: ComponentKind::TransposeMatrix,
    },
    Registration {
        guids: &["{8600a3fc-30f0-4df6-b126-aaa79ece5bfe}"],
        names: &["Swap Rows", "SwapR"],
        kind: ComponentKind::SwapRows,
    },
    Registration {
        guids: &["{4cebcaf7-9a6a-435b-8f8f-95a62bacb0f2}"],
        names: &["Swap Columns", "SwapC"],
        kind: ComponentKind::SwapColumns,
    },
    Registration {
        guids: &["{f986e79a-1215-4822-a1e7-3311dbdeb851}"],
        names: &["Invert Matrix", "MInvert"],
        kind: ComponentKind::InvertMatrix,
    },
];

impl ComponentKind {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::ConstructMatrix => "Construct Matrix",
            Self::DeconstructMatrix => "Deconstruct Matrix",
            Self::TransposeMatrix => "Transpose Matrix",
            Self::SwapRows => "Swap Rows",
            Self::SwapColumns => "Swap Columns",
            Self::InvertMatrix => "Invert Matrix",
        }
    }
}

impl Component for ComponentKind {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        match self {
            Self::ConstructMatrix => evaluate_construct_matrix(inputs),
            Self::DeconstructMatrix => evaluate_deconstruct_matrix(inputs),
            Self::TransposeMatrix => evaluate_transpose_matrix(inputs),
            Self::SwapRows => evaluate_swap_rows(inputs),
            Self::SwapColumns => evaluate_swap_columns(inputs),
            Self::InvertMatrix => evaluate_invert_matrix(inputs),
        }
    }
}

fn evaluate_construct_matrix(inputs: &[Value]) -> ComponentResult {
    if inputs.len() < 2 {
        return Err(ComponentError::new(
            "Construct Matrix verwacht minimaal twee inputs (R en C)",
        ));
    }

    let rows = coerce_dimension(&inputs[0], "Construct Matrix")?;
    let columns = coerce_dimension(&inputs[1], "Construct Matrix")?;
    if rows == 0 || columns == 0 {
        return Err(ComponentError::new(
            "Construct Matrix vereist positieve afmetingen",
        ));
    }

    let values = collect_numbers(inputs.get(2));
    let matrix = create_matrix(rows, columns, &values, values.is_empty());

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_MATRIX.to_owned(), Value::Matrix(matrix));

    Ok(outputs)
}

fn evaluate_deconstruct_matrix(inputs: &[Value]) -> ComponentResult {
    let Some(matrix_value) = inputs.get(0) else {
        return Err(ComponentError::new(
            "Deconstruct Matrix verwacht een matrix input",
        ));
    };

    let matrix = coerce_matrix(matrix_value, "Deconstruct Matrix")?;

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_ROWS.to_owned(), Value::Number(matrix.rows as f64));
    outputs.insert(PIN_COLUMNS.to_owned(), Value::Number(matrix.columns as f64));
    outputs.insert(
        PIN_VALUES.to_owned(),
        Value::List(matrix.values.iter().copied().map(Value::Number).collect()),
    );

    Ok(outputs)
}

fn evaluate_transpose_matrix(inputs: &[Value]) -> ComponentResult {
    let Some(matrix_value) = inputs.get(0) else {
        return Err(ComponentError::new(
            "Transpose Matrix verwacht een matrix input",
        ));
    };

    let matrix = coerce_matrix(matrix_value, "Transpose Matrix")?;
    let transposed = transpose_matrix(&matrix);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_MATRIX.to_owned(), Value::Matrix(transposed));

    Ok(outputs)
}

fn evaluate_swap_rows(inputs: &[Value]) -> ComponentResult {
    let Some(matrix_value) = inputs.get(0) else {
        return Err(ComponentError::new("Swap Rows verwacht een matrix input"));
    };

    let matrix = coerce_matrix(matrix_value, "Swap Rows")?;
    let row_a = normalize_index(inputs.get(1), matrix.rows);
    let row_b = normalize_index(inputs.get(2), matrix.rows);
    let swapped = swap_rows(&matrix, row_a, row_b);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_MATRIX.to_owned(), Value::Matrix(swapped));

    Ok(outputs)
}

fn evaluate_swap_columns(inputs: &[Value]) -> ComponentResult {
    let Some(matrix_value) = inputs.get(0) else {
        return Err(ComponentError::new(
            "Swap Columns verwacht een matrix input",
        ));
    };

    let matrix = coerce_matrix(matrix_value, "Swap Columns")?;
    let column_a = normalize_index(inputs.get(1), matrix.columns);
    let column_b = normalize_index(inputs.get(2), matrix.columns);
    let swapped = swap_columns(&matrix, column_a, column_b);

    let mut outputs = BTreeMap::new();
    outputs.insert(PIN_MATRIX.to_owned(), Value::Matrix(swapped));

    Ok(outputs)
}

fn evaluate_invert_matrix(inputs: &[Value]) -> ComponentResult {
    let Some(matrix_value) = inputs.get(0) else {
        return Err(ComponentError::new(
            "Invert Matrix verwacht een matrix input",
        ));
    };

    let matrix = coerce_matrix(matrix_value, "Invert Matrix")?;
    let tolerance = extract_tolerance(inputs.get(1));
    let (inverted, success) = invert_matrix(&matrix, tolerance);

    let mut outputs = BTreeMap::new();
    if let Some(result) = inverted {
        outputs.insert(PIN_MATRIX.to_owned(), Value::Matrix(result));
    }
    outputs.insert(PIN_SUCCESS.to_owned(), Value::Boolean(success));

    Ok(outputs)
}

fn coerce_dimension(value: &Value, context: &str) -> Result<usize, ComponentError> {
    let number = coerce_number(value, context)?;
    if !number.is_finite() {
        return Err(ComponentError::new(format!(
            "{} verwacht een geheel getal, kreeg {}",
            context, number
        )));
    }
    let truncated = number.trunc();
    if truncated <= 0.0 {
        return Err(ComponentError::new(format!(
            "{} verwacht een positief aantal, kreeg {}",
            context, number
        )));
    }
    if truncated > (usize::MAX as f64) {
        return Err(ComponentError::new(format!(
            "{} dimensie is te groot: {}",
            context, number
        )));
    }
    Ok(truncated as usize)
}

fn coerce_number(value: &Value, context: &str) -> Result<f64, ComponentError> {
    if let Some(number) = try_coerce_number(value) {
        return Ok(number);
    }
    Err(ComponentError::new(format!(
        "{} verwacht een getal, kreeg {}",
        context,
        value.kind()
    )))
}

fn try_coerce_number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => Some(*number),
        Value::Boolean(boolean) => Some(if *boolean { 1.0 } else { 0.0 }),
        Value::List(values) if values.len() == 1 => values.get(0).and_then(try_coerce_number),
        _ => None,
    }
}

fn collect_numbers(value: Option<&Value>) -> Vec<f64> {
    let mut result = Vec::new();
    if let Some(value) = value {
        collect_numbers_inner(value, &mut result);
    }
    result
}

fn collect_numbers_inner(value: &Value, result: &mut Vec<f64>) {
    match value {
        Value::Number(number) => result.push(*number),
        Value::Boolean(boolean) => result.push(if *boolean { 1.0 } else { 0.0 }),
        Value::Matrix(matrix) => result.extend(matrix.values.iter().copied()),
        Value::List(values) => {
            for entry in values {
                collect_numbers_inner(entry, result);
            }
        }
        _ => {}
    }
}

fn coerce_matrix(value: &Value, context: &str) -> Result<Matrix, ComponentError> {
    match value {
        Value::Matrix(matrix) => Ok(matrix.clone()),
        Value::List(values) if values.is_empty() => Err(ComponentError::new(format!(
            "{} verwacht een matrix met waarden",
            context
        ))),
        Value::List(values) => interpret_list_as_matrix(values).ok_or_else(|| {
            ComponentError::new(format!("{} kon lijst niet naar matrix omzetten", context))
        }),
        Value::Number(_) | Value::Boolean(_) => {
            let numbers = collect_numbers(Some(value));
            Ok(Matrix {
                rows: 1,
                columns: numbers.len(),
                values: numbers,
            })
        }
        other => Err(ComponentError::new(format!(
            "{} verwacht een matrix, kreeg {}",
            context,
            other.kind()
        ))),
    }
}

fn interpret_list_as_matrix(values: &[Value]) -> Option<Matrix> {
    if values.is_empty() {
        return None;
    }
    if values.iter().all(|entry| matches!(entry, Value::List(_))) {
        let rows = values.len();
        let mut columns = 0_usize;
        let mut row_data: Vec<Vec<f64>> = Vec::with_capacity(rows);
        for entry in values {
            let Value::List(row_values) = entry else {
                continue;
            };
            let numbers = flatten_numbers(row_values);
            columns = columns.max(numbers.len());
            row_data.push(numbers);
        }
        if columns == 0 {
            return None;
        }
        let mut flat = vec![0.0; rows * columns];
        for (r, row_numbers) in row_data.into_iter().enumerate() {
            for c in 0..columns {
                let value = row_numbers.get(c).copied().unwrap_or(0.0);
                flat[r * columns + c] = value;
            }
        }
        return Some(Matrix {
            rows,
            columns,
            values: flat,
        });
    }

    let numbers = flatten_numbers(values);
    if numbers.is_empty() {
        None
    } else {
        Some(Matrix {
            rows: 1,
            columns: numbers.len(),
            values: numbers,
        })
    }
}

fn flatten_numbers(values: &[Value]) -> Vec<f64> {
    let mut result = Vec::new();
    for value in values {
        collect_numbers_inner(value, &mut result);
    }
    result
}

fn create_matrix(rows: usize, columns: usize, values: &[f64], identity_fallback: bool) -> Matrix {
    let total = rows * columns;
    let mut flat = vec![0.0; total];
    let count = values.len().min(total);
    flat[..count].copy_from_slice(&values[..count]);
    if identity_fallback && count == 0 {
        let diagonal = rows.min(columns);
        for i in 0..diagonal {
            flat[i * columns + i] = 1.0;
        }
    }
    Matrix {
        rows,
        columns,
        values: flat,
    }
}

fn transpose_matrix(matrix: &Matrix) -> Matrix {
    let mut result = vec![0.0; matrix.rows * matrix.columns];
    for r in 0..matrix.rows {
        for c in 0..matrix.columns {
            result[c * matrix.rows + r] = matrix.values[r * matrix.columns + c];
        }
    }
    Matrix {
        rows: matrix.columns,
        columns: matrix.rows,
        values: result,
    }
}

fn normalize_index(value: Option<&Value>, size: usize) -> Option<usize> {
    let Some(value) = value else { return None };
    let number = try_coerce_number(value)?;
    if !number.is_finite() {
        return None;
    }
    let rounded = number.round();
    if (number - rounded).abs() > 1e-6 {
        return None;
    }
    let rounded_i = rounded as isize;
    if rounded_i >= 0 && (rounded_i as usize) < size {
        return Some(rounded_i as usize);
    }
    let adjusted = rounded_i - 1;
    if adjusted >= 0 && (adjusted as usize) < size {
        return Some(adjusted as usize);
    }
    None
}

fn swap_rows(matrix: &Matrix, row_a: Option<usize>, row_b: Option<usize>) -> Matrix {
    let Some(row_a) = row_a else {
        return matrix.clone();
    };
    let Some(row_b) = row_b else {
        return matrix.clone();
    };
    if row_a == row_b || row_a >= matrix.rows || row_b >= matrix.rows {
        return matrix.clone();
    }
    let mut values = matrix.values.clone();
    for c in 0..matrix.columns {
        let index_a = row_a * matrix.columns + c;
        let index_b = row_b * matrix.columns + c;
        values.swap(index_a, index_b);
    }
    Matrix {
        rows: matrix.rows,
        columns: matrix.columns,
        values,
    }
}

fn swap_columns(matrix: &Matrix, column_a: Option<usize>, column_b: Option<usize>) -> Matrix {
    let Some(column_a) = column_a else {
        return matrix.clone();
    };
    let Some(column_b) = column_b else {
        return matrix.clone();
    };
    if column_a == column_b || column_a >= matrix.columns || column_b >= matrix.columns {
        return matrix.clone();
    }
    let mut values = matrix.values.clone();
    for r in 0..matrix.rows {
        let index_a = r * matrix.columns + column_a;
        let index_b = r * matrix.columns + column_b;
        values.swap(index_a, index_b);
    }
    Matrix {
        rows: matrix.rows,
        columns: matrix.columns,
        values,
    }
}

fn extract_tolerance(value: Option<&Value>) -> f64 {
    value
        .and_then(try_coerce_number)
        .map(|number| number.abs())
        .filter(|tolerance| *tolerance > 0.0)
        .unwrap_or(DEFAULT_TOLERANCE)
}

fn invert_matrix(matrix: &Matrix, tolerance: f64) -> (Option<Matrix>, bool) {
    if matrix.rows != matrix.columns {
        return (None, false);
    }
    let size = matrix.rows;
    let mut augmented = vec![vec![0.0; size * 2]; size];
    for r in 0..size {
        for c in 0..size {
            augmented[r][c] = matrix.values[r * size + c];
        }
        for c in 0..size {
            augmented[r][size + c] = if r == c { 1.0 } else { 0.0 };
        }
    }

    for col in 0..size {
        let mut pivot_row = col;
        let mut pivot_value = augmented[pivot_row][col].abs();
        for r in (col + 1)..size {
            let value = augmented[r][col].abs();
            if value > pivot_value {
                pivot_value = value;
                pivot_row = r;
            }
        }
        if pivot_value <= tolerance {
            return (None, false);
        }
        if pivot_row != col {
            augmented.swap(col, pivot_row);
        }
        let pivot = augmented[col][col];
        for c in 0..size * 2 {
            augmented[col][c] /= pivot;
        }
        for r in 0..size {
            if r == col {
                continue;
            }
            let factor = augmented[r][col];
            if factor.abs() <= tolerance {
                augmented[r][col] = 0.0;
                continue;
            }
            for c in 0..size * 2 {
                augmented[r][c] -= factor * augmented[col][c];
            }
            augmented[r][col] = 0.0;
        }
    }

    let mut result = vec![0.0; size * size];
    for r in 0..size {
        for c in 0..size {
            let value = augmented[r][size + c];
            result[r * size + c] = if value.abs() <= tolerance { 0.0 } else { value };
        }
    }

    (
        Some(Matrix {
            rows: size,
            columns: size,
            values: result,
        }),
        true,
    )
}