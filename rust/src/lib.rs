use std::fs;

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
