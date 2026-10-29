// Idea 4: enum and match.
//
// An `enum` lists every choice a value can be. A choice can carry data inside it, like
// Wave(Direction). `match` must handle EVERY choice, so adding a new gesture and forgetting
// to say what it does is a compile error, not a silent bug.
// Run it with:  cargo run --example 04_enum_match
// The Python version is python/04_gestures_if_elif.py (it silently ignores a new gesture).

enum Direction {
    Left,
    Right,
}

enum Gesture {
    Point,
    Pinch,
    DoublePinch,
    Wave(Direction),
    // Fist, // <- uncomment this new gesture: error[E0004]: non-exhaustive patterns: `Gesture::Fist` not covered
}

fn what_it_does(gesture: Gesture) -> &'static str {
    match gesture {
        Gesture::Point => "move the cursor",
        Gesture::Pinch => "click",
        Gesture::DoublePinch => "double-click",
        Gesture::Wave(Direction::Left) => "previous app",
        Gesture::Wave(Direction::Right) => "next app",
    }
}

fn main() {
    println!("point       -> {}", what_it_does(Gesture::Point));
    println!("pinch       -> {}", what_it_does(Gesture::Pinch));
    println!("double      -> {}", what_it_does(Gesture::DoublePinch));
    println!("wave left   -> {}", what_it_does(Gesture::Wave(Direction::Left)));
    println!("wave right  -> {}", what_it_does(Gesture::Wave(Direction::Right)));
}
