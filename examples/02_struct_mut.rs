// Idea 2: struct and let / let mut.
//
// A `struct` groups related values under one name. Here a Point holds an x and a y.
// A value can only change if you declare it with `let mut`. Rust refuses the rest.
// Run it with:  cargo run --example 02_struct_mut
// The Python version is python/02_dataclass_point.py (Python lets anything change).

struct Point {
    x: f32,
    y: f32,
}

fn main() {
    let finger = Point { x: 120.0, y: 80.0 };
    println!("finger at ({}, {})", finger.x, finger.y);

    // `cursor` is declared with `mut`, so it may move.
    let mut cursor = Point { x: 0.0, y: 0.0 };
    cursor.x = finger.x;
    cursor.y = finger.y;
    println!("cursor at ({}, {})", cursor.x, cursor.y);

    // Uncomment ONE of these lines to see the compiler stop you:
    // finger.x = 300.0; // error[E0594]: cannot assign to `finger.x`, as `finger` is not declared as mutable
    // let clicks = 0; clicks = clicks + 1; // error[E0384]: cannot assign twice to immutable variable `clicks`
}
