#!/usr/bin/env python3

import math

print('Compute rounding errors')

def log2(x):
    return math.log(x)/math.log(2)

def error(n,d,m):
    t1 = n * log2(n / (n + d))
    t2 = (d + m) * log2((d + m) / m)
    return t1 + t2

def show_error(n, d, m):
    e1 = error(n,d,m)
    e2 = error(m,d,n)
    print(f'{n:.6f}..{d:.6f}..{m:.6f} -> {e1:.8f} :: {e2:.8f}')

show_error(1, 1, 2)

print()

# Project a quarter circle onto the interval [1/2, 1] (equally-spaced angles are i/w, where 0/w corresponds to 1/2 and w/w corresponds to 1)
def p(i, w):
    return (1 + math.cos((math.pi*w - math.pi*i)/(2*w))) / 2

def show_p_error(i, w):
    p1 = p(i, w)
    p2 = p(i + 1, w);
    n = p1
    d = p2 - p1
    m = 1 - p2
    show_error(n * w, d * w, m * w)

for w in range(1, 100, 10):
    print()
    print(f'Width: w={w}')
    for i in range(w - 1):
        show_p_error(i, w)

print()
print('Width: w=1000')
for i in range(1, 999, 100):
    show_p_error(i, 1000)
