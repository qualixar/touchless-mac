// Chapter 4: turning hand shapes and movements into gestures.
//
// The idea of this chapter is `enum` and `match`. An enum lists every choice there can be.
// `match` makes the compiler check that we wrote something for EACH choice.
use crate::hand::{Hand, INDEX_TIP, MIDDLE_TIP, THUMB_TIP};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Pinch: press when thumb and finger are closer than 2.5 cm, re-arm when they open past 5 cm.
const PINCH_PRESS_M: f32 = 0.025;
const PINCH_RELEASE_M: f32 = 0.05;
const LEFT_PINCH_FRAMES: u32 = 2;
const RIGHT_PINCH_FRAMES: u32 = 3;

/// A second pinch within 1 second and 40 points of the first one is a double-click.
const DOUBLE_PINCH_WINDOW: Duration = Duration::from_millis(1000);
const DOUBLE_PINCH_RADIUS: f32 = 40.0;
/// After a double-click, ignore pinches for a moment so a bouncing third pinch can't click again.
const QUIET_AFTER_DOUBLE: Duration = Duration::from_millis(400);

/// A click lands where the cursor was this many pictures ago (about 165 ms), before the
/// fingers started closing and dragged the hand.
const CURSOR_HISTORY: usize = 6;

/// A wave must be faster than this many frame-widths per second, and cooldown stops repeats.
const WAVE_SPEED: f32 = 0.9;
const WAVE_COOLDOWN: Duration = Duration::from_millis(800);
const WAVE_LOOK_BACK: Duration = Duration::from_millis(200);
const WAVE_MIN_SPAN: f32 = 0.1; // seconds of movement needed before we judge the speed

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
}

/// Everything the hand can "say". Note that `Wave` carries a Direction inside it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gesture {
    Point,
    Pinch,
    DoublePinch,
    RightPinch,
    Wave(Direction),
}

/// A gesture, and the screen spot where it should happen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Event {
    pub gesture: Gesture,
    pub at: (f32, f32),
}

/// What the hand looked like in one camera picture, boiled down to the numbers gestures need.
pub struct Reading {
    pub now: Instant,
    /// Where the cursor is on the screen.
    pub cursor: (f32, f32),
    pub thumb_to_index_m: f32,
    pub thumb_to_middle_m: f32,
    /// Palm centre in frame widths, with x mirrored (moving your hand right makes x grow).
    pub palm: (f32, f32),
    pub palm_open: bool,
}

impl Reading {
    pub fn new(now: Instant, cursor: (f32, f32), hand: &Hand, frame_width: f32) -> Reading {
        let palm = hand.palm_centre();
        Reading {
            now,
            cursor,
            thumb_to_index_m: hand.distance_m(THUMB_TIP, INDEX_TIP),
            thumb_to_middle_m: hand.distance_m(THUMB_TIP, MIDDLE_TIP),
            palm: (1.0 - palm.x / frame_width, palm.y / frame_width),
            palm_open: hand.is_open_palm(),
        }
    }
}

/// A debounced pinch. It fires once, after the fingers stayed closed for `hold` pictures,
/// and cannot fire again until they have opened wide.
struct Pinch {
    hold: u32,
    closed_for: u32,
    down: bool,
}

impl Pinch {
    fn new(hold: u32) -> Pinch {
        Pinch { hold, closed_for: 0, down: false }
    }

    /// Feed one distance (metres). Gives back `true` on the picture where the pinch fires.
    fn update(&mut self, distance_m: f32) -> bool {
        self.closed_for = if distance_m < PINCH_PRESS_M { self.closed_for + 1 } else { 0 };
        if !self.down && self.closed_for >= self.hold {
            self.down = true;
            return true;
        }
        if self.down && distance_m > PINCH_RELEASE_M {
            self.down = false;
        }
        false
    }
}

/// Open-hand wave detector: how fast the palm centre moved over the last ~200 ms.
struct SwipeDetector {
    trail: VecDeque<(Instant, f32, f32)>,
    cooldown_until: Option<Instant>,
}

impl SwipeDetector {
    fn new() -> SwipeDetector {
        SwipeDetector { trail: VecDeque::new(), cooldown_until: None }
    }

    fn update(&mut self, now: Instant, palm: (f32, f32), open: bool) -> Option<Direction> {
        if !open {
            self.trail.clear();
            return None;
        }
        self.trail.push_back((now, palm.0, palm.1));
        while self.trail.front().is_some_and(|(t, _, _)| now.duration_since(*t) > WAVE_LOOK_BACK) {
            self.trail.pop_front();
        }
        let (start, x0, y0) = *self.trail.front()?;
        let seconds = now.duration_since(start).as_secs_f32();
        let cooling_down = self.cooldown_until.is_some_and(|until| now < until);
        if seconds < WAVE_MIN_SPAN || cooling_down {
            return None;
        }
        // Speeds in frame widths per second.
        let direction = wave_direction((palm.0 - x0) / seconds, (palm.1 - y0) / seconds);
        if direction.is_some() {
            self.cooldown_until = Some(now + WAVE_COOLDOWN);
            self.trail.clear();
        }
        direction
    }
}

/// Is this movement a wave? It must be fast, and at least twice as fast along its main
/// direction as across it. Screen y grows downwards, so "up" is a negative vy.
fn wave_direction(vx: f32, vy: f32) -> Option<Direction> {
    if vx.abs() > WAVE_SPEED && vx.abs() > 2.0 * vy.abs() {
        Some(if vx > 0.0 { Direction::Right } else { Direction::Left })
    } else if vy < -WAVE_SPEED && vy.abs() > 2.0 * vx.abs() {
        Some(Direction::Up)
    } else {
        None
    }
}

/// Remembers the last single click so a quick second pinch can become a double-click.
struct DoubleClickRule {
    last_click: Option<(Instant, (f32, f32))>,
    quiet_until: Option<Instant>,
}

impl DoubleClickRule {
    fn new() -> DoubleClickRule {
        DoubleClickRule { last_click: None, quiet_until: None }
    }

    fn is_quiet(&self, now: Instant) -> bool {
        self.quiet_until.is_some_and(|until| now < until)
    }

    /// macOS only counts a double-click if both clicks land within about half a second,
    /// which a hand in the air rarely manages. So WE decide: a second pinch near the first
    /// becomes a DoublePinch at the FIRST pinch's spot (the item is already selected).
    fn left_pinch(&mut self, now: Instant, spot: (f32, f32)) -> Event {
        let first = self.last_click.filter(|&(at, first_spot)| {
            now.duration_since(at) < DOUBLE_PINCH_WINDOW && distance(spot, first_spot) < DOUBLE_PINCH_RADIUS
        });
        match first {
            Some((_, first_spot)) => {
                self.last_click = None;
                self.quiet_until = Some(now + QUIET_AFTER_DOUBLE);
                Event { gesture: Gesture::DoublePinch, at: first_spot }
            }
            None => {
                self.last_click = Some((now, spot));
                Event { gesture: Gesture::Pinch, at: spot }
            }
        }
    }
}

fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

/// Looks at one Reading per picture and says which gesture just happened.
pub struct GestureDetector {
    left: Pinch,
    right: Pinch,
    swipes: SwipeDetector,
    clicks: DoubleClickRule,
    cursor_history: VecDeque<(f32, f32)>,
}

impl GestureDetector {
    pub fn new() -> GestureDetector {
        GestureDetector {
            left: Pinch::new(LEFT_PINCH_FRAMES),
            right: Pinch::new(RIGHT_PINCH_FRAMES),
            swipes: SwipeDetector::new(),
            clicks: DoubleClickRule::new(),
            cursor_history: VecDeque::with_capacity(CURSOR_HISTORY + 1),
        }
    }

    /// The hand left the picture: forget where the cursor has been.
    pub fn lost(&mut self) {
        self.cursor_history.clear();
    }

    /// What should the mouse do for this picture? `None` means "leave the cursor alone":
    /// a pinch is being held, or we are in the quiet moment after a double-click.
    pub fn update(&mut self, reading: &Reading) -> Option<Event> {
        self.remember(reading.cursor);
        let wave = self.swipes.update(reading.now, reading.palm, reading.palm_open);
        let left_fired = self.left.update(reading.thumb_to_index_m);
        // A right pinch is ignored while the left one is down, so one pinch can't do both.
        let right_fired = self.right.update(reading.thumb_to_middle_m) && !self.left.down;

        if let Some(direction) = wave {
            return Some(Event { gesture: Gesture::Wave(direction), at: reading.cursor });
        }
        if left_fired || right_fired {
            if self.clicks.is_quiet(reading.now) {
                return None;
            }
            let spot = self.cursor_before_pinch(reading.cursor);
            if right_fired {
                return Some(Event { gesture: Gesture::RightPinch, at: spot });
            }
            return Some(self.clicks.left_pinch(reading.now, spot));
        }
        if self.left.down || self.right.down {
            return None; // freeze the cursor while a pinch is held
        }
        Some(Event { gesture: Gesture::Point, at: reading.cursor })
    }

    fn remember(&mut self, cursor: (f32, f32)) {
        self.cursor_history.push_back(cursor);
        if self.cursor_history.len() > CURSOR_HISTORY {
            self.cursor_history.pop_front();
        }
    }

    fn cursor_before_pinch(&self, now_cursor: (f32, f32)) -> (f32, f32) {
        self.cursor_history.front().copied().unwrap_or(now_cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLOSE: f32 = 0.02; // 2 cm: fingers touching
    const OPEN: f32 = 0.08; // 8 cm: fingers apart
    const FRAME: u64 = 33; // milliseconds between pictures at 30 fps

    fn at(base: Instant, ms: u64) -> Instant {
        base + Duration::from_millis(ms)
    }

    /// A reading with a resting open hand, which each test then changes.
    fn reading(now: Instant, cursor: (f32, f32), index_m: f32, middle_m: f32) -> Reading {
        Reading { now, cursor, thumb_to_index_m: index_m, thumb_to_middle_m: middle_m, palm: (0.5, 0.3), palm_open: false }
    }

    #[test]
    fn pinch_fires_once_after_two_closed_pictures() {
        let mut pinch = Pinch::new(2);
        assert!(!pinch.update(CLOSE));
        assert!(pinch.update(CLOSE));
        assert!(!pinch.update(CLOSE), "holding the pinch must not fire again");
    }

    #[test]
    fn pinch_ignores_a_single_closed_picture() {
        let mut pinch = Pinch::new(2);
        assert!(!pinch.update(CLOSE));
        assert!(!pinch.update(OPEN));
        assert!(!pinch.update(CLOSE), "the count starts again after an open picture");
    }

    #[test]
    fn pinch_rearms_only_after_opening_past_five_cm() {
        let mut pinch = Pinch::new(2);
        pinch.update(CLOSE);
        assert!(pinch.update(CLOSE));
        // 4 cm is in between: the pinch is still "down", so closing again does nothing.
        pinch.update(0.04);
        assert!(!pinch.update(CLOSE) && !pinch.update(CLOSE));
        pinch.update(0.06); // opened wide: re-armed
        pinch.update(CLOSE);
        assert!(pinch.update(CLOSE));
    }

    #[test]
    fn double_click_needs_a_second_pinch_soon_and_near() {
        let base = Instant::now();
        let mut rule = DoubleClickRule::new();
        let first = rule.left_pinch(at(base, 0), (100.0, 100.0));
        assert_eq!(first.gesture, Gesture::Pinch);

        // 0.5 s later, 10 points away: a double-click, at the FIRST spot.
        let second = rule.left_pinch(at(base, 500), (108.0, 106.0));
        assert_eq!(second, Event { gesture: Gesture::DoublePinch, at: (100.0, 100.0) });
    }

    #[test]
    fn double_click_is_refused_when_too_slow_or_too_far() {
        let base = Instant::now();
        let mut rule = DoubleClickRule::new();
        rule.left_pinch(at(base, 0), (100.0, 100.0));
        let too_slow = rule.left_pinch(at(base, 1200), (100.0, 100.0));
        assert_eq!(too_slow.gesture, Gesture::Pinch);

        let mut rule = DoubleClickRule::new();
        rule.left_pinch(at(base, 0), (100.0, 100.0));
        let too_far = rule.left_pinch(at(base, 500), (200.0, 100.0));
        assert_eq!(too_far.gesture, Gesture::Pinch);
    }

    #[test]
    fn quiet_for_400_ms_after_a_double_click() {
        let base = Instant::now();
        let mut rule = DoubleClickRule::new();
        rule.left_pinch(at(base, 0), (50.0, 50.0));
        rule.left_pinch(at(base, 300), (50.0, 50.0));
        assert!(rule.is_quiet(at(base, 600)));
        assert!(!rule.is_quiet(at(base, 701)));
    }

    /// Feed a closed-then-open pinch through the detector. Returns what the closing frames produced.
    fn pinch_once(det: &mut GestureDetector, base: Instant, start_ms: u64, cursor: (f32, f32)) -> Vec<Option<Event>> {
        let mut out = Vec::new();
        for (i, distance) in [CLOSE, CLOSE, OPEN, OPEN].into_iter().enumerate() {
            let now = at(base, start_ms + i as u64 * FRAME);
            out.push(det.update(&reading(now, cursor, distance, OPEN)));
        }
        out
    }

    #[test]
    fn detector_points_when_nothing_is_happening() {
        let mut det = GestureDetector::new();
        let event = det.update(&reading(Instant::now(), (10.0, 20.0), OPEN, OPEN));
        assert_eq!(event, Some(Event { gesture: Gesture::Point, at: (10.0, 20.0) }));
    }

    #[test]
    fn detector_pinch_then_quick_pinch_is_a_double_pinch() {
        let base = Instant::now();
        let mut det = GestureDetector::new();
        let first = pinch_once(&mut det, base, 0, (300.0, 300.0));
        assert_eq!(first[1].map(|e| e.gesture), Some(Gesture::Pinch));
        let second = pinch_once(&mut det, base, 400, (302.0, 300.0));
        assert_eq!(second[1].map(|e| e.gesture), Some(Gesture::DoublePinch));
    }

    #[test]
    fn detector_ignores_a_pinch_during_the_quiet_moment() {
        let base = Instant::now();
        let mut det = GestureDetector::new();
        pinch_once(&mut det, base, 0, (300.0, 300.0));
        pinch_once(&mut det, base, 400, (300.0, 300.0)); // the double-click
        let third = pinch_once(&mut det, base, 560, (300.0, 300.0));
        assert_eq!(third[1], None, "a third pinch right after a double must not click");
    }

    #[test]
    fn click_lands_where_the_cursor_was_about_165_ms_ago() {
        let base = Instant::now();
        let mut det = GestureDetector::new();
        // The cursor drifts 10 points per picture while the fingers close.
        let mut fired = None;
        for i in 0..8u64 {
            let distance = if i >= 6 { CLOSE } else { OPEN };
            let cursor = (100.0 + i as f32 * 10.0, 50.0);
            fired = det.update(&reading(at(base, i * FRAME), cursor, distance, OPEN)).filter(|e| e.gesture == Gesture::Pinch);
            if fired.is_some() {
                break;
            }
        }
        // The pinch fires on picture 7 (cursor x = 170). The oldest remembered cursor is picture 2.
        assert_eq!(fired.map(|e| e.at), Some((120.0, 50.0)));
    }

    #[test]
    fn right_pinch_needs_three_pictures() {
        let base = Instant::now();
        let mut det = GestureDetector::new();
        let results: Vec<_> = (0..3u64)
            .map(|i| det.update(&reading(at(base, i * FRAME), (5.0, 5.0), OPEN, CLOSE)))
            .collect();
        assert_eq!(results[0].map(|e| e.gesture), Some(Gesture::Point));
        assert_eq!(results[1].map(|e| e.gesture), Some(Gesture::Point));
        assert_eq!(results[2].map(|e| e.gesture), Some(Gesture::RightPinch));
    }

    #[test]
    fn right_pinch_is_blocked_while_the_left_pinch_is_down() {
        let base = Instant::now();
        let mut det = GestureDetector::new();
        det.update(&reading(at(base, 0), (5.0, 5.0), CLOSE, OPEN));
        det.update(&reading(at(base, FRAME), (5.0, 5.0), CLOSE, OPEN)); // left pinch fires
        for i in 2..8u64 {
            let event = det.update(&reading(at(base, i * FRAME), (5.0, 5.0), CLOSE, CLOSE));
            assert_eq!(event, None, "the cursor stays frozen and no right click happens");
        }
    }

    #[test]
    fn cursor_is_frozen_while_a_pinch_is_held() {
        let base = Instant::now();
        let mut det = GestureDetector::new();
        pinch_once(&mut det, base, 0, (1.0, 1.0));
        det.update(&reading(at(base, 500), (1.0, 1.0), CLOSE, OPEN));
        det.update(&reading(at(base, 533), (1.0, 1.0), CLOSE, OPEN));
        assert_eq!(det.update(&reading(at(base, 566), (9.0, 9.0), 0.04, OPEN)), None);
    }

    /// Slide an open palm along a line and report the first wave, if any.
    fn wave_test(det: &mut GestureDetector, base: Instant, start_ms: u64, from: (f32, f32), speed: (f32, f32), open: bool) -> Option<Direction> {
        for i in 0..8u64 {
            let seconds = (i * FRAME) as f32 / 1000.0;
            let palm = (from.0 + speed.0 * seconds, from.1 + speed.1 * seconds);
            let mut r = reading(at(base, start_ms + i * FRAME), (0.0, 0.0), OPEN, OPEN);
            r.palm = palm;
            r.palm_open = open;
            if let Some(Event { gesture: Gesture::Wave(direction), .. }) = det.update(&r) {
                return Some(direction);
            }
        }
        None
    }

    #[test]
    fn open_hand_waves_in_three_directions() {
        let base = Instant::now();
        let right = wave_test(&mut GestureDetector::new(), base, 0, (0.3, 0.4), (1.5, 0.0), true);
        let left = wave_test(&mut GestureDetector::new(), base, 0, (0.7, 0.4), (-1.5, 0.0), true);
        let up = wave_test(&mut GestureDetector::new(), base, 0, (0.5, 0.6), (0.0, -1.5), true);
        assert_eq!((right, left, up), (Some(Direction::Right), Some(Direction::Left), Some(Direction::Up)));
    }

    #[test]
    fn slow_movement_closed_hands_and_downward_moves_are_not_waves() {
        let base = Instant::now();
        let slow = wave_test(&mut GestureDetector::new(), base, 0, (0.3, 0.4), (0.5, 0.0), true);
        let closed = wave_test(&mut GestureDetector::new(), base, 0, (0.3, 0.4), (1.5, 0.0), false);
        let down = wave_test(&mut GestureDetector::new(), base, 0, (0.5, 0.3), (0.0, 1.5), true);
        assert_eq!((slow, closed, down), (None, None, None));
    }

    #[test]
    fn a_second_wave_must_wait_for_the_cooldown() {
        let base = Instant::now();
        let mut det = GestureDetector::new();
        assert_eq!(wave_test(&mut det, base, 0, (0.3, 0.4), (1.5, 0.0), true), Some(Direction::Right));
        // Straight away: still cooling down (the first wave used up to 8 pictures = 264 ms).
        assert_eq!(wave_test(&mut det, base, 300, (0.3, 0.4), (1.5, 0.0), true), None);
        // Well after the 800 ms cooldown.
        assert_eq!(wave_test(&mut det, base, 2000, (0.3, 0.4), (1.5, 0.0), true), Some(Direction::Right));
    }

    #[test]
    fn wave_direction_wants_a_clear_main_direction() {
        assert_eq!(wave_direction(1.0, 0.0), Some(Direction::Right));
        assert_eq!(wave_direction(-1.0, 0.2), Some(Direction::Left));
        assert_eq!(wave_direction(0.0, -1.0), Some(Direction::Up));
        assert_eq!(wave_direction(1.0, 0.7), None, "diagonal moves are not waves");
        assert_eq!(wave_direction(0.8, 0.0), None, "below 0.9 frame widths per second");
    }
}
