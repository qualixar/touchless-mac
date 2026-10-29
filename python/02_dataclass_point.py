# Idea 2, the Python way: a dataclass groups values, and anyone can change them any time.
#
#   python python/02_dataclass_point.py
#
# Rust's version is examples/02_struct_mut.rs. In Rust a value only changes if it was
# declared with `mut`.
from dataclasses import dataclass


@dataclass
class Point:
    x: float
    y: float


finger = Point(120.0, 80.0)
print(f"finger at ({finger.x}, {finger.y})")

# Nothing stopped this change. Python is happy even if it was a bug.
finger.x = 300.0
print(f"finger at ({finger.x}, {finger.y})")
