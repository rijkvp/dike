import time, random
a, b = map(int, input().split())
time.sleep(random.uniform(2, 8))
if a == 666:
    exit("fatal error")
print(a + b)
