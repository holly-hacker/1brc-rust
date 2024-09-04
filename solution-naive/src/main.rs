use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{BufRead, BufReader},
};

fn main() {
    let file_name = std::env::args().nth(1).expect("No file name given");
    let file = File::open(file_name).expect("Could not open file");
    let mut reader = BufReader::new(file);

    let mut hash_map = HashMap::new();
    let mut line = String::new();
    while reader.read_line(&mut line).expect("read line") != 0 {
        let (left, right) = line
            .trim_end_matches('\n')
            .split_once(';')
            .expect("split_once");
        let num = right.parse::<f32>().expect("parse");

        let data = hash_map.entry(left.to_string()).or_insert(SomeData {
            min: num,
            max: num,
            sum: 0.,
            count: 0,
        });

        data.min = data.min.min(num);
        data.max = data.max.max(num);
        data.sum += num;
        data.count += 1;

        line.clear();
    }

    // move into btreemap to sort by key
    let sorted = hash_map.into_iter().collect::<BTreeMap<_, _>>();

    println!("Station;Min;Max;Avg");
    for (station, data) in sorted.iter() {
        let avg = data.sum / data.count as f32;
        println!("{};{};{};{}", station, data.min, data.max, avg);
    }
}

struct SomeData {
    min: f32,
    max: f32,
    sum: f32,
    count: u32,
}
