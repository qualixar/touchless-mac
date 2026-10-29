# touchless-mac

Control your Mac with your hand, built one Rust idea at a time. This is the code from the
YouTube tutorial "Learn Rust by building a touchless Mac".

This is chapter 4 (`ch4-enum`): `enum` and `match`. Gestures: point to move, pinch to click, double pinch
to double-click, thumb and middle finger to right-click, wave an open hand to switch apps.

    bash scripts/get-models.sh
    cargo run --release
    cargo run --example 04_enum_match

macOS will ask for Camera and Accessibility permission for your Terminal app.
Later tags add error messages and calibration; the final README lives on the last tag.
