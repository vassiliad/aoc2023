# VV: madness :)
import numpy

values_x =  [65, 196, 327]
values_y =  [3947, 35153, 97459]

values_x = [i for i in range(len(values_x))]
params = numpy.polyfit(values_x, values_y, 2)

print(params)

def calc(target):
    return target*target*params[0] + target*params[1] + params[2]

for (x, y) in zip(values_x, values_y):
    guess = calc(x)
    print(x, y, guess, guess-y)

target = 26501365//131
print(calc(target))
