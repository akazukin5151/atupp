use plotters::coord::types::{RangedCoordf64, RangedCoordi32};
use plotters::coord::Shift;
use plotters::prelude::*;
use rstar::RTree;
use std::fs;

/// Describes a visualization that searches the R* tree and save the result as csv
/// The result can be used to plot with python
/// The generic type T is anything the search function returns
pub trait Search<T> {
    /// Search the tree and output it to a file (actually stdout)
    /// The python script can read the result and plot it
    fn search_to_file(&self, tree: &RTree<(f64, f64)>, pp_lines: &[&str]);

    /// The function that searches the R* tree.
    /// The stations are stored in the tree. For every population point
    /// in pp_lines, the function searches for the nearest neighbours within
    /// max_distance. It returns anything the visualization needs, such as...
    ///
    /// cumulative_props.rs:
    /// - the population of the point if a station within max_distance is found
    ///
    /// stations_within_pp.rs:
    /// - the number of stations within max_distance of any population point
    ///
    /// quadrants.rs:
    /// - the population of every point and the number of stations within
    ///   max_distance of it
    fn search(
        tree: &RTree<(f64, f64)>,
        pp_lines: &[&str],
        max_distance: f64,
    ) -> T;
}

/// Describes a visualization that searches the R* tree and plots the result in rust
/// It requires the visualization to implement Search, as it relies on the search
/// function. The result of the search function can be anything (U), as long
/// as it can be transformed into T
pub trait Plot<T, U>: Search<U> {
    /// Search the tree and immediately plot the results with rust.
    /// Use when python cannot handle the amount of data
    fn search_to_plot(tree: &RTree<(f64, f64)>, pp_lines: &[&str]);

    /// The function that does the plotting
    fn plot(data: T) -> Result<(), Box<dyn std::error::Error>>;
}

// TODO: use serde
pub fn parse_csv_line(line: &str) -> Vec<&str> {
    line.split(',')
        .map(|x| {
            let a = x.strip_prefix('"').unwrap_or(x);
            let b = a
                .strip_suffix('"')
                .unwrap_or_else(|| a.strip_suffix("\"\n").unwrap_or(a));
            b
        })
        .collect()
}

pub fn load_stations(path: &str) -> Vec<(f64, f64)> {
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

pub type Chart<'a, 'b> = ChartContext<
    'a,
    BitMapBackend<'b>,
    Cartesian2d<RangedCoordf64, RangedCoordi32>,
>;

pub fn plot_vline(
    root: &DrawingArea<BitMapBackend, Shift>,
    chart: &Chart,
    x_value: f64,
    modifier: i32,
    top_y: i32,
    stroke: ShapeStyle,
) -> Result<(), Box<dyn std::error::Error>> {
    let drawing_area = chart.plotting_area();
    let mapped = drawing_area.map_coordinate(&(x_value, 0));
    let p: PathElement<(i32, i32)> = PathElement::new(
        [(mapped.0, mapped.1 - modifier), (mapped.0, top_y)],
        stroke,
    );
    root.draw(&p)?;
    Ok(())
}

pub fn plot_hline(
    root: &DrawingArea<BitMapBackend, Shift>,
    chart: &Chart,
    y_value: i32,
    modifier: i32,
    left_x: f64,
    stroke: ShapeStyle,
) -> Result<(), Box<dyn std::error::Error>> {
    let drawing_area = chart.plotting_area();
    let mapped = drawing_area.map_coordinate(&(0., y_value));
    let end = drawing_area.map_coordinate(&(left_x, y_value));
    let p: PathElement<(i32, i32)> = PathElement::new(
        [(mapped.0, mapped.1 - modifier), (end.0, end.1)],
        stroke,
    );
    root.draw(&p)?;
    Ok(())
}
