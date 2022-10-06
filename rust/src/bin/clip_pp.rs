use src::parse_csv_line;
use geo::Contains;
use geojson::GeoJson;
use rayon::prelude::*;
use std::fs::{self, File};
use std::io::{self, Read, Write};

pub fn main() {
    let mut args = std::env::args();

    let (boundaries, pp, flip_coords) = if args.nth(1).unwrap() == "london" {
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
    println!("{}", lines[0]);

    lines[1..].into_par_iter().for_each(|line| {
        process(line, &polygons, flip_coords, &mut io::stdout())
    });
}

fn load_polygons(path: &str) -> geo::GeometryCollection {
    let mut f = File::open(path).unwrap();
    let mut geojson_str = String::new();
    f.read_to_string(&mut geojson_str).unwrap();
    let geojson: GeoJson = geojson_str.parse().unwrap();
    let geometry: geo::geometry::Geometry<f64> = geojson.try_into().unwrap();
    geometry.try_into().unwrap()
}

fn process<W: Write>(
    line: &str,
    polygons: &geo::GeometryCollection,
    flip_coords: bool,
    mut stdout: W,
) {
    if line.is_empty() {
        return;
    }
    let xs = parse_csv_line(line);

    if xs.is_empty() {
        return;
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
        writeln!(stdout, "{}", line).unwrap();
    }
}

#[cfg(test)]
mod test {
    use std::str::from_utf8;

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

        let mut vec = vec![];
        process(line, &polygons, true, &mut vec);

        let stdout = from_utf8(&vec).unwrap();
        assert_eq!(stdout, format!("{}\n", line));
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

        let mut vec = vec![];
        process(line, &polygons, false, &mut vec);

        let stdout = from_utf8(&vec).unwrap();
        assert_eq!(stdout, format!("{}\n", line));
    }
}
