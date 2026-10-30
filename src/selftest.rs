// A camera-free check of the hand finder: run it on a photo of a hand.
//
//   cargo run --release -- --selftest path/to/photo.jpg
//
// It prints where the index fingertip is and how far thumb and index are apart. It is the same
// code path as the live app (palm search, then the hand model, then one tracking step), so it
// is a quick way to see that your models and ONNX Runtime work, without a camera or permissions.
use crate::error::{AppError, Result};
use crate::hand::{Detector, INDEX_TIP, Point, THUMB_TIP};
use image::RgbImage;

pub struct StillReport {
    pub confidence: f32,
    pub index_tip: Point,
    pub thumb_to_index_cm: f32,
    /// How far the index tip moves when we run the model again from its own tracking crop.
    pub tracking_shift_px: f32,
}

/// Find the hand in a still picture. `None` means no hand was found.
pub fn check_still(detector: &mut Detector, picture: &RgbImage) -> Result<Option<StillReport>> {
    let Some(crop) = detector.find_palm(picture)? else {
        return Ok(None);
    };
    let Some(hand) = detector.landmarks(picture, crop)? else {
        return Ok(None);
    };
    let tip = hand.points[INDEX_TIP];

    // The tracking step: look again, this time from the crop the live app would use next frame.
    let shift = match detector.landmarks(picture, hand.next_crop())? {
        Some(again) => (again.points[INDEX_TIP].x - tip.x).hypot(again.points[INDEX_TIP].y - tip.y),
        None => f32::NAN,
    };
    Ok(Some(StillReport {
        confidence: hand.confidence,
        index_tip: tip,
        thumb_to_index_cm: hand.distance_m(THUMB_TIP, INDEX_TIP) * 100.0,
        tracking_shift_px: shift,
    }))
}

/// Run the check on a photo file and print the result.
pub fn run(detector: &mut Detector, path: &str) -> Result<()> {
    let picture = image::open(path)
        .map_err(|error| AppError::FileProblem(format!("could not open {path}: {error}")))?
        .to_rgb8();
    match check_still(detector, &picture)? {
        Some(r) => println!(
            "{path}: hand {:.2}, index tip at ({:.0}, {:.0}), thumb-index {:.1} cm, tracking re-run moves it {:.1} px",
            r.confidence, r.index_tip.x, r.index_tip.y, r.thumb_to_index_cm, r.tracking_shift_px
        ),
        None => println!("{path}: no hand found"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// The photo checks use Google's MediaPipe sample photos, which are not stored in this repo.
    /// Run them with:
    ///   TOUCHLESS_PHOTOS=/folder/with/the/photos cargo test --release -- --ignored
    /// The expected numbers are what the first working version of this app measured.
    fn check(name: &str, expected_tip: (f32, f32)) -> StillReport {
        let folder = std::env::var("TOUCHLESS_PHOTOS")
            .expect("set TOUCHLESS_PHOTOS to the folder with the photos");
        let picture = image::open(Path::new(&folder).join(name))
            .expect("photo should open")
            .to_rgb8();
        let mut detector =
            Detector::new(Path::new("models/palm.onnx"), Path::new("models/hand.onnx"))
                .expect("models");
        let report = check_still(&mut detector, &picture)
            .expect("check should run")
            .expect("a hand should be found");
        let miss = (report.index_tip.x - expected_tip.0).hypot(report.index_tip.y - expected_tip.1);
        assert!(
            miss <= 3.0,
            "{name}: index tip at {:?}, expected {expected_tip:?}, off by {miss:.1} px",
            report.index_tip
        );
        report
    }

    #[test]
    #[ignore = "needs the models and the sample photos; see the comment above"]
    fn pointing_up_photo() {
        let report = check("pointing_up.jpg", (168.0, 75.0));
        assert!(
            (report.thumb_to_index_cm - 8.0).abs() < 0.5,
            "thumb-index {:.1} cm",
            report.thumb_to_index_cm
        );
    }

    #[test]
    #[ignore = "needs the models and the sample photos; see the comment above"]
    fn tilted_photo_keeps_its_finger_and_its_distance() {
        let report = check("rot135.jpg", (343.0, 330.0));
        assert!(
            (report.thumb_to_index_cm - 8.1).abs() < 0.5,
            "tilt must not change the distance"
        );
    }

    #[test]
    #[ignore = "needs the models and the sample photos; see the comment above"]
    fn victory_photo() {
        check("victory.jpg", (198.0, 62.0));
    }
}
