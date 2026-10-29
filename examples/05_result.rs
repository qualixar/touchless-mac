// Idea 5: Result and the ? operator.
//
// A function that can fail returns Result<T, E>: either Ok(value) or Err(error).
// The `?` operator means "if this failed, return the error from MY function right now".
// Run it with:  cargo run --example 05_result
// The Python version is python/05_try_except.py (try/except around the same mistake).
use std::num::ParseFloatError;

/// Turn text such as "1.5" (centimetres) into metres.
fn cm_to_metres(text: &str) -> Result<f32, ParseFloatError> {
    let cm: f32 = text.trim().parse()?; // a failed parse leaves this function here
    Ok(cm / 100.0)
}

/// Same idea, but with a friendly message for the person running the program.
fn describe_pinch(text: &str) -> Result<&'static str, String> {
    let metres = cm_to_metres(text).map_err(|error| format!("'{text}' is not a distance in cm ({error})"))?;
    Ok(if metres < 0.025 { "pinch" } else { "open hand" })
}

fn main() {
    for text in ["1.5", "8", "eight"] {
        match describe_pinch(text) {
            Ok(label) => println!("{text:>5} cm -> {label}"),
            Err(message) => println!("{text:>5} cm -> problem: {message}"),
        }
    }

    // Uncomment either line to see the compiler stop you:
    // let metres: f32 = cm_to_metres("8"); // error[E0308]: expected `f32`, found `Result<f32, ParseFloatError>`
    // let cm: f32 = "8".parse()?; // error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option`
}
