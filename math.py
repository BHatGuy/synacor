import itertools
#_ + _ * _^2 + _^3 - _ = 399

#red corroded shiny concave blue
coins = [2, 3, 5, 7, 9]

for perm in itertools.permutations(coins):
    res = perm[0] + perm[1] * perm[2]**2 + perm[3]**3 - perm[4]
    if res == 399:
        print(perm)
        break
    


