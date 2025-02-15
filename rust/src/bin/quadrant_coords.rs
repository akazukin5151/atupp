// Usage: target/release/quadrant_coords [city] [X meters] [point_type]

use plotters::prelude::Quartiles;
use rayon::prelude::*;
use rstar::RTree;
use src::{load_stations, parse_csv_line, Search};
use std::fs;

enum PointType {
    Red,
    Orange,
    Blue,
    Green,
}

impl PointType {
    fn to_cond(
        &self,
        pop: f64,
        n_stations: f64,
        pop_q3: f64,
        n_stations_q3: f64,
    ) -> bool {
        match self {
            PointType::Red => {
                // points with normal population but lots of stations
                pop <= pop_q3 && n_stations > n_stations_q3
            }
            PointType::Orange => {
                // points with high population but few stations
                pop > pop_q3 && n_stations <= n_stations_q3
            }
            PointType::Blue => {
                // points with high population and lots of stations
                pop > pop_q3 && n_stations > n_stations_q3
            }
            PointType::Green => {
                // points with low population and few stations
                pop <= pop_q3 && n_stations <= n_stations_q3
            }
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let city = &args[1];
    let distance_threshold = args[2].parse().unwrap();
    let filter_by = &args[3];
    let point_type = match filter_by.as_str() {
        "red" => PointType::Red,
        "orange" => PointType::Orange,
        "blue" => PointType::Blue,
        "green" => PointType::Green,
        _ => panic!("unknown point type"),
    };
    let outfile = &args[4];
    let pp_path = format!("../data/{}_pp_meters.csv", city);

    // TODO: fix this inconsistency...
    let stations_path = if city == "london" {
        "../data/london_trains/stations/station_coords_meters.csv"
    } else {
        "../data/tokyo_trains/coords_meters.csv"
    };

    inner_main(
        &pp_path,
        stations_path,
        distance_threshold,
        point_type,
        outfile,
    );
}

fn inner_main(
    pp_path: &str,
    stations_path: &str,
    distance_threshold: f64,
    point_type: PointType,
    outfile: &str,
) {
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

    eprintln!("calculating Q3 of population points...");
    // TODO: for better code quality, pass this to the trait functions
    // so that the parsing isn't duplicated. in practice it doesn't
    // have a noticable impact on speed so not highest priority
    let populations: Vec<_> = pp_lines
        .clone()
        .into_par_iter()
        .map(|pp_line| {
            let xs = parse_csv_line(pp_line);

            // a line in pp looks like this
            // lat/lon, lat/lon, pop, x, y
            let pop: f64 = xs[2].parse().unwrap();
            pop
        })
        .collect();

    let pop_q3 = Quartiles::new(&populations).values()[3] as f64;

    eprintln!("calculating Q3 of n stations...");
    let n_stations_vec = count_n_stations(&tree, &pp_lines, distance_threshold);
    let n_stations_q3 = Quartiles::new(&n_stations_vec).values()[3] as f64;

    let q = QuadrantCoords {
        pop_q3,
        n_stations_q3,
        distance_threshold,
        point_type,
        outfile,
    };
    q.search_to_file(&tree, &pp_lines);
}

struct QuadrantCoords<'a> {
    pop_q3: f64,
    n_stations_q3: f64,
    distance_threshold: f64,
    point_type: PointType,
    outfile: &'a str,
}

fn count_n_stations(
    tree: &RTree<(f64, f64)>,
    pp_lines: &[&str],
    max_distance: f64,
) -> Vec<i32> {
    let max_distance_squared = max_distance * max_distance;

    let pop_within_dist: Vec<_> = pp_lines
        .into_par_iter()
        .map(|pp_line| {
            let mut n_stations = 0;
            let xs = parse_csv_line(pp_line);

            // a line in pp looks like this
            // lat/lon, lat/lon, pop, x, y
            let x: f64 = xs[3].parse().unwrap();
            let y: f64 = xs[4].parse().unwrap();

            for _ in tree.locate_within_distance((x, y), max_distance_squared) {
                n_stations += 1;
            }
            n_stations
        })
        .collect();

    pop_within_dist
}

impl Search<Vec<(f64, f64)>> for QuadrantCoords<'_> {
    fn search_to_file(&self, tree: &RTree<(f64, f64)>, pp_lines: &[&str]) {
        eprintln!("searching...");

        let xys = self.search(tree, pp_lines, self.distance_threshold);
        let res: Vec<_> = xys
            .into_iter()
            .map(|(x, y)| format!("{},{}", x, y))
            .collect();

        let joined = "x,y\n".to_string() + &res.join("\n");
        fs::write(self.outfile, joined).unwrap();
    }

    fn search(
        &self,
        tree: &RTree<(f64, f64)>,
        pp_lines: &[&str],
        max_distance: f64,
    ) -> Vec<(f64, f64)> {
        let max_distance_squared = max_distance * max_distance;

        let pop_within_dist: Vec<_> = pp_lines
            .into_par_iter()
            .filter_map(|pp_line| {
                let mut n_stations = 0;
                let xs = parse_csv_line(pp_line);

                // a line in pp looks like this
                // lat/lon, lat/lon, pop, x, y
                let pop: f64 = xs[2].parse().unwrap();
                let x: f64 = xs[3].parse().unwrap();
                let y: f64 = xs[4].parse().unwrap();

                for _ in
                    tree.locate_within_distance((x, y), max_distance_squared)
                {
                    n_stations += 1;
                }
                let n_stations = n_stations as f64;
                if self.point_type.to_cond(
                    pop,
                    n_stations,
                    self.pop_q3,
                    self.n_stations_q3,
                ) {
                    Some((x, y))
                } else {
                    None
                }
            })
            .collect();

        pop_within_dist
    }
}
