use polars::prelude::*;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

const ERROR_FLAGS: [&str; 2] = ["Loop time", "ERROR"];
const MPI_FLAGS: [&str; 2] = [
    "MPI task timing breakdown",
    "Per MPI rank memory allocation",
];

/** This Rust code uses the Polars library to parse log files,
particularly from LAMMPS simulations. The goal is to read
specific data blocks from the log file and convert them
into a DataFrame format for further analysis. The parsing
logic focuses on extracting data between specific MPI
flags and handling error flags appropriately. */
pub struct LogLammpsReader {
    log_file_name: PathBuf,
    thermo_run_number: u32,
}

impl LogLammpsReader {
    /** Constructor to create a new instance of LogLammpsReader.
    It takes a log file name and an optional thermo run number. */
    pub fn new(
        log_file_name: PathBuf,
        run_number: Option<u32>,
    ) -> Result<DataFrame, Box<dyn std::error::Error>> {
        LogLammpsReader {
            log_file_name,
            thermo_run_number: run_number.unwrap_or_default(),
        }
        .parse()
    }

    /// Method to parse the log file and convert the log file into a DataFrame.
    fn parse(&self) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let mut current_thermo_run_num: u32 = 0;
        let mut data_flag: bool = false;
        let mut minimization_flag: bool = false;
        let mut log_header: Vec<String> = Vec::new();
        let mut log_data: Vec<Vec<f64>> = Vec::new();

        let log_file: File = File::open(&self.log_file_name)?;
        let log_reader: BufReader<File> = BufReader::new(log_file);

        for line_result in log_reader.lines() {
            let line: String = line_result?;

            // Check for MPI flags to set minimization and run flags.
            if !minimization_flag || !data_flag {
                if line.starts_with(MPI_FLAGS[0]) {
                    minimization_flag = true;
                } else if line.starts_with(MPI_FLAGS[1]) && minimization_flag {
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
                minimization_flag = false;
                data_flag = false;
                current_thermo_run_num += 1;
                if current_thermo_run_num > self.thermo_run_number {
                    break;
                }
                log_header.clear();
                continue;
            }

            // Skip lines if the current run number does not match the specified run number.
            if self.thermo_run_number != current_thermo_run_num {
                continue;
            }

            // Parse data rows and filter out invalid rows.
            let row: Vec<f64> = line
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();

            if row.len() != log_header.len() {
                continue;
            }

            log_data.push(row);
        }

        if log_data.is_empty() {
            return Err(format!(
                "No data found in the log file for run {}",
                self.thermo_run_number
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
}
