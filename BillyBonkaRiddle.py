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

new_possible = set(permutations(arr))

# print(new_possible)

for person in range(4):
    print(f'ITERATION: {person}')
    possible = new_possible
    new_possible = possible.copy()

    collections = defaultdict(set)
    unique_observations = set()
    observation_to_perm = defaultdict(set)

    for perm in possible:
        see_counts = defaultdict(int)
        for i in range(person+1, 5):
            see_counts[perm[i]] += 1

        observation = tuple(sorted(see_counts.items()))
        unique_observations.add(observation)
        collections[perm[i]].add(observation)
        observation_to_perm[observation].add(perm)

    for x in unique_observations:
        print(x)

    for x in collections:
        print(x, len(collections[x]), collections[x])

    for x in observation_to_perm:
        print(x, observation_to_perm[x])

# (0, 0, 2, 1, 0, 2, 0) vs (0, 0, 0, 1, 2, 2, 0)
    new_possible = set()

    for observation in unique_observations:
        found_in_places = 0
        for color in collections:
            if observation in collections[color]:
                found_in_places += 1
                continue

        if (ANS[person] == CAN and found_in_places <= 1) or (ANS[person] == CANNOT and found_in_places >= 2):
            new_possible |= observation_to_perm[observation]

    print()
    print(new_possible)

print()
print(Counter(x[4] for x in new_possible))

# x = defaultdict(int)
# x[1] += 10
# x[2] += 20

# print(tuple(sorted(x.items())))

    
# test-copy: {(0, 2, 0, 0, 0, 2, 1), (2, 1, 0, 0, 0, 0, 2), (1, 2, 0, 0, 0, 0, 2), (0, 1, 0, 0, 0, 2, 2), (2, 2, 0, 0, 0, 1, 0), (0, 2, 0, 0, 0, 1, 2), (2, 1, 0, 0, 0, 2, 0), (1, 2, 0, 0, 0, 2, 0), (2, 2, 0, 0, 0, 0, 1)}
# test:      {(0, 2, 0, 0, 0, 2, 1), (2, 1, 0, 0, 0, 0, 2), (1, 2, 0, 0, 0, 0, 2), (0, 1, 0, 0, 0, 2, 2), (2, 2, 0, 0, 0, 1, 0), (0, 2, 0, 0, 0, 1, 2), (2, 1, 0, 0, 0, 2, 0), (1, 2, 0, 0, 0, 2, 0), (2, 2, 0, 0, 0, 0, 1), (1, 0, 0, 0, 0, 2, 2), (2, 0, 0, 0, 0, 1, 2), (2, 0, 0, 0, 0, 2, 1)}
