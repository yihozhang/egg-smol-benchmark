import csv
import matplotlib.pyplot as plt
import numpy as np

data = []
with open('benchmarks.csv', newline='') as csvfile:
    reader = csv.reader(csvfile)
    for row in reader:
        data.append((int(row[0][9:]), row[1], float(row[2]), float(row[3])))
X = 3
egg = list(map(lambda x: (x[X], x[2]/1e9), filter(lambda x: x[1] == 'Egg', data)))
egg = sorted(egg, key = lambda x: x[1])
egg_x = list(map(lambda x:x[0], egg))
egg_y = list(map(lambda x:x[1], egg))
plt.plot(egg_y, egg_x, label="egg")

egglog = list(map(lambda x: (x[X], x[2]/1e9), filter(lambda x: x[1] == 'Egglog', data)))
egglog = sorted(egglog, key = lambda x: x[1])
egglog_x = list(map(lambda x:x[0], egglog))
egglog_y = list(map(lambda x:x[1], egglog))
plt.plot(egglog_y, egglog_x, label="egglog")
plt.legend(loc='upper right')
plt.savefig("plot.png")
plt.show()