use crate::DumpLammpsReader;
use polars::prelude::*;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub struct AnalyzeLammps {
    file_name: PathBuf,
}

impl AnalyzeLammps {
    pub fn mean_square_displacements(
        file_name: PathBuf,
    ) -> Result<BTreeMap<u64, f64>, Box<dyn std::error::Error>> {
        let mut msd_map: BTreeMap<u64, f64> = BTreeMap::new();
        let dump_data: BTreeMap<u64, DataFrame> = DumpLammpsReader::parse(file_name)?;
        for (timestep, df) in dump_data {
            let msd = df
                .clone()
                .lazy()
                .select([(col("x").pow(2) + col("y").pow(2) + col("z").pow(2)).alias("r_squared")])
                .select([
                    col("r_squared").mean().alias("msd"), // compute MSD
                ])
                .collect()?; // msd is a DataFrame with a single value

            if let Some(msd_value) = msd.column("msd")?.f64()?.get(0) {
                msd_map.insert(timestep, msd_value);
            }
        }
        Ok(msd_map)
    }
}
