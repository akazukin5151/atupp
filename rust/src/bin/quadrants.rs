use plotters::prelude::*;
use rayon::prelude::*;
use rstar::RTree;
use src::{load_stations, parse_csv_line, Plot, Search};
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

    Quadrants::search_to_plot(&tree, &pp_lines);
}

struct Quadrants;

impl Search<Vec<(f64, i32)>> for Quadrants {
    fn search_to_file(&self, tree: &RTree<(f64, f64)>, pp_lines: &[&str]) {
        eprintln!("searching...");
        let mut wtr = csv::Writer::from_writer(io::stdout());
        wtr.write_record(&["population", "n_stations"]).unwrap();
        let wtr = Arc::new(Mutex::new(wtr));

        let n_stations = Self::n_stations_within_dist(tree, pp_lines, 500.);
        let mut w = wtr.lock().unwrap();
        for (pop, n_stations) in n_stations {
            w.write_record(&[format!("{}", pop), format!("{}", n_stations)])
                .unwrap();
        }
    }

    fn n_stations_within_dist(
        tree: &RTree<(f64, f64)>,
        pp_lines: &[&str],
        max_distance: f64,
    ) -> Vec<(f64, i32)> {
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
            (*pop_within_dist.lock().unwrap()).push((pop, n_stations));
        });

        Arc::try_unwrap(pop_within_dist)
            .unwrap()
            .into_inner()
            .unwrap()
    }
}

impl Plot<Vec<(f64, i32)>> for Quadrants {
    fn search_to_plot(tree: &RTree<(f64, f64)>, pp_lines: &[&str]) {
        eprintln!("searching...");

        let mut result = vec![];
        let n_stations = Self::n_stations_within_dist(tree, pp_lines, 500.);
        result.extend(n_stations);

        Self::plot(result).unwrap();
    }

    fn plot(data: Vec<(f64, i32)>) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new("../out/rust_q.png", (1024, 768))
            .into_drawing_area();

        root.fill(&WHITE)?;

        let max_x_value = data.iter().map(|x| x.0).fold(f64::NAN, f64::max);
        let mut scatter_ctx = ChartBuilder::on(&root)
            .x_label_area_size(40_i32)
            .y_label_area_size(40_i32)
            .build_cartesian_2d(
                0_f64..max_x_value,
                0..data.iter().map(|x| x.1).max().unwrap(),
            )?;

        scatter_ctx
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()?;

        scatter_ctx.draw_series(
            data.iter()
                .map(|(x, y)| Circle::new((*x, *y), 2_i32, GREEN.filled())),
        )?;

        root.present().unwrap();

        Ok(())
    }
}
