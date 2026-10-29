// Chapters 2 and 4: moving the Mac's cursor, clicking and pressing keys.
//
// `enigo` sends real mouse and keyboard events to macOS. macOS only allows that if you switch
// on "Accessibility" for the app that started us (usually Terminal). See the README.
use crate::gesture::{Direction, Event, Gesture};
use enigo::{Button, Coordinate, Direction as Press, Enigo, Key, Keyboard, Mouse as _, Settings};

/// The cursor moves only when it would travel at least this many points. It lets the
/// cursor rest on a small target while your hand wobbles a little.
const DEAD_ZONE: f32 = 3.0;

pub struct Mouse {
    enigo: Enigo,
    shown: Option<(f32, f32)>, // where we last put the cursor when pointing
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

    /// Do what a gesture means. `match` must cover every Gesture: add a new one to the enum
    /// and the compiler points here until we say what it does.
    pub fn perform(&mut self, event: Event) {
        let (x, y) = event.at;
        match event.gesture {
            Gesture::Point => self.point_at(x, y),
            Gesture::Pinch => {
                self.jump_to(x, y);
                self.click(Button::Left);
            }
            Gesture::DoublePinch => {
                // Two instant clicks at the same spot: macOS reads them as a double-click.
                self.jump_to(x, y);
                self.click(Button::Left);
                self.click(Button::Left);
            }
            Gesture::RightPinch => {
                self.jump_to(x, y);
                self.click(Button::Right);
            }
            Gesture::Wave(Direction::Right) => self.shortcut(&[Key::Meta], Key::Tab), // next app
            Gesture::Wave(Direction::Left) => self.shortcut(&[Key::Meta, Key::Shift], Key::Tab), // previous app
            Gesture::Wave(Direction::Up) => self.shortcut(&[Key::Control], Key::UpArrow), // Mission Control
        }
    }

    /// Move the cursor, unless the move is smaller than the dead zone.
    fn point_at(&mut self, x: f32, y: f32) {
        if let Some((shown_x, shown_y)) = self.shown {
            if (x - shown_x).hypot(y - shown_y) < DEAD_ZONE {
                return;
            }
        }
        self.jump_to(x, y);
        self.shown = Some((x, y));
    }

    fn jump_to(&mut self, x: f32, y: f32) {
        self.enigo.move_mouse(x as i32, y as i32, Coordinate::Abs).expect("could not move the mouse");
    }

    fn click(&mut self, button: Button) {
        self.enigo.button(button, Press::Click).expect("could not click");
    }

    /// Hold the modifier keys, tap the main key, then let go in the opposite order.
    fn shortcut(&mut self, held: &[Key], key: Key) {
        for modifier in held {
            self.enigo.key(*modifier, Press::Press).expect("could not press a key");
        }
        self.enigo.key(key, Press::Click).expect("could not press a key");
        for modifier in held.iter().rev() {
            self.enigo.key(*modifier, Press::Release).expect("could not release a key");
        }
    }
}
