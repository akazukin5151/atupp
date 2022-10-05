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

It reads the entire pp file into memory, then process parts of the file in parallel across multiple CPUs. To avoid accumulating a potentially huge result, it is better to write them immediately. To simplify things, writing is done by printing to stdout instead of writing to a file. Use a UNIX pipe to divert stdout to a file.

These are rust programs in parallel (the first two rows). For comparison, these are the timings for serial and python.

## Reproject stations and population points into WGS84, Pseudo-Mercator, EPSG:3857

```sh
python python/reproj_to_meters.py london_trains
python python/reproj_to_meters.py tokyo_trains
python python/reproj_to_meters.py london_pp
python python/reproj_to_meters.py tokyo_pp
```

Distance matrix requires the coordinates to be in meters rather than lat/long.

# Analysis

## Calculate the distance matrix for station to population point

```sh
cd rust
cargo b --release --bin matrix
target/release/matrix london > ../data/london_matrix.csv
target/release/matrix tokyo > ../data/tokyo_matrix.csv
```

273 london stations * 1044719 london pps = 2.85e8

london_matrix.csv has 2.84e8 lines and 19.5 GB

19.5 GB = 19.5e9 = 1.95e10

so the average line is 68.37 bytes

1412 tokyo stations * 6083761 tokyo pps = 8.59e9

8.59e9 lines * 68.37 bytes = 5.873e11 bytes = 597.3e9 bytes = 597.3 GB

