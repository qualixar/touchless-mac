// Chapter 6: calibration. Your desk, your camera and your arm are not the author's.
//
// For about six seconds you point at the four corners of the screen, one after another. We
// record where your index knuckle is each time, and build the "reach box" from those four
// spots (see ReachBox in steer.rs). The box is saved, so the next run starts straight away.
use crate::camera::Webcam;
use crate::error::Result;
use crate::hand::{INDEX_KNUCKLE, Tracker};
use crate::options::CalibrationMode;
use crate::steer::{ReachBox, hand_position};
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Time given to each corner. The first part is for moving your hand there; only the rest is recorded.
const CORNER_TIME: Duration = Duration::from_millis(1500);
const SETTLE_TIME: Duration = Duration::from_millis(500);
/// A moment at the start to raise your hand into view.
const GET_READY_TIME: Duration = Duration::from_secs(2);
/// A corner needs at least this many pictures with a hand in them (one second is about 30).
const MIN_SAMPLES_PER_CORNER: usize = 8;
/// The box is shrunk by this share of its size on every side, so the screen edges are
/// reachable without stretching your arm all the way.
const INWARD_MARGIN: f32 = 0.05;
/// A box smaller than this (in frame widths) means you did not really move your hand.
const MIN_BOX_WIDTH: f32 = 0.05;
const MIN_BOX_HEIGHT: f32 = 0.03;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

/// The order we ask for: clockwise from the top-left.
const CORNERS: [Corner; 4] = [Corner::TopLeft, Corner::TopRight, Corner::BottomRight, Corner::BottomLeft];

impl Corner {
    fn name(self) -> &'static str {
        match self {
            Corner::TopLeft => "TOP-LEFT",
            Corner::TopRight => "TOP-RIGHT",
            Corner::BottomRight => "BOTTOM-RIGHT",
            Corner::BottomLeft => "BOTTOM-LEFT",
        }
    }
}

/// Knuckle positions (in reach-box units) recorded for each corner, in CORNERS order.
type Samples = [Vec<(f32, f32)>; 4];

/// Why a calibration could not be used. These are not crashes: we fall back to the built-in box.
#[derive(Debug, PartialEq)]
enum CalibrationProblem {
    NotEnoughHand(Corner),
    Misordered,
    TooSmall,
}

impl fmt::Display for CalibrationProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalibrationProblem::NotEnoughHand(corner) => {
                write!(f, "I could not see your hand for the {} corner", corner.name())
            }
            CalibrationProblem::Misordered => write!(f, "the four corners were not in the expected order"),
            CalibrationProblem::TooSmall => write!(f, "your hand barely moved between corners"),
        }
    }
}

/// The middle value of a list (less upset by one bad picture than the average).
fn median(values: &[f32]) -> f32 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    sorted[sorted.len() / 2]
}

/// The typical knuckle position for one corner.
fn corner_spot(samples: &[(f32, f32)]) -> (f32, f32) {
    let xs: Vec<f32> = samples.iter().map(|s| s.0).collect();
    let ys: Vec<f32> = samples.iter().map(|s| s.1).collect();
    (median(&xs), median(&ys))
}

/// Build the reach box from the four recorded corners.
fn build_box(samples: &Samples) -> std::result::Result<ReachBox, CalibrationProblem> {
    for (corner, recorded) in CORNERS.iter().zip(samples) {
        if recorded.len() < MIN_SAMPLES_PER_CORNER {
            return Err(CalibrationProblem::NotEnoughHand(*corner));
        }
    }
    let [top_left, top_right, bottom_right, bottom_left] = [0, 1, 2, 3].map(|i| corner_spot(&samples[i]));

    // Each edge of the box is the average of the two corners on that side.
    let left = (top_left.0 + bottom_left.0) / 2.0;
    let right = (top_right.0 + bottom_right.0) / 2.0;
    let top = (top_left.1 + top_right.1) / 2.0;
    let bottom = (bottom_left.1 + bottom_right.1) / 2.0;
    if right <= left || bottom <= top {
        return Err(CalibrationProblem::Misordered);
    }
    let (width, height) = (right - left, bottom - top);
    if width < MIN_BOX_WIDTH || height < MIN_BOX_HEIGHT {
        return Err(CalibrationProblem::TooSmall);
    }
    Ok(ReachBox {
        x0: left + width * INWARD_MARGIN,
        x1: right - width * INWARD_MARGIN,
        y0: top + height * INWARD_MARGIN,
        y1: bottom - height * INWARD_MARGIN,
    })
}

// ---- remembering the box between runs ----

/// One line of text: "x0 x1 y0 y1".
fn format_box(reach: &ReachBox) -> String {
    format!("{:.4} {:.4} {:.4} {:.4}\n", reach.x0, reach.x1, reach.y0, reach.y1)
}

/// Read the text back. Anything odd gives `None`, and we simply calibrate again.
fn parse_box(text: &str) -> Option<ReachBox> {
    let numbers: Vec<f32> = text.split_whitespace().map(|word| word.parse().ok()).collect::<Option<_>>()?;
    let [x0, x1, y0, y1] = numbers[..] else {
        return None;
    };
    let sensible = [x0, x1, y0, y1].iter().all(|v| v.is_finite() && (-1.0..=2.0).contains(v));
    (sensible && x0 < x1 && y0 < y1).then_some(ReachBox { x0, x1, y0, y1 })
}

/// Where the box is saved: the user's config folder (on a Mac, ~/Library/Application Support).
fn default_box_file() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("touchless-mac").join("reach-box.txt"))
}

fn save_box(path: &Path, reach: &ReachBox) -> Result<()> {
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder)?;
    }
    std::fs::write(path, format_box(reach))?;
    Ok(())
}

fn load_box(path: &Path) -> Option<ReachBox> {
    parse_box(&std::fs::read_to_string(path).ok()?)
}

// ---- the live part ----

/// Decide which reach box to use for this run, following the command-line choice.
pub fn choose_reach_box(mode: CalibrationMode, webcam: &mut Webcam, tracker: &mut Tracker) -> Result<ReachBox> {
    let file = default_box_file();
    match mode {
        CalibrationMode::Skip => {
            println!("Calibration skipped. Using the built-in reach box.");
            return Ok(ReachBox::DEFAULT);
        }
        CalibrationMode::UseSaved => {
            if let Some(saved) = file.as_deref().and_then(load_box) {
                println!("Using your saved calibration. (Run with --recalibrate to redo it.)");
                return Ok(saved);
            }
        }
        CalibrationMode::Redo => {}
    }

    let samples = record_corners(webcam, tracker)?;
    let reach = match build_box(&samples) {
        Ok(reach) => reach,
        Err(problem) => return Ok(fall_back(&problem.to_string())),
    };
    println!("Calibration done.");
    match file {
        Some(path) => match save_box(&path, &reach) {
            Ok(()) => println!("Saved to {}", path.display()),
            Err(error) => eprintln!("Could not save the calibration ({error}). You will be asked again next time."),
        },
        None => eprintln!("No config folder found, so the calibration is not saved."),
    }
    Ok(reach)
}

fn fall_back(reason: &str) -> ReachBox {
    eprintln!("Calibration did not work: {reason}.");
    eprintln!("Using the built-in reach box instead. Run with --recalibrate to try again.");
    ReachBox::DEFAULT
}

/// Ask for each corner in turn and record the knuckle.
fn record_corners(webcam: &mut Webcam, tracker: &mut Tracker) -> Result<Samples> {
    println!("\nCalibration (about 6 seconds). Raise your hand so the camera can see it.");
    println!("You will point your index finger at the four corners of the SCREEN, one by one.");
    watch_for(GET_READY_TIME, webcam, tracker, |_, _| {})?;

    let mut samples: Samples = Default::default();
    for (slot, corner) in samples.iter_mut().zip(CORNERS) {
        println!("Point at the {} corner and hold", corner.name());
        watch_for(CORNER_TIME, webcam, tracker, |elapsed, position| {
            if elapsed >= SETTLE_TIME {
                slot.push(position);
            }
        })?;
        println!("  {} spots recorded", slot.len());
    }
    Ok(samples)
}

/// Look through the camera for `duration`, calling `on_hand(elapsed, knuckle position)` for
/// every picture that has a hand in it.
fn watch_for(
    duration: Duration,
    webcam: &mut Webcam,
    tracker: &mut Tracker,
    mut on_hand: impl FnMut(Duration, (f32, f32)),
) -> Result<()> {
    let started = Instant::now();
    while started.elapsed() < duration {
        let frame = webcam.grab()?;
        if let Some(hand) = tracker.see(&frame)? {
            on_hand(started.elapsed(), hand_position(hand.points[INDEX_KNUCKLE], frame.width() as f32));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Thirty slightly shaky readings around a spot.
    fn around(x: f32, y: f32) -> Vec<(f32, f32)> {
        (0..30).map(|i| (x + 0.002 * ((i % 3) as f32 - 1.0), y + 0.002 * ((i % 5) as f32 - 2.0))).collect()
    }

    /// A normal calibration: the hand covers x 0.40 to 0.60 and y 0.20 to 0.40.
    fn good_samples() -> Samples {
        [around(0.40, 0.20), around(0.60, 0.20), around(0.60, 0.40), around(0.40, 0.40)]
    }

    #[test]
    fn box_comes_from_the_four_corners_with_a_small_margin() {
        let reach = build_box(&good_samples()).unwrap();
        // Width 0.20 and height 0.20, so the 5% margin is 0.01 on each side.
        assert!((reach.x0 - 0.41).abs() < 0.003 && (reach.x1 - 0.59).abs() < 0.003, "{reach:?}");
        assert!((reach.y0 - 0.21).abs() < 0.003 && (reach.y1 - 0.39).abs() < 0.003, "{reach:?}");
    }

    #[test]
    fn one_bad_picture_does_not_move_the_box() {
        let mut samples = good_samples();
        samples[0].push((0.95, 0.95)); // the hand was lost for a moment and the model jumped
        let reach = build_box(&samples).unwrap();
        assert!((reach.x0 - 0.41).abs() < 0.003, "{reach:?}");
    }

    #[test]
    fn the_corners_you_pointed_at_reach_the_corners_of_the_screen() {
        let reach = build_box(&good_samples()).unwrap();
        assert_eq!(reach.map(0.40, 0.20), (0.0, 0.0), "pointing at the top-left corner");
        assert_eq!(reach.map(0.60, 0.40), (1.0, 1.0), "pointing at the bottom-right corner");
        let (middle_x, middle_y) = reach.map(0.50, 0.30);
        assert!((middle_x - 0.5).abs() < 0.01 && (middle_y - 0.5).abs() < 0.01);
    }

    #[test]
    fn a_corner_with_too_few_pictures_fails() {
        let mut samples = good_samples();
        samples[2].truncate(7); // under a quarter of a second of hand
        assert_eq!(build_box(&samples), Err(CalibrationProblem::NotEnoughHand(Corner::BottomRight)));
        samples[2] = around(0.60, 0.40);
        samples[2].truncate(8);
        assert!(build_box(&samples).is_ok(), "eight pictures is enough");
    }

    #[test]
    fn corners_in_the_wrong_places_fail() {
        // Left and right swapped: the "right" edge ends up left of the "left" edge.
        let swapped = [around(0.60, 0.20), around(0.40, 0.20), around(0.40, 0.40), around(0.60, 0.40)];
        assert_eq!(build_box(&swapped), Err(CalibrationProblem::Misordered));
    }

    #[test]
    fn a_hand_that_did_not_move_fails() {
        let still = [around(0.5, 0.3), around(0.5, 0.3), around(0.5, 0.3), around(0.5, 0.3)];
        assert!(matches!(build_box(&still), Err(CalibrationProblem::Misordered | CalibrationProblem::TooSmall)));
        let tiny = [around(0.49, 0.29), around(0.51, 0.29), around(0.51, 0.31), around(0.49, 0.31)];
        assert_eq!(build_box(&tiny), Err(CalibrationProblem::TooSmall));
    }

    #[test]
    fn the_box_survives_being_written_and_read_back() {
        let reach = ReachBox { x0: 0.4123, x1: 0.6, y0: 0.21, y1: 0.39 };
        let back = parse_box(&format_box(&reach)).unwrap();
        assert!((back.x0 - reach.x0).abs() < 1e-4 && (back.y1 - reach.y1).abs() < 1e-4);
    }

    #[test]
    fn damaged_box_files_are_ignored() {
        assert_eq!(parse_box(""), None);
        assert_eq!(parse_box("0.4 0.6 0.2"), None);
        assert_eq!(parse_box("0.4 0.6 0.2 abc"), None);
        assert_eq!(parse_box("0.6 0.4 0.2 0.4"), None, "x0 must be left of x1");
        assert_eq!(parse_box("0.4 0.6 0.2 NaN"), None);
        assert_eq!(parse_box("0.4 0.6 0.2 0.4 0.9"), None, "too many numbers");
    }

    #[test]
    fn save_then_load_through_a_real_file() {
        let folder = std::env::temp_dir().join(format!("touchless-mac-test-{}", std::process::id()));
        let path = folder.join("nested").join("reach-box.txt");
        let reach = ReachBox { x0: 0.41, x1: 0.59, y0: 0.21, y1: 0.39 };
        save_box(&path, &reach).unwrap();
        assert_eq!(load_box(&path).map(|b| (b.x0, b.y1)), Some((0.41, 0.39)));
        assert_eq!(load_box(&folder.join("missing.txt")), None);
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
