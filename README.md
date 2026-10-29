# touchless-mac

Control your Mac with your hand. Point your index finger to move the cursor, pinch to click, wave to
switch apps. It runs on the camera already in your Mac, with no extra hardware.

This is also a Rust tutorial. The code is split into six tagged chapters, and each one adds a visible
ability to the app while teaching one Rust idea. It is the code from the YouTube tutorial
"Learn Rust by building a touchless Mac" (link to be added when the video is published).

Air gestures are not a touchscreen. The cursor still wobbles a little, and very small targets are hard to hit.

## What you need

A Mac running macOS with a built-in or USB camera, and Rust installed through [rustup](https://rustup.rs).
The edition used is 2024, so you need Rust 1.85 or newer (`rustup update` if unsure).

## Run it

    git clone <this repository>
    cd touchless-mac
    bash scripts/get-models.sh      # downloads the two hand models, about 8 MB
    cargo run --release

The first build takes a few minutes because Cargo downloads and compiles the crates. Then macOS asks for two permissions:

1. **Camera.** Allow it when the box appears. If you said no by mistake: System Settings, Privacy & Security, Camera.
2. **Accessibility,** which lets the app move the mouse and press keys. System Settings, Privacy & Security, Accessibility.

Both permissions belong to the app you started the program from, usually Terminal (or iTerm, VS Code, and so on).
If a permission is missing, the app says which one and where to switch it on; it does not crash.

## Calibration

The first time, the app asks you to point your index finger at the four corners of the screen, one after another
(about six seconds). Follow the prompts in the terminal, for example "Point at the TOP-LEFT corner and hold".
It records where your index knuckle is for each corner and builds your personal "reach box" with a small margin inside,
so you can reach the whole screen, including the Dock, without stretching your arm.

The box is saved in your config folder (`~/Library/Application Support/touchless-mac/reach-box.txt`) and reused next time.

    cargo run --release -- --recalibrate    # measure again
    cargo run --release -- --no-calibrate   # skip it and use the built-in box

If calibration fails (for example, the camera could not see your hand), the app says so and uses the built-in box,
which was measured on the author's desk.

## Gestures

| Gesture | What happens | macOS shortcut |
|---|---|---|
| Point with the index finger | The cursor follows your index knuckle | none |
| Pinch thumb and index finger | Click, where the cursor was a moment before the fingers closed | none |
| Two quick pinches, close together | Double-click (opens a file or app) | none |
| Pinch thumb and middle finger | Right-click | none |
| Open hand, wave right | Next app | Cmd+Tab |
| Open hand, wave left | Previous app | Cmd+Shift+Tab |
| Open hand, wave up | Mission Control | Ctrl+Up |

A pinch counts at under 2.5 cm between the fingertips, measured in 3D from the hand model, so tilting your hand does not change it.
The cursor holds still while a pinch is down. Stop with Ctrl-C.

## The chapters

Check out a tag to see the code as it stood at the end of that chapter, for example `git checkout ch3-option`.

| Tag | Rust idea | The app gains |
|---|---|---|
| `ch1-cargo` | Cargo and crates | It sees the camera |
| `ch2-struct` | `struct`, `let` and `let mut` | The cursor follows your finger |
| `ch3-option` | `Option` | The hand leaves the picture, the cursor waits |
| `ch4-enum` | `enum` and `match` | Pinch, double pinch, right pinch and waves |
| `ch5-result` | `Result` and `?` | Clear messages for missing permissions and models |
| `ch6-calibrate` | Putting it together | Start-up calibration |

`examples/` has one tiny program per idea that needs no camera: `cargo run --example 01_cargo`, then `02_struct_mut`,
`03_option`, `04_enum_match` and `05_result`. Each file has a commented-out line; uncomment it to see the real compiler error.
`python/` has the Python versions shown first in the video. `03_none_crash.py` crashes on purpose.

## Where things are

| File | What it does |
|---|---|
| `src/camera.rs` | Opens the camera and hands out pictures |
| `src/hand.rs` | Runs the palm and hand models, tracks the hand between pictures |
| `src/steer.rs` | Reach box and the One Euro smoothing filter: hand position to screen position |
| `src/gesture.rs` | `Gesture` enum, pinch debounce, double-click rule, wave detector |
| `src/mouse.rs` | Moves the cursor, clicks and presses shortcuts |
| `src/error.rs` | Our error type, with messages that say what to do |
| `src/calibrate.rs` | Four-corner calibration, saved between runs |
| `src/options.rs`, `src/selftest.rs` | Command-line options and a camera-free check |

## Check your install without a camera

    cargo run --release -- --selftest some-photo-of-a-hand.jpg

This runs the same hand finder on a photo and prints where the index fingertip is.
It needs neither camera nor mouse permission. Run the tests with `cargo test`.

## Credits

The hand tracking uses Google's MediaPipe hand models (palm detection and 21-point hand landmarks),
converted to ONNX by OpenCV and run with ONNX Runtime through the `ort` crate. The pipeline follows MediaPipe's:
a rotated crop, then tracking from the palm joints. The pinch and smoothing numbers come from measuring this app against it.

- Google MediaPipe hand models: Apache License 2.0 (to be confirmed).
- OpenCV ONNX conversions of those models (`opencv/palm_detection_mediapipe`, `opencv/handpose_estimation_mediapipe` on Hugging Face): licence to be confirmed.
- The One Euro filter is from Casiez, Roussel and Vogel, CHI 2012.

The models are not stored in this repository; `scripts/get-models.sh` downloads them from Hugging Face and checks their SHA-256 fingerprints.

## Licence

The code in this repository is MIT licensed. See [LICENSE](LICENSE).
