"""Bounded screenshot gate for the 1024x768 diagnostic room-create dialog.

Calibration comes from control-03 positive and glyph-03/04 negative screenshots.
This is an image heuristic, not proof of client state; retain the screenshot and
inspect it independently before claiming that a dialog was visible.
"""
from PIL import Image


def dialog_candidate(path):
    with Image.open(path) as image:
        rgb = image.convert('RGB')
        if rgb.size != (1024, 768):
            return False
        checks = [((210, 200), (12, 223, 255)), ((400, 600), (12, 223, 255)),
                  ((300, 460), (0, 86, 161)), ((700, 530), (8, 190, 255))]
        return all(max(abs(a - b) for a, b in zip(rgb.getpixel(point), expected)) <= 8
                   for point, expected in checks)
