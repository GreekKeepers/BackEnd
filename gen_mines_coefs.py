import math


multipliers = []

for num_mines in range(1, 25):
    multipliers_per_mines = []
    for g in range(1, (25 - num_mines) + 1):
        multiplier = 1
        divisor = 1
        for f in range(0, g):
            multiplier *= 25 - num_mines - f
            divisor *= 25 - f
        mult = (9900 * (10**9)) // ((multiplier * (10**9)) // divisor)

        multipliers_per_mines.append(f'"{round(mult / 10000, 4)}"')
    multipliers.append(multipliers_per_mines)


print(multipliers)
