# touchless-mac

Control your Mac with your hand, built one Rust idea at a time. This is the code from the
YouTube tutorial "Learn Rust by building a touchless Mac".

This is chapter 2 (`ch2-struct`): `struct` and `let mut`. The cursor follows your index finger.
There are no clicks yet, and the program stops if your hand leaves the picture (chapter 3 fixes that).

    bash scripts/get-models.sh
    cargo run --release
    cargo run --example 02_struct_mut

macOS will ask for Camera and Accessibility permission for your Terminal app.
Later tags add safe no-hand handling, gestures, error messages and calibration; the final README lives on the last tag.
