"""End host feedback when native rank/return takes ownership of the screen.

Loaded after room-observer.py. Breakpoints only read the native lifecycle and
remove host observer control files; they never write client memory or call it.
"""
import json
import time
import gdb


class FeedbackLifetimeProbe(gdb.Breakpoint):
    def __init__(self, address, kind):
        super().__init__('*'+hex(address), internal=True)
        self.kind = kind

    def stop(self):
        removed = []
        for name in ['feedback-on', 'udp-window.json']:
            path = release99.parent/name
            if path.exists():
                path.unlink()
                removed.append(name)
        print('G1_FEEDBACK_LIFETIME '+json.dumps(dict(
            kind=self.kind, ns=time.monotonic_ns(), removed=removed)), flush=True)
        return False


FeedbackLifetimeProbe(0xe610a4, 'rank_end')
FeedbackLifetimeProbe(0xe64950, 'return_begin')
