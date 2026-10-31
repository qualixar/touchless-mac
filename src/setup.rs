// Setup helpers: where the models live, which macOS setting to open, and `touchless-mac doctor`.
//
// A tool you install should work from any folder. So instead of only looking in ./models,
// we look in a short list of places, in order, and use the first one that has both files.
use crate::error::{AppError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub const PALM: &str = "palm.onnx";
pub const HAND: &str = "hand.onnx";

/// The folders we search for the models, in order:
/// 1. ./models (a source checkout, where the tutorial runs `cargo run`),
/// 2. models/ next to the program itself (the release download and the installer),
/// 3. ../share/touchless-mac/models next to the program (a Homebrew-style layout),
/// 4. ~/Library/Application Support/touchless-mac/models.
pub fn model_search_path() -> Vec<PathBuf> {
    let mut places = vec![PathBuf::from("models")];
    if let Some(dir) = program_folder() {
        places.push(dir.join("models"));
        places.push(dir.join("../share/touchless-mac/models"));
    }
    if let Some(dir) = dirs::config_dir() {
        places.push(dir.join("touchless-mac").join("models"));
    }
    places
}

/// The folder that holds the running program, with symlinks resolved, so a link in
/// ~/.local/bin still finds the models installed next to the real file.
fn program_folder() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let real = exe.canonicalize().unwrap_or(exe);
    real.parent().map(Path::to_path_buf)
}

/// Pick the models folder: the one given with --models, or the first place that has both files.
pub fn find_models(explicit: Option<&str>) -> Result<PathBuf> {
    if let Some(dir) = explicit {
        return Ok(PathBuf::from(dir));
    }
    first_with_models(&model_search_path()).ok_or_else(|| {
        AppError::ModelMissing(
            model_search_path()
                .iter()
                .map(|p| format!("  {}", p.display()))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    })
}

fn first_with_models(places: &[PathBuf]) -> Option<PathBuf> {
    places
        .iter()
        .find(|dir| dir.join(PALM).is_file() && dir.join(HAND).is_file())
        .cloned()
}

/// The two privacy panes the app needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pane {
    Camera,
    Accessibility,
}

impl Pane {
    fn url(self) -> &'static str {
        match self {
            Pane::Camera => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Camera"
            }
            Pane::Accessibility => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
            }
        }
    }
}

/// Open System Settings on the right pane. Best effort: if it fails, the printed message
/// already says where to go, so we stay quiet.
pub fn open_settings(pane: Pane) {
    let _ = Command::new("open").arg(pane.url()).status();
}

/// Which pane, if any, fixes this error.
pub fn pane_for(error: &AppError) -> Option<Pane> {
    match error {
        AppError::CameraDenied => Some(Pane::Camera),
        AppError::AccessibilityDenied => Some(Pane::Accessibility),
        _ => None,
    }
}

// macOS answers "may this process control the computer?" without showing any prompt.
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

pub fn accessibility_allowed() -> bool {
    // SAFETY: AXIsProcessTrusted takes no arguments and only reads the permission state.
    unsafe { AXIsProcessTrusted() }
}

/// One line of the doctor report.
pub struct Check {
    pub ok: bool,
    pub what: String,
    pub fix: Option<String>,
}

pub fn print_report(checks: &[Check]) -> bool {
    println!("touchless-mac {} doctor\n", env!("CARGO_PKG_VERSION"));
    for check in checks {
        let mark = if check.ok { "ok  " } else { "FIX " };
        println!("  {mark} {}", check.what);
        if let Some(fix) = &check.fix {
            println!("       {fix}");
        }
    }
    let all_ok = checks.iter().all(|c| c.ok);
    println!(
        "\n{}",
        if all_ok {
            "Everything is ready. Run  touchless-mac  and point at the screen."
        } else {
            "Fix the lines marked FIX, then run  touchless-mac doctor  again."
        }
    );
    all_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_place_with_both_files_wins() {
        let root = std::env::temp_dir().join(format!("tm-setup-{}", std::process::id()));
        let (empty, half, full) = (root.join("a"), root.join("b"), root.join("c"));
        for dir in [&empty, &half, &full] {
            std::fs::create_dir_all(dir).unwrap();
        }
        std::fs::write(half.join(PALM), b"x").unwrap();
        std::fs::write(full.join(PALM), b"x").unwrap();
        std::fs::write(full.join(HAND), b"x").unwrap();
        let found = first_with_models(&[empty.clone(), half.clone(), full.clone()]);
        assert_eq!(found, Some(full));
        assert_eq!(first_with_models(&[empty, half]), None);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn explicit_folder_is_used_as_given() {
        assert_eq!(
            find_models(Some("/tmp/m")).unwrap(),
            PathBuf::from("/tmp/m")
        );
    }

    #[test]
    fn search_starts_with_the_local_models_folder() {
        assert_eq!(model_search_path()[0], PathBuf::from("models"));
    }

    #[test]
    fn only_permission_errors_open_settings() {
        assert_eq!(pane_for(&AppError::CameraDenied), Some(Pane::Camera));
        assert_eq!(
            pane_for(&AppError::AccessibilityDenied),
            Some(Pane::Accessibility)
        );
        assert_eq!(pane_for(&AppError::ModelMissing("x".into())), None);
    }
}
