"""Drive the observed KR 00967 sign-in form on a dedicated Wine desktop."""
import time

from PIL import ImageChops, ImageStat


# Root coordinates for the 1024x768 capture of the centered 800x600 client.
# Exclude the leading pixels where the empty edit caret blinks.
PANEL = (420, 488, 607, 545)
FIELDS = ((530, 580, 602, 600), (530, 604, 602, 625))


def distance(reference, current, box):
    if reference.size != (1024, 768) or current.size != reference.size:
        raise ValueError("KR Wine login requires a 1024x768 desktop capture")
    a, b = (image.crop(box).convert("RGB") for image in (reference, current))
    return sum(ImageStat.Stat(ImageChops.difference(a, b)).mean) / 3


def wait_for_panel(capture, reference, timeout=35):
    """A visible HWND can still be displaying a black startup frame."""
    end = time.monotonic() + timeout
    consecutive = 0
    while time.monotonic() < end:
        if distance(reference, capture(), PANEL) < 12:
            consecutive += 1
            if consecutive >= 2:
                return
        else:
            consecutive = 0
        time.sleep(.25)
    raise TimeoutError("KR login panel did not render")


def sign_in(capture, reference, xdo, credentials, pause=time.sleep):
    """Type local fixture credentials; verify each field cleared before typing."""
    login, password = credentials
    if any(not field or len(field) > 16 or not field.isascii() or not field.isalnum()
           for field in credentials):
        raise ValueError("expected short ASCII fixture credentials")

    def click(x, y):
        xdo("mousemove", str(x), str(y))
        pause(.15)
        xdo("mousedown", "1")
        try:
            pause(.15)
        finally:
            xdo("mouseup", "1")
        pause(.2)

    def wait_for_field(box, filled, error):
        for _ in range(9):
            if (distance(reference, capture(), box) >= 4) == filled:
                return
            pause(.22)
        raise RuntimeError(error)

    # Ctrl+A is not a reliable clear operation in the client's custom edit box.
    # Verify the blank field against the same build's rendered empty form.
    for box, y, value in zip(FIELDS, (590, 615), (login, password), strict=True):
        click(557, y)
        # The client draws its own cursor over the field, even in a root capture.
        xdo("mousemove", "700", "550")
        xdo("key", "End")
        xdo("key", "--repeat", "24", "--delay", "28", "BackSpace")
        wait_for_field(box, False, "KR login field did not clear; sign-in not sent")
        xdo("type", "--clearmodifiers", "--delay", "90", "--", value)
        wait_for_field(box, True, "KR login field did not receive input; sign-in not sent")
    click(514, 667)
