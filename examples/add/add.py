import time, random
a, b = map(int, input().split())
time.sleep(random.uniform(0.5, 3))
if a == 666:
    exit("fatal error")
elif a == 777:
    time.sleep(1000)
print(a + b)
