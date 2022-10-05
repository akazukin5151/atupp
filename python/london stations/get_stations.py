import json
import urllib.request
import pandas as pd

def main(line_name):
    url = f"https://api.tfl.gov.uk/Line/{line_name}/StopPoints"

    hdr = {
        'Cache-Control': 'no-cache',
    }

    req = urllib.request.Request(url, headers=hdr)

    req.get_method = lambda: 'GET'
    response = urllib.request.urlopen(req)
    x = (response.read())
    j = json.loads(x.decode('utf-8'))

    l = [(y['commonName'], y['lat'], y['lon']) for y in j]
    df = pd.DataFrame(l)
    df.to_csv(f'data/london_trains/stoppoints by line/{line_name}.csv')


with open('data/london_trains/lines/line_ids.txt', 'r') as f:
    file = f.read()

remove_quotes = file.replace('"', '')
splitted = remove_quotes.split('\n')
# remove last line which is an empty string
lines = splitted[:-1]

for line in lines:
    main(line)
