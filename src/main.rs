// Chapter 2: struct and let mut. The cursor follows your finger.
mod camera;
mod hand;
mod mouse;
mod steer;

use anyhow::Result;
use camera::Webcam;
use hand::{Detector, INDEX_KNUCKLE, Tracker};
use mouse::Mouse;
use std::time::Instant;
use steer::{ReachBox, Steering};

fn main() -> Result<()> {
    let mut webcam = Webcam::open();
    let detector = Detector::new("models/palm.onnx", "models/hand.onnx")?;
    let mut tracker = Tracker::new(detector);
    let mut mouse = Mouse::new();

    let (screen_width, screen_height) = mouse.screen_size();
    let mut steering = Steering::new(ReachBox::DEFAULT, screen_width, screen_height);
    println!("Screen {screen_width} x {screen_height}. Point with your index finger. Ctrl-C stops.");

    // `let mut` means "this value is allowed to change". Without `mut`, Rust refuses.
    let mut previous_frame_at = Instant::now();
    let mut first_hand = true;
    loop {
        let frame = webcam.grab();

        // Chapter 3 fixes this line: if the hand leaves the picture, the program stops here.
        let hand = tracker.see(&frame)?.expect("no hand in the picture");

        if first_hand {
            println!("Found your hand (confidence {:.2}).", hand.confidence);
            first_hand = false;
        }

        let dt = previous_frame_at.elapsed().as_secs_f32().max(1e-3);
        previous_frame_at = Instant::now();

        let knuckle = hand.points[INDEX_KNUCKLE];
        let (x, y) = steering.aim(knuckle, frame.width() as f32, dt);
        mouse.move_to(x, y);
    }
}
