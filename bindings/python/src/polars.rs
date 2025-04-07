use polars::prelude::DataFrame;
use pyo3::pyclass;

#[pyclass]
#[repr(transparent)]
#[derive(Clone)]
pub struct PyDataFrame {
    pub df: DataFrame,
}

impl From<DataFrame> for PyDataFrame {
    fn from(df: DataFrame) -> Self {
        PyDataFrame { df }
    }
}

impl PyDataFrame {
    pub fn new(df: DataFrame) -> Self {
        PyDataFrame { df }
    }
}
