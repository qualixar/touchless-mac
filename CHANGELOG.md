# Changelog

## 0.1.0 (30 October 2026)

The first release.

- **Pointing:** the cursor follows your index knuckle through a One Euro filter, and holds still when your hand leaves the picture.
- **Clicks:** a thumb-to-index pinch clicks, two quick pinches double-click, and a thumb-to-middle pinch right-clicks. Each click lands where the cursor was just before your fingers closed.
- **Scroll:** touch your thumb to your ring finger and move your hand; the page follows it. The pose was chosen from real recordings because it almost never happens by accident.
- **Waves:** open hand right or left switches apps, up opens Mission Control.
- **Calibration:** on the first run the app speaks each of the four corners and builds your personal reach box. A box that comes out too small is widened, so the cursor never gets jumpy.
- **Options:** `--speed 0.25–4`, `--invert-scroll`, `--recalibrate`, `--no-calibrate`, `--models` and `--selftest` for a camera-free check.
- **Clear errors:** a missing camera or Accessibility permission, or missing models, gives a message that says what to do.
- **The tutorial:** tags `ch1-cargo` to `ch6-calibrate` hold the code at the end of each chapter of the video "Learn Rust by building a touchless Mac".
