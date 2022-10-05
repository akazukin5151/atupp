use geo::EuclideanDistance;
use rayon::prelude::*;
use src::parse_csv_line;
use std::fs::File;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};

pub fn main() {
    let mut args = std::env::args();

    let (pp, stations) = if args.nth(1).unwrap() == "london" {
        let pp = "../data/london_pp_meters.csv";
        let stations =
            "../data/london_trains/stations/station_coords_meters.csv";
        (pp, stations)
    } else {
        let pp = "../data/tokyo_pp_meters.csv";
        let stations = "../data/tokyo_trains/coords_meters.csv";
        (pp, stations)
    };

    let stations = load_stations(stations);
    let pp_reader = File::open(pp).unwrap();
    calculate_matrix(pp_reader, stations, io::stdout());
}

fn calculate_matrix<R: Read, W: Write + Send>(
    mut pp_reader: R,
    stations: Vec<geo::Point>,
    mut stdout: W,
) {
    writeln!(stdout, "pp_x,pp_y,pop,dist").unwrap();
    let stdout = Arc::new(Mutex::new(stdout));

    let mut file = String::new();
    pp_reader.read_to_string(&mut file).unwrap();
    let pp_lines = file.split('\n');
    let pp_lines: Vec<_> =
        pp_lines.skip(1).filter(|line| !line.is_empty()).collect();

    pp_lines
        .into_par_iter()
        .map(|pp_line| (pp_line, stations.clone()))
        .for_each(|(pp_line, stations)| {
            let xs = parse_csv_line(pp_line);

            // a line in pp looks like this
            // lat/lon, lat/lon, pop, x, y
            let pop = xs[2];
            let x: f64 = xs[3].parse().unwrap();
            let y: f64 = xs[4].parse().unwrap();
            let point = geo::Point::new(x, y);

            stations.into_par_iter().for_each(|station| {
                let mut s = stdout.lock().unwrap();
                writeln!(
                    s,
                    "{},{},{},{}",
                    point.x(),
                    point.y(),
                    pop,
                    station.euclidean_distance(&point),
                )
                .unwrap();
            })
        });
}

fn load_stations(path: &str) -> Vec<geo::Point> {
    let mut fd = File::open(path).unwrap();
    let mut file = String::new();
    fd.read_to_string(&mut file).unwrap();
    let lines = file.split('\n');
    lines
        .skip(1)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let xs = parse_csv_line(line);

            // both london and tokyo is (name, lat, lon, x, y)
            let x: f64 = xs[3].parse().unwrap();
            let y: f64 = xs[4].parse().unwrap();
            geo::Point::new(x, y)
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use geo::point;
    use std::str::from_utf8;

    #[test]
    fn test_counts_equal() {
        let pps = "\n7,9,25,0.0,0.0\n".as_bytes();
        let stations = vec![point!(x: 0.0, y: 0.0)];
        let mut output = vec![];

        calculate_matrix(pps, stations, &mut output);

        let s = from_utf8(&output).unwrap();
        assert_eq!(s, "pp_x,pp_y,pop,dist\n0,0,25,0\n");
    }

    #[test]
    fn test_counts_at_100() {
        let pps = "\n7,9,25,100.0,100.0\n".as_bytes();
        let stations = vec![point!(x: 0.0, y: 0.0)];
        let mut output = vec![];

        calculate_matrix(pps, stations, &mut output);

        let s = from_utf8(&output).unwrap();
        assert_eq!(s, "pp_x,pp_y,pop,dist\n100,100,25,141.4213562373095\n");
    }

    #[test]
    fn test_counts_at_500() {
        // 500m away from unit
        // c = 500, assume a = b
        // 500^2 = 2a^2
        // a = sqrt(500^2/2)
        let side_length = (500.0_f64.powi(2) / 2.0).powf(0.5);
        let pps = format!("\n7,9,25,{},{}\n", side_length, side_length);
        let pps = pps.as_bytes();
        let stations = vec![point!(x: 0.0, y: 0.0)];
        let mut output = vec![];

        calculate_matrix(pps, stations, &mut output);

        let s = from_utf8(&output).unwrap();
        assert_eq!(
            s,
            format!(
                "pp_x,pp_y,pop,dist\n{},{},25,500\n",
                side_length, side_length
            )
        );
    }

    #[test]
    fn test_counts_beyond_500() {
        let pps = "\n7,9,25,1000.0,1000.0\n".as_bytes();
        let stations = vec![point!(x: 0.0, y: 0.0)];
        let mut output = vec![];

        calculate_matrix(pps, stations, &mut output);

        let s = from_utf8(&output).unwrap();
        assert_eq!(s, "pp_x,pp_y,pop,dist\n1000,1000,25,1414.213562373095\n");
    }

    #[test]
    fn test_counts_combined_1_station() {
        let side_length = (500.0_f64.powi(2) / 2.0).powf(0.5);
        let pps = format!("\n7,9,25,0.0,0.0\n7,9,25,100.0,100.0\n7,9,25,{},{}\n7,9,25,1000.0,1000.0\n", side_length, side_length);
        let pps = pps.as_bytes();
        let stations = vec![point!(x: 0.0, y: 0.0)];
        let mut output = vec![];

        calculate_matrix(pps, stations, &mut output);

        let s = from_utf8(&output).unwrap();
        let mut result: Vec<_> = s.split('\n').collect();
        result.sort_unstable();
        assert_eq!(
            result,
            vec![
                "",
                "0,0,25,0",
                "100,100,25,141.4213562373095",
                "1000,1000,25,1414.213562373095",
                &format!("{},{},25,500", side_length, side_length),
                "pp_x,pp_y,pop,dist"
            ]
        );
    }

    #[test]
    fn test_counts_combined_multiple_stations() {
        let side_length = (500.0_f64.powi(2) / 2.0).powf(0.5);
        let pps = format!("\n7,9,25,0.0,0.0\n7,9,25,100.0,100.0\n7,9,25,{},{}\n7,9,25,1000.0,1000.0\n", side_length, side_length);
        let pps = pps.as_bytes();
        let stations = vec![
            point!(x: 0.0, y: 0.0),
            point!(x: 10.0, y: 20.0),
            point!(x: 1100.0, y: 1100.0),
            point!(x: 10000.0, y: 10000.0),
        ];
        let mut output = vec![];

        calculate_matrix(pps, stations, &mut output);

        let s = from_utf8(&output).unwrap();
        let mut result: Vec<_> = s.split('\n').collect();
        result.sort_unstable();
        assert_eq!(
            result,
            vec![
                "",
                "0,0,25,0",
                "0,0,25,14142.13562373095",
                "0,0,25,1555.6349186104046",
                "0,0,25,22.360679774997898",
                "100,100,25,120.41594578792295",
                "100,100,25,14000.71426749364",
                "100,100,25,141.4213562373095",
                "100,100,25,1414.213562373095",
                "1000,1000,25,12727.922061357855",
                "1000,1000,25,1393.0183056945089",
                "1000,1000,25,141.4213562373095",
                "1000,1000,25,1414.213562373095",
                &format!(
                    "{},{},25,1055.6349186104046",
                    side_length, side_length
                ),
                &format!(
                    "{},{},25,13642.13562373095",
                    side_length, side_length
                ),
                &format!(
                    "{},{},25,478.83900902537545",
                    side_length, side_length
                ),
                &format!("{},{},25,500", side_length, side_length),
                "pp_x,pp_y,pop,dist"
            ]
        );
    }
}
