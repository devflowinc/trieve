import sys
from collections import Counter
from urllib.parse import urlparse

if len(sys.argv) != 2:
    print("Usage: python parse_results.py <results_file>")
    sys.exit()

FILE = sys.argv[1]
OUT = FILE + ".out"

sums = Counter()

with open(FILE) as f:
    lines = f.readlines()
    for line in lines:
        sums[urlparse(line).hostname] += 1

print(sums)

with open(OUT, 'w') as f:
    for k, v in sums.most_common():
        f.write('Name "{}": {}\n'.format(k, v))
