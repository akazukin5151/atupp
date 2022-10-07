import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

london = pd.read_csv('data/london_props.csv')
london['City'] = 'London'

tokyo = pd.read_csv('data/tokyo_props.csv')
tokyo['City'] = 'Tokyo'

df = pd.concat([london, tokyo])

_, g = plt.subplots(figsize=(15, 9))
sns.barplot(data=df, x='max_dist', y='prop', hue='City', ax=g)

sns.despine()
g.set_xticklabels(g.get_xticklabels(), rotation=90)
g.set_xlabel('Distance')
g.set_ylabel('Proportion of population within distance of a station')

plt.tight_layout()
plt.savefig('out/props.png')
