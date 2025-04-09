use chrono::{NaiveDate, NaiveDateTime};
#[cfg(feature = "polars")]
use polars::PyDataFrame;
use pyo3::exceptions::PyException;
use pyo3::types::PyList;
use pyo3::IntoPyObjectExt;
use pyo3::{create_exception, prelude::*, wrap_pyfunction};

use types::{PySheetProperty, PyStyle, RangeInjectionManifestItem};
use xlsx::base::calc_result::CalcResult;
use xlsx::base::expressions::parser::ArrayNode;
use xlsx::base::expressions::types::CellReferenceIndex;
use xlsx::base::types::Style;
use xlsx::base::Model;

use xlsx::export::{save_to_icalc, save_to_xlsx};
use xlsx::import;

#[cfg(feature = "polars")]
mod polars;
mod types;
use crate::types::PyCellType;

create_exception!(_ironcalc, WorkbookError, PyException);

/// This is a model implementing the 'raw' API
#[pyclass]
pub struct PyModel {
    model: Model,
}

#[pymethods]
impl PyModel {
    /// Saves the model to an xlsx file
    pub fn save_to_xlsx(&self, file: &str) -> PyResult<()> {
        save_to_xlsx(&self.model, file).map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    /// Saves the model to file in the internal binary ic format
    pub fn save_to_icalc(&self, file: &str) -> PyResult<()> {
        save_to_icalc(&self.model, file).map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    /// Evaluates the workbook
    pub fn evaluate(&mut self) {
        self.model.evaluate()
    }

    // Set values

    /// Set an input
    #[pyo3(signature = (sheet, row, column, value, reevaluate=true))]
    pub fn set_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: Py<PyAny>,
        reevaluate: bool,
    ) -> PyResult<()> {
        let result = Python::with_gil(|py| {
            // Try to extract various types in order of specificity
            if let Ok(string_val) = value.extract::<String>(py) {
                if string_val.starts_with('=') || string_val.starts_with('\'') {
                    // Handle formulas or text that starts with '
                    self.model
                        .set_user_input(sheet, row, column, string_val)
                        .map_err(|e| WorkbookError::new_err(e.to_string()))
                } else {
                    // Normal text
                    self.model
                        .update_cell_with_text(sheet, row, column, &string_val)
                        .map_err(|e| WorkbookError::new_err(e.to_string()))
                }
            } else if let Ok(bool_val) = value.extract::<bool>(py) {
                // Handle boolean values
                self.model
                    .update_cell_with_bool(sheet, row, column, bool_val)
                    .map_err(|e| WorkbookError::new_err(e.to_string()))
            } else if let Ok(float_val) = value.extract::<f64>(py) {
                // Handle floating point numbers
                self.model
                    .update_cell_with_number(sheet, row, column, float_val)
                    .map_err(|e| WorkbookError::new_err(e.to_string()))
            } else if let Ok(int_val) = value.extract::<i64>(py) {
                // Handle integers
                self.model
                    .update_cell_with_number(sheet, row, column, int_val as f64)
                    .map_err(|e| WorkbookError::new_err(e.to_string()))
            } else if let Ok(date_val) = value.extract::<NaiveDate>(py) {
                // Handle dates
                self.model
                    .update_cell_with_number(
                        sheet,
                        row,
                        column,
                        naivedate_to_excel_timestamp(date_val),
                    )
                    .map_err(|e| WorkbookError::new_err(e.to_string()))
            } else if let Ok(date_time_val) = value.extract::<NaiveDateTime>(py) {
                // Handle date times
                self.model
                    .update_cell_with_number(
                        sheet,
                        row,
                        column,
                        naivedatetime_to_excel_timestamp(date_time_val),
                    )
                    .map_err(|e| WorkbookError::new_err(e.to_string()))
            } else {
                // For any other type, convert to string and use set_user_input
                let value_str = value.call_method0(py, "__str__")?.extract::<String>(py)?;
                self.model
                    .set_user_input(sheet, row, column, value_str)
                    .map_err(|e| WorkbookError::new_err(e.to_string()))
            }
        });

        if reevaluate {
            self.evaluate();
        }

        result
    }

    pub fn clear_cell_contents(&mut self, sheet: u32, row: i32, column: i32) -> PyResult<()> {
        self.model
            .cell_clear_contents(sheet, row, column)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn clear_range(
        &mut self,
        sheet: u32,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
    ) -> PyResult<()> {
        for row in start_row..=end_row {
            for column in start_column..=end_column {
                self.clear_cell_contents(sheet, row, column)?;
            }
        }
        Ok(())
    }

    /// Set multiple inputs at once using a single GIL acquisition
    ///
    /// Takes an iterable of (sheet, row, column, value) tuples and applies them as a batch.
    /// This is much more efficient than calling set_user_input repeatedly for large datasets.
    ///
    /// Example:
    ///     model.set_user_inputs_batch([
    ///         (0, 0, 0, "Header"),
    ///         (0, 1, 0, 100),
    ///         (0, 2, 0, 200),
    ///     ])
    #[pyo3(signature = (inputs, reevaluate=true))]
    pub fn set_user_inputs_batch(
        &mut self,
        inputs: Vec<(u32, i32, i32, Py<PyAny>)>,
        reevaluate: bool,
    ) -> PyResult<()> {
        Python::with_gil(|py| {
            // Process each item in the iterator
            for tuple in inputs.iter() {
                let sheet: u32 = tuple.0;
                let row: i32 = tuple.1;
                let column: i32 = tuple.2;
                let value = tuple.3.clone_ref(py);

                // Logic similar to set_user_input but without acquiring the GIL each time
                if let Ok(string_val) = value.extract::<String>(py) {
                    if string_val.starts_with('=') || string_val.starts_with('\'') {
                        // Handle formulas or text that starts with '
                        self.model
                            .set_user_input(sheet, row, column, string_val)
                            .map_err(|e| WorkbookError::new_err(e.to_string()))?;
                    } else {
                        // Normal text
                        self.model
                            .update_cell_with_text(sheet, row, column, &string_val)
                            .map_err(|e| WorkbookError::new_err(e.to_string()))?;
                    }
                } else if let Ok(bool_val) = value.extract::<bool>(py) {
                    // Handle boolean values
                    self.model
                        .update_cell_with_bool(sheet, row, column, bool_val)
                        .map_err(|e| WorkbookError::new_err(e.to_string()))?;
                } else if let Ok(float_val) = value.extract::<f64>(py) {
                    // Handle floating point numbers
                    self.model
                        .update_cell_with_number(sheet, row, column, float_val)
                        .map_err(|e| WorkbookError::new_err(e.to_string()))?;
                } else if let Ok(int_val) = value.extract::<i64>(py) {
                    // Handle integers
                    self.model
                        .update_cell_with_number(sheet, row, column, int_val as f64)
                        .map_err(|e| WorkbookError::new_err(e.to_string()))?;
                } else if let Ok(date_val) = value.extract::<NaiveDate>(py) {
                    // Handle dates
                    self.model
                        .update_cell_with_number(
                            sheet,
                            row,
                            column,
                            naivedate_to_excel_timestamp(date_val),
                        )
                        .map_err(|e| WorkbookError::new_err(e.to_string()))?;
                } else if let Ok(date_time_val) = value.extract::<NaiveDateTime>(py) {
                    // Handle date times
                    self.model
                        .update_cell_with_number(
                            sheet,
                            row,
                            column,
                            naivedatetime_to_excel_timestamp(date_time_val),
                        )
                        .map_err(|e| WorkbookError::new_err(e.to_string()))?;
                } else {
                    // For any other type, convert to string and use set_user_input
                    let value_str = value.call_method0(py, "__str__")?.extract::<String>(py)?;
                    self.model
                        .set_user_input(sheet, row, column, value_str)
                        .map_err(|e| WorkbookError::new_err(e.to_string()))?;
                }
            }

            // Re-evaluate the model if requested
            if reevaluate {
                self.evaluate();
            }

            Ok(())
        })
    }

    /// Get raw value
    pub fn get_cell_content(&self, sheet: u32, row: i32, column: i32) -> PyResult<String> {
        self.model
            .get_cell_content(sheet, row, column)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    /// Get cell type
    pub fn get_cell_type(&self, sheet: u32, row: i32, column: i32) -> PyResult<PyCellType> {
        self.model
            .get_cell_type(sheet, row, column)
            .map(|cell_type| cell_type.into())
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    /// Get formatted value
    pub fn get_formatted_cell_value(&self, sheet: u32, row: i32, column: i32) -> PyResult<String> {
        self.model
            .get_formatted_cell_value(sheet, row, column)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    // Set styles
    pub fn set_cell_style(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        py_style: &PyStyle,
    ) -> PyResult<()> {
        let style: Style = py_style.into();
        self.model
            .set_cell_style(sheet, row, column, &style)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    // Get styles
    pub fn get_cell_style(&self, sheet: u32, row: i32, column: i32) -> PyResult<PyStyle> {
        let style = self
            .model
            .get_style_for_cell(sheet, row, column)
            .map_err(|e| WorkbookError::new_err(e.to_string()))?;
        Ok(style.into())
    }

    pub fn evaluate_cell(
        &mut self,
        py: Python,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> PyResult<PyObject> {
        let cell_reference = CellReferenceIndex { sheet, row, column };

        let result: CalcResult = self.model.evaluate_cell(cell_reference);
        calc_result_to_py_any(result, py)
    }

    pub fn evaluate_cells(
        &mut self,
        py: Python,
        cells: Vec<(u32, i32, i32)>,
    ) -> PyResult<Vec<PyObject>> {
        let mut results = Vec::new();
        for (sheet, row, column) in cells {
            let result = self.evaluate_cell(py, sheet, row, column)?;
            results.push(result);
        }
        Ok(results)
    }

    // column widths, row heights
    // insert/delete rows/columns

    pub fn insert_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> PyResult<()> {
        self.model
            .insert_rows(sheet, row, row_count)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn insert_columns(&mut self, sheet: u32, column: i32, column_count: i32) -> PyResult<()> {
        self.model
            .insert_columns(sheet, column, column_count)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn delete_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> PyResult<()> {
        self.model
            .delete_rows(sheet, row, row_count)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn delete_columns(&mut self, sheet: u32, column: i32, column_count: i32) -> PyResult<()> {
        self.model
            .delete_columns(sheet, column, column_count)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn get_column_width(&self, sheet: u32, column: i32) -> PyResult<f64> {
        self.model
            .get_column_width(sheet, column)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn get_row_height(&self, sheet: u32, row: i32) -> PyResult<f64> {
        self.model
            .get_row_height(sheet, row)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn set_column_width(&mut self, sheet: u32, column: i32, width: f64) -> PyResult<()> {
        self.model
            .set_column_width(sheet, column, width)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn set_row_height(&mut self, sheet: u32, row: i32, height: f64) -> PyResult<()> {
        self.model
            .set_row_height(sheet, row, height)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    // frozen rows/columns

    pub fn get_frozen_columns_count(&self, sheet: u32) -> PyResult<i32> {
        self.model
            .get_frozen_columns_count(sheet)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn get_frozen_rows_count(&self, sheet: u32) -> PyResult<i32> {
        self.model
            .get_frozen_rows_count(sheet)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn set_frozen_columns_count(&mut self, sheet: u32, column_count: i32) -> PyResult<()> {
        self.model
            .set_frozen_columns(sheet, column_count)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn set_frozen_rows_count(&mut self, sheet: u32, row_count: i32) -> PyResult<()> {
        self.model
            .set_frozen_rows(sheet, row_count)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    // Manipulate sheets (add/remove/rename/change color)
    pub fn get_worksheets_properties(&self) -> PyResult<Vec<PySheetProperty>> {
        Ok(self
            .model
            .get_worksheets_properties()
            .into_iter()
            .map(|s| PySheetProperty {
                name: s.name,
                state: s.state,
                sheet_id: s.sheet_id,
                color: s.color,
            })
            .collect())
    }

    pub fn set_sheet_color(&mut self, sheet: u32, color: &str) -> PyResult<()> {
        self.model
            .set_sheet_color(sheet, color)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn add_sheet(&mut self, sheet_name: &str) -> PyResult<()> {
        self.model
            .add_sheet(sheet_name)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn new_sheet(&mut self) {
        self.model.new_sheet();
    }

    pub fn delete_sheet(&mut self, sheet: u32) -> PyResult<()> {
        self.model
            .delete_sheet(sheet)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn rename_sheet(&mut self, sheet: u32, new_name: &str) -> PyResult<()> {
        self.model
            .rename_sheet_by_index(sheet, new_name)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn get_sheet_index_by_name(&self, sheet_name: &str) -> PyResult<Option<i32>> {
        Ok(self
            .model
            .get_sheet_index_by_name(sheet_name)
            .map(|index| index as i32))
    }

    #[allow(clippy::panic)]
    pub fn test_panic(&self) -> PyResult<()> {
        panic!("This function panics for testing panic handling");
    }
}

fn calc_result_to_py_any(result: CalcResult, py: Python) -> PyResult<PyObject> {
    match result {
        CalcResult::String(s) => s.into_py_any(py),
        CalcResult::Number(n) => n.into_py_any(py),
        CalcResult::Boolean(b) => b.into_py_any(py),
        CalcResult::Error { error, message, .. } => (error.to_string(), message).into_py_any(py),
        CalcResult::Range { left, right } => (
            (left.sheet, left.row, left.column),
            (right.sheet, right.row, right.column),
        )
            .into_py_any(py),
        CalcResult::EmptyCell => Ok(py.None()),
        CalcResult::EmptyArg => Ok(py.None()),
        CalcResult::Array(arr) => {
            // Convert to Python list of lists
            let py_list_result = PyList::new(
                py,
                arr.iter().map(|row| {
                    match PyList::new(
                        py,
                        row.iter().map(|node| match node {
                            ArrayNode::Boolean(b) => {
                                b.into_py_any(py).unwrap_or_else(|_| py.None())
                            }
                            ArrayNode::Number(n) => n.into_py_any(py).unwrap_or_else(|_| py.None()),
                            ArrayNode::String(s) => s.into_py_any(py).unwrap_or_else(|_| py.None()),
                            ArrayNode::Error(e) => {
                                e.to_string().into_py_any(py).unwrap_or_else(|_| py.None())
                            }
                        }),
                    ) {
                        Ok(row_list) => row_list.into_py_any(py).unwrap_or_else(|_| py.None()),
                        Err(_) => py.None(),
                    }
                }),
            );

            match py_list_result {
                Ok(py_arr) => py_arr.into_py_any(py),
                Err(e) => Err(e),
            }
        }
    }
}

fn naivedate_to_excel_timestamp(date: NaiveDate) -> f64 {
    // Excel's epoch starts at December 30, 1899
    // Using December 30, 1899 instead of January 1, 1900 is intentional
    // This accounts for Excel's leap year error in 1900
    let excel_epoch = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();

    // Calculate days between the dates
    let days_since_epoch = date.signed_duration_since(excel_epoch).num_days();

    // Convert to f64 (Excel uses floating point for timestamps)
    days_since_epoch as f64
}

fn naivedatetime_to_excel_timestamp(dt: NaiveDateTime) -> f64 {
    // Excel's epoch starts at December 30, 1899
    let excel_epoch = NaiveDate::from_ymd_opt(1899, 12, 30)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    // Calculate total seconds between the dates
    let duration_since_epoch = dt.signed_duration_since(excel_epoch);

    // Convert to days (integer part)
    let days = duration_since_epoch.num_days() as f64;

    // Calculate the remaining seconds for the fractional part
    let remaining_seconds = duration_since_epoch.num_seconds() % 86400;

    // Convert remaining seconds to fraction of a day
    let fraction_of_day = remaining_seconds as f64 / 86400.0;

    // Combine the days and fraction of day
    days + fraction_of_day
}

// Create methods

/// Loads a function from an xlsx file
#[pyfunction]
pub fn load_from_xlsx(file_path: &str, locale: &str, tz: &str) -> PyResult<PyModel> {
    let model = import::load_from_xlsx(file_path, locale, tz)
        .map_err(|e| WorkbookError::new_err(e.to_string()))?;
    Ok(PyModel { model })
}

/// Loads a function from icalc binary representation
#[pyfunction]
pub fn load_from_icalc(file_name: &str) -> PyResult<PyModel> {
    let model =
        import::load_from_icalc(file_name).map_err(|e| WorkbookError::new_err(e.to_string()))?;
    Ok(PyModel { model })
}

/// Creates an empty model
#[pyfunction]
pub fn create(name: &str, locale: &str, tz: &str) -> PyResult<PyModel> {
    let model =
        Model::new_empty(name, locale, tz).map_err(|e| WorkbookError::new_err(e.to_string()))?;
    Ok(PyModel { model })
}

#[pyfunction]
#[allow(clippy::panic)]
pub fn test_panic() {
    panic!("This function panics for testing panic handling");
}

#[pymodule]
fn ironcalc(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Add the package version to the module
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    // Add the functions to the module using the `?` operator
    m.add_function(wrap_pyfunction!(create, m)?)?;
    m.add_function(wrap_pyfunction!(load_from_xlsx, m)?)?;
    m.add_function(wrap_pyfunction!(load_from_icalc, m)?)?;
    m.add_function(wrap_pyfunction!(test_panic, m)?)?;

    Ok(())
}
