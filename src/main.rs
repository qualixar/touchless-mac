// Chapter 4: enum and match. Pinch clicks, double pinch double-clicks, waves switch apps.
mod camera;
mod gesture;
mod hand;
mod mouse;
mod steer;

use anyhow::Result;
use camera::Webcam;
use gesture::{GestureDetector, Reading};
use hand::{Detector, INDEX_KNUCKLE, Tracker};
use mouse::Mouse;
use std::time::Instant;
use steer::{ReachBox, Steering};

fn main() -> Result<()> {
    let mut webcam = Webcam::open();
    let detector = Detector::new("models/palm.onnx", "models/hand.onnx")?;
    let mut tracker = Tracker::new(detector);
    let mut gestures = GestureDetector::new();
    let mut mouse = Mouse::new();

    let (screen_width, screen_height) = mouse.screen_size();
    let mut steering = Steering::new(ReachBox::DEFAULT, screen_width, screen_height);
    println!("Screen {screen_width} x {screen_height}. Point to move, pinch to click. Ctrl-C stops.");

    let mut previous_frame_at = Instant::now();
    let mut hand_was_visible = false;
    loop {
        let frame = webcam.grab();
        let now = Instant::now();
        let dt = now.duration_since(previous_frame_at).as_secs_f32().max(1e-3);
        previous_frame_at = now;

        // `see` returns an Option<Hand>: either `Some(hand)` or `None`.
        // `match` makes us write down what happens in BOTH cases, so we can't forget one.
        match tracker.see(&frame)? {
            Some(hand) => {
                if !hand_was_visible {
                    println!("Found your hand (confidence {:.2}).", hand.confidence);
                }
                hand_was_visible = true;

                let knuckle = hand.points[INDEX_KNUCKLE];
                let cursor = steering.aim(knuckle, frame.width() as f32, dt);

                // The detector turns the hand into a Gesture; the mouse does what it means.
                let reading = Reading::new(now, cursor, &hand, frame.width() as f32);
                if let Some(event) = gestures.update(&reading) {
                    mouse.perform(event);
                }
            }
            None => {
                if hand_was_visible {
                    println!("Hand left the picture. The cursor waits.");
                }
                hand_was_visible = false;
                // Do not touch the mouse. Forget the smoothing so there is no jump later.
                steering.lost();
                gestures.lost();
            }
        }
    }
}
