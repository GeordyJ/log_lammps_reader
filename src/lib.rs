use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;

mod reader;
use reader::LogLammpsReader;

/** This Rust code integrates with Python using PyO3 and PyPolars
to provide a Python interface for reading and processing LAMMPS
log files. The main function `new` serves as a bridge between
Rust and Python, allowing Python code to call Rust functions to
parse log files. It utilizes the LogLammpsReader struct from the
`reader` module to handle the actual parsing and conversion of log
file data into a DataFrame. */
#[pyfunction]
fn new(log_file_name: &str, thermo_run_number: Option<u32>) -> PyResult<PyDataFrame> {
    match LogLammpsReader::new(log_file_name.into(), thermo_run_number) {
        Ok(df) => Ok(PyDataFrame(df)),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
            "LogLammpsReader error: {}",
            e
        ))),
    }
}

/// Adds the rust function to the python module.
#[pymodule]
fn log_lammps_reader(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(new, m)?)?;
    Ok(())
}
