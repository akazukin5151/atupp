// Usage: target/release/cumulative_props [city]

use rayon::prelude::*;
use rstar::RTree;
use src::{load_stations, parse_csv_line, Search};
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

    let o = CumulativeProps { pp_path };
    o.search_to_file(&tree, &pp_lines);
}

struct CumulativeProps {
    pp_path: &'static str,
}

impl Search<f64> for CumulativeProps {
    fn search_to_file(&self, tree: &RTree<(f64, f64)>, pp_lines: &[&str]) {
        eprintln!("getting city population...");
        let city_pop = total_city_pop(self.pp_path);
        dbg!(city_pop);

        eprintln!("searching...");
        let mut wtr = csv::Writer::from_writer(io::stdout());
        wtr.write_record(&["max_dist", "prop"]).unwrap();
        let wtr = Arc::new(Mutex::new(wtr));

        let dists: Vec<_> = (100..=3000).step_by(100).collect();
        dists.into_par_iter().for_each(|max_dist| {
            let pop_within =
                self.search(tree, pp_lines, max_dist as f64);
            wtr.lock()
                .unwrap()
                .write_record(&[
                    format!("{}", max_dist),
                    format!("{}", pop_within / city_pop),
                ])
                .unwrap();
        });
    }

    fn search(
        &self,
        tree: &RTree<(f64, f64)>,
        pp_lines: &[&str],
        max_distance: f64,
    ) -> f64 {
        let max_distance_squared = max_distance * max_distance;

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
}

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
