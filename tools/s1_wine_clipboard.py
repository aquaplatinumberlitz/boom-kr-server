"""Read the Unicode text copied by BoomServer's button in its private Wine prefix."""
import ctypes
from ctypes import wintypes
import sys


def copied():
    user = ctypes.WinDLL('user32', use_last_error=True)
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    user.GetClipboardData.argtypes = [wintypes.UINT]
    user.GetClipboardData.restype = wintypes.HANDLE
    kernel.GlobalLock.argtypes = [wintypes.HANDLE]
    kernel.GlobalLock.restype = wintypes.LPVOID
    kernel.GlobalUnlock.argtypes = [wintypes.HANDLE]
    if not user.OpenClipboard(None):
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        handle = user.GetClipboardData(13)  # CF_UNICODETEXT
        if not handle:
            raise RuntimeError('No Unicode text on the private clipboard')
        pointer = kernel.GlobalLock(handle)
        if not pointer:
            raise ctypes.WinError(ctypes.get_last_error())
        try:
            return ctypes.wstring_at(pointer)
        finally:
            kernel.GlobalUnlock(handle)
    finally:
        user.CloseClipboard()


if __name__ == '__main__':
    code = copied()
    if not code.startswith('S1-1-') or len(code) != 91:
        sys.exit('The Copy button did not put an S1 code on the clipboard')
    print(code)
