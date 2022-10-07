# Data sources

## data/pp/jpn_population_2020.csv
https://data.humdata.org/dataset/japan-high-resolution-population-density-maps-demographic-estimates

CRS: WGS84, EPSG:4326

## data/pp/population_gbr_2019-07-01.csv
https://data.humdata.org/dataset/united-kingdom-high-resolution-population-density-maps-demographic-estimates

CRS: WGS84, EPSG:4326

## data/tokyo boundaries/gadm
https://gadm.org/download_country_v3.html

CRS: WGS84, EPSG:4326

## data/london boundaries/statistical-gis-boundaries-london
https://data.london.gov.uk/dataset/statistical-gis-boundary-files-london

**CRS: British National Grid, EPSG:27700**

## London station coordinates

[TfL public API](https://api.tfl.gov.uk)

## Tokyo station coordinates

[https://www.odpt.org/](https://www.odpt.org/)

# Download the data

## Fetch London lines data

```sh
cd data/london_trains/lines/
curl "https://api.tfl.gov.uk/Line/Mode/tube/Route" > tube_lines.json
```

## Get London stations

```sh
cd data/london_trains/lines/
jq '.[].id' tube_lines.json > line_ids.txt
```

This command selects all `id` values in every train line. The output is every train line separated by newline. You can avoid installing `jq` by writing a python script or similar. 

## Fetch London station coordinates

```sh
python python/london stations/get_stations.py
```

This command queries the TfL API. For every train line found in the above step, it queries every station on that line.

## Combine London station coordinates

```sh
python python/london stations/station_coords.py
```

## Fetch Tokyo Lines data

The Tokyo data is from [https://www.odpt.org/](https://www.odpt.org/). Go to [https://developer-dc.odpt.org/en/info](https://developer-dc.odpt.org/en/info) and create an account. Get your consumer key and export it as an environment variable, then make a query for stations.

```sh
export CONSUMERKEY="your consumer key"
curl -X GET https://api.odpt.org/api/v4/odpt:Station?acl:consumerKey=$CONSUMERKEY
```

## TODO: fix up Tokyo lines data

# Data preprocessing

## Convert London boundaries to WGS84, EPSG:4326

Use QGIS, export it as GeoJSON

## Extract Tokyo boundaries from the Japan boundaries

In QGIS, select the prefectures Tokyo, Saitama, Chiba, Kanagawa, Tochigi, Ibaraki, Gunma. Export it as GeoJSON

This is a pretty wide definition of "Greater Tokyo", but the Japanese government uses "Capital ring" (首都圏), which includes even Yamanashi.

## Clip the national population points to its cities

```sh
cd rust
cargo b --release --bin clip_pp
target/release/clip_pp london > ../data/london_pp.csv
target/release/clip_pp tokyo > ../data/tokyo_pp.csv
```

It reads the entire population point file into memory, then splits up the file to process the bits in parallel across multiple CPUs. To avoid accumulating a potentially huge result, it writes the results immediately after calculating it. To simplify things, writing is done by printing to stdout instead of writing to a file. Use a UNIX pipe to divert stdout to a file. Rust already uses a mutex when writing to stdout, so there are no race conditions.

This is the first step that takes significant time (28 minutes for Tokyo). The time complexity is O(m), where m is the number of population points. Technically the city boundaries are multi-polygons so every polygon is compared, but in practice the number of population points dominates and it is always possible to dissolve the multi-polygons into one.

## Reproject stations and population points into WGS84, Pseudo-Mercator, EPSG:3857

```sh
python python/reproj_to_meters.py london_trains
python python/reproj_to_meters.py tokyo_trains
python python/reproj_to_meters.py london_pp
python python/reproj_to_meters.py tokyo_pp
```

Calculating distances requires the coordinates to be in meters rather than lat/long.

# Analysis

## Cumulative population within a certain distance of a train station

```sh
cd rust
cargo b --release --bin cumulative_props
target/release/cumulative_props london > ../data/london_props.csv
target/release/cumulative_props tokyo > ../data/tokyo_props.csv
cd ..
python python/plot_props.py
```

## Stations within population points

```sh
cd rust
cargo b --release --bin stations_within_pp
target/release/stations_within_pp london
target/release/stations_within_pp tokyo
```

A brute force search has time complexity O(n\*m), where n is the number of stations and m is the number of population points. There are millions to billions of population points so asymptotic growth is really important here.

This program uses a r\* tree, which is O(log(n)) for searching distances, and O(n\*log(n)) for insertion. If we only insert the stations (Tokyo has only ~1000), then insertion time is negligible. Bulk loading the stations will also reduce tree building time.

There are m population points, so searching for the nearest station for every population point is O(m\*log(n)). Because the number of population points m >>> number of stations n, m >>> log(n), so it's basically O(m). This is significantly faster than O(n\*m).

## Population points and number of stations within 500m of them

```sh
cd rust
cargo b --release --bin quadrants
target/release/quadrants 3000
```

## Map of population points with normal population but high number of stops within X meters

```sh
cd rust
cargo b --release --bin quadrant_coords
# Usage:
# target/release/quadrant_coords [city] [X meters] [Q3 of population] [Q3 of n_stations]
target/release/quadrant_coords london 3000 10.608587 3.0 > ../data/london_reds.csv
target/release/quadrant_coords tokyo 3000 8.678337 2.0 > ../data/tokyo_reds.csv
```

