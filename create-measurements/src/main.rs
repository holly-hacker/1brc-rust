//! Create measurements, based on https://github.com/ifnesi/1brc

use std::io::{BufWriter, Write};

mod stations;

const RECORD_COUNT: usize = 1_000_000_000;
const PROGRESS_INTERVAL: usize = 10_000_000;
const MAX_DEV: f64 = 10.;

fn main() {
    println!("Hello, world!");

    fastrand::seed(0);

    let time_start = std::time::Instant::now();

    let file = std::fs::File::create("measurements.csv").expect("create measurements.csv");
    let mut writer = BufWriter::with_capacity(50 * 1024 * 1024, file);

    let mut next_progress_report = 0;
    for i in 0..RECORD_COUNT {
        let station = &stations::STATIONS[fastrand::usize(..stations::STATIONS.len())];
        let temperature = station.1 + (fastrand::f64() * 2. - 1.) * MAX_DEV;
        let line = format!("{};{}\n", station.0, (temperature * 10.).round() / 10.);
        writer.write(line.as_bytes()).unwrap();

        if i == next_progress_report {
            println!("{} records written ({}%)", i, (i * 100) / RECORD_COUNT);
            next_progress_report += PROGRESS_INTERVAL;
        }
    }

    let end = time_start.elapsed();
    println!("Elapsed: {:?}", end);
}
