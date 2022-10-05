use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use src::parse_csv_line;
use std::fs::{self, File};
use std::io::Read;
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

    let bar = ProgressBar::new(fd.metadata().unwrap().len());
    bar.set_style(ProgressStyle::with_template(
            "[{elapsed}] {bytes_per_sec:15} {wide_bar} {percent} ({eta:15})"
        )
        .unwrap()
    );

    // 5 seems to be the maxima, beyond that the speed does not increase anymore
    let mut buf = [0; 10_usize.pow(5)];

    let mut current_block = String::new();
    let mut chars_for_next = String::new();

    while let Ok(n) = fd.read(&mut buf) {
        let block = String::from_utf8_lossy(&buf);
        if !chars_for_next.is_empty() {
            current_block.push_str(&chars_for_next);
            chars_for_next.clear();
        }
        if block.ends_with('\n') {
            current_block.push_str(&block);
        } else {
            // remove all chars from the back until we find a newline
            // store the removed chars in chars_for_next
            // in the next iteration, join the removed chars and the next block
            // in that order
            let (complete_block, for_next) = block.rsplit_once('\n').unwrap();
            current_block.push_str(complete_block);
            current_block.push('\n');
            chars_for_next = for_next.to_string();
        }
        let lines: Vec<_> = current_block.split('\n').collect();
        lines[..]
            .into_par_iter()
            .filter(|line| !line.is_empty())
            .for_each(|line| {
                bar.inc(1);
                if *line == "pp_x,pp_y,pop,dist" {
                    return;
                }
                let xs = parse_csv_line(line);
                // pp_x, pp_y, pop, dist
                let pop: f64 = xs[2].parse().unwrap();
                let r = xs[3].parse();
                let distance: f64 = r.unwrap();
                if distance <= max_distance {
                    *pop_within_dist.lock().unwrap() += pop;
                    //dbg!(&pop_within_dist);
                }
            });
        if n == 0 {
            break;
        }
        buf.fill(0);
    }

    bar.finish();

    Arc::try_unwrap(pop_within_dist)
        .unwrap()
        .into_inner()
        .unwrap()
}

fn total_city_pop(pp_path: &str) -> f64 {
    let file = fs::read_to_string(pp_path).unwrap();
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
