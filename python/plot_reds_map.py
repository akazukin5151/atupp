import sys
import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns

def main(city):
    grays = pd.read_csv(f'data/{city}_pp_meters.csv')
    grays = grays.sample(1000)
    grays['x'] = grays['x'].astype(float)
    grays['y'] = grays['y'].astype(float)

    reds = pd.read_csv(f'data/{city}_reds.csv')
    reds = reds.sample(1000)
    reds['x'] = reds['x'].astype(float)
    reds['y'] = reds['y'].astype(float)

    _, ax = plt.subplots()
    sns.scatterplot(data=grays, x='x', y='y', ax=ax, color='gray')
    sns.scatterplot(data=reds, x='x', y='y', ax=ax, color='red')
    plt.savefig(f'out/{city}_reds.png')


main(sys.argv[1])
