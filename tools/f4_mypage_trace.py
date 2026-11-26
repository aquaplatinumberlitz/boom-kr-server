"""GDB extension for the KR 00967 MyPage control/guard trace in CP401.

Source in GDB after attaching to the stopped, isolated Wine client:
    source tools/f4_mypage_trace.py
    f4-mypage-trace start 0x400000 /absolute/new/trace.jsonl
    continue

Run the hover and click through the same input controller, recording their
timestamps. Interrupt after the click window, then use `f4-mypage-trace stop`
and detach. The observer verifies instruction bytes before setting breakpoints
and refuses a different image layout. It never forces any gate or sends packets.
"""

import json
import time
from pathlib import Path

import gdb


IMAGE_BASE = 0x400000
MAX_EVENTS = 256
SITES = (
    ("lobby_callback", 0xDA1E80, "558bec51894dfc8b", None),
    ("lobby_guard", 0xDA1E95, "85c07402", "zero"),
    ("mypage_entry", 0xD73C50, "558bec6aff686946650164a1", "slot"),
    ("defer_state", 0xD73C8E, "83f8030f85", "eax"),
    ("profile_guard", 0xD73D30, "85c07505", "zero"),
    ("builder_201", 0xD73E52, "e8b90d3f00", "opcode"),
    ("builder_702", 0xD7406B, "e8a00b3f00", "opcode"),
    ("builder_548", 0xD740FD, "e80e0b3f00", "opcode"),
    ("builder_69", 0xD7418A, "e8810a3f00", "opcode"),
)


def sample(frame, inferior, kind):
    if kind is None:
        return {}
    if kind in ("zero", "eax"):
        value = int(frame.read_register("eax")) & 0xFFFFFFFF
        return {"zero": value == 0} if kind == "zero" else {"state": value}
    sp = int(frame.read_register("esp")) & 0xFFFFFFFF
    if kind == "slot":
        # thiscall entry: return address, a2, signed low word of a3, a4.
        return {"slot": int.from_bytes(inferior.read_memory(sp + 8, 2), "little", signed=True)}
    # At the call instruction, the builder's opcode is already on the stack.
    return {"opcode": int.from_bytes(inferior.read_memory(sp, 4), "little")}


class Site(gdb.Breakpoint):
    def __init__(self, address, label, kind, trace):
        super().__init__(f"*{address:#x}", internal=True)
        self.address, self.label, self.kind, self.trace = address, label, kind, trace
        self.silent = True

    def stop(self):
        trace = self.trace
        if trace.count == MAX_EVENTS:
            if not trace.overflow:
                trace.write("event_limit", limit=MAX_EVENTS)
                trace.overflow = True
            return False
        trace.count += 1
        try:
            fields = sample(gdb.selected_frame(), gdb.selected_inferior(), self.kind)
        except (gdb.error, ValueError):
            fields = {"read_error": True}
        trace.write(self.label, address=f"{self.address:#x}", **fields)
        return False


class Trace(gdb.Command):
    def __init__(self):
        super().__init__("f4-mypage-trace", gdb.COMMAND_USER)
        self.output = None
        self.points = []

    def write(self, event, **fields):
        print(json.dumps(dict(event=event, unix_ns=time.time_ns(), **fields)), file=self.output, flush=True)

    def stop(self):
        for point in self.points:
            if point.is_valid():
                point.delete()
        self.points.clear()
        if self.output is not None:
            self.write("observer_stopped", hits=self.count, overflow=self.overflow)
            self.output.close()
            self.output = None

    def invoke(self, arg, _from_tty):
        argv = gdb.string_to_argv(arg)
        if argv == ["stop"]:
            self.stop()
            return
        if len(argv) != 3 or argv[0] != "start" or self.output is not None:
            raise gdb.GdbError("usage: f4-mypage-trace start BASE /absolute/new/trace.jsonl | stop")
        try:
            base = int(argv[1], 0)
            path = Path(argv[2])
            if not 0 < base < 0xFFFFFFFF or not path.is_absolute():
                raise ValueError("expected an image base and an absolute output path")
            inferior = gdb.selected_inferior()
            sites = [(base + va - IMAGE_BASE, label, bytes.fromhex(code), kind)
                     for label, va, code, kind in SITES]
            for address, label, code, _ in sites:
                if bytes(inferior.read_memory(address, len(code))) != code:
                    raise ValueError(f"unexpected code at {label} ({address:#x})")
            self.output = path.open("x", encoding="utf-8")
        except (OSError, ValueError, gdb.error) as exc:
            raise gdb.GdbError(str(exc)) from None
        self.count = 0
        self.overflow = False
        try:
            for address, label, _, kind in sites:
                self.points.append(Site(address, label, kind, self))
            self.write("observer_ready", base=f"{base:#x}", sites=len(sites))
        except (OSError, gdb.error):
            self.stop()
            raise


Trace()
