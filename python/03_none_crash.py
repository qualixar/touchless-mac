# Idea 3, the Python way: a function that might find nothing returns None, and the
# crash happens later, far from the mistake.
#
#   python python/03_none_crash.py      (this one really crashes, on purpose)
#
# Rust's version is examples/03_option.rs, where the compiler makes you check first.
def find_hand(picture):
    for y, row in enumerate(picture):
        if "#" in row:
            return (row.index("#"), y)
    return None  # no hand in the picture


with_hand = ["....", "..#.", "...."]
without_hand = ["....", "....", "...."]

hand = find_hand(with_hand)
print("hand at", hand)

hand = find_hand(without_hand)
print("looking for the hand again...")
cursor_x = hand.x  # AttributeError: 'NoneType' object has no attribute 'x'
print("the cursor moves to", cursor_x)
