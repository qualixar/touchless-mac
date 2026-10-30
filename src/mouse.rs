// Chapters 2, 4 and 5: moving the Mac's cursor, clicking and pressing keys.
//
// `enigo` sends real mouse and keyboard events to macOS. macOS only allows that if you switch
// on "Accessibility" for the app that started us (usually Terminal). See the README.
use crate::error::Result;
use crate::gesture::{Direction, Event, Gesture};
use enigo::{
    Axis, Button, Coordinate, Direction as Press, Enigo, Key, Keyboard, Mouse as _, Settings,
};

/// The cursor moves only when it would travel at least this many points. It lets the
/// cursor rest on a small target while your hand wobbles a little.
const DEAD_ZONE: f32 = 3.0;

pub struct Mouse {
    enigo: Enigo,
    shown: Option<(f32, f32)>, // where we last put the cursor when pointing
    scroll_sign: i32,          // -1 after --invert-scroll
}

impl Mouse {
    /// Without macOS Accessibility permission this fails with `AppError::AccessibilityDenied`
    /// (see error.rs), which explains where to switch it on.
    pub fn new() -> Result<Mouse> {
        let enigo = Enigo::new(&Settings::default())?;
        Ok(Mouse {
            enigo,
            shown: None,
            scroll_sign: 1,
        })
    }

    /// Flip the scroll direction (for `--invert-scroll`).
    pub fn invert_scroll(&mut self) {
        self.scroll_sign = -self.scroll_sign;
    }

    pub fn screen_size(&self) -> Result<(i32, i32)> {
        Ok(self.enigo.main_display()?)
    }

    /// Do what a gesture means. `match` must cover every Gesture: add a new one to the enum
    /// and the compiler points here until we say what it does.
    pub fn perform(&mut self, event: Event) -> Result<()> {
        let (x, y) = event.at;
        match event.gesture {
            Gesture::Point => self.point_at(x, y),
            Gesture::Pinch => {
                self.jump_to(x, y)?;
                self.click(Button::Left)
            }
            Gesture::DoublePinch => {
                // Two instant clicks at the same spot: macOS reads them as a double-click.
                self.jump_to(x, y)?;
                self.click(Button::Left)?;
                self.click(Button::Left)
            }
            Gesture::RightPinch => {
                self.jump_to(x, y)?;
                self.click(Button::Right)
            }
            Gesture::Wave(Direction::Right) => self.shortcut(&[Key::Meta], Key::Tab), // next app
            Gesture::Wave(Direction::Left) => self.shortcut(&[Key::Meta, Key::Shift], Key::Tab), // previous app
            Gesture::Wave(Direction::Up) => self.shortcut(&[Key::Control], Key::UpArrow), // Mission Control
            // enigo counts positive lines as "scroll down", which moves the page up.
            Gesture::Scroll(lines) => Ok(self
                .enigo
                .scroll(lines * self.scroll_sign, Axis::Vertical)?),
        }
    }

    /// Move the cursor, unless the move is smaller than the dead zone.
    fn point_at(&mut self, x: f32, y: f32) -> Result<()> {
        let far_enough = match self.shown {
            Some((shown_x, shown_y)) => (x - shown_x).hypot(y - shown_y) >= DEAD_ZONE,
            None => true, // the very first move always goes through
        };
        if !far_enough {
            return Ok(());
        }
        self.jump_to(x, y)?;
        self.shown = Some((x, y));
        Ok(())
    }

    fn jump_to(&mut self, x: f32, y: f32) -> Result<()> {
        Ok(self.enigo.move_mouse(x as i32, y as i32, Coordinate::Abs)?)
    }

    fn click(&mut self, button: Button) -> Result<()> {
        Ok(self.enigo.button(button, Press::Click)?)
    }

    /// Hold the modifier keys, tap the main key, then let go in the opposite order.
    fn shortcut(&mut self, held: &[Key], key: Key) -> Result<()> {
        for modifier in held {
            self.enigo.key(*modifier, Press::Press)?;
        }
        self.enigo.key(key, Press::Click)?;
        for modifier in held.iter().rev() {
            self.enigo.key(*modifier, Press::Release)?;
        }
        Ok(())
    }
}
