use polars::prelude::*;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

const ERROR_FLAGS: [&str; 2] = ["Loop time", "ERROR"];
const MPI_FLAG: &str = "Per MPI rank memory allocation";

/** This Rust code uses the Polars library to parse log files,
particularly from LAMMPS simulations. The goal is to read
specific data blocks from the log file and convert them
into a DataFrame format for further analysis. */
pub struct LogLammpsReader {
    log_file_name: PathBuf,
}

impl LogLammpsReader {
    /** Constructor to create a new instance of LogLammpsReader.

    Parameters:
    log_file_name: File path for the LAMMPS log file
    thermo_run_number: The index of the run thermo (default = 0)*/
    pub fn new(
        log_file_name: PathBuf,
        run_number: Option<u32>,
    ) -> Result<DataFrame, Box<dyn std::error::Error>> {
        LogLammpsReader { log_file_name }.parse_lammps_log(run_number.unwrap_or_default())
    }

    pub fn log_starts_with(
        log_file_name: PathBuf,
        prefix_key: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        LogLammpsReader { log_file_name }.parse_log_starts_with(prefix_key)
    }

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
        thermo_run_number: u32,
    ) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let mut current_thermo_run_num: u32 = 0;
        let mut data_flag: bool = false;
        let mut log_header: Vec<String> = Vec::new();
        let mut log_data: Vec<Vec<f64>> = Vec::new();

        let log_reader: BufReader<File> = LogLammpsReader::log_buffer_reader(&self.log_file_name)?;

        for line_result in log_reader.lines() {
            let line: String = line_result?;

            // Check for MPI flag to set minimization and data flags.
            if !data_flag {
                if line.starts_with(MPI_FLAG) {
                    data_flag = true;
                }
                continue;
            }

            // Capture the header line once the flags are set.
            if log_header.is_empty() {
                log_header = line.split_whitespace().map(String::from).collect();
                continue;
            }

            // Reset flags and increase run number upon encountering error flags.
            if line.starts_with(ERROR_FLAGS[0]) || line.starts_with(ERROR_FLAGS[1]) {
                data_flag = false;
                current_thermo_run_num += 1;
                if current_thermo_run_num > thermo_run_number {
                    break;
                }
                log_header.clear();
                continue;
            }

            // Skip lines if the current run number does not match the specified run number.
            if thermo_run_number != current_thermo_run_num {
                continue;
            }

            // Parse data rows
            let row: Vec<f64> = line
                .split_whitespace()
                .filter_map(|s: &str| s.parse().ok())
                .collect();

            // filter out invalid rows.
            if row.len() != log_header.len() {
                continue;
            }

            log_data.push(row);
        }

        if log_data.is_empty() {
            return Err(format!(
                "No data found in the log file for run: {}\nThis may be caused due to
                \n1. Incorrect 'run_number' parameter (Try 'run_number = {}')
                \n2. Unsual format of log file",
                thermo_run_number,
                thermo_run_number - 1
            )
            .into());
        }

        // Convert the parsed data into a polars Series
        let columns: Vec<Series> = (0..log_data[0].len())
            .map(|i| {
                let column_data: Vec<f64> = log_data.par_iter().map(|row| row[i]).collect();
                Series::new(&log_header[i], column_data)
            })
            .collect();

        Ok(DataFrame::new(columns)?)
    }

    fn parse_log_starts_with(
        &self,
        prefix_key: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut matched: Vec<String> = Vec::new();
        let log_reader: BufReader<File> = LogLammpsReader::log_buffer_reader(&self.log_file_name)?;
        for line_result in log_reader.lines() {
            let line: String = line_result?;
            if line.starts_with(prefix_key) {
                matched.push(line)
            }
        }
        Ok(matched)
    }
}
