use plotters::prelude::Quartiles;
use rayon::prelude::*;
use rstar::RTree;
use src::{load_stations, parse_csv_line, Search};
use std::{
    fs, io,
    sync::{Arc, Mutex},
};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let city = &args[1];
    let distance_threshold = args[2].parse().unwrap();
    let n_stations_q3 = args[3].parse().unwrap();

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
        n_stations_q3,
        distance_threshold,
    );
}

fn inner_main(
    pp_path: &str,
    stations_path: &str,
    n_stations_q3: f64,
    distance_threshold: f64,
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

    // TODO: for better code quality, pass this to the trait functions
    // so that the parsing isn't duplicated. in practice it doesn't
    // have a noticable impact on speed so not highest priority
    let populations: Vec<_> = pp_lines.clone().into_par_iter().map(|pp_line| {
        let xs = parse_csv_line(pp_line);

        // a line in pp looks like this
        // lat/lon, lat/lon, pop, x, y
        let pop: f64 = xs[2].parse().unwrap();
        pop
    }).collect();

    let pop_q3 = Quartiles::new(&populations).values()[3] as f64;

    let q = QuadrantCoords {
        pop_q3,
        n_stations_q3,
        distance_threshold,
    };
    q.search_to_file(&tree, &pp_lines);
}

struct QuadrantCoords {
    // TODO: calculate this without manual input
    // all n_stations needs to be counted first, then calculate Q3
    // one pass to do that count, calculate Q3, second pass to filter based on Q3
    pop_q3: f64,
    n_stations_q3: f64,
    distance_threshold: f64,
}

impl Search<Vec<(f64, f64)>> for QuadrantCoords {
    fn search_to_file(&self, tree: &RTree<(f64, f64)>, pp_lines: &[&str]) {
        eprintln!("searching...");
        let mut wtr = csv::Writer::from_writer(io::stdout());
        wtr.write_record(&["x", "y"]).unwrap();
        let wtr = Arc::new(Mutex::new(wtr));

        let xys = self.search(tree, pp_lines, self.distance_threshold);
        let mut w = wtr.lock().unwrap();
        for (x, y) in xys {
            w.write_record(&[format!("{}", x), format!("{}", y)])
                .unwrap();
        }
    }

    fn search(
        &self,
        tree: &RTree<(f64, f64)>,
        pp_lines: &[&str],
        max_distance: f64,
    ) -> Vec<(f64, f64)> {
        let max_distance_squared = max_distance * max_distance;

        let pop_within_dist =
            Arc::new(Mutex::new(Vec::with_capacity(pp_lines.len())));

        pp_lines.into_par_iter().for_each(|pp_line| {
            let mut n_stations = 0;
            let xs = parse_csv_line(pp_line);

            // a line in pp looks like this
            // lat/lon, lat/lon, pop, x, y
            let pop: f64 = xs[2].parse().unwrap();
            let x: f64 = xs[3].parse().unwrap();
            let y: f64 = xs[4].parse().unwrap();

            for _ in tree.locate_within_distance((x, y), max_distance_squared) {
                n_stations += 1;
            }
            // points with normal population but lots of stations
            if pop <= self.pop_q3 && (n_stations as f64) >= self.n_stations_q3 {
                (*pop_within_dist.lock().unwrap()).push((x, y));
            }
            // TODO: points with high population but few stations
            //if pop >= self.pop_q3 && (n_stations as f64) <= self.n_stations_q3 {
            //    (*pop_within_dist.lock().unwrap()).push((x, y));
            //}
        });

        Arc::try_unwrap(pop_within_dist)
            .unwrap()
            .into_inner()
            .unwrap()
    }
}
