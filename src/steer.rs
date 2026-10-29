// Chapter 2: from "where is my finger in the picture" to "where is the cursor on the screen".
//
// Two steps:
//   1. A reach box says which part of the camera picture counts as "the whole screen".
//   2. A One Euro filter smooths the shaking out of the hand's position.
use crate::hand::Point;

/// The part of the camera picture your hand moves in, in "frame widths" (1.0 = the full
/// picture width, for both x and y so that a move means the same distance in each direction).
/// x is already mirrored, so moving your hand to the right moves x up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReachBox {
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
}

impl ReachBox {
    /// The box measured on the author's desk. Calibration (chapter 6) replaces it with yours.
    pub const DEFAULT: ReachBox = ReachBox { x0: 0.38, x1: 0.64, y0: 0.20, y1: 0.40 };

    /// Where a hand position falls inside the box: (0, 0) is the top-left corner,
    /// (1, 1) the bottom-right. Positions outside the box are held at the edge.
    pub fn map(&self, x: f32, y: f32) -> (f32, f32) {
        let nx = ((x - self.x0) / (self.x1 - self.x0)).clamp(0.0, 1.0);
        let ny = ((y - self.y0) / (self.y1 - self.y0)).clamp(0.0, 1.0);
        (nx, ny)
    }
}

/// One Euro filter (Casiez et al., CHI 2012). It smooths a lot when the hand moves slowly
/// (so the cursor doesn't shake) and very little when it moves fast (so it doesn't lag).
pub struct OneEuro {
    min_cutoff: f32,
    beta: f32,
    smoothed: Option<f32>,
    speed: f32,
}

impl OneEuro {
    pub fn new(min_cutoff: f32, beta: f32) -> OneEuro {
        OneEuro { min_cutoff, beta, smoothed: None, speed: 0.0 }
    }

    /// How much of the new value to let in. 1.0 means "use it as it is".
    fn alpha(cutoff: f32, dt: f32) -> f32 {
        let tau = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
        1.0 / (1.0 + tau / dt)
    }

    /// Smooth one new value. `dt` is the time since the last value, in seconds.
    pub fn filter(&mut self, value: f32, dt: f32) -> f32 {
        let Some(previous) = self.smoothed else {
            self.smoothed = Some(value); // the very first value passes straight through
            return value;
        };
        self.speed += Self::alpha(1.0, dt) * ((value - previous) / dt - self.speed);
        let cutoff = self.min_cutoff + self.beta * self.speed.abs();
        let smoothed = previous + Self::alpha(cutoff, dt) * (value - previous);
        self.smoothed = Some(smoothed);
        smoothed
    }

    /// Forget the past. Used when the hand disappears, so the cursor doesn't glide
    /// from the old spot when the hand comes back.
    pub fn reset(&mut self) {
        self.smoothed = None;
        self.speed = 0.0;
    }
}

/// Turns the index knuckle's position in the camera picture into a screen position.
pub struct Steering {
    reach: ReachBox,
    screen: (f32, f32),
    filter_x: OneEuro,
    filter_y: OneEuro,
}

impl Steering {
    pub fn new(reach: ReachBox, screen_width: i32, screen_height: i32) -> Steering {
        // Steadier when nearly still (min cutoff 0.6 Hz), still quick on big moves (beta 10).
        Steering {
            reach,
            screen: (screen_width as f32, screen_height as f32),
            filter_x: OneEuro::new(0.6, 10.0),
            filter_y: OneEuro::new(0.6, 10.0),
        }
    }

    /// The screen position for this knuckle position.
    pub fn aim(&mut self, knuckle: Point, frame_width: f32, dt: f32) -> (f32, f32) {
        // We steer with the index knuckle: it barely moves when you pinch, unlike the fingertip.
        // Both axes are divided by the frame WIDTH so a move means the same distance either way.
        let x = self.filter_x.filter(1.0 - knuckle.x / frame_width, dt); // 1.0 - x mirrors the picture
        let y = self.filter_y.filter(knuckle.y / frame_width, dt);
        let (nx, ny) = self.reach.map(x, y);
        (nx * (self.screen.0 - 1.0), ny * (self.screen.1 - 1.0))
    }

    /// The hand left the picture: forget the smoothing.
    pub fn lost(&mut self) {
        self.filter_x.reset();
        self.filter_y.reset();
    }
}
