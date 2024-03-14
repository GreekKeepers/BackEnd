import math


initial_deck = []

for i in range(4):
    for j in range(1, 14):
        initial_deck.append({"number": j, "suit": i})

print(initial_deck)
