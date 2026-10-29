// Chapter 3: Option. The hand can leave the picture, and the app stays calm.
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

    let mut previous_frame_at = Instant::now();
    let mut hand_was_visible = false;
    loop {
        let frame = webcam.grab();
        let dt = previous_frame_at.elapsed().as_secs_f32().max(1e-3);
        previous_frame_at = Instant::now();

        // `see` returns an Option<Hand>: either `Some(hand)` or `None`.
        // `match` makes us write down what happens in BOTH cases, so we can't forget one.
        match tracker.see(&frame)? {
            Some(hand) => {
                if !hand_was_visible {
                    println!("Found your hand (confidence {:.2}).", hand.confidence);
                }
                hand_was_visible = true;

                let knuckle = hand.points[INDEX_KNUCKLE];
                let (x, y) = steering.aim(knuckle, frame.width() as f32, dt);
                mouse.move_to(x, y);
            }
            None => {
                if hand_was_visible {
                    println!("Hand left the picture. The cursor waits.");
                }
                hand_was_visible = false;
                // Do not touch the mouse. Forget the smoothing so there is no jump later.
                steering.lost();
            }
        }
    }
}
