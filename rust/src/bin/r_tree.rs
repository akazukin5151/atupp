// replace matrix.rs and cumulative.rs
// matrix.rs is brute force O(n*m) search
// where n is the number of stations and m is the number of population points
// which can reach millions to billions!
// cumulative also goes through every point to compare distances: O(n)

// this program uses a r* tree, which is O(log(n)) for searching distances,
// and O(n*log(n)) for insertion
// if we only insert the stations (tokyo has only ~1000), then insertion time is
// negligible.
// there are m population points, so searching for the nearest station
// for every population point is O(m*log(n)).
// m is much larger than log(n) so it's basically O(m)
// Vec<(pp, nearest_station)
// Vec<(pp, nearest_station_distance)
// from here a O(1) comparison can be made for any distance threshold.
// for example filter all nearest stations to be within 500m

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

    eprintln!("loading tree...");
    let tree: RTree<(f64, f64)> = RTree::bulk_load(stations);

    // the pp file is just a few hundred MB, which can fit into RAM
    eprintln!("loading population points...");
    let file = fs::read_to_string(pp_path).unwrap();

    eprintln!("getting city population...");
    let city_pop = total_city_pop(pp_path);
    dbg!(city_pop);

    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(&["max_dist", "prop"]).unwrap();
    let wtr = Arc::new(Mutex::new(wtr));

    let dists: Vec<_> = (100..=2000).step_by(100).collect();
    dists.into_par_iter().for_each(|max_dist| {
        let pop_within = pop_within_dist(&tree, &file, max_dist as f64);
        wtr.lock()
            .unwrap()
            .write_record(&[
                format!("{}", max_dist),
                format!("{}", pop_within / city_pop),
            ])
            .unwrap();
    });
}

fn pop_within_dist(
    tree: &RTree<(f64, f64)>,
    file: &str,
    max_distance: f64,
) -> f64 {
    let max_distance_squared = max_distance * max_distance;

    let pp_lines: Vec<_> = file
        .split('\n')
        .skip(1)
        .filter(|line| !line.is_empty())
        .collect();

    let pop_within_dist = Arc::new(Mutex::new(0.0));

    pp_lines.into_par_iter().for_each(|pp_line| {
        let xs = parse_csv_line(pp_line);

        // a line in pp looks like this
        // lat/lon, lat/lon, pop, x, y
        let pop: f64 = xs[2].parse().unwrap();
        let x: f64 = xs[3].parse().unwrap();
        let y: f64 = xs[4].parse().unwrap();

        if let Some((_, nearest_dist_squared)) =
            tree.nearest_neighbor_iter_with_distance_2(&(x, y)).next()
        {
            if nearest_dist_squared <= max_distance_squared {
                *pop_within_dist.lock().unwrap() += pop;
            }
        };
    });

    Arc::try_unwrap(pop_within_dist)
        .unwrap()
        .into_inner()
        .unwrap()
}

// TODO: copied from matrix
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

// TODO: copied from cumulative
fn total_city_pop(pp_path: &str) -> f64 {
    let file = fs::read_to_string(pp_path).unwrap();
    let mut sum = 0.0;
    for line in file.split('\n').skip(1).filter(|line| !line.is_empty()) {
        let xs = parse_csv_line(line);
        let pop: f64 = xs[2].parse().unwrap();
        sum += pop;
    }
    sum
}
