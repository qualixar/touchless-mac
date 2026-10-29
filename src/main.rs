// touchless-mac: control your Mac with your hand. Chapter 6 adds start-up calibration.
mod calibrate;
mod camera;
mod error;
mod gesture;
mod hand;
mod mouse;
mod options;
mod selftest;
mod steer;

use camera::Webcam;
use error::Result;
use gesture::{GestureDetector, Reading};
use hand::{Detector, INDEX_KNUCKLE, Tracker};
use mouse::Mouse;
use options::{Options, USAGE};
use std::path::Path;
use std::time::Instant;
use steer::{ReachBox, Steering};

fn main() {
    // `run` returns a Result. If it is an Err, we print its message and stop calmly:
    // no panic, no stack trace, just what went wrong and what to do about it.
    if let Err(error) = run() {
        eprintln!("\n{error}\n");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let options = Options::parse(&args)?;
    if options.show_help {
        println!("{USAGE}");
        return Ok(());
    }

    // Cheap checks first: if the models are missing, say so before asking for any permission.
    let models = Path::new(&options.models_dir);
    let mut detector = Detector::new(&models.join("palm.onnx"), &models.join("hand.onnx"))?;
    if !options.selftest_images.is_empty() {
        for path in &options.selftest_images {
            selftest::run(&mut detector, path)?;
        }
        return Ok(());
    }

    let mut tracker = Tracker::new(detector);
    let mut webcam = Webcam::open()?;
    let mut mouse = Mouse::new()?;
    let reach = calibrate::choose_reach_box(options.calibration, &mut webcam, &mut tracker)?;
    follow_hand(&mut webcam, &mut tracker, &mut mouse, reach)
}

/// The main loop: camera picture in, mouse action out, for ever (Ctrl-C stops it).
fn follow_hand(webcam: &mut Webcam, tracker: &mut Tracker, mouse: &mut Mouse, reach: ReachBox) -> Result<()> {
    let (screen_width, screen_height) = mouse.screen_size()?;
    let mut steering = Steering::new(reach, screen_width, screen_height);
    let mut gestures = GestureDetector::new();
    println!("Screen {screen_width} x {screen_height}. Point to move, pinch to click. Ctrl-C stops.");

    let mut previous_frame_at = Instant::now();
    let mut hand_was_visible = false;
    loop {
        let frame = webcam.grab()?;
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
                    mouse.perform(event)?;
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
