// Chapter 1: the camera.
//
// Everything here comes from the `nokhwa` crate. We only wrap it so the rest of the
// program can say "give me the next picture" and not care how.
use image::RgbImage;
use nokhwa::Camera;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use std::sync::mpsc;
use std::time::Duration;

pub struct Webcam {
    camera: Camera,
}

impl Webcam {
    /// Ask macOS for camera permission, then open the first camera at its fastest setting.
    ///
    /// For now a failure stops the program with a message (`expect`). Chapter 5 replaces
    /// this with proper error handling.
    pub fn open() -> Webcam {
        // The first time, macOS shows a "Terminal would like to access the camera" box.
        // We wait here until the user answers it.
        let (answer_tx, answer_rx) = mpsc::channel();
        nokhwa::nokhwa_initialize(move |granted| {
            let _ = answer_tx.send(granted);
        });
        let granted = answer_rx
            .recv_timeout(Duration::from_secs(60))
            .expect("macOS never answered the camera permission question");
        assert!(granted, "Camera access was not allowed for this app");

        let wanted = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        let mut camera = Camera::new(CameraIndex::Index(0), wanted).expect("could not find a camera");
        camera.open_stream().expect("could not start the camera");
        println!("camera: {}", camera.camera_format());
        Webcam { camera }
    }

    /// Wait for the next picture from the camera.
    pub fn grab(&mut self) -> RgbImage {
        let buffer = self.camera.frame().expect("the camera stopped sending pictures");
        buffer.decode_image::<RgbFormat>().expect("could not read the camera picture")
    }
}
