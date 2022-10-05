import sys
import geopandas as gpd
from pathlib import Path
from shapely.geometry import Point

arg = sys.argv[1]
if arg == 'london_trains':
    file = 'data/london_trains/stations/station_coords.csv'
    extractor = ((lambda r: r.lon), (lambda r: r.lat))
elif arg == 'tokyo_trains':
    file = 'data/tokyo_trains/coords.csv'
    extractor = ((lambda r: r.lon), (lambda r: r.lat))
elif arg == 'london_pp':
    file = 'data/london_pp.csv'
    extractor = ((lambda r: r.Lon), (lambda r: r.Lat))
elif arg == 'tokyo_pp':
    file = 'data/tokyo_pp.csv'
    extractor = ((lambda r: r.longitude), (lambda r: r.latitude))

out_path = Path(file)
out_path = out_path.with_stem(out_path.stem + '_meters')

df = gpd.read_file(file)
df['geometry'] = df.apply(
    lambda r: Point(float(extractor[0](r)), float(extractor[1](r)))
    , axis=1
)
df = df.set_crs(epsg=4326)
df = df.to_crs(epsg=3857)
df['x'] = df['geometry'].apply(lambda p: p.x)
df['y'] = df['geometry'].apply(lambda p: p.y)
df.drop('geometry', axis=1, inplace=True)
df.to_csv(out_path, index=False)
