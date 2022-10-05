use rayon::prelude::*;
use src::parse_csv_line;
use std::fs::File;
use std::io::Read;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};

fn main() {
    let mut args = std::env::args();

    let (pp_path, matrix_path) = if args.nth(1).unwrap() == "london" {
        let pp_path = "../data/london_pp.csv";
        let matrix_path = "../data/london_matrix.csv";
        (pp_path, matrix_path)
    } else {
        let pp_path = "../data/tokyo_pp.csv";
        let matrix_path = "../data/tokyo_matrix.csv";
        (pp_path, matrix_path)
    };
    let city_pop = total_city_pop(pp_path);
    let pop_within_500 = pop_within_dist(matrix_path, 500.0);
    println!("{}", pop_within_500 / city_pop);
}

fn pop_within_dist(matrix_path: &str, max_distance: f64) -> f64 {
    let pop_within_dist = Arc::new(Mutex::new(0.0));

    let mut fd = File::open(matrix_path).unwrap();
    let mut buf = [0; 10_usize.pow(9)];
    while let Ok(n) = fd.read(&mut buf) {
        let block = String::from_utf8_lossy(&buf);
        let lines: Vec<_> = block.split('\n').collect();
        lines.into_par_iter().for_each(|line| {
            let xs = parse_csv_line(line);
            // pp_x, pp_y, pop, dist
            let pop: f64 = xs[2].parse().unwrap();
            let distance: f64 = xs[4].parse().unwrap();
            if distance <= max_distance {
                *pop_within_dist.lock().unwrap() += pop;
            }
        });
        if n == 0 {
            break;
        }
        buf.fill(0);
    }
    Arc::try_unwrap(pop_within_dist)
        .unwrap()
        .into_inner()
        .unwrap()
}

fn total_city_pop(pp_path: &str) -> f64 {
    let mut fd = File::open(pp_path).unwrap();
    let mut file = Vec::new();
    fd.read_to_end(&mut file).unwrap();
    let file = from_utf8(&file).unwrap();
    let mut sum = 0.0;
    for line in file.split('\n').skip(1).filter(|line| !line.is_empty()) {
        let xs = parse_csv_line(line);
        let pop: f64 = xs[2].parse().unwrap();
        sum += pop;
    }
    sum
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_london_pop() {
        let pp_path = "../data/london_pp.csv";
        assert_eq!(total_city_pop(pp_path).floor(), 9_079_712.0);
    }

    #[test]
    fn test_tokyo_pop() {
        let pp_path = "../data/tokyo_pp.csv";
        assert_eq!(total_city_pop(pp_path).floor(), 43_574_358.0);
    }
}
