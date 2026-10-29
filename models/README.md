# Models

The app needs two neural-network files here. They are not stored in git (about 8 MB together).
Download them with:

    bash scripts/get-models.sh

That creates:

| File | What it does | Source |
|---|---|---|
| `models/palm.onnx` | Finds a palm anywhere in the camera picture | [opencv/palm_detection_mediapipe](https://huggingface.co/opencv/palm_detection_mediapipe) (`palm_detection_mediapipe_2023feb.onnx`) |
| `models/hand.onnx` | Gives 21 hand points for one hand | [opencv/handpose_estimation_mediapipe](https://huggingface.co/opencv/handpose_estimation_mediapipe) (`handpose_estimation_mediapipe_2023feb.onnx`) |

Both are OpenCV's ONNX conversions of Google MediaPipe's hand models. Licence: to be confirmed (being verified separately).
The script checks each download against a SHA-256 fingerprint, so a changed or damaged file is rejected.
