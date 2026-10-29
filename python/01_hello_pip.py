# Idea 1, the Python way: install a package with pip, then run the file.
#
#   pip install pillow
#   python python/01_hello_pip.py
#
# Rust's version is examples/01_cargo.rs. In Rust, Cargo does both steps for you.
from PIL import Image

print("Hello from Python!")

# A tiny picture, 4 pixels wide and 3 pixels tall, from the Pillow package.
picture = Image.new("RGB", (4, 3))
picture.putpixel((1, 2), (255, 0, 0))  # one red pixel
print(f"the picture is {picture.width} x {picture.height} pixels")
print(f"pixel (1, 2) is {picture.getpixel((1, 2))}")
