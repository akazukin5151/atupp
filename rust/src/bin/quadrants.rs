// Usage: target/release/quadrant [X meters]

use plotters::style::full_palette::GREY;
use plotters::{prelude::*, style::full_palette::ORANGE};
use rayon::prelude::*;
use rstar::RTree;
use src::{
    load_stations, parse_csv_line, plot_hline, plot_vline, Plot, Search,
};
use std::cmp::Ordering;
use std::{
    fs, io,
    sync::{Arc, Mutex},
};

fn main() {
    let mut args = std::env::args();
    let distance_threshold = args.nth(1).unwrap().parse().unwrap();

    for city in ["london", "tokyo"] {
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
            format!("../out/{}_quadrant.png", city),
            distance_threshold,
        );
    }
}

fn inner_main(
    pp_path: &str,
    stations_path: &str,
    out_filename: String,
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

    let q = Quadrants {
        out_filename,
        distance_threshold,
    };
    q.search_to_plot(&tree, &pp_lines);
}

struct Quadrants {
    out_filename: String,
    distance_threshold: f64,
}

impl Search<Vec<(f64, i32)>> for Quadrants {
    fn search_to_file(&self, tree: &RTree<(f64, f64)>, pp_lines: &[&str]) {
        eprintln!("searching...");
        let mut wtr = csv::Writer::from_writer(io::stdout());
        wtr.write_record(&["population", "n_stations"]).unwrap();
        let wtr = Arc::new(Mutex::new(wtr));

        let n_stations = self.search(tree, pp_lines, self.distance_threshold);
        let mut w = wtr.lock().unwrap();
        for (pop, n_stations) in n_stations {
            w.write_record(&[format!("{}", pop), format!("{}", n_stations)])
                .unwrap();
        }
    }

    fn search(
        &self,
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

impl Plot<Vec<(f64, i32)>, Vec<(f64, i32)>> for Quadrants {
    fn search_to_plot(&self, tree: &RTree<(f64, f64)>, pp_lines: &[&str]) {
        eprintln!("searching...");
        let result = self.search(tree, pp_lines, self.distance_threshold);
        self.plot(result).unwrap();
    }

    fn plot(
        &self,
        data: Vec<(f64, i32)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let populations: Vec<_> = data.iter().map(|x| x.0).collect();
        let pop_q3 = Quartiles::new(&populations).values()[3];
        let n_stations: Vec<_> = data.iter().map(|x| x.1).collect();
        let n_stations_q3 = Quartiles::new(&n_stations).values()[3];

        let root = BitMapBackend::new(&self.out_filename, (1024, 768))
            .into_drawing_area();

        root.fill(&WHITE)?;
        let roots = root.split_by_breakpoints([10_i32], [668_i32]);
        let left_box_area = &roots[0];
        let scatterplot_area = &roots[1];
        let bottom_box_area = &roots[3];
        //let (upper, bottom_box_area) = root.split_vertically(668);
        //let (left_box_area, scatterplot_area) = upper.split_horizontally(10);

        let max_x_value = data.iter().map(|x| x.0).fold(f64::NAN, f64::max);
        let max_y_value = data.iter().map(|x| x.1).max().unwrap();
        let mut scatter_ctx = ChartBuilder::on(scatterplot_area)
            .margin(20_i32)
            .x_label_area_size(40_i32)
            .y_label_area_size(40_i32)
            .build_cartesian_2d(
                0_f64..max_x_value,
                0..max_y_value,
            )?;

        scatter_ctx
            .configure_mesh()
            .y_desc(format!(
                "Number of stations within {} m of a population point",
                self.distance_threshold
            ))
            .axis_desc_style(("sans-serif", 20_i32).into_text_style(scatterplot_area))
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()?;

        scatter_ctx.draw_series(data.iter().map(|(x, y)| {
            let pq3 = &(pop_q3 as f64);
            let y_ = *y as f32;
            let color = match (
                x.partial_cmp(pq3).unwrap(),
                y_.partial_cmp(&n_stations_q3).unwrap(),
            ) {
                (Ordering::Less, Ordering::Greater) => RED.filled(),
                (Ordering::Equal, Ordering::Greater) => RED.filled(),
                (Ordering::Greater, Ordering::Less) => ORANGE.filled(),
                (Ordering::Greater, Ordering::Equal) => ORANGE.filled(),
                (Ordering::Less, Ordering::Less) => GREEN.filled(),
                (Ordering::Equal, Ordering::Less) => GREEN.filled(),
                (Ordering::Less, Ordering::Equal) => GREEN.filled(),
                (Ordering::Greater, Ordering::Greater) => BLUE.filled(),
                (Ordering::Equal, Ordering::Equal) => BLUE.filled(),
            };
            Circle::new((*x, *y), 2_i32, color)
        }))?;

        plot_vline(
            scatterplot_area,
            &scatter_ctx,
            pop_q3.into(),
            0,
            0,
            BLUE.stroke_width(1),
        )
        .unwrap();

        plot_hline(
            scatterplot_area,
            &scatter_ctx,
            n_stations_q3 as i32,
            0,
            max_x_value,
            BLUE.filled(),
        )
        .unwrap();

        let mut chart = ChartBuilder::on(bottom_box_area)
            .margin(20_i32)
            .margin_top(0)
            .x_label_area_size(40_i32)
            .y_label_area_size(40_i32)
            .build_cartesian_2d(0_f32..(max_x_value as f32), 0..2_i32)?;

        chart
            .configure_mesh()
            .x_desc("Population of point")
            .axis_desc_style(("sans-serif", 20_i32).into_text_style(bottom_box_area))
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()?;

        let pops: Vec<_> = data.iter().map(|x| x.0).collect();
        let pop_quartiles = Quartiles::new(&pops);
        let boxplot = Boxplot::new_horizontal(1, &pop_quartiles).width(20);
        chart.draw_series([boxplot])?;

        // TODO: copied from stations_within_pp.rs
        let lower = pop_quartiles.values()[0];
        let upper = pop_quartiles.values()[4];
        let outliers = pops.iter().filter(|x| {
            let x = **x as f32;
            x < lower || x > upper
        });
        chart
            .draw_series(outliers.map(|x| {
                Circle::new((*x as f32, 1_i32), 2_i32, GREY.filled())
            }))?;

        let mut chart = ChartBuilder::on(left_box_area)
            .margin(20_i32)
            .margin_right(0_i32)
            .x_label_area_size(40_i32)
            .y_label_area_size(40_i32)
            .build_cartesian_2d(0..2_i32, 0_f32..(max_y_value as f32))?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()?;

        let n_stations: Vec<_> = data.iter().map(|x| x.1).collect();
        let n_stations_quartiles = Quartiles::new(&n_stations);
        let boxplot = Boxplot::new_vertical(1, &n_stations_quartiles).width(20);
        chart.draw_series([boxplot])?;

        // TODO: copied from stations_within_pp.rs
        let lower = n_stations_quartiles.values()[0];
        let upper = n_stations_quartiles.values()[4];
        let outliers = n_stations.iter().filter(|x| {
            let x = **x as f32;
            x < lower || x > upper
        });
        chart
            .draw_series(outliers.map(|x| {
                Circle::new((1_i32, *x as f32), 2_i32, GREY.filled())
            }))?;

        root.present().unwrap();

        Ok(())
    }
}
