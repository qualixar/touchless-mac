# Third-party models

The release download includes two model files. They are not part of this repository's MIT-licensed code.

| File | Original | Licence |
|---|---|---|
| `palm.onnx` | OpenCV's ONNX conversion of Google MediaPipe's palm detector: [opencv/palm_detection_mediapipe](https://huggingface.co/opencv/palm_detection_mediapipe), file `palm_detection_mediapipe_2023feb.onnx` | Apache License 2.0 |
| `hand.onnx` | OpenCV's ONNX conversion of Google MediaPipe's hand landmark model: [opencv/handpose_estimation_mediapipe](https://huggingface.co/opencv/handpose_estimation_mediapipe), file `handpose_estimation_mediapipe_2023feb.onnx` | Apache License 2.0 |

The files are redistributed unchanged (renamed only) and are checked against SHA-256 fingerprints in `scripts/get-models.sh`. A copy of the Apache License 2.0 is in `LICENSE-Apache-2.0.txt` next to this file.
