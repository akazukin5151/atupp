use plotters::prelude::*;
use rayon::prelude::*;
use rstar::RTree;
use src::parse_csv_line;
use std::{
    fs,
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

    let mut result = vec![];
    let n_stations = n_stations_within_dist(&tree, &pp_lines, 500.);
    result.extend(n_stations);

    plot(result).unwrap();
}

fn plot(data: Vec<(f64, i32)>) -> Result<(), Box<dyn std::error::Error>> {
    let root =
        BitMapBackend::new("../out/rust_q.png", (1024, 768)).into_drawing_area();

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

fn n_stations_within_dist(
    tree: &RTree<(f64, f64)>,
    pp_lines: &Vec<&str>,
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