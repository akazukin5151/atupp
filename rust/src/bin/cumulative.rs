use src::parse_csv_line;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};

fn main() {}

fn test(path: &str, max_distance: f64) {
    let points = Arc::new(Mutex::new(Vec::new()));

    let mut fd = File::open(path).unwrap();
    let mut buf = [0; 10_usize.pow(9)];
    while let Ok(n) = fd.read(&mut buf) {
        let block = String::from_utf8_lossy(&buf);
        let lines: Vec<_> = block.split('\n').collect();
        lines.into_par_iter().for_each(|line| {
            let xs = parse_csv_line(line);
            let pp_x = xs[2].to_string();
            let pp_y = xs[3].to_string();
            let distance: f64 = xs[4].parse().unwrap();
            if distance <= max_distance {
                let mut points = points.lock().unwrap();
                points.push((pp_x, pp_y));
            }
        });
        if n == 0 {
            break
        }
        buf.fill(0);
    }
}
