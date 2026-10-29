// Idea 1: Cargo and crates.
//
// Cargo is Rust's project tool: it downloads packages, builds the program and runs it.
// A package of Rust code is called a "crate". This example uses one crate, `image`,
// which is listed in Cargo.toml. Run it with:  cargo run --example 01_cargo
//
// The Python version is python/01_hello_pip.py (pip install, then python file.py).
use image::{Rgb, RgbImage};

fn main() {
    println!("Hello from Rust!");

    // A tiny picture, 4 pixels wide and 3 pixels tall, from the `image` crate.
    let mut picture = RgbImage::new(4, 3);
    picture.put_pixel(1, 2, Rgb([255, 0, 0])); // one red pixel
    println!("the picture is {} x {} pixels", picture.width(), picture.height());
    println!("pixel (1, 2) is {:?}", picture.get_pixel(1, 2));
}
