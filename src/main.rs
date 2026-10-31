// touchless-mac: control your Mac with your hand. Chapter 6 adds start-up calibration.
mod calibrate;
mod camera;
mod error;
mod gesture;
mod hand;
mod mouse;
mod options;
mod selftest;
mod setup;
mod steer;

use camera::Webcam;
use error::Result;
use gesture::{GestureDetector, Reading};
use hand::{Detector, INDEX_KNUCKLE, Tracker};
use mouse::Mouse;
use options::{Options, USAGE};
use std::time::Instant;
use steer::{ReachBox, Steering};

fn main() {
    // `run` returns a Result. If it is an Err, we print its message and stop calmly:
    // no panic, no stack trace, just what went wrong and what to do about it.
    if let Err(error) = run() {
        eprintln!("\n{error}\n");
        // A missing permission is fixed in System Settings, so open the right page for the user.
        if let Some(pane) = setup::pane_for(&error) {
            eprintln!("Opening that page in System Settings for you...\n");
            setup::open_settings(pane);
        }
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
    if options.show_version {
        println!("touchless-mac {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if options.doctor {
        return doctor(options.models_dir.as_deref());
    }

    // Cheap checks first: if the models are missing, say so before asking for any permission.
    let models = setup::find_models(options.models_dir.as_deref())?;
    let mut detector = Detector::new(&models.join(setup::PALM), &models.join(setup::HAND))?;
    if !options.selftest_images.is_empty() {
        for path in &options.selftest_images {
            selftest::run(&mut detector, path)?;
        }
        return Ok(());
    }

    let mut tracker = Tracker::new(detector);
    let mut webcam = Webcam::open()?;
    let mut mouse = Mouse::new()?;
    if options.invert_scroll {
        mouse.invert_scroll();
    }
    let reach = calibrate::choose_reach_box(options.calibration, &mut webcam, &mut tracker)?
        .with_speed(options.speed);
    follow_hand(&mut webcam, &mut tracker, &mut mouse, reach)
}

/// `touchless-mac doctor`: check everything a first run needs and say how to fix what is missing.
fn doctor(models_dir: Option<&str>) -> Result<()> {
    let mut checks = Vec::new();
    match setup::find_models(models_dir)
        .and_then(|dir| Detector::new(&dir.join(setup::PALM), &dir.join(setup::HAND)).map(|_| dir))
    {
        Ok(dir) => checks.push(setup::Check {
            ok: true,
            what: format!(
                "Hand models load from {}",
                dir.canonicalize().unwrap_or(dir).display()
            ),
            fix: None,
        }),
        Err(error) => checks.push(setup::Check {
            ok: false,
            what: "Hand models".into(),
            fix: Some(error.to_string().replace('\n', "\n       ")),
        }),
    }
    let trusted = setup::accessibility_allowed();
    checks.push(setup::Check {
        ok: trusted,
        what: "Accessibility (lets the app move the mouse)".into(),
        fix: (!trusted).then(|| {
            setup::open_settings(setup::Pane::Accessibility);
            "Opened System Settings > Privacy & Security > Accessibility: switch on the app \
             you run this from (usually Terminal)."
                .into()
        }),
    });
    match Webcam::open().and_then(|mut cam| cam.grab()) {
        Ok(frame) => checks.push(setup::Check {
            ok: true,
            what: format!("Camera works ({} x {})", frame.width(), frame.height()),
            fix: None,
        }),
        Err(error) => {
            if let Some(pane) = setup::pane_for(&error) {
                setup::open_settings(pane);
            }
            checks.push(setup::Check {
                ok: false,
                what: "Camera".into(),
                fix: Some(error.to_string().replace('\n', "\n       ")),
            })
        }
    }
    let saved = calibrate::saved_box_exists();
    checks.push(setup::Check {
        ok: true,
        what: if saved {
            "Calibration saved (run with --recalibrate to redo it)".into()
        } else {
            "No calibration yet: the first run asks you to point at four corners (about 13 s)"
                .into()
        },
        fix: None,
    });
    setup::print_report(&checks);
    Ok(())
}

/// The main loop: camera picture in, mouse action out, for ever (Ctrl-C stops it).
fn follow_hand(
    webcam: &mut Webcam,
    tracker: &mut Tracker,
    mouse: &mut Mouse,
    reach: ReachBox,
) -> Result<()> {
    let (screen_width, screen_height) = mouse.screen_size()?;
    let mut steering = Steering::new(reach, screen_width, screen_height);
    let mut gestures = GestureDetector::new();
    println!(
        "Screen {screen_width} x {screen_height}. Point to move, pinch to click. Ctrl-C stops."
    );

    let mut previous_frame_at = Instant::now();
    let mut hand_was_visible = false;
    loop {
        let frame = webcam.grab()?;
        let now = Instant::now();
        let dt = now
            .duration_since(previous_frame_at)
            .as_secs_f32()
            .max(1e-3);
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
