// Chapter 2: moving the Mac's cursor.
//
// `enigo` sends real mouse events to macOS. macOS only allows that if you switch on
// "Accessibility" for the app that started us (usually Terminal). See the README.
use enigo::{Coordinate, Enigo, Mouse as _, Settings};

/// The cursor moves only when it would travel at least this many points. It lets the
/// cursor rest on a small target while your hand wobbles a little.
const DEAD_ZONE: f32 = 3.0;

pub struct Mouse {
    enigo: Enigo,
    shown: Option<(f32, f32)>, // where we last put the cursor
}

impl Mouse {
    /// For now, missing permission stops the program with a message. Chapter 5 improves that.
    pub fn new() -> Mouse {
        let enigo = Enigo::new(&Settings::default()).expect("macOS Accessibility permission is missing");
        Mouse { enigo, shown: None }
    }

    pub fn screen_size(&self) -> (i32, i32) {
        self.enigo.main_display().expect("could not read the screen size")
    }

    /// Move the cursor, unless the move is smaller than the dead zone.
    pub fn move_to(&mut self, x: f32, y: f32) {
        if let Some((shown_x, shown_y)) = self.shown {
            if (x - shown_x).hypot(y - shown_y) < DEAD_ZONE {
                return;
            }
        }
        self.enigo.move_mouse(x as i32, y as i32, Coordinate::Abs).expect("could not move the mouse");
        self.shown = Some((x, y));
    }
}
