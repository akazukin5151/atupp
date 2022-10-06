import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

def main(city_name):
    df = pd.read_csv(f'data/{city_name}_quadrant.csv')
    df = df.sample(100000)

    sns.scatterplot(data=df, x='population', y='n_stations')

    plt.tight_layout()
    plt.savefig(f'out/{city_name}_quadrant.png')
    plt.close()


main('london')
main('tokyo')
