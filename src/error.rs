// Chapter 5: Result and `?`.
//
// A function that can fail returns `Result<T, E>`: either `Ok(value)` or `Err(error)`.
// The `?` operator means "if this failed, return the error from MY function right now".
// This file defines our own error type, so every failure comes with a message that tells
// the user what to do about it.
use std::fmt;

/// Shorthand: `Result<Hand>` means `Result<Hand, AppError>`.
pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    /// macOS Camera permission was refused.
    CameraDenied,
    /// The camera is missing, busy or stopped working. Holds the technical detail.
    CameraProblem(String),
    /// macOS Accessibility permission (needed to move the mouse) was refused.
    AccessibilityDenied,
    MouseProblem(String),
    /// A model file is not where we looked. Holds the path.
    ModelMissing(String),
    ModelProblem(String),
    FileProblem(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::CameraDenied => write!(
                f,
                "The camera is not allowed.\n\
                 Open System Settings > Privacy & Security > Camera, switch on the app you started\n\
                 this from (usually Terminal), then run the program again."
            ),
            AppError::CameraProblem(detail) => write!(
                f,
                "The camera did not work: {detail}\n\
                 Check that a camera is connected and that no other app (Zoom, FaceTime) is using it."
            ),
            AppError::AccessibilityDenied => write!(
                f,
                "macOS does not let this app control the mouse yet.\n\
                 Open System Settings > Privacy & Security > Accessibility, switch on the app you\n\
                 started this from (usually Terminal), then run the program again.\n\
                 If it is already on, switch it off and on again."
            ),
            AppError::MouseProblem(detail) => write!(f, "Could not control the mouse: {detail}"),
            AppError::ModelMissing(path) => write!(
                f,
                "Cannot find the hand model file: {path}\n\
                 Download the models with:  bash scripts/get-models.sh\n\
                 and run the program from the folder that contains the models/ folder."
            ),
            AppError::ModelProblem(detail) => write!(
                f,
                "A hand model could not be loaded or run: {detail}\n\
                 If the download was interrupted, run  bash scripts/get-models.sh  again."
            ),
            AppError::FileProblem(detail) => write!(f, "A file problem: {detail}"),
        }
    }
}

impl std::error::Error for AppError {}

// These `From` impls are what let `?` turn a library's error into ours automatically.

impl From<nokhwa::NokhwaError> for AppError {
    fn from(error: nokhwa::NokhwaError) -> AppError {
        AppError::CameraProblem(error.to_string())
    }
}

impl From<enigo::NewConError> for AppError {
    fn from(error: enigo::NewConError) -> AppError {
        match error {
            enigo::NewConError::NoPermission => AppError::AccessibilityDenied,
            other => AppError::MouseProblem(other.to_string()),
        }
    }
}

impl From<enigo::InputError> for AppError {
    fn from(error: enigo::InputError) -> AppError {
        AppError::MouseProblem(error.to_string())
    }
}

impl<R> From<ort::Error<R>> for AppError {
    fn from(error: ort::Error<R>) -> AppError {
        AppError::ModelProblem(error.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> AppError {
        AppError::FileProblem(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_accessibility_permission_explains_where_to_switch_it_on() {
        let error: AppError = enigo::NewConError::NoPermission.into();
        assert!(matches!(error, AppError::AccessibilityDenied));
        let message = error.to_string();
        assert!(message.contains("Accessibility") && message.contains("System Settings"), "{message}");
    }

    #[test]
    fn other_mouse_setup_errors_keep_their_detail() {
        let error: AppError = enigo::NewConError::Reply.into();
        assert!(matches!(&error, AppError::MouseProblem(d) if d.contains("reply")), "{error}");
    }

    #[test]
    fn camera_denied_names_the_privacy_setting() {
        let message = AppError::CameraDenied.to_string();
        assert!(message.contains("Privacy & Security > Camera"), "{message}");
    }

    #[test]
    fn missing_model_names_the_file_and_the_download_script() {
        let message = AppError::ModelMissing("models/palm.onnx".into()).to_string();
        assert!(message.contains("models/palm.onnx") && message.contains("scripts/get-models.sh"), "{message}");
    }

    #[test]
    fn question_mark_converts_library_errors() {
        fn read(path: &str) -> Result<String> {
            Ok(std::fs::read_to_string(path)?) // io::Error becomes AppError by itself
        }
        let error = read("/definitely/not/here.txt").unwrap_err();
        assert!(matches!(error, AppError::FileProblem(_)));
    }
}
