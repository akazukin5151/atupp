import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

london = pd.read_csv('data/london_stations_within_pp.csv')
london['city'] = 'london'

tokyo = pd.read_csv('data/tokyo_stations_within_pp.csv')
tokyo['city'] = 'tokyo'

df = pd.concat([london, tokyo])

g = sns.boxplot(data=df, x='max_dist', y='n_stations', hue='city')

#g.set_xticklabels(g.get_xticklabels(), rotation=90)
plt.tight_layout()
plt.savefig('out/n_stations.png')
