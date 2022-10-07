import sys
import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns

def main(ax, city, point_type):
    reds = pd.read_csv(f'data/{city}_{point_type}.csv')
    reds = reds.sample(1000)
    reds['x'] = reds['x'].astype(float)
    reds['y'] = reds['y'].astype(float)

    sns.scatterplot(data=reds, x='x', y='y', ax=ax, color=point_type.removesuffix('s'))


_, ax = plt.subplots()
city = sys.argv[1]
for point_type in {'reds', 'oranges', 'blues', 'greens'}:
    main(ax, city, point_type)

plt.savefig(f'out/{city}_quadrants_map.png')
