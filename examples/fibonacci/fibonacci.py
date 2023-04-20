def fibbonacci(n):
    if n <= 1:
        return n
    else:
        return fibbonacci(n-1) + fibbonacci(n-2)

n = int(input())
if n == 9:
    # Program randomly crashes test
    exit(2)
elif n == 15:
    # Program has wrong output test
    print('8375')
else:
    print(fibbonacci(n))
