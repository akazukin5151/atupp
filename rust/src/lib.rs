use std::fs;
use rstar::RTree;

pub trait Search<T> {
    fn search_to_file(
        &self,
        tree: &RTree<(f64, f64)>,
        pp_lines: &[&str],
    );
    fn n_stations_within_dist(
        tree: &RTree<(f64, f64)>,
        pp_lines: &[&str],
        max_distance: f64,
    ) -> T;
}

pub trait Plot<T> {
    fn search_to_plot(tree: &RTree<(f64, f64)>, pp_lines: &[&str]);
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
