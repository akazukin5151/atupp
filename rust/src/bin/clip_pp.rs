// Usage: target/release/clip_pp [city]

use geo::Contains;
use geojson::GeoJson;
use rayon::prelude::*;
use src::parse_csv_line;
use std::fs::{self, File};
use std::io::Read;

pub fn main() {
    let args: Vec<_> = std::env::args().collect();

    let (boundaries, pp, flip_coords) = if args[1] == "london" {
        let boundaries = "../data/london boundaries/london.geojson";
        let pp = "../data/pp/population_gbr_2019-07-01.csv";
        let flip_coords = true;
        (boundaries, pp, flip_coords)
    } else {
        let boundaries = "../data/tokyo boundaries/clipped.geojson";
        let pp = "../data/pp/jpn_population_2020.csv";
        let flip_coords = false;
        (boundaries, pp, flip_coords)
    };

    let polygons = load_polygons(boundaries);

    let file = fs::read_to_string(pp).unwrap();

    let lines: Vec<_> = file.split('\n').collect();

    let result: Vec<_> = lines[1..]
        .into_par_iter()
        .filter_map(|line| process(line, &polygons, flip_coords))
        .collect();

    let joined = lines[0].to_string() + "\n" + &result.join("\n");
    let out_path = args[2].clone();
    fs::write(out_path, joined).unwrap();
}

fn load_polygons(path: &str) -> geo::GeometryCollection {
    let mut f = File::open(path).unwrap();
    let mut geojson_str = String::new();
    f.read_to_string(&mut geojson_str).unwrap();
    let geojson: GeoJson = geojson_str.parse().unwrap();
    let geometry: geo::geometry::Geometry<f64> = geojson.try_into().unwrap();
    geometry.try_into().unwrap()
}

fn process<'a>(
    line: &'a str,
    polygons: &geo::GeometryCollection,
    flip_coords: bool,
) -> Option<&'a str> {
    if line.is_empty() {
        return None;
    }
    let xs = parse_csv_line(line);

    if xs.is_empty() {
        return None;
    }

    // lat is y, lon is x
    let point = if flip_coords {
        let lon: f64 = xs[1].parse().unwrap();
        let lat: f64 = xs[0].parse().unwrap();
        geo::Point::new(lon, lat)
    } else {
        let lon: f64 = xs[0].parse().unwrap();
        let lat: f64 = xs[1].parse().unwrap();
        geo::Point::new(lon, lat)
    };

    if polygons.contains(&point) {
        Some(line)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_london_polygons() {
        let polygons =
            load_polygons("../data/london boundaries/london.geojson");
        let p = geo::Point::new(-0.1270, 51.4475);
        assert!(polygons.contains(&p));
    }

    #[test]
    fn test_london_process() {
        let polygons =
            load_polygons("../data/london boundaries/london.geojson");

        let line =
            r#""51.5781944444857","-0.24125000000019298","9.821008556019821""#;

        let out = process(line, &polygons, true).unwrap();

        assert_eq!(out, line);
    }

    #[test]
    fn test_tokyo_polygons() {
        let polygons =
            load_polygons("../data/tokyo boundaries/clipped.geojson");
        let p = geo::Point::new(139.689, 35.682);
        assert!(polygons.contains(&p));
    }

    #[test]
    fn test_tokyo_process() {
        let polygons =
            load_polygons("../data/tokyo boundaries/clipped.geojson");

        let line =
            r#""139.80944444445794","35.66361111110322","17.34286880493164""#;

        let out = process(line, &polygons, false).unwrap();

        assert_eq!(out, line);
    }
}
