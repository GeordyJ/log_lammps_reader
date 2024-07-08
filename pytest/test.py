import time
import polars as pl


def test_log_lammps_reader(filename):
    start_time = time.time()
    import log_lammps_reader

    df = log_lammps_reader.new(filename)
    df = df.filter(pl.col('Time') > 1)   # Use polars to filter data.
    end_time = time.time()
    execution_time = end_time - start_time

    Time = df.get_column('Time') ** 2
    print(f'Execution time for log_lammps_reader: {execution_time} seconds')
    print(Time)


def test_lammps_logfile(filename):
    start_time = time.time()
    import lammps_logfile

    df = lammps_logfile.File(filename)
    end_time = time.time()
    execution_time = end_time - start_time

    Time = df.get('Time', 1)
    print(f'Execution time for lammps_logfile: {execution_time} seconds')
    print(Time)


if __name__ == '__main__':
    filename = '/Users/geordy/github/tjat/log.lammps'
    test_log_lammps_reader(filename)
    test_lammps_logfile(filename)
