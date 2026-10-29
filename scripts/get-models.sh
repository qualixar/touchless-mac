#!/bin/bash
# Download the two hand models into models/ (or into the folder you pass as the first argument).
#
#   bash scripts/get-models.sh
#
# The files are about 8 MB together and are not stored in git. After downloading, the script
# checks each file against a known SHA-256 fingerprint, so you know you got the right bytes.
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
dest="${1:-$here/models}"
mkdir -p "$dest"

PALM_URL="https://huggingface.co/opencv/palm_detection_mediapipe/resolve/main/palm_detection_mediapipe_2023feb.onnx"
HAND_URL="https://huggingface.co/opencv/handpose_estimation_mediapipe/resolve/main/handpose_estimation_mediapipe_2023feb.onnx"
PALM_SHA256="78ff51c38496b7fc8b8ebdb6cc8c1abb02fa6c38427c6848254cdaba57fcce7c"
HAND_SHA256="db0898ae717b76b075d9bf563af315b29562e11f8df5027a1ef07b02bef6d81c"

# fetch <url> <file name> <expected sha256>
fetch() {
    local url="$1" name="$2" expected="$3"
    echo "Downloading $name ..."
    curl --fail --location --silent --show-error --output "$dest/$name.part" "$url"
    local actual
    actual="$(shasum -a 256 "$dest/$name.part" | cut -d' ' -f1)"
    if [ "$actual" != "$expected" ]; then
        rm -f "$dest/$name.part"
        echo "ERROR: $name does not match the expected fingerprint." >&2
        echo "  expected $expected" >&2
        echo "  got      $actual" >&2
        exit 1
    fi
    mv "$dest/$name.part" "$dest/$name"
    echo "  ok: $dest/$name"
}

fetch "$PALM_URL" palm.onnx "$PALM_SHA256"
fetch "$HAND_URL" hand.onnx "$HAND_SHA256"
echo "Done. Next: cargo run --release"
