// Chapter 6: command-line options, read without any extra crate.
use crate::error::{AppError, Result};

pub const USAGE: &str = "\
Usage: touchless-mac [options]
       touchless-mac doctor      Check models, camera and permissions, and say what to fix

  --recalibrate        Run the four-corner calibration again and save the result
  --no-calibrate       Skip calibration and use the built-in reach box
  --speed <0.25-4>     Cursor speed: 1 is normal, 2 twice as fast, 0.5 half (default: 1)
  --invert-scroll      Scroll the other way
  --models <folder>    Folder with palm.onnx and hand.onnx (default: found automatically)
  --selftest <photo>.. Check the hand finder on photo files; no camera or mouse needed
  -V, --version        Show the version
  -h, --help           Show this text

Without options, the app uses your saved calibration, or calibrates the first time.";

/// What to do about the reach box at start-up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CalibrationMode {
    /// Use the saved box if there is one, otherwise calibrate (the default).
    UseSaved,
    /// `--recalibrate`: always calibrate and save.
    Redo,
    /// `--no-calibrate`: use the built-in box.
    Skip,
}

#[derive(Debug, PartialEq)]
pub struct Options {
    pub calibration: CalibrationMode,
    /// `--models`: a folder to use instead of searching the usual places.
    pub models_dir: Option<String>,
    pub speed: f32,
    pub invert_scroll: bool,
    pub selftest_images: Vec<String>,
    pub show_help: bool,
    pub show_version: bool,
    /// `touchless-mac doctor`
    pub doctor: bool,
}

impl Options {
    /// Read the arguments that follow the program name.
    pub fn parse(args: &[String]) -> Result<Options> {
        let mut options = Options {
            calibration: CalibrationMode::UseSaved,
            models_dir: None,
            speed: 1.0,
            invert_scroll: false,
            selftest_images: Vec::new(),
            show_help: false,
            show_version: false,
            doctor: false,
        };
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--recalibrate" => options.set_calibration(CalibrationMode::Redo)?,
                "--no-calibrate" => options.set_calibration(CalibrationMode::Skip)?,
                "--speed" => {
                    let value = args
                        .next()
                        .ok_or_else(|| bad("--speed needs a number, for example --speed 1.5"))?;
                    options.speed = value
                        .parse::<f32>()
                        .ok()
                        .filter(|s| (0.25..=4.0).contains(s))
                        .ok_or_else(|| bad("--speed must be a number from 0.25 to 4"))?;
                }
                "--invert-scroll" => options.invert_scroll = true,
                "--models" => {
                    options.models_dir = Some(
                        args.next()
                            .ok_or_else(|| bad("--models needs a folder name"))?
                            .clone(),
                    );
                }
                "--selftest" => {
                    options.selftest_images = args.by_ref().cloned().collect();
                    if options.selftest_images.is_empty() {
                        return Err(bad("--selftest needs at least one photo file"));
                    }
                }
                "-h" | "--help" => options.show_help = true,
                "-V" | "--version" => options.show_version = true,
                "doctor" => options.doctor = true,
                other => return Err(bad(&format!("unknown option '{other}'"))),
            }
        }
        Ok(options)
    }

    fn set_calibration(&mut self, mode: CalibrationMode) -> Result<()> {
        if self.calibration != CalibrationMode::UseSaved && self.calibration != mode {
            return Err(bad(
                "--recalibrate and --no-calibrate cannot be used together",
            ));
        }
        self.calibration = mode;
        Ok(())
    }
}

fn bad(detail: &str) -> AppError {
    AppError::BadArguments(detail.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Options> {
        Options::parse(&args.iter().map(|a| a.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn no_arguments_means_saved_calibration_and_default_models() {
        let options = parse(&[]).unwrap();
        assert_eq!(options.calibration, CalibrationMode::UseSaved);
        assert_eq!(options.models_dir, None);
        assert!(!options.doctor && !options.show_version);
        assert!(options.selftest_images.is_empty() && !options.show_help);
    }

    #[test]
    fn calibration_flags_are_read() {
        assert_eq!(
            parse(&["--recalibrate"]).unwrap().calibration,
            CalibrationMode::Redo
        );
        assert_eq!(
            parse(&["--no-calibrate"]).unwrap().calibration,
            CalibrationMode::Skip
        );
    }

    #[test]
    fn both_calibration_flags_together_are_refused() {
        assert!(parse(&["--recalibrate", "--no-calibrate"]).is_err());
    }

    #[test]
    fn selftest_takes_every_photo_after_it() {
        let options = parse(&["--selftest", "a.jpg", "b.jpg"]).unwrap();
        assert_eq!(options.selftest_images, vec!["a.jpg", "b.jpg"]);
        assert!(parse(&["--selftest"]).is_err());
    }

    #[test]
    fn models_flag_needs_a_folder_and_unknown_flags_are_refused() {
        assert_eq!(
            parse(&["--models", "/tmp/m"])
                .unwrap()
                .models_dir
                .as_deref(),
            Some("/tmp/m")
        );
        assert!(parse(&["--models"]).is_err());
        let message = parse(&["--wobble"]).unwrap_err().to_string();
        assert!(
            message.contains("--wobble") && message.contains("Usage"),
            "{message}"
        );
    }

    #[test]
    fn doctor_and_version_are_read() {
        assert!(parse(&["doctor"]).unwrap().doctor);
        assert!(parse(&["--version"]).unwrap().show_version);
        assert!(parse(&["-V"]).unwrap().show_version);
    }
}
