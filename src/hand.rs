// Chapters 2 and 4: finding the hand.
//
// Two small neural networks from Google MediaPipe do the hard part:
//   1. the palm model looks at the whole picture and says "a palm is here",
//   2. the hand model looks at a crop around that palm and returns 21 hand points.
// We run them with ONNX Runtime (the `ort` crate). This file is the plumbing around them.
// You don't need to read every line: the ideas for this chapter are `struct` and `let mut`.
use crate::error::{AppError, Result};
use image::{RgbImage, imageops, imageops::FilterType};
use ort::{session::Session, value::Tensor};
use std::path::Path;

// Names for the hand points we use (the model numbers them 0 to 20).
pub const WRIST: usize = 0;
pub const THUMB_TIP: usize = 4;
pub const INDEX_KNUCKLE: usize = 5;
pub const INDEX_TIP: usize = 8;
pub const MIDDLE_TIP: usize = 12;
pub const RING_TIP: usize = 16;

/// The 12 points of the palm itself. We leave out the fingertips on purpose: fingertips
/// move a lot, and a box drawn around them would swing about and make the cursor shake.
const PALM_JOINTS: [usize; 12] = [0, 1, 2, 3, 5, 6, 9, 10, 13, 14, 17, 18];
const MIDDLE_KNUCKLE: usize = 9;

const PALM_INPUT: u32 = 192; // the palm model wants a 192 x 192 picture
const HAND_INPUT: u32 = 224; // the hand model wants a 224 x 224 picture
const MIN_CONFIDENCE: f32 = 0.5;

/// One spot in the camera picture, in pixels.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// What the hand model sees: 21 points and how sure it is.
#[derive(Clone, Debug)]
pub struct Hand {
    pub points: [Point; 21],
    /// The same 21 points in metres (3D, origin near the hand centre). Distances measured here
    /// don't change when the hand tilts towards the camera, unlike distances in pixels.
    pub world: [[f32; 3]; 21],
    pub confidence: f32,
}

/// Where to look for the hand: a square around a centre, turned so the fingers point "up".
#[derive(Clone, Copy, Debug)]
pub struct Crop {
    pub cx: f32,
    pub cy: f32,
    pub side: f32,
    /// Direction from the wrist towards the fingers, as a unit vector in picture pixels.
    pub up: (f32, f32),
}

/// A direction of length 1.
fn unit(dx: f32, dy: f32) -> (f32, f32) {
    let length = (dx * dx + dy * dy).sqrt().max(1e-6);
    (dx / length, dy / length)
}

fn sigmoid(value: f32) -> f32 {
    1.0 / (1.0 + (-value).exp())
}

/// Read one spot of the picture, blending the four nearest pixels (bilinear sampling).
/// Spots outside the picture are black.
fn sample(img: &RgbImage, x: f32, y: f32) -> [f32; 3] {
    let (w, h) = img.dimensions();
    if x < 0.0 || y < 0.0 || x >= (w - 1) as f32 || y >= (h - 1) as f32 {
        return [0.0; 3];
    }
    let (x0, y0) = (x.floor() as u32, y.floor() as u32);
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let pixel = |px, py| img.get_pixel(px, py).0;
    let (a, b, c, d) = (
        pixel(x0, y0),
        pixel(x0 + 1, y0),
        pixel(x0, y0 + 1),
        pixel(x0 + 1, y0 + 1),
    );
    std::array::from_fn(|i| {
        let top = a[i] as f32 * (1.0 - fx) + b[i] as f32 * fx;
        let bottom = c[i] as f32 * (1.0 - fx) + d[i] as f32 * fx;
        (top * (1.0 - fy) + bottom * fy) / 255.0
    })
}

/// The palm model's grid of "anchor" positions: 24x24 cells with 2 anchors each, then 12x12 with 6.
fn palm_anchors() -> Vec<(f32, f32)> {
    let mut anchors = Vec::with_capacity(2016);
    for (grid, per_cell) in [(24u32, 2), (12u32, 6)] {
        for y in 0..grid {
            for x in 0..grid {
                for _ in 0..per_cell {
                    anchors.push((
                        (x as f32 + 0.5) / grid as f32,
                        (y as f32 + 0.5) / grid as f32,
                    ));
                }
            }
        }
    }
    anchors
}

/// The camera picture shrunk to fit a 192 x 192 square, with black bars where it doesn't reach.
struct Letterbox {
    canvas: RgbImage,
    ratio: f32,
    left: u32,
    top: u32,
}

fn letterbox(img: &RgbImage) -> Letterbox {
    let (w, h) = img.dimensions();
    let ratio = PALM_INPUT as f32 / w.max(h) as f32;
    let (small_w, small_h) = ((w as f32 * ratio) as u32, (h as f32 * ratio) as u32);
    let small = imageops::resize(img, small_w, small_h, FilterType::Triangle);
    let mut canvas = RgbImage::new(PALM_INPUT, PALM_INPUT);
    let (left, top) = ((PALM_INPUT - small_w) / 2, (PALM_INPUT - small_h) / 2);
    imageops::replace(&mut canvas, &small, left as i64, top as i64);
    Letterbox {
        canvas,
        ratio,
        left,
        top,
    }
}

/// Turn a picture into the flat list of numbers (0.0 to 1.0) that the models read.
fn to_model_input(img: &RgbImage) -> Vec<f32> {
    img.pixels()
        .flat_map(|p| p.0.map(|c| c as f32 / 255.0))
        .collect()
}

pub struct Detector {
    palm_model: Session,
    hand_model: Session,
    anchors: Vec<(f32, f32)>,
}

impl Detector {
    /// Load both models from disk (see models/README.md for how to download them).
    pub fn new(palm_path: &Path, hand_path: &Path) -> Result<Detector> {
        for path in [palm_path, hand_path] {
            if !path.exists() {
                return Err(AppError::ModelMissing(path.display().to_string()));
            }
        }
        let environment = ort::init().with_name("touchless-mac").build()?;
        let palm_model = Session::builder(&environment)?.commit_from_file(palm_path)?;
        let hand_model = Session::builder(&environment)?.commit_from_file(hand_path)?;
        Ok(Detector {
            palm_model,
            hand_model,
            anchors: palm_anchors(),
        })
    }

    /// Look at the whole picture for a palm. Gives back a crop for the hand model, or
    /// `None` when no palm is confident enough.
    pub fn find_palm(&mut self, img: &RgbImage) -> Result<Option<Crop>> {
        let boxed = letterbox(img);
        let input = Tensor::from_array(([1usize, 192, 192, 3], to_model_input(&boxed.canvas)))?;
        let outputs = self.palm_model.run(ort::inputs![input])?;
        let (_, boxes) = outputs[0].try_extract_tensor::<f32>()?;
        let (_, scores) = outputs[1].try_extract_tensor::<f32>()?;

        // Each of the 2016 anchors has a score. We take the best one.
        let (best, raw_score) =
            scores.iter().enumerate().fold(
                (0, f32::MIN),
                |acc, (i, &s)| if s > acc.1 { (i, s) } else { acc },
            );
        if sigmoid(raw_score) < MIN_CONFIDENCE {
            return Ok(None);
        }
        Ok(Some(palm_to_crop(boxes, best, self.anchors[best], &boxed)))
    }

    /// Run the 21-point hand model on a crop. The crop is turned so the hand points up,
    /// and sampled straight into the 224 x 224 square the model expects.
    pub fn landmarks(&mut self, img: &RgbImage, crop: Crop) -> Result<Option<Hand>> {
        let to_picture = crop_to_picture(crop);
        let mut data = Vec::with_capacity((HAND_INPUT * HAND_INPUT * 3) as usize);
        for v in 0..HAND_INPUT {
            for u in 0..HAND_INPUT {
                let (x, y) = to_picture(u as f32 + 0.5, v as f32 + 0.5);
                data.extend_from_slice(&sample(img, x, y));
            }
        }
        let input = Tensor::from_array(([1usize, 224, 224, 3], data))?;
        let outputs = self.hand_model.run(ort::inputs![input])?;
        let (_, landmarks) = outputs[0].try_extract_tensor::<f32>()?;
        let (_, confidence) = outputs[1].try_extract_tensor::<f32>()?;
        let (_, world_points) = outputs[3].try_extract_tensor::<f32>()?;
        if confidence[0] < MIN_CONFIDENCE {
            return Ok(None);
        }
        let points = std::array::from_fn(|i| {
            let (x, y) = to_picture(landmarks[i * 3], landmarks[i * 3 + 1]);
            Point { x, y }
        });
        let world = std::array::from_fn(|i| {
            [
                world_points[i * 3],
                world_points[i * 3 + 1],
                world_points[i * 3 + 2],
            ]
        });
        Ok(Some(Hand {
            points,
            world,
            confidence: confidence[0],
        }))
    }
}

/// Turn the palm model's best guess into a crop for the hand model.
fn palm_to_crop(boxes: &[f32], best: usize, anchor: (f32, f32), boxed: &Letterbox) -> Crop {
    let row = &boxes[best * 18..best * 18 + 18];
    let (ax, ay) = anchor;
    // Numbers in the model's 0..1 space become picture pixels again.
    let scale = PALM_INPUT as f32 / boxed.ratio;
    let (pad_x, pad_y) = (
        boxed.left as f32 / boxed.ratio,
        boxed.top as f32 / boxed.ratio,
    );
    let to_pixels = |x: f32, y: f32| {
        (
            (x / 192.0 + ax) * scale - pad_x,
            (y / 192.0 + ay) * scale - pad_y,
        )
    };

    // Palm keypoint 0 is the wrist and keypoint 2 the middle finger's base: they give the direction.
    let wrist = to_pixels(row[4], row[5]);
    let middle = to_pixels(row[8], row[9]);
    let up = unit(middle.0 - wrist.0, middle.1 - wrist.1);

    let (cx, cy) = to_pixels(row[0], row[1]);
    let size = (row[2] / 192.0).max(row[3] / 192.0) * scale;
    // Same numbers as Google's pipeline: move the box half a palm towards the fingers,
    // then make it 2.6 times bigger so the whole hand fits.
    Crop {
        cx: cx + up.0 * 0.5 * size,
        cy: cy + up.1 * 0.5 * size,
        side: size * 2.6,
        up,
    }
}

/// Gives back a function that converts a spot inside the 224 x 224 crop to a spot in the
/// camera picture. It undoes the turn, the scale and the shift.
fn crop_to_picture(crop: Crop) -> impl Fn(f32, f32) -> (f32, f32) {
    let (ux, uy) = crop.up;
    let right = (-uy, ux); // the crop's x axis, in picture pixels
    let down = (-ux, -uy); // the crop's y axis, in picture pixels
    let pixels_per_step = crop.side / HAND_INPUT as f32;
    let half = HAND_INPUT as f32 / 2.0;
    move |a, b| {
        let (a, b) = ((a - half) * pixels_per_step, (b - half) * pixels_per_step);
        (
            crop.cx + a * right.0 + b * down.0,
            crop.cy + a * right.1 + b * down.1,
        )
    }
}

impl Hand {
    /// Where to look for the hand in the NEXT picture, based on where it is now.
    /// This is much cheaper than searching the whole picture again.
    pub fn next_crop(&self) -> Crop {
        let (wrist, middle) = (self.points[WRIST], self.points[MIDDLE_KNUCKLE]);
        let up = unit(middle.x - wrist.x, middle.y - wrist.y);
        let right = (-up.1, up.0);

        // Measure the palm along the hand's own axes, so a tilted hand doesn't make the box bigger.
        let (mut a_min, mut a_max, mut b_min, mut b_max) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for p in PALM_JOINTS.iter().map(|&i| self.points[i]) {
            let a = p.x * right.0 + p.y * right.1;
            let b = p.x * up.0 + p.y * up.1;
            a_min = a_min.min(a);
            a_max = a_max.max(a);
            b_min = b_min.min(b);
            b_max = b_max.max(b);
        }
        let (centre_a, centre_b) = ((a_min + a_max) / 2.0, (b_min + b_max) / 2.0);
        let size = (a_max - a_min).max(b_max - b_min);
        let shift = 0.1 * size; // a little towards the fingers
        let cx = centre_a * right.0 + (centre_b + shift) * up.0;
        let cy = centre_a * right.1 + (centre_b + shift) * up.1;
        Crop {
            cx,
            cy,
            side: size * 2.0,
            up,
        }
    }
}

impl Hand {
    /// Distance in metres between two hand points (3D, so it ignores tilt).
    pub fn distance_m(&self, a: usize, b: usize) -> f32 {
        let (p, q) = (self.world[a], self.world[b]);
        ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
    }

    /// An open hand: every finger is stretched, meaning its tip is further from the wrist
    /// than its middle joint.
    pub fn is_open_palm(&self) -> bool {
        let from_wrist = |i: usize| {
            let (wrist, p) = (self.points[WRIST], self.points[i]);
            (p.x - wrist.x).hypot(p.y - wrist.y)
        };
        let stretched =
            |tip: usize, middle_joint: usize| from_wrist(tip) > from_wrist(middle_joint) * 1.1;
        stretched(8, 6) && stretched(12, 10) && stretched(16, 14) && stretched(20, 18)
    }

    /// The middle of the palm (wrist plus the four knuckles), in picture pixels.
    pub fn palm_centre(&self) -> Point {
        const PALM_CENTRE_POINTS: [usize; 5] = [0, 5, 9, 13, 17];
        let sum_x: f32 = PALM_CENTRE_POINTS.iter().map(|&i| self.points[i].x).sum();
        let sum_y: f32 = PALM_CENTRE_POINTS.iter().map(|&i| self.points[i].y).sum();
        Point {
            x: sum_x / 5.0,
            y: sum_y / 5.0,
        }
    }
}

/// Finds the hand in each camera picture and remembers it for the next one.
pub struct Tracker {
    detector: Detector,
    last: Option<Hand>,
}

impl Tracker {
    pub fn new(detector: Detector) -> Tracker {
        Tracker {
            detector,
            last: None,
        }
    }

    /// Look for the hand in this picture. First try near where it was last time (fast).
    /// If that fails, search the whole picture (slower). `None` means no hand is visible.
    pub fn see(&mut self, img: &RgbImage) -> Result<Option<Hand>> {
        let tracked = match &self.last {
            Some(previous) => self.detector.landmarks(img, previous.next_crop())?,
            None => None,
        };
        let hand = match tracked {
            Some(hand) => Some(hand),
            None => self.search(img)?,
        };
        self.last = hand.clone();
        Ok(hand)
    }

    fn search(&mut self, img: &RgbImage) -> Result<Option<Hand>> {
        match self.detector.find_palm(img)? {
            Some(crop) => self.detector.landmarks(img, crop),
            None => Ok(None),
        }
    }
}
