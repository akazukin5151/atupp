use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use src::parse_csv_line;
use std::fs::File;
use std::io::Read;

fn main() {
    let mut args = std::env::args();

    let matrix_path = if args.nth(1).unwrap() == "london" {
        "../data/london_matrix.csv"
    } else {
        "../data/tokyo_matrix.csv"
    };
    check_matrix_csv(matrix_path)
}

fn check_matrix_csv(matrix_path: &str) {
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
        let x = lines[..]
            .into_par_iter()
            .filter(|line| !line.is_empty())
            .find_first(|line| {
                bar.inc(1);
                if **line == "pp_x,pp_y,pop,dist" {
                    return false;
                }
                let xs = parse_csv_line(line);
                if xs.len() != 4 {
                    dbg!(&line);
                    return true;
                }
                // pp_x, pp_y, pop, dist
                let r: Result<f64, _> = xs[3].parse();
                if r.is_err() {
                    dbg!(&line);
                    return true;
                }
                false
            });
        if x.is_some() {
            return;
        }
        if n == 0 {
            break;
        }
        buf.fill(0);
    }

    bar.finish();
}
