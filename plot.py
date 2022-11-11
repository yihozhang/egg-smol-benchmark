import csv
import matplotlib.pyplot as plt
import numpy as np

data = []
with open('benchmarks.csv', newline='') as csvfile:
    reader = csv.reader(csvfile)
    for row in reader:
        data.append((int(row[0][9:]), row[1], float(row[2]), float(row[3])))

def smooth(data):
    for i in range(1, len(data)):
        data[i] = (data[i][0], max(data[i][1], data[i-1][1]))
    return data

X = 3
plt.figure(1)
egglog = list(map(lambda x: (x[X], x[2]/1e9), filter(lambda x: x[1] == 'Egglog', data)))
egglog = smooth(egglog)
# egglog = sorted(egglog, key = lambda x: x[1])
egglog_x = list(map(lambda x:x[0], egglog))
egglog_y = list(map(lambda x:x[1], egglog))
plt.plot(egglog_y, egglog_x, label="EqLog")

egglognaive = list(map(lambda x: (x[X], x[2]/1e9), filter(lambda x: x[1] == 'EgglogNaive', data)))
# egglognaive = sorted(egglognaive, key = lambda x: x[1])
egglognaive = smooth(egglognaive)
egglognaive_x = list(map(lambda x:x[0], egglognaive))
egglognaive_y = list(map(lambda x:x[1], egglognaive))
plt.plot(egglognaive_y, egglognaive_x, label="EqLogNI")

egg = list(map(lambda x: (x[X], x[2]/1e9), filter(lambda x: x[1] == 'Egg', data)))
# egg = sorted(egg, key = lambda x: x[1])
egg = smooth(egg)
egg_x = list(map(lambda x:x[0], egg))
egg_y = list(map(lambda x:x[1], egg))
plt.plot(egg_y, egg_x, label="egg")
plt.xlabel("Time (s)")
plt.ylabel("E-node numbers")
plt.ylim((10**4, None))
plt.yscale('log')

plt.legend(loc='lower right')
plt.savefig("microbenchmarks.pdf")

plt.show()

naive_speedup = egg[-1][1] / egglognaive[-1][1]
print("EgglogNaive speedup over egg: " + str(naive_speedup))

# plt.figure(2)
# X = 0
# egglog = list(map(lambda x: (x[X], x[2]/1e9), filter(lambda x: x[1] == 'Egglog', data)))
# # egglog = sorted(egglog, key = lambda x: x[1])
# egglog_x = list(map(lambda x:x[0], egglog))
# egglog_y = list(map(lambda x:x[1], egglog))
# plt.plot(egglog_x, egglog_y, label="egglog")

# egg = list(map(lambda x: (x[X], x[2]/1e9), filter(lambda x: x[1] == 'Egg', data)))
# # egg = sorted(egg, key = lambda x: x[1])
# egg_x = list(map(lambda x:x[0], egg))
# egg_y = list(map(lambda x:x[1], egg))
# plt.plot(egg_x, egg_y, label="egg")

# egglog = list(map(lambda x: (x[X], x[2]/1e9), filter(lambda x: x[1] == 'EgglogNaive', data)))
# # egglog = sorted(egglog, key = lambda x: x[1])
# egglog_x = list(map(lambda x:x[0], egglog))
# egglog_y = list(map(lambda x:x[1], egglog))
# plt.plot(egglog_x, egglog_y, label="egglog-naive")
# plt.legend(loc='upper right')
# plt.savefig("time-iter.png")
