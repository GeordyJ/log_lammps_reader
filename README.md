# Log LAMMPS Reader

Log LAMMPS Reader is a high-performance Rust library and Python extension for reading LAMMPS log files and converting them into DataFrames using the Polars library. This project leverages PyO3 to create a Python module that interfaces with Rust code, ensuring both speed and efficiency.

This package returns a polars DataFrame allowing the user to use powerful data manipulations (e.g filters) provided through polars.

## Features

- **High-speed** reading of LAMMPS log files
- Converts log data into Polars DataFrames
- Exposes functionality to Python through a PyO3 module
- Gets thermo data for multiple thermo runs.

## Requirements

- Rust (latest stable version recommended)
- Python 3.6 or later
- Cargo (Rust package manager)

## Installation

### Python

To build and install the Python module, follow these steps:


1. Ensure you have `maturin` and `polars` installed:

   ```bash
   pip install maturin polars
   ```

2. Compile the Rust packages and install the python module.

    ```bash
    maturin develop --release
    ```

## Usage Examples

### Python
```python
import polars as pl
import numpy as np
import log_lammps_reader


thermo_number = 0 # Choose the nth number of thermo run
df = log_lammps_reader.new('log.lammps', thermo_number) # polars DataFrame
equilibrated_df = df.filter(pl.col('Time') > 1) # Use polars to filter data.
time = df.get_column('Time') # Get any thermo column
time_squared = time ** 2 # use broadcasting operations similar to numpy
step = np.array(df.get_column('Step')) # or use numpy
```

### Rust

First install using `cargo build --release` and add it to your project
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
