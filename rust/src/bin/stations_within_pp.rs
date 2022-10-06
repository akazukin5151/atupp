use rayon::prelude::*;
use rstar::RTree;
use src::parse_csv_line;
use std::{
    fs, io,
    sync::{Arc, Mutex},
};

fn main() {
    let mut args = std::env::args();

    let (pp_path, stations_path) = if args.nth(1).unwrap() == "london" {
        let pp_path = "../data/london_pp_meters.csv";
        let stations_path =
            "../data/london_trains/stations/station_coords_meters.csv";
        (pp_path, stations_path)
    } else {
        let pp_path = "../data/tokyo_pp_meters.csv";
        let stations_path = "../data/tokyo_trains/coords_meters.csv";
        (pp_path, stations_path)
    };

    eprintln!("loading stations...");
    let stations = load_stations(stations_path);

    eprintln!("building tree...");
    let tree: RTree<(f64, f64)> = RTree::bulk_load(stations);

    // the pp file is just a few hundred MB, which can fit into RAM
    eprintln!("reading population points...");
    let file = fs::read_to_string(pp_path).unwrap();

    let pp_lines: Vec<_> = file
        .split('\n')
        .skip(1)
        .filter(|line| !line.is_empty())
        .collect();

    eprintln!("searching...");
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(&["max_dist", "n_stations"]).unwrap();
    let wtr = Arc::new(Mutex::new(wtr));

    let dists: Vec<_> = (100..=3000).step_by(100).collect();
    dists.into_par_iter().for_each(|max_dist| {
        let pop_within = pop_within_dist(&tree, &pp_lines, max_dist as f64);
        wtr.lock()
            .unwrap()
            .write_record(&[
                format!("{}", max_dist),
                format!(
                    "{}",
                    pop_within
                        .iter()
                        .map(|x| format!("{}", x))
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            ])
            .unwrap();
    });
}

fn pop_within_dist(
    tree: &RTree<(f64, f64)>,
    pp_lines: &Vec<&str>,
    max_distance: f64,
) -> Vec<i32> {
    let max_distance_squared = max_distance * max_distance;

    let pop_within_dist =
        Arc::new(Mutex::new(Vec::with_capacity(pp_lines.len())));

    pp_lines.into_par_iter().for_each(|pp_line| {
        let mut n_stations = 0;
        let xs = parse_csv_line(pp_line);

        // a line in pp looks like this
        // lat/lon, lat/lon, pop, x, y
        let x: f64 = xs[3].parse().unwrap();
        let y: f64 = xs[4].parse().unwrap();

        for _ in tree.locate_within_distance((x, y), max_distance_squared) {
            n_stations += 1;
        }
        (*pop_within_dist.lock().unwrap()).push(n_stations);
    });

    Arc::try_unwrap(pop_within_dist)
        .unwrap()
        .into_inner()
        .unwrap()
}

fn load_stations(path: &str) -> Vec<(f64, f64)> {
    let file = fs::read_to_string(path).unwrap();
    let lines = file.split('\n');
    lines
        .skip(1)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let xs = parse_csv_line(line);

            // both london and tokyo is (name, lat, lon, x, y)
            let x: f64 = xs[3].parse().unwrap();
            let y: f64 = xs[4].parse().unwrap();
            (x, y)
        })
        .collect()
}

