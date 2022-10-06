import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

london = pd.read_csv('data/london_props.csv')
london['city'] = 'london'

tokyo = pd.read_csv('data/tokyo_props.csv')
tokyo['city'] = 'tokyo'

df = pd.concat([london, tokyo])

g = sns.barplot(data=df, x='max_dist', y='prop', hue='city')

g.set_xticklabels(g.get_xticklabels(), rotation=90)
plt.tight_layout()
plt.savefig('out/props.png')
