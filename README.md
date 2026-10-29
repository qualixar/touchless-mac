# touchless-mac

Control your Mac with your hand, built one Rust idea at a time. This is the code from the
YouTube tutorial "Learn Rust by building a touchless Mac".

This is chapter 3 (`ch3-option`): `Option`. The cursor follows your index finger, and when your hand
leaves the picture the cursor simply waits. There are no clicks yet.

    bash scripts/get-models.sh
    cargo run --release
    cargo run --example 03_option

macOS will ask for Camera and Accessibility permission for your Terminal app.
Later tags add gestures, error messages and calibration; the final README lives on the last tag.
