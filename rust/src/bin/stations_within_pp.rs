// Usage: target/release/stations_within_pp

use plotters::{prelude::*, style::full_palette::GREY};
use rayon::prelude::*;
use rstar::RTree;
use src::{load_stations, parse_csv_line, Plot, Search};
use std::fs;

fn main() {
    for city in ["london", "tokyo"] {
        let pp_path = format!("../data/{}_pp_meters.csv", city);

        // TODO: fix this inconsistency...
        let stations_path = if city == "london" {
            "../data/london_trains/stations/station_coords_meters.csv"
        } else {
            "../data/tokyo_trains/coords_meters.csv"
        };

        inner_main(&pp_path, stations_path, format!("../out/{}_box.png", city));
    }
}

fn inner_main(pp_path: &str, stations_path: &str, out_filename: String) {
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

    let s = StationWithinPP { out_filename };
    s.search_to_plot(&tree, &pp_lines);
}

struct StationWithinPP {
    out_filename: String,
}

impl Search<Vec<i32>> for StationWithinPP {
    fn search_to_file(&self, tree: &RTree<(f64, f64)>, pp_lines: &[&str]) {
        // this isn't actually used, but here for debugging i suppose
        eprintln!("searching...");

        let dists: Vec<_> = (100..=3000).step_by(100).collect();
        let result: Vec<_> = dists
            .into_par_iter()
            .flat_map(|max_dist| {
                let n_stations = self.search(tree, pp_lines, max_dist as f64);
                let v: Vec<_> = n_stations
                    .iter()
                    .map(|num| format!("{},{}", max_dist, num))
                    .collect();
                v
            })
            .collect();

        println!("max_dist,n_stations");
        for r in result {
            println!("{}", r);
        }
    }

    fn search(
        &self,
        tree: &RTree<(f64, f64)>,
        pp_lines: &[&str],
        max_distance: f64,
    ) -> Vec<i32> {
        let max_distance_squared = max_distance * max_distance;

        let pop_within_dist: Vec<_> = pp_lines
            .into_par_iter()
            .map(|pp_line| {
                let xs = parse_csv_line(pp_line);

                // a line in pp looks like this
                // lat/lon, lat/lon, pop, x, y
                let x: f64 = xs[3].parse().unwrap();
                let y: f64 = xs[4].parse().unwrap();

                tree.locate_within_distance((x, y), max_distance_squared)
                    .count() as i32
            })
            .collect();

        pop_within_dist
    }
}

// the result of the search function is Vec<i32> (U),
// but actual data collected is that result plus the distance threshold,
// over multiple distances, which is T
impl Plot<Vec<(i32, Vec<i32>, Vec<i32>)>, Vec<i32>> for StationWithinPP {
    fn search_to_plot(&self, tree: &RTree<(f64, f64)>, pp_lines: &[&str]) {
        eprintln!("searching...");
        let dists: Vec<_> = (100..=3000).step_by(100).collect();
        let data: Vec<_> = dists
            .into_par_iter()
            .map(|max_dist| {
                let n_stations = self.search(tree, pp_lines, max_dist as f64);
                let quartiles = Quartiles::new(&n_stations);
                let lower = quartiles.values()[0];
                let upper = quartiles.values()[4];
                let outliers: Vec<_> = n_stations
                    .iter()
                    .filter(|x| {
                        let x = **x as f32;
                        x < lower || x > upper
                    })
                    .cloned()
                    .collect();
                (max_dist, n_stations, outliers)
            })
            .collect();
        self.plot(data).unwrap();
    }

    fn plot(
        &self,
        data: Vec<(i32, Vec<i32>, Vec<i32>)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(&self.out_filename, (1500, 768))
            .into_drawing_area();

        root.fill(&WHITE)?;

        let max_y_value: f32 = *data
            .iter()
            .map(|(_, n_stations, _)| n_stations.iter().max().unwrap_or(&0))
            .max()
            .unwrap_or(&0) as f32;

        let max_x_value =
            *data.iter().map(|(dist, _, _)| dist).max().unwrap_or(&0);

        let mut scatter_ctx = ChartBuilder::on(&root)
            .margin(20_i32)
            .x_label_area_size(40_i32)
            .y_label_area_size(40_i32)
            .build_cartesian_2d(0..max_x_value, 0.0..max_y_value)?;

        scatter_ctx
            .configure_mesh()
            .y_desc("Number of stations within distance threshold")
            .x_desc("Distance threshold")
            .x_labels((max_x_value / 100) as usize)
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()?;

        scatter_ctx.draw_series(data.iter().map(
            |(distance, n_stations, _)| {
                let n_stations_quartiles = Quartiles::new(n_stations);
                Boxplot::new_vertical(*distance, &n_stations_quartiles)
                    .width(20)
            },
        ))?;

        scatter_ctx.draw_series(data.iter().flat_map(
            |(max_dist, _, outliers)| {
                outliers.iter().map(|y_value| {
                    Circle::new(
                        (*max_dist, *y_value as f32),
                        2_i32,
                        GREY.filled(),
                    )
                })
            },
        ))?;

        root.present().unwrap();

        Ok(())
    }
}
