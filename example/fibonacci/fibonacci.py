# Really slow & unoptimized fibonacci example
def fibbonacci(n):
    if n <= 1:
        return n
    else:
        return fibbonacci(n-1) + fibbonacci(n -2)
print(fibbonacci(int(input())))