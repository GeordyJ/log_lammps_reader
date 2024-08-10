use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;

mod reader;
use reader::LogLammpsReader;

/**
### Parameters:
`log_file_name`: File path for the LAMMPS log file
`thermo_run_number`: The index of the run thermo (default = 0)
Note:
The default thermo_run_number includes the MPI minimization data
So usually what you need will start at index 1
*/
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

/**
### Parameters:
`log_file_name`: File path for the LAMMPS log file
`prefix_key`: The string key in which the line in the log
    file starts with
Note:
This returns a list of strings that satisfy the above key.
*/
#[pyfunction]
fn log_starts_with(log_file_name: &str, prefix_key: &str) -> PyResult<Vec<String>> {
    match LogLammpsReader::log_starts_with(log_file_name.into(), prefix_key) {
        Ok(matched_lines) => Ok(matched_lines),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
            "LogLammpsReader error: {}",
            e
        ))),
    }
}

/** Adds the rust function to the python module.
This Rust code integrates with Python using PyO3 and PyPolars
to provide a Python interface for reading and processing LAMMPS
log files. The main function `new` serves as a bridge between
Rust and Python, allowing Python code to call Rust functions to
parse log files. It utilizes the LogLammpsReader struct from the
`reader` module to handle the actual parsing and conversion of log
file data into a DataFrame. */
#[pymodule]
fn log_lammps_reader(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(new, m)?)?;
    m.add_function(wrap_pyfunction!(log_starts_with, m)?)?;
    Ok(())
}
