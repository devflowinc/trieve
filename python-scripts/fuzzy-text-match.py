from thefuzz import fuzz
import sys

print(fuzz.token_set_ratio(sys.argv[1], sys.argv[2]))
