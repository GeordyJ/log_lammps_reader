use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;
use std::collections::HashMap;

mod dump_reader;
mod log_reader;
use dump_reader::DumpLammpsReader;
use log_reader::LogLammpsReader;

/**
### Parameters:
`log_file_name`: File path for the LAMMPS log file
`requried_thermo_run_id`: The index of the run thermo output (default = 0)
Note:
The default requried_thermo_run_id includes the MPI minimization data
So usually what you need will start at index 1
*/
#[pyfunction]
#[pyo3(signature = (log_file_name, requried_thermo_run_id=None))]
fn parse(log_file_name: &str, requried_thermo_run_id: Option<u32>) -> PyResult<PyDataFrame> {
    match LogLammpsReader::parse(log_file_name.into(), requried_thermo_run_id) {
        Ok(df) => Ok(PyDataFrame(df)),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
            "LogLammpsReader error: {}",
            e
        ))),
    }
}

/**
Parses a LAMMPS dump file and returns a HashMap/dict of timesteps and polars DataFrame objects.

# Arguments
* `dump_file_name` - A string slice representing the name of the LAMMPS dump file to be parsed.

# Returns
* `dict{int,polars.DataFrame}` - A Python result containing a HashMap where the keys
  are timesteps (int) and the values are polars DataFrame objects, or a Python exception if an error occurs.

# Errors
 Returns a `PyException` if the `DumpLammpsReader::parse` function fails.
*/
#[pyfunction]
#[pyo3(signature = (dump_file_name))]
fn parse_dump(dump_file_name: &str) -> PyResult<HashMap<u64, PyDataFrame>> {
    match DumpLammpsReader::parse(dump_file_name.into()) {
        Ok(df_map) => Ok(df_map
            .into_iter()
            .map(|(timestep, df)| (timestep, PyDataFrame(df)))
            .collect()),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
            "DumpLammpsReader error: {}",
            e
        ))),
    }
}

/**
### Depreciation Warning: Use .parse() instead of .new()
*/
#[pyfunction]
#[pyo3(signature = (log_file_name, requried_thermo_run_id=None))]
fn new(log_file_name: &str, requried_thermo_run_id: Option<u32>) -> PyResult<PyDataFrame> {
    println!(
        "Depreciation Warning: The .new() function has been renamed to .parse(). It will be removed in future versions!"
    );
    match LogLammpsReader::parse(log_file_name.into(), requried_thermo_run_id) {
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
fn log_lammps_reader(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(new, m)?)?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_dump, m)?)?;
    m.add_function(wrap_pyfunction!(log_starts_with, m)?)?;
    Ok(())
}
