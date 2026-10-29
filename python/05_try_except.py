# Idea 5, the Python way: wrap the risky call in try/except.
# Nothing forces you to; forget it and the program stops with a traceback.
#
#   python python/05_try_except.py
#
# Rust's version is examples/05_result.rs, where failure is part of the function's type.
def cm_to_metres(text):
    return float(text.strip()) / 100.0


def describe_pinch(text):
    try:
        metres = cm_to_metres(text)
    except ValueError as error:
        return f"problem: '{text}' is not a distance in cm ({error})"
    return "pinch" if metres < 0.025 else "open hand"


for text in ["1.5", "8", "eight"]:
    print(f"{text:>5} cm -> {describe_pinch(text)}")
