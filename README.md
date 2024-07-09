# Log LAMMPS Reader

Log LAMMPS Reader is a high-performance Rust library and Python extension for reading LAMMPS log files and converting them into DataFrames using the [Polars](https://pola.rs/) library. This project leverages [PyO3](https://pyo3.rs/) to create a Python module that interfaces with Rust code, ensuring both speed and efficiency.

This package returns a polars DataFrame allowing the user to use powerful data manipulations (e.g filters) provided through polars.

## Features

- **High-speed** reading of LAMMPS log files
- Converts log data into Polars DataFrames
- Exposes functionality to Python through a PyO3 module
- Gets thermo data for multiple thermo runs
- Better data parsing, skips rows if they are invalid (e.g missing newline, non-numeric characters in the log)
- Only stores the needed thermo run data specified by user

## Installation

### Python

Using pip:

```bash
pip install log-lammps-reader
```

## Build From Source

Alternatively, to build the Python module, follow these steps:

### Requirements

- Rust (latest stable version recommended)
- Python 3.8 or later
- Cargo (Rust package manager)

1. Ensure you have `maturin` installed:

   ```bash
   pip install maturin
   ```

2. Compile the Rust packages and install the python module.

    ```bash
    maturin develop --release
    ```

## Usage Examples

### Python

```python
import log_lammps_reader

thermo_number = 0 # Choose the nth number of thermo run
df = log_lammps_reader.new('log.lammps') # polars DataFrame for 1st thermo run

# Or choose the nth number of thermo run (default n = 0)
df = log_lammps_reader.new('log.lammps', n) 
time = df.get_column('Time') # Get any thermo column
time_squared = time ** 2 # use broadcasting operations similar to numpy

# Use polars to filter the results.
import polars as pl
equilibrated_df = df.filter(pl.col('Time') > 1) 

# Convert data to numpy if needed
import numpy as np
step = np.array(df.get_column('Step'))
```

Example of a DataFrame for a LAMMPS log file.

```python
>>> import log_lammps_reader
>>> df = log_lammps_reader.new('log.lammps')
>>> df
shape: (10_000_002, 10)
┌──────────────┬───────────┬───────────┬───────────┬───┬───────┬────────────┬───────────┬───────────┐
│ Step         ┆ Time      ┆ Temp      ┆ Press     ┆ … ┆ Atoms ┆ PotEng     ┆ KinEng    ┆ TotEng    │
│ ---          ┆ ---       ┆ ---       ┆ ---       ┆   ┆ ---   ┆ ---        ┆ ---       ┆ ---       │
│ f64          ┆ f64       ┆ f64       ┆ f64       ┆   ┆ f64   ┆ f64        ┆ f64       ┆ f64       │
╞══════════════╪═══════════╪═══════════╪═══════════╪═══╪═══════╪════════════╪═══════════╪═══════════╡
│ 61.0         ┆ 0.0       ┆ 298.0     ┆ 57.20028  ┆ … ┆ 519.0 ┆ -14.776112 ┆ 19.953113 ┆ 5.1770012 │
│ 70.0         ┆ 0.009     ┆ 296.73074 ┆ 60.840723 ┆ … ┆ 519.0 ┆ -14.721924 ┆ 19.868128 ┆ 5.1462039 │
│ 80.0         ┆ 0.019     ┆ 292.56952 ┆ 72.565657 ┆ … ┆ 519.0 ┆ -14.530972 ┆ 19.589506 ┆ 5.0585341 │
│ 90.0         ┆ 0.029     ┆ 285.36347 ┆ 92.936408 ┆ … ┆ 519.0 ┆ -14.18668  ┆ 19.107012 ┆ 4.9203316 │
│ 100.0        ┆ 0.039     ┆ 275.29149 ┆ 121.91127 ┆ … ┆ 519.0 ┆ -13.681587 ┆ 18.432625 ┆ 4.7510379 │
│ …            ┆ …         ┆ …         ┆ …         ┆ … ┆ …     ┆ …          ┆ …         ┆ …         │
│ 1.0000003e8  ┆ 99999.969 ┆ 301.90216 ┆ 225.03035 ┆ … ┆ 519.0 ┆ -11.279288 ┆ 20.214389 ┆ 8.9351011 │
│ 1.0000004e8  ┆ 99999.979 ┆ 301.99266 ┆ 220.86566 ┆ … ┆ 519.0 ┆ -11.33326  ┆ 20.220449 ┆ 8.8871881 │
│ 1.0000005e8  ┆ 99999.989 ┆ 302.04158 ┆ 215.55467 ┆ … ┆ 519.0 ┆ -11.406581 ┆ 20.223724 ┆ 8.8171428 │
│ 1.0000006e8  ┆ 99999.999 ┆ 301.61379 ┆ 210.565   ┆ … ┆ 519.0 ┆ -11.471215 ┆ 20.195081 ┆ 8.723866  │
│ 1.00000061e8 ┆ 100000.0  ┆ 301.52726 ┆ 210.15164 ┆ … ┆ 519.0 ┆ -11.475823 ┆ 20.189287 ┆ 8.7134637 │
└──────────────┴───────────┴───────────┴───────────┴───┴───────┴────────────┴───────────┴───────────┘
>>> df.get_column('Time')
shape: (10_000_002,)
Series: 'Time' [f64]
[
        0.0
        0.009
        0.019
        0.029
        0.039
        …
        99999.969
        99999.979
        99999.989
        99999.999
        100000.0
]
>>> df.get_column('Time').mean()
50000.00399999919
>>> df.get_column('Time').std()
28867.520676357886
>>>
```

### Rust

Clone the repo and add it to your project

```rust
use log_lammps_reader::LogLammpsReader;

fn main() {
    let log_file_name = "log.lammps";
    let run_number = Some(0);

    match LogLammpsReader::new(log_file_name.into(), run_number) {
        Ok(df) => println!("DataFrame read successfully: {:?}", df),
        Err(e) => eprintln!("Error reading DataFrame: {}", e),
    }
}
```
