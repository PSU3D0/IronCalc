use ordered_float::OrderedFloat;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use xlsx::base::expressions::pattern_matching::{
    CriteriaPair, CriteriaValue, LookupValue, PatternError, PatternKey, SubstitutionRegistry,
    TableRange,
};
use xlsx::base::expressions::types::CellReferenceIndex;

/// Python wrapper for PatternError
#[pyclass]
#[derive(Clone)]
pub struct PyPatternError {
    #[pyo3(get)]
    pub message: String,
}

impl From<PatternError> for PyPatternError {
    fn from(error: PatternError) -> Self {
        PyPatternError {
            message: error.to_string(),
        }
    }
}

/// Python wrapper for CriteriaValue
#[pyclass]
#[derive(Clone)]
pub struct PyCriteriaValue {
    inner: CriteriaValue,
}

#[pymethods]
impl PyCriteriaValue {
    /// Create a string criteria value
    #[staticmethod]
    pub fn string(value: String) -> Self {
        PyCriteriaValue {
            inner: CriteriaValue::String(value),
        }
    }

    /// Create a number criteria value
    #[staticmethod]
    pub fn number(value: f64) -> Self {
        PyCriteriaValue {
            inner: CriteriaValue::Number(OrderedFloat(value)),
        }
    }

    /// Create a cell reference criteria value
    #[staticmethod]
    pub fn cell_ref(sheet: u32, row: i32, column: i32) -> Self {
        PyCriteriaValue {
            inner: CriteriaValue::CellReference(CellReferenceIndex { sheet, row, column }),
        }
    }

    /// Get the string representation
    pub fn __str__(&self) -> String {
        match &self.inner {
            CriteriaValue::String(s) => format!("String({})", s),
            CriteriaValue::Number(n) => format!("Number({})", n.0),
            CriteriaValue::CellReference(cell) => format!(
                "CellRef(sheet={}, row={}, column={})",
                cell.sheet, cell.row, cell.column
            ),
        }
    }

    /// Get the debug representation
    pub fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl From<PyCriteriaValue> for CriteriaValue {
    fn from(py_criteria: PyCriteriaValue) -> Self {
        py_criteria.inner
    }
}

impl From<CriteriaValue> for PyCriteriaValue {
    fn from(criteria: CriteriaValue) -> Self {
        PyCriteriaValue { inner: criteria }
    }
}

/// Python wrapper for CriteriaPair
#[pyclass]
#[derive(Clone)]
pub struct PyCriteriaPair {
    #[pyo3(get)]
    pub column: i32,
    #[pyo3(get)]
    pub criteria: PyCriteriaValue,
}

#[pymethods]
impl PyCriteriaPair {
    #[new]
    pub fn new(column: i32, criteria: PyCriteriaValue) -> Self {
        PyCriteriaPair { column, criteria }
    }

    pub fn __str__(&self) -> String {
        format!(
            "CriteriaPair(column={}, criteria={})",
            self.column,
            self.criteria.__str__()
        )
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl From<PyCriteriaPair> for CriteriaPair {
    fn from(py_pair: PyCriteriaPair) -> Self {
        CriteriaPair {
            column: py_pair.column,
            criteria: py_pair.criteria.into(),
        }
    }
}

impl From<CriteriaPair> for PyCriteriaPair {
    fn from(pair: CriteriaPair) -> Self {
        PyCriteriaPair {
            column: pair.column,
            criteria: pair.criteria.into(),
        }
    }
}

/// Python wrapper for LookupValue
#[pyclass]
#[derive(Clone)]
pub struct PyLookupValue {
    inner: LookupValue,
}

#[pymethods]
impl PyLookupValue {
    /// Create a string lookup value
    #[staticmethod]
    pub fn string(value: String) -> Self {
        PyLookupValue {
            inner: LookupValue::String(value),
        }
    }

    /// Create a number lookup value
    #[staticmethod]
    pub fn number(value: f64) -> Self {
        PyLookupValue {
            inner: LookupValue::Number(OrderedFloat(value)),
        }
    }

    /// Create a cell reference lookup value
    #[staticmethod]
    pub fn cell_ref(sheet: u32, row: i32, column: i32) -> Self {
        PyLookupValue {
            inner: LookupValue::CellReference(CellReferenceIndex { sheet, row, column }),
        }
    }

    pub fn __str__(&self) -> String {
        match &self.inner {
            LookupValue::String(s) => format!("String({})", s),
            LookupValue::Number(n) => format!("Number({})", n.0),
            LookupValue::CellReference(cell) => format!(
                "CellRef(sheet={}, row={}, column={})",
                cell.sheet, cell.row, cell.column
            ),
        }
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl From<PyLookupValue> for LookupValue {
    fn from(py_lookup: PyLookupValue) -> Self {
        py_lookup.inner
    }
}

impl From<LookupValue> for PyLookupValue {
    fn from(lookup: LookupValue) -> Self {
        PyLookupValue { inner: lookup }
    }
}

/// Python wrapper for TableRange
#[pyclass]
#[derive(Clone)]
pub struct PyTableRange {
    #[pyo3(get)]
    pub sheet: Option<String>,
    #[pyo3(get)]
    pub start_row: i32,
    #[pyo3(get)]
    pub start_column: i32,
    #[pyo3(get)]
    pub end_row: i32,
    #[pyo3(get)]
    pub end_column: i32,
}

#[pymethods]
impl PyTableRange {
    #[new]
    #[pyo3(signature = (sheet, start_row, start_column, end_row, end_column))]
    pub fn new(
        sheet: Option<String>,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
    ) -> Self {
        PyTableRange {
            sheet,
            start_row,
            start_column,
            end_row,
            end_column,
        }
    }

    pub fn __str__(&self) -> String {
        format!(
            "TableRange(sheet={:?}, {}:{} to {}:{})",
            self.sheet, self.start_row, self.start_column, self.end_row, self.end_column
        )
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl From<PyTableRange> for TableRange {
    fn from(py_range: PyTableRange) -> Self {
        TableRange {
            sheet: py_range.sheet,
            start_row: py_range.start_row,
            start_column: py_range.start_column,
            end_row: py_range.end_row,
            end_column: py_range.end_column,
        }
    }
}

impl From<TableRange> for PyTableRange {
    fn from(range: TableRange) -> Self {
        PyTableRange {
            sheet: range.sheet,
            start_row: range.start_row,
            start_column: range.start_column,
            end_row: range.end_row,
            end_column: range.end_column,
        }
    }
}

/// Python wrapper for PatternKey with builder patterns
#[pyclass]
#[derive(Clone)]
pub struct PyPatternKey {
    inner: PatternKey,
}

#[pymethods]
impl PyPatternKey {
    /// Create a SUMIFS pattern builder
    #[staticmethod]
    pub fn sumifs() -> PySumifsBuilder {
        PySumifsBuilder::new()
    }

    /// Create a COUNTIFS pattern builder
    #[staticmethod]
    pub fn countifs() -> PyCountifsBuilder {
        PyCountifsBuilder::new()
    }

    /// Create a AVERAGEIFS pattern builder
    #[staticmethod]
    pub fn averageifs() -> PyAverageifsBuilder {
        PyAverageifsBuilder::new()
    }

    /// Create a VLOOKUP pattern builder
    #[staticmethod]
    pub fn vlookup() -> PyVlookupBuilder {
        PyVlookupBuilder::new()
    }

    /// Validate the pattern
    pub fn validate(&self) -> PyResult<()> {
        self.inner.validate().map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Pattern validation failed: {}", e))
        })
    }

    /// Get the pattern type as a string
    pub fn pattern_type(&self) -> String {
        match &self.inner {
            PatternKey::Sumifs { .. } => "SUMIFS".to_string(),
            PatternKey::Countifs { .. } => "COUNTIFS".to_string(),
            PatternKey::Averageifs { .. } => "AVERAGEIFS".to_string(),
            PatternKey::Vlookup { .. } => "VLOOKUP".to_string(),
        }
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }

    pub fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::hash::DefaultHasher::new();
        self.inner.hash(&mut hasher);
        hasher.finish()
    }

    pub fn __eq__(&self, other: &PyPatternKey) -> bool {
        self.inner == other.inner
    }
}

impl From<PyPatternKey> for PatternKey {
    fn from(py_pattern: PyPatternKey) -> Self {
        py_pattern.inner
    }
}

impl From<PatternKey> for PyPatternKey {
    fn from(pattern: PatternKey) -> Self {
        PyPatternKey { inner: pattern }
    }
}

/// Builder for SUMIFS patterns
#[pyclass]
pub struct PySumifsBuilder {
    sheet: Option<String>,
    sum_column: Option<i32>,
    criteria_pairs: Vec<CriteriaPair>,
}

#[pymethods]
impl PySumifsBuilder {
    #[new]
    pub fn new() -> Self {
        PySumifsBuilder {
            sheet: None,
            sum_column: None,
            criteria_pairs: Vec::new(),
        }
    }

    /// Set the sheet name
    pub fn sheet(mut slf: PyRefMut<Self>, sheet: String) -> PyRefMut<Self> {
        slf.sheet = Some(sheet);
        slf
    }

    /// Set the sum column
    pub fn sum_column(mut slf: PyRefMut<Self>, column: i32) -> PyRefMut<Self> {
        slf.sum_column = Some(column);
        slf
    }

    /// Add a string criteria
    pub fn criteria_string(mut slf: PyRefMut<Self>, column: i32, value: String) -> PyRefMut<Self> {
        slf.criteria_pairs.push(CriteriaPair {
            column,
            criteria: CriteriaValue::String(value),
        });
        slf
    }

    /// Add a number criteria
    pub fn criteria_number(mut slf: PyRefMut<Self>, column: i32, value: f64) -> PyRefMut<Self> {
        slf.criteria_pairs.push(CriteriaPair {
            column,
            criteria: CriteriaValue::Number(OrderedFloat(value)),
        });
        slf
    }

    /// Add a cell reference criteria
    pub fn criteria_cell(
        mut slf: PyRefMut<Self>,
        column: i32,
        sheet: u32,
        row: i32,
        cell_column: i32,
    ) -> PyRefMut<Self> {
        slf.criteria_pairs.push(CriteriaPair {
            column,
            criteria: CriteriaValue::CellReference(CellReferenceIndex {
                sheet,
                row,
                column: cell_column,
            }),
        });
        slf
    }

    /// Build the pattern
    pub fn build(&self) -> PyResult<PyPatternKey> {
        if let Some(sum_column) = self.sum_column {
            if !self.criteria_pairs.is_empty() {
                let pattern = PatternKey::Sumifs {
                    sheet: self.sheet.clone(),
                    sum_column,
                    criteria_pairs: self.criteria_pairs.clone(),
                };
                Ok(PyPatternKey { inner: pattern })
            } else {
                Err(pyo3::exceptions::PyValueError::new_err(
                    "At least one criteria pair is required",
                ))
            }
        } else {
            Err(pyo3::exceptions::PyValueError::new_err(
                "Sum column is required",
            ))
        }
    }
}

/// Builder for COUNTIFS patterns
#[pyclass]
pub struct PyCountifsBuilder {
    sheet: Option<String>,
    criteria_pairs: Vec<CriteriaPair>,
}

#[pymethods]
impl PyCountifsBuilder {
    #[new]
    pub fn new() -> Self {
        PyCountifsBuilder {
            sheet: None,
            criteria_pairs: Vec::new(),
        }
    }

    /// Set the sheet name
    pub fn sheet(mut slf: PyRefMut<Self>, sheet: String) -> PyRefMut<Self> {
        slf.sheet = Some(sheet);
        slf
    }

    /// Add a string criteria
    pub fn criteria_string(mut slf: PyRefMut<Self>, column: i32, value: String) -> PyRefMut<Self> {
        slf.criteria_pairs.push(CriteriaPair {
            column,
            criteria: CriteriaValue::String(value),
        });
        slf
    }

    /// Add a number criteria
    pub fn criteria_number(mut slf: PyRefMut<Self>, column: i32, value: f64) -> PyRefMut<Self> {
        slf.criteria_pairs.push(CriteriaPair {
            column,
            criteria: CriteriaValue::Number(OrderedFloat(value)),
        });
        slf
    }

    /// Add a cell reference criteria
    pub fn criteria_cell(
        mut slf: PyRefMut<Self>,
        column: i32,
        sheet: u32,
        row: i32,
        cell_column: i32,
    ) -> PyRefMut<Self> {
        slf.criteria_pairs.push(CriteriaPair {
            column,
            criteria: CriteriaValue::CellReference(CellReferenceIndex {
                sheet,
                row,
                column: cell_column,
            }),
        });
        slf
    }

    /// Build the pattern
    pub fn build(&self) -> PyResult<PyPatternKey> {
        if !self.criteria_pairs.is_empty() {
            let pattern = PatternKey::Countifs {
                sheet: self.sheet.clone(),
                criteria_pairs: self.criteria_pairs.clone(),
            };
            Ok(PyPatternKey { inner: pattern })
        } else {
            Err(pyo3::exceptions::PyValueError::new_err(
                "At least one criteria pair is required",
            ))
        }
    }
}

/// Builder for AVERAGEIFS patterns
#[pyclass]
pub struct PyAverageifsBuilder {
    sheet: Option<String>,
    average_column: Option<i32>,
    criteria_pairs: Vec<CriteriaPair>,
}

#[pymethods]
impl PyAverageifsBuilder {
    #[new]
    pub fn new() -> Self {
        PyAverageifsBuilder {
            sheet: None,
            average_column: None,
            criteria_pairs: Vec::new(),
        }
    }

    /// Set the sheet name
    pub fn sheet(mut slf: PyRefMut<Self>, sheet: String) -> PyRefMut<Self> {
        slf.sheet = Some(sheet);
        slf
    }

    /// Set the average column
    pub fn average_column(mut slf: PyRefMut<Self>, column: i32) -> PyRefMut<Self> {
        slf.average_column = Some(column);
        slf
    }

    /// Add a string criteria
    pub fn criteria_string(mut slf: PyRefMut<Self>, column: i32, value: String) -> PyRefMut<Self> {
        slf.criteria_pairs.push(CriteriaPair {
            column,
            criteria: CriteriaValue::String(value),
        });
        slf
    }

    /// Add a number criteria
    pub fn criteria_number(mut slf: PyRefMut<Self>, column: i32, value: f64) -> PyRefMut<Self> {
        slf.criteria_pairs.push(CriteriaPair {
            column,
            criteria: CriteriaValue::Number(OrderedFloat(value)),
        });
        slf
    }

    /// Add a cell reference criteria
    pub fn criteria_cell(
        mut slf: PyRefMut<Self>,
        column: i32,
        sheet: u32,
        row: i32,
        cell_column: i32,
    ) -> PyRefMut<Self> {
        slf.criteria_pairs.push(CriteriaPair {
            column,
            criteria: CriteriaValue::CellReference(CellReferenceIndex {
                sheet,
                row,
                column: cell_column,
            }),
        });
        slf
    }

    /// Build the pattern
    pub fn build(&self) -> PyResult<PyPatternKey> {
        if let Some(average_column) = self.average_column {
            if !self.criteria_pairs.is_empty() {
                let pattern = PatternKey::Averageifs {
                    sheet: self.sheet.clone(),
                    average_column,
                    criteria_pairs: self.criteria_pairs.clone(),
                };
                Ok(PyPatternKey { inner: pattern })
            } else {
                Err(pyo3::exceptions::PyValueError::new_err(
                    "At least one criteria pair is required",
                ))
            }
        } else {
            Err(pyo3::exceptions::PyValueError::new_err(
                "Average column is required",
            ))
        }
    }
}

/// Builder for VLOOKUP patterns
#[pyclass]
pub struct PyVlookupBuilder {
    sheet: Option<String>,
    lookup_value: Option<LookupValue>,
    table_range: Option<TableRange>,
    column_index: Option<u32>,
    exact_match: Option<bool>,
}

#[pymethods]
impl PyVlookupBuilder {
    #[new]
    pub fn new() -> Self {
        PyVlookupBuilder {
            sheet: None,
            lookup_value: None,
            table_range: None,
            column_index: None,
            exact_match: None,
        }
    }

    /// Set the sheet name
    pub fn sheet(mut slf: PyRefMut<Self>, sheet: String) -> PyRefMut<Self> {
        slf.sheet = Some(sheet);
        slf
    }

    /// Set lookup value as string
    pub fn lookup_string(mut slf: PyRefMut<Self>, value: String) -> PyRefMut<Self> {
        slf.lookup_value = Some(LookupValue::String(value));
        slf
    }

    /// Set lookup value as number
    pub fn lookup_number(mut slf: PyRefMut<Self>, value: f64) -> PyRefMut<Self> {
        slf.lookup_value = Some(LookupValue::Number(OrderedFloat(value)));
        slf
    }

    /// Set lookup value as cell reference
    pub fn lookup_cell(
        mut slf: PyRefMut<Self>,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> PyRefMut<Self> {
        slf.lookup_value = Some(LookupValue::CellReference(CellReferenceIndex {
            sheet,
            row,
            column,
        }));
        slf
    }

    /// Set table range
    pub fn table_range(
        mut slf: PyRefMut<Self>,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
    ) -> PyRefMut<Self> {
        slf.table_range = Some(TableRange {
            sheet: None,
            start_row,
            start_column,
            end_row,
            end_column,
        });
        slf
    }

    /// Set table range with sheet
    pub fn table_range_with_sheet(
        mut slf: PyRefMut<Self>,
        sheet: String,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
    ) -> PyRefMut<Self> {
        slf.table_range = Some(TableRange {
            sheet: Some(sheet),
            start_row,
            start_column,
            end_row,
            end_column,
        });
        slf
    }

    /// Set column index
    pub fn column_index(mut slf: PyRefMut<Self>, index: u32) -> PyRefMut<Self> {
        slf.column_index = Some(index);
        slf
    }

    /// Set exact match requirement
    pub fn exact_match(mut slf: PyRefMut<Self>, exact: bool) -> PyRefMut<Self> {
        slf.exact_match = Some(exact);
        slf
    }

    /// Build the pattern
    pub fn build(&self) -> PyResult<PyPatternKey> {
        if let (Some(lookup_value), Some(table_range), Some(column_index), Some(exact_match)) = (
            &self.lookup_value,
            &self.table_range,
            &self.column_index,
            &self.exact_match,
        ) {
            let pattern = PatternKey::Vlookup {
                sheet: self.sheet.clone(),
                lookup_value: lookup_value.clone(),
                table_range: table_range.clone(),
                column_index: *column_index,
                exact_match: *exact_match,
            };
            Ok(PyPatternKey { inner: pattern })
        } else {
            Err(pyo3::exceptions::PyValueError::new_err("All VLOOKUP fields are required: lookup_value, table_range, column_index, exact_match"))
        }
    }
}

/// Python wrapper for SubstitutionRegistry
#[pyclass]
pub struct PySubstitutionRegistry {
    inner: SubstitutionRegistry,
}

#[pymethods]
impl PySubstitutionRegistry {
    #[new]
    pub fn new() -> Self {
        PySubstitutionRegistry {
            inner: SubstitutionRegistry::new(),
        }
    }

    /// Add a substitution pattern
    pub fn add_substitution(&mut self, pattern: &PyPatternKey, value: f64) {
        self.inner.add_substitution(pattern.inner.clone(), value);
    }

    /// Get a substitution value
    pub fn get_substitution(&mut self, pattern: &PyPatternKey) -> Option<f64> {
        self.inner.get_substitution(&pattern.inner).copied()
    }

    /// Check if registry has a pattern
    pub fn has_substitution(&mut self, pattern: &PyPatternKey) -> bool {
        self.inner.get_substitution(&pattern.inner).is_some()
    }

    /// Set registry active state
    pub fn set_active(&mut self, active: bool) {
        self.inner.set_active(active);
    }

    /// Check if registry is active
    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    /// Get number of patterns
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Clear all substitutions
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Get statistics as a dictionary
    pub fn stats(&self) -> Py<PyDict> {
        let stats = self.inner.stats();
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("total_lookups", stats.total_lookups())
                .unwrap();
            dict.set_item("successful_lookups", stats.successful_lookups)
                .unwrap();
            dict.set_item("failed_lookups", stats.failed_lookups)
                .unwrap();
            dict.set_item("inactive_lookups", stats.inactive_lookups)
                .unwrap();
            dict.set_item("success_rate", stats.success_rate()).unwrap();
            dict.into()
        })
    }

    /// Add multiple substitutions at once
    pub fn add_bulk(&mut self, patterns: Vec<(PyPatternKey, f64)>) -> usize {
        let mut count = 0;
        for (py_pattern, value) in patterns {
            self.inner.add_substitution(py_pattern.inner, value);
            count += 1;
        }
        count
    }
}
