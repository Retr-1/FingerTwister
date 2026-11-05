from itertools import permutations
from collections import defaultdict, Counter
RED = 0
BLUE = 1
WHITE = 2
COLOR_COUNTS = {
    RED: 4,
    WHITE: 2,
    BLUE:1
}
CANNOT = 0
CAN = 1
ANS = [CANNOT, CAN, CAN, CAN]
arr = []
for x in COLOR_COUNTS:
    for i in range(COLOR_COUNTS[x]):
        arr.append(x)

possible = set(permutations(arr))
# print(possible)
# for i in range(len(ANS)):
#     new_possible = possible.copy()
#     for perm in possible:
#         see_counts = defaultdict(int)
#         for j in range(i+1, 5):
#             see_counts[perm[j]] += 1

        # Criterions
        # 1. Has only one color left after seeing
        # 2. Has only one hat color in remaining possible

new_possible = possible.copy()
for perm in possible:
    see_counts = defaultdict(int)
    for i in range(1, 5):
        see_counts[perm[i]] += 1

    excluded = 0
    for color in COLOR_COUNTS:
        if COLOR_COUNTS[color] - see_counts[color] == 0:
            excluded += 1

    if excluded >= 2:
        new_possible.remove(perm)

# print(new_possible)

for k in range(2,5):
    possible = new_possible
    new_possible = possible.copy()

    collections = defaultdict(set)
    unique_observations = set()
    observation_to_perm = defaultdict(set)

    for perm in possible:
        see_counts = defaultdict(int)
        for i in range(k, 5):
            see_counts[perm[i]] += 1

        observation = tuple(sorted(see_counts.items()))
        unique_observations.add(observation)
        collections[perm[i]].add(observation)
        observation_to_perm[observation].add(perm)

    for x in unique_observations:
        print(x)

    for x in collections:
        print(x, len(collections[x]), collections[x])


    new_possible = set()

    for observation in unique_observations:
        found_in_places = 0
        for color in collections:
            if observation in collections[color]:
                found_in_places += 1
                continue

        if found_in_places <= 1:
            new_possible |= observation_to_perm[observation]

    print()
    print(new_possible)

print()
print(Counter(x[4] for x in new_possible))

# x = defaultdict(int)
# x[1] += 10
# x[2] += 20

# print(tuple(sorted(x.items())))

    

