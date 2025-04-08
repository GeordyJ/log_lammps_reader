use crate::DumpLammpsReader;
use anyhow::Result;
use polars::prelude::*;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub struct AnalyzeLammps {
    file_name: PathBuf,
}

impl AnalyzeLammps {
    pub fn unwrap(mut trajectories: Vec<DataFrame>, box_bounds: &DataFrame) -> Vec<DataFrame> {
        for (i, df) in trajectories.iter_mut().enumerate() {
            let xlo = box_bounds
                .column("xlo")
                .unwrap()
                .f64()
                .unwrap()
                .get(i)
                .unwrap();
            let xhi = box_bounds
                .column("xhi")
                .unwrap()
                .f64()
                .unwrap()
                .get(i)
                .unwrap();
            let ylo = box_bounds
                .column("ylo")
                .unwrap()
                .f64()
                .unwrap()
                .get(i)
                .unwrap();
            let yhi = box_bounds
                .column("yhi")
                .unwrap()
                .f64()
                .unwrap()
                .get(i)
                .unwrap();
            let zlo = box_bounds
                .column("zlo")
                .unwrap()
                .f64()
                .unwrap()
                .get(i)
                .unwrap();
            let zhi = box_bounds
                .column("zhi")
                .unwrap()
                .f64()
                .unwrap()
                .get(i)
                .unwrap();

            let lx = xhi - xlo;
            let ly = yhi - ylo;
            let lz = zhi - zlo;

            let x_vals = df.column("x").unwrap().f64().unwrap();
            let y_vals = df.column("y").unwrap().f64().unwrap();
            let z_vals = df.column("z").unwrap().f64().unwrap();

            let mut x_unwrapped = vec![x_vals.get(0).unwrap()];
            let mut y_unwrapped = vec![y_vals.get(0).unwrap()];
            let mut z_unwrapped = vec![z_vals.get(0).unwrap()];

            for j in 1..x_vals.len() {
                let mut xj = x_vals.get(j).unwrap();
                let dx = xj - x_unwrapped[j - 1];
                if dx > lx / 2.0 {
                    xj -= lx;
                } else if dx < -lx / 2.0 {
                    xj += lx;
                }
                x_unwrapped.push(xj);

                let mut yj = y_vals.get(j).unwrap();
                let dy = yj - y_unwrapped[j - 1];
                if dy > ly / 2.0 {
                    yj -= ly;
                } else if dy < -ly / 2.0 {
                    yj += ly;
                }
                y_unwrapped.push(yj);

                let mut zj = z_vals.get(j).unwrap();
                let dz = zj - z_unwrapped[j - 1];
                if dz > lz / 2.0 {
                    zj -= lz;
                } else if dz < -lz / 2.0 {
                    zj += lz;
                }
                z_unwrapped.push(zj);
            }

            df.replace("x".into(), Series::new("x".into(), x_unwrapped))
                .unwrap();
            df.replace("y".into(), Series::new("y".into(), y_unwrapped))
                .unwrap();
            df.replace("z".into(), Series::new("z".into(), z_unwrapped))
                .unwrap();
        }

        trajectories
    }
    pub fn mean_square_displacement(
        file_name: PathBuf,
        unwrap_trajectory: bool,
    ) -> Result<BTreeMap<u64, f64>, Box<dyn std::error::Error>> {
        let mut msd_map: BTreeMap<u64, f64> = BTreeMap::new();
        let mut system = DumpLammpsReader {
            dump_file_name: file_name,
            timesteps: Vec::new(),
            trajectories: Vec::new(),
            box_state: DataFrame::empty(),
        };
        system.parse_lammps_dump()?;
        if unwrap_trajectory {
            system.trajectories = Self::unwrap(system.trajectories, &system.box_state);
        }

        let dump_data: BTreeMap<u64, DataFrame> = system.get_dump_map()?;

        // Assume first timestep is initial positions
        let Some(first_df) = system.trajectories.iter().next() else {
        return Err("Empty dump data".into());
    };

        let initial_positions = first_df
            .clone()
            .lazy()
            .select([
                col("id"),
                col("x").alias("x0"),
                col("y").alias("y0"),
                col("z").alias("z0"),
            ])
            .collect()?;

        for (timestep, df) in dump_data {
            let msd_df = df
                .lazy()
                .join(
                    initial_positions.clone().lazy(),
                    [col("id")],
                    [col("id")],
                    JoinArgs::new(JoinType::Inner),
                ) // Corrected join
                .with_columns([((col("x") - col("x0")).pow(2)
                    + (col("y") - col("y0")).pow(2)
                    + (col("z") - col("z0")).pow(2))
                .alias("r_squared")])
                .select([col("r_squared").mean().alias("msd")])
                .collect()?;

            if let Some(msd_value) = msd_df.column("msd")?.f64()?.get(0) {
                msd_map.insert(timestep, msd_value);
            }
        }

        Ok(msd_map)
    }
}
