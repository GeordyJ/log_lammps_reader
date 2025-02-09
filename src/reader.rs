use polars::prelude::*;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/** This Rust code uses the Polars library to parse log files,
particularly from LAMMPS simulations. The goal is to read
specific data blocks from the log file and convert them
into a DataFrame format for further analysis. */
pub struct LogLammpsReader {
    log_file_name: PathBuf,
}

const MPI_FLAG: &str = "Per MPI rank memory allocation";
const ERROR_FLAGS: [&str; 2] = ["Loop time", "ERROR"];
const UNSUPPORTED_THERMO_STYLES: [&str; 2] = ["thermo_style multi", "thermo_style yaml"];

impl LogLammpsReader {
    /** Constructor to create a new instance of LogLammpsReader.

    ### Parameters:
    log_file_name: File path for the LAMMPS log file
    requried_thermo_run_id: The index of the run thermo (default = 0)

    Returns a polars DataFrame object*/
    pub fn new(
        log_file_name: PathBuf,
        requried_thermo_run_id: Option<u32>,
    ) -> Result<DataFrame, Box<dyn std::error::Error>> {
        LogLammpsReader { log_file_name }
            .parse_lammps_log(requried_thermo_run_id.unwrap_or_default())
    }

    /** Constructor to create a new instance of LogLammpsReader.

    ### Parameters:
    log_file_name: File path for the LAMMPS log file
    prefix_key: The prefix string to find the lines in log file

    Returns a vector of strings. */
    pub fn log_starts_with(
        log_file_name: PathBuf,
        prefix_key: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        LogLammpsReader { log_file_name }.parse_log_starts_with(prefix_key)
    }

    /// Returns a BufReader for a certain file
    fn log_buffer_reader(
        log_file_name: &PathBuf,
    ) -> Result<BufReader<File>, Box<dyn std::error::Error>> {
        let log_file: File = File::open(log_file_name).map_err(|_| {
            format!(
                "Log file at '{}' not found...\nCheck 'log_file_name' parameter",
                log_file_name.display()
            )
        })?;
        Ok(BufReader::new(log_file))
    }

    /// Method to parse the log file and convert the log file into a DataFrame.
    fn parse_lammps_log(
        &self,
        req_thermo_run_id: u32,
    ) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let mut thermo_run_id: u32 = 0;
        let mut data_flag: bool = false;
        let mut log_header_str: String = String::new();
        let mut raw_log_data: Vec<String> = Vec::new();

        let log_reader: BufReader<File> = LogLammpsReader::log_buffer_reader(&self.log_file_name)?;

        for line_result in log_reader.lines() {
            let line: String = line_result?;

            // Check for MPI flag to set minimization and data flags.
            if !data_flag {
                if UNSUPPORTED_THERMO_STYLES
                    .iter()
                    .any(|&flag| line.starts_with(flag))
                {
                    return Err(format!(
                        "This thermo style '{}' is not supported. Use 'one' or 'custom'",
                        line
                    )
                    .into());
                }
                if line.starts_with(MPI_FLAG) {
                    data_flag = true;
                }
                continue;
            }
            if log_header_str.is_empty() {
                log_header_str = line;
                continue;
            }

            // Reset flags and increase run number upon encountering any error flags.
            if ERROR_FLAGS.iter().any(|&flag| line.starts_with(flag)) {
                data_flag = false;
                thermo_run_id += 1;
                if thermo_run_id > req_thermo_run_id {
                    break;
                }
                log_header_str.clear();
                continue;
            }
            if thermo_run_id != req_thermo_run_id {
                continue;
            }
            raw_log_data.push(line);
        }

        let log_header: Vec<String> = log_header_str
            .split_whitespace()
            .map(String::from)
            .collect();

        let log_data: Vec<Vec<f64>> = raw_log_data
            .into_par_iter()
            .filter_map(|s| {
                let row: Vec<f64> = s
                    .split_whitespace()
                    .filter_map(|num| num.parse::<f64>().ok())
                    .collect();

                if row.len() == log_header.len() {
                    Some(row)
                } else {
                    None
                }
            })
            .collect();

        if log_data.is_empty() {
            return Err(format!(
                "No data found in the log file for run: {}\nThis may be caused due to:
                \n1. Incorrect 'run_number' parameter (Try 'run_number = {}')
                \n2. Unsual format of log file",
                req_thermo_run_id,
                req_thermo_run_id.saturating_sub(1)
            )
            .into());
        }

        // Convert the parsed data into a polars Series
        let columns: Vec<Column> = (0..log_header.len())
            .map(|index: usize| {
                let column_data: Vec<f64> = log_data
                    .par_iter()
                    .map(|row: &Vec<f64>| row[index])
                    .collect();
                Column::new((&log_header[index]).into(), column_data)
            })
            .collect();

        Ok(DataFrame::new(columns)?)
    }

    /// Returns all instance of a prefix string in a file
    fn parse_log_starts_with(
        &self,
        prefix_key: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut matched_lines: Vec<String> = Vec::new();
        let log_reader: BufReader<File> = LogLammpsReader::log_buffer_reader(&self.log_file_name)?;
        for line_result in log_reader.lines() {
            let line: String = line_result?;
            if line.starts_with(prefix_key) {
                matched_lines.push(line)
            }
        }
        Ok(matched_lines)
    }
}
