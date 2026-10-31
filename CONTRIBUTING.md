# Contributing

Thanks for helping. touchless-mac is also the companion code for a beginner Rust tutorial, so code that is easy to read wins over code that is clever.

## Set up

```sh
git clone https://github.com/qualixar/touchless-mac.git && cd touchless-mac
bash scripts/get-models.sh
cargo run --release -- doctor
```

## Before you open a pull request

```sh
cargo fmt
cargo clippy --release --all-targets -- -D warnings
cargo test --release
```

CI runs the same three checks on macOS.

## Ground rules

- **One change per pull request,** with a test when the change has logic in it.
- **Explain the Rust.** New code keeps the plain-English comments the rest of the code base has; a beginner should be able to follow it.
- **Leave the chapter tags alone.** The tags `ch1-cargo` to `ch6-calibrate` are the code exactly as each film chapter left it, so changes go on `main` only.
- **Measure gesture changes.** If you change how a gesture is detected, say how you checked it doesn't fire by accident. The scroll pose was chosen from recordings of ordinary pointing for exactly this reason.
- **Keep it local.** No network calls at run time. The app never sends camera frames anywhere.

## Reporting bugs

Use the bug report form and paste the output of `touchless-mac doctor`. For security problems, see [SECURITY.md](SECURITY.md).
