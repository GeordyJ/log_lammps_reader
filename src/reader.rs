use polars::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub struct LogLammpsReader {
    log_file_name: PathBuf,
    run_number: u32,
}

impl LogLammpsReader {
    pub fn new(
        log_file_name: PathBuf,
        run_number: Option<u32>,
    ) -> Result<DataFrame, Box<dyn std::error::Error>> {
        LogLammpsReader {
            log_file_name,
            run_number: run_number.unwrap_or_default(),
        }
        .parse()
    }

    fn parse(&self) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let log_file = File::open(&self.log_file_name)?;
        let log_reader = BufReader::new(log_file);

        let mut current_run_num: u32 = 0;
        let mut run_flag = false;
        let mut minimization_flag = false;
        let mut log_header: Vec<String> = Vec::new();
        let mut log_data: Vec<Vec<f64>> = Vec::new();

        for line_result in log_reader.lines() {
            let line = line_result?;

            if !minimization_flag || !run_flag {
                if line.starts_with("MPI task timing breakdown") {
                    minimization_flag = true;
                } else if line.starts_with("Per MPI rank memory allocation") && minimization_flag {
                    run_flag = true;
                }
                continue;
            }

            if log_header.is_empty() {
                log_header = line.split_whitespace().map(String::from).collect();
                continue;
            }

            if line.starts_with("Loop time") || line.starts_with("ERROR") {
                minimization_flag = false;
                run_flag = false;
                current_run_num += 1;
                if current_run_num > self.run_number {
                    break;
                }
                log_header.clear();
                continue;
            }

            if self.run_number != current_run_num {
                continue;
            }

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
            return Err(
                format!("No data found in the log file for run {}", self.run_number).into(),
            );
        }

        let columns: Vec<Series> = (0..log_data[0].len())
            .map(|i| {
                let column_data: Vec<f64> = log_data.iter().map(|row| row[i]).collect();
                Series::new(&log_header[i], column_data)
            })
            .collect();

        Ok(DataFrame::new(columns)?)
    }
}
