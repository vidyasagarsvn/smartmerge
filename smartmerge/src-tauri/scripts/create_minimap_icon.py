#!/usr/bin/env python3
from PIL import Image, ImageDraw

# Create a 64x64 PNG with a minimap-like icon
size = (64, 64)
img = Image.new("RGBA", size, (255, 255, 255, 0))
draw = ImageDraw.Draw(img)

# Draw a rounded rectangle background
draw.rounded_rectangle(
    [4, 4, 60, 60],
    radius=8,
    fill=(240, 240, 240, 255),
    outline=(180, 180, 180, 255),
    width=2,
)

# Draw vertical bars to represent a minimap
for i in range(8):
    x = 10 + i * 6
    draw.rectangle([x, 14, x + 3, 50], fill=(100, 160, 220, 255))

# Draw a highlighted viewport rectangle
draw.rectangle([18, 28, 46, 38], outline=(255, 120, 60, 255), width=2)

import os

output_path = os.path.abspath(
    os.path.join(os.path.dirname(__file__), "..", "icons", "menu", "minimap.png")
)
img.save(output_path)
print(f"minimap.png created at {output_path}")
