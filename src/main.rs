// Chapter 1: Cargo and crates. The app can see.
//
// `cargo run --release` builds this program and starts it. Cargo also downloads the
// two crates listed in Cargo.toml the first time.
mod camera;

use std::time::Instant;

const FRAMES_TO_GRAB: u32 = 90; // about three seconds

fn main() {
    println!("touchless-mac, chapter 1: can the app see?");
    let mut webcam = camera::Webcam::open();

    let started = Instant::now();
    for number in 1..=FRAMES_TO_GRAB {
        let frame = webcam.grab();
        if number == 1 {
            println!("first picture: {} x {} pixels", frame.width(), frame.height());
        }
    }

    let seconds = started.elapsed().as_secs_f32();
    println!("{FRAMES_TO_GRAB} pictures in {seconds:.1} s = {:.1} per second", FRAMES_TO_GRAB as f32 / seconds);
}
