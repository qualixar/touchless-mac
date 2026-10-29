# touchless-mac

Control your Mac with your hand, built one Rust idea at a time. This is the code from the
YouTube tutorial "Learn Rust by building a touchless Mac".

This is chapter 5 (`ch5-result`): `Result` and `?`. A missing Camera or Accessibility permission, a missing
model file or a broken camera now gives a clear message that says what to do, instead of a crash.

    bash scripts/get-models.sh
    cargo run --release
    cargo run --example 05_result

macOS will ask for Camera and Accessibility permission for your Terminal app.
The next tag adds calibration; the final README lives on the last tag.
