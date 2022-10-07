import sys
import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns

def main(ax, city, point_type):
    reds = pd.read_csv(f'data/{city}_{point_type}s.csv')
    reds = reds.sample(1000)
    reds['x'] = reds['x'].astype(float)
    reds['y'] = reds['y'].astype(float)
    reds['point_type'] = point_type
    return reds


_, ax = plt.subplots()
city = sys.argv[1]

grays = pd.read_csv(f'data/{city}_pp_meters.csv')
grays = grays.sample(1000)
grays['x'] = grays['x'].astype(float)
grays['y'] = grays['y'].astype(float)

df = None

for point_type in {'red', 'orange', 'blue', 'green'}:
    color_df = main(ax, city, point_type)
    if df is None:
        df = color_df
    else:
        df = pd.concat([df, color_df])

g = sns.FacetGrid(data=df, col='point_type')
for col_val, ax in g.axes_dict.items():
    sns.scatterplot(ax=ax, data=grays, x='x', y='y', color='gray')
    sns.scatterplot(
        ax=ax, data=df[df['point_type'] == col_val], x='x', y='y', color=col_val
    )
    ax.set_axis_off()

g.savefig(f'out/{city}_quadrants_map.png')
