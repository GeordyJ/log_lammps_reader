use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;

mod reader;
use reader::LogLammpsReader;

/// Reads a LAMMPS log file and returns a DataFrame.
#[pyfunction]
fn new(log_file_name: &str, run_number: Option<u32>) -> PyResult<PyDataFrame> {
    match LogLammpsReader::new(log_file_name.into(), run_number) {
        Ok(df) => Ok(PyDataFrame(df)),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
            "LogLammpsReader error: {}",
            e
        ))),
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn log_lammps_reader(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(new, m)?)?;
    Ok(())
}
