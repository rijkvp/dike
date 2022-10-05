#!/bin/python3
# Really slow & unoptimized fibonacci example
def fibbonacci(n):
    if n <= 1:
        return n
    else:
        return fibbonacci(n-1) + fibbonacci(n-2)

n = int(input())
if n == 9:
    exit(2)
else:
    print(fibbonacci(n))
