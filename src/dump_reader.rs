use polars::prelude::*;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/** This Rust code parses LAMMPS dump files */
pub struct DumpLammpsReader {
    pub dump_file_name: PathBuf,
    pub timesteps: Vec<u64>,
    pub trajectories: Vec<DataFrame>,
    pub box_state: DataFrame,
}

impl DumpLammpsReader {
    // API to parse LAMMPS dump file
    pub fn parse(
        dump_file_name: PathBuf,
    ) -> Result<BTreeMap<u64, DataFrame>, Box<dyn std::error::Error>> {
        let mut system = DumpLammpsReader {
            dump_file_name,
            timesteps: Vec::new(),
            trajectories: Vec::new(),
            box_state: DataFrame::empty(),
        };
        system.parse_lammps_dump()?;
        system.get_dump_map()
    }

    pub fn parse_state(dump_file_name: PathBuf) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let mut system = DumpLammpsReader {
            dump_file_name,
            timesteps: Vec::new(),
            trajectories: Vec::new(),
            box_state: DataFrame::empty(),
        };
        system.parse_lammps_dump()?;
        Ok(system.box_state)
    }

    // Parse LAMMPS dump file
    pub fn parse_lammps_dump(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut timesteps: Vec<String> = Vec::new();
        let mut atoms: Vec<String> = Vec::new();
        let mut single_dump_data: Vec<String> = Vec::new();
        let mut dump_data: Vec<Vec<String>> = Vec::new();
        let mut header: String = String::new();
        let mut start_parse_data: bool = false;
        let mut parse_in_progress: bool = false;
        let mut x_bounds: Vec<String> = Vec::new();
        let mut y_bounds: Vec<String> = Vec::new();
        let mut z_bounds: Vec<String> = Vec::new();
        let dump_file: File = File::open(&self.dump_file_name).map_err(|_| {
            format!(
                "Dump file at '{}' not found...\nCheck 'dump_file_name' parameter",
                &self.dump_file_name.display()
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
                atoms.push(lines.next().unwrap().unwrap());
                start_parse_data = false;
            }
            if line.starts_with("ITEM: BOX BOUNDS") {
                start_parse_data = false;
                x_bounds.push(lines.next().unwrap().unwrap());
                y_bounds.push(lines.next().unwrap().unwrap());
                z_bounds.push(lines.next().unwrap().unwrap());
                continue;
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

        if !single_dump_data.is_empty() {
            dump_data.push(single_dump_data);
        }

        // INFO: Begin parsing data into types
        self.timesteps = timesteps
            .iter()
            .map(|x| x.split_whitespace().last().unwrap().parse::<u64>().unwrap())
            .collect();
        let atoms: Vec<u64> = atoms
            .iter()
            .map(|x| x.split_whitespace().last().unwrap().parse::<u64>().unwrap())
            .collect();
        let header = header
            .strip_prefix("ITEM: ATOMS")
            .expect("Failed to parse header...")
            .split_whitespace()
            .collect::<Vec<&str>>();

        self.trajectories = dump_data
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

        //INFO: Parse system information in a dataframe
        let (xlo, xhi): (Vec<f64>, Vec<f64>) = x_bounds
            .iter()
            .map(|s| {
                let mut split = s.split_whitespace();
                (
                    split.next().unwrap().parse::<f64>().unwrap(),
                    split.next().unwrap().parse::<f64>().unwrap(),
                )
            })
            .unzip();

        let (ylo, yhi): (Vec<f64>, Vec<f64>) = y_bounds
            .iter()
            .map(|s| {
                let mut split = s.split_whitespace();
                (
                    split.next().unwrap().parse::<f64>().unwrap(),
                    split.next().unwrap().parse::<f64>().unwrap(),
                )
            })
            .unzip();

        let (zlo, zhi): (Vec<f64>, Vec<f64>) = z_bounds
            .iter()
            .map(|s| {
                let mut split = s.split_whitespace();
                (
                    split.next().unwrap().parse::<f64>().unwrap(),
                    split.next().unwrap().parse::<f64>().unwrap(),
                )
            })
            .unzip();

        self.box_state = df![
            "timestep" => &self.timesteps,
            "atoms" => atoms,
            "xlo" => xlo,
            "xhi" => xhi,
            "ylo" => ylo,
            "yhi" => yhi,
            "zlo" => zlo,
            "zhi" => zhi,
        ]?;

        Ok(())
    }
    pub fn get_dump_map(&self) -> Result<BTreeMap<u64, DataFrame>, Box<dyn std::error::Error>> {
        let data_map: BTreeMap<u64, DataFrame> = self
            .timesteps
            .iter()
            .cloned()
            .zip(self.trajectories.iter().cloned())
            .collect();

        Ok(data_map)
    }
}
