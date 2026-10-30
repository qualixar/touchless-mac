// Chapters 1 and 5: the camera.
//
// Everything here comes from the `nokhwa` crate. We only wrap it so the rest of the
// program can say "give me the next picture" and not care how.
use crate::error::{AppError, Result};
use image::RgbImage;
use nokhwa::Camera;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use std::sync::mpsc;
use std::time::Duration;

/// How long we wait for the person to answer macOS's camera permission box.
const PERMISSION_WAIT: Duration = Duration::from_secs(60);

pub struct Webcam {
    camera: Camera,
}

impl Webcam {
    /// Ask macOS for camera permission, then open the first camera at its fastest setting.
    /// Every step that can fail uses `?`, so the error travels up to `main`, which prints it.
    pub fn open() -> Result<Webcam> {
        if !ask_for_permission()? {
            return Err(AppError::CameraDenied);
        }
        let wanted =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        let mut camera = Camera::new(CameraIndex::Index(0), wanted)?;
        camera.open_stream()?;
        println!("camera: {}", camera.camera_format());
        Ok(Webcam { camera })
    }

    /// Wait for the next picture from the camera.
    pub fn grab(&mut self) -> Result<RgbImage> {
        let buffer = self.camera.frame()?;
        Ok(buffer.decode_image::<RgbFormat>()?)
    }
}

/// The first time, macOS shows a "Terminal would like to access the camera" box. We wait
/// until the user answers it. `Ok(true)` means allowed.
fn ask_for_permission() -> Result<bool> {
    let (answer_tx, answer_rx) = mpsc::channel();
    nokhwa::nokhwa_initialize(move |granted| {
        let _ = answer_tx.send(granted);
    });
    answer_rx.recv_timeout(PERMISSION_WAIT).map_err(|_| {
        AppError::CameraProblem("macOS never answered the camera permission question".to_string())
    })
}
