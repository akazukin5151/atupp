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
        let stations = "../data/london_trains/stations/station_coords_meters.csv";
        (pp, stations)
    } else {
        let pp = "../data/tokyo_pp_meters.csv";
        let stations = "../data/tokyo_trains/coords_meters.csv";
        (pp, stations)
    };

    let stations = load_stations(stations);
    calculate_matrix(pp, stations, io::stdout());
}

fn calculate_matrix<W: Write + Send>(
    pp_path: &str,
    stations: Vec<geo::Point>,
    mut stdout: W,
) {
    writeln!(stdout, "pp_x,pp_y,pop,dist").unwrap();
    let stdout = Arc::new(Mutex::new(stdout));

    let mut fd = File::open(pp_path).unwrap();
    let mut file = String::new();
    fd.read_to_string(&mut file).unwrap();
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

//#[cfg(test)]
//mod test {
//    use super::*;
//    use geo::point;
//    use std::str::from_utf8;
//
//    #[test]
//    fn test_counts_equal() {
//        let pps = vec![point!(x: 0.0, y: 0.0)];
//        let stations = vec![point!(x: 0.0, y: 0.0)];
//        let mut output = vec![];
//
//        count_pps_within_500m_of_stations(pps, stations, &mut output);
//
//        let s = from_utf8(&output).unwrap();
//        assert_eq!(s, "0,0\n");
//    }
//
//    #[test]
//    fn test_counts_at_100() {
//        let pps = vec![point!(x: 100.0, y: 100.0)];
//        let stations = vec![point!(x: 0.0, y: 0.0)];
//        let mut output = vec![];
//
//        count_pps_within_500m_of_stations(pps, stations, &mut output);
//
//        let s = from_utf8(&output).unwrap();
//        assert_eq!(s, "0,0\n");
//    }
//
//    #[test]
//    fn test_counts_at_500() {
//        // 500m away from unit
//        // c = 500, assume a = b
//        // 500^2 = 2a^2
//        // a = sqrt(500^2/2)
//        let side_length = (500.0_f64.powi(2) / 2.0).powf(0.5);
//        let pps = vec![point!(x: side_length, y: side_length)];
//        let stations = vec![point!(x: 0.0, y: 0.0)];
//        let mut output = vec![];
//
//        assert_eq!(pps[0].euclidean_distance(&stations[0]), 500.0);
//
//        count_pps_within_500m_of_stations(pps, stations, &mut output);
//
//        let s = from_utf8(&output).unwrap();
//        assert_eq!(s, "0,0\n");
//    }
//
//    #[test]
//    fn test_counts_beyond_500() {
//        let pps = vec![point!(x: 1000.0, y: 1000.0)];
//        let stations = vec![point!(x: 0.0, y: 0.0)];
//        let mut output = vec![];
//
//        count_pps_within_500m_of_stations(pps, stations, &mut output);
//
//        let s = from_utf8(&output).unwrap();
//        assert_eq!(s, "");
//    }
//
//    #[test]
//    fn test_counts_combined_1_station() {
//        let side_length = (500.0_f64.powi(2) / 2.0).powf(0.5);
//        let pps = vec![
//            point!(x: 0.0, y: 0.0),
//            point!(x: 100.0, y: 100.0),
//            point!(x: side_length, y: side_length),
//            point!(x: 1000.0, y: 1000.0),
//        ];
//        let stations = vec![point!(x: 0.0, y: 0.0)];
//        let mut output = vec![];
//
//        count_pps_within_500m_of_stations(pps, stations, &mut output);
//
//        let s = from_utf8(&output).unwrap();
//        assert_eq!(s, "0,0\n0,0\n0,0\n");
//    }
//
//    #[test]
//    fn test_counts_combined_multiple_stations() {
//        let side_length = (500.0_f64.powi(2) / 2.0).powf(0.5);
//        let pps = vec![
//            point!(x: 0.0, y: 0.0),
//            point!(x: 100.0, y: 100.0),
//            point!(x: side_length, y: side_length),
//            point!(x: 1000.0, y: 1000.0),
//        ];
//        let stations = vec![
//            point!(x: 0.0, y: 0.0),
//            point!(x: 10.0, y: 20.0),
//            point!(x: 1100.0, y: 1100.0),
//            point!(x: 10000.0, y: 10000.0),
//        ];
//        let mut output = vec![];
//
//        count_pps_within_500m_of_stations(pps, stations, &mut output);
//
//        let s = from_utf8(&output).unwrap();
//        let mut stations_within_500m: Vec<_> = s.split('\n').collect();
//        stations_within_500m.sort_unstable();
//        assert_eq!(
//            stations_within_500m,
//            vec![
//                "",
//                "0,0",
//                "0,0",
//                "0,0",
//                "10,20",
//                "10,20",
//                "10,20",
//                "1100,1100"
//            ]
//        );
//    }
//}
