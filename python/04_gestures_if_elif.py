# Idea 4, the Python way: gestures are strings and an if/elif chain decides what to do.
# A new gesture that nobody added to the chain is silently ignored.
#
#   python python/04_gestures_if_elif.py
#
# Rust's version is examples/04_enum_match.rs, where the compiler refuses to build.
def what_it_does(gesture):
    if gesture == "point":
        return "move the cursor"
    elif gesture == "pinch":
        return "click"
    elif gesture == "double_pinch":
        return "double-click"
    elif gesture == "wave_left":
        return "previous app"
    elif gesture == "wave_right":
        return "next app"
    # Nothing handles "fist". Python returns None and does not warn anyone.


for gesture in ["point", "pinch", "double_pinch", "wave_right", "fist"]:
    print(f"{gesture:<12} -> {what_it_does(gesture)}")

# A typo is ignored the same way:
print(f"{'pinch ':<12} -> {what_it_does('pinch ')}")
