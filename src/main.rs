mod reader;
use reader::LogLammpsReader;

/** Tests the LogLammpsReader by getting arguments from
 CLI. The first is the filename and the second is the optional
thermo run number */
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <log_file_name> [run_number]", args[0]);
        std::process::exit(1);
    }

    let log_file_name = &args[1];
    let thermo_run_number = if args.len() > 2 {
        args[2].parse::<u32>().ok()
    } else {
        None
    };

    // Create a LogLammpsReader instance and read the DataFrame
    match LogLammpsReader::new(log_file_name.into(), thermo_run_number) {
        Ok(df) => {
            println!("DataFrame read successfully: {:?}", df);
        }
        Err(e) => {
            eprintln!("Error reading DataFrame: {}", e);
            std::process::exit(1);
        }
    }
}
