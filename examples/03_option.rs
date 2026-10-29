// Idea 3: Option.
//
// "Might be missing" is part of the type. find_hand returns Option<Point>: either
// Some(point) or None. Rust will not let you use the point until you check which one it is.
// Run it with:  cargo run --example 03_option
// The Python version is python/03_none_crash.py (it crashes three lines after the mistake).

struct Point {
    x: usize,
    y: usize,
}

/// Look for the hand, drawn as a `#` in a tiny text picture.
fn find_hand(picture: &[&str]) -> Option<Point> {
    for (y, row) in picture.iter().enumerate() {
        if let Some(x) = row.find('#') {
            return Some(Point { x, y });
        }
    }
    None
}

fn main() {
    let with_hand = ["....", "..#.", "...."];
    let without_hand = ["....", "....", "...."];

    for picture in [with_hand, without_hand] {
        match find_hand(&picture) {
            Some(hand) => println!("hand at ({}, {})", hand.x, hand.y),
            None => println!("no hand, the cursor waits"),
        }
    }

    // `if let` is the short form when you only care about the Some case.
    if let Some(hand) = find_hand(&with_hand) {
        println!("again: x is {}", hand.x);
    }

    // Uncomment to see the compiler stop you from using the Option without checking it:
    // println!("{}", find_hand(&without_hand).x); // error[E0609]: no field `x` on type `Option<Point>`
}
