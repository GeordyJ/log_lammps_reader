use polars::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/** This Rust code parses LAMMPS dump files */
pub struct DumpLammpsReader {
    dump_file_name: PathBuf,
}

impl DumpLammpsReader {
    // API to parse LAMMPS dump file
    pub fn parse(
        dump_file_name: PathBuf,
    ) -> Result<HashMap<u64, DataFrame>, Box<dyn std::error::Error>> {
        DumpLammpsReader { dump_file_name }.parse_lammps_dump()
    }

    // Parse LAMMPS dump file
    fn parse_lammps_dump(&self) -> Result<HashMap<u64, DataFrame>, Box<dyn std::error::Error>> {
        let mut timesteps: Vec<String> = Vec::new();
        //let mut atoms: Vec<String> = Vec::new();
        let mut single_dump_data: Vec<String> = Vec::new();
        let mut dump_data: Vec<Vec<String>> = Vec::new();
        let mut header: String = String::new();
        let mut start_parse_data: bool = false;
        let mut parse_in_progress: bool = false;
        //let mut raw_box_bounds: Vec<String> = Vec::new();
        let dump_file: File = File::open(&self.dump_file_name).map_err(|_| {
            format!(
                "Dump file at '{}' not found...\nCheck 'dump_file_name' parameter",
                self.dump_file_name.display()
            )
        })?;
        let dump_reader: BufReader<File> = BufReader::new(dump_file);
        let mut lines = dump_reader.lines().peekable();
        while let Some(line_result) = lines.next() {
            let line: String = line_result?;
            if line.starts_with("ITEM: TIMESTEP") {
                timesteps.push(lines.next().unwrap().unwrap());
                start_parse_data = false;
            }
            if line.starts_with("ITEM: NUMBER OF ATOMS") {
                //atoms.push(lines.next().unwrap().unwrap());
                start_parse_data = false;
            }
            if line.starts_with("ITEM: BOX BOUNDS") {
                start_parse_data = false;
                continue;
                //raw_box_bounds.push(lines.next().unwrap().unwrap());
                //raw_box_bounds.push(lines.next().unwrap().unwrap());
                //raw_box_bounds.push(lines.next().unwrap().unwrap());
            }
            if line.starts_with("ITEM: ATOMS") {
                header = line;
                start_parse_data = true;
                if parse_in_progress {
                    dump_data.push(single_dump_data.clone());
                    single_dump_data.clear();
                }
                continue;
            }
            // start parsing ATOMS
            if start_parse_data {
                parse_in_progress = true;
                single_dump_data.push(line);
            }
        }
        let timesteps: Vec<u64> = timesteps
            .iter()
            .map(|x| x.split_whitespace().last().unwrap().parse::<u64>().unwrap())
            .collect();
        //let atoms: Vec<u64> = atoms
        //    .iter()
        //    .map(|x| x.split_whitespace().last().unwrap().parse::<u64>().unwrap())
        //    .collect();
        let header = header
            .strip_prefix("ITEM: ATOMS")
            .expect("Failed to strip prefix")
            .split_whitespace()
            .collect::<Vec<&str>>();

        let dump_data: Vec<DataFrame> = dump_data
            .par_iter()
            .map(|single_dump_data| {
                // Parse data
                let parsed_data: Vec<Vec<String>> = single_dump_data
                    .iter()
                    .map(|line| line.split_whitespace().map(String::from).collect())
                    .collect();

                // Transpose data into columns
                let mut columns: Vec<Column> = Vec::new();
                for (col_idx, col_name) in header.iter().enumerate() {
                    let col_data: Vec<&str> = parsed_data
                        .iter()
                        .map(|row| row[col_idx].as_str())
                        .collect();

                    let series = if col_data.iter().all(|v| v.parse::<i64>().is_ok()) {
                        let col_values: Vec<i64> =
                            col_data.iter().map(|v| v.parse().unwrap()).collect();
                        Column::new(col_name.to_string().into(), col_values)
                    } else if col_data.iter().all(|v| v.parse::<f64>().is_ok()) {
                        let col_values: Vec<f64> =
                            col_data.iter().map(|v| v.parse().unwrap()).collect();
                        Column::new(col_name.to_string().into(), col_values)
                    } else {
                        let col_values: Vec<String> =
                            col_data.iter().map(|&v| v.to_string()).collect();
                        Column::new(col_name.to_string().into(), col_values)
                    };

                    columns.push(series);
                }
                DataFrame::new(columns).expect("Failed to create DataFrame")
            })
            .collect();
        let data_map: HashMap<u64, DataFrame> = timesteps
            .iter()
            .cloned()
            .zip(dump_data.iter().cloned())
            .collect();
        Ok(data_map)
    }
}
