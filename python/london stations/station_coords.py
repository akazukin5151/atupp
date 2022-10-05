import pandas as pd

with open('data/london_trains/lines/line_ids.txt', 'r') as f:
    file = f.read()

remove_quotes = file.replace('"', '')
splitted = remove_quotes.split('\n')
# remove last line which is an empty string
lines = splitted[:-1]

dfs = None
for line in lines:
    df = pd.read_csv(f'data/london_trains/stoppoints by line/{line}.csv')
    if dfs is None:
        dfs = df
    else:
        dfs = pd.concat([dfs, df])

dfs.columns = ['unused', 'station_name', 'lat', 'lon']
dfs.drop(columns='unused', inplace=True)
dfs.drop_duplicates(inplace=True)
dfs.to_csv('data/london_trains/stations/station_coords.csv', index=False)
