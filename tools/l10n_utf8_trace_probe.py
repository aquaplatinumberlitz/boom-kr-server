"""Read-only one-shot title conversion observer for a private diagnostic client.

Captured pointers and before/after bytes are observations only; the output buffer
may be temporary and is not assumed to reach a UI control or network packet.
"""
import gdb
import hashlib
import json

SOURCES = {bytes.fromhex(value): name for name, value in (
    ('DefaultRoomTitle0', 'c1f1b0c5bfee20b0d4c0d3c7d8bfe4'),
    ('DefaultRoomTitle1', 'bec6b9abb3aa20bfc0bcbcbfe4'),
    ('DefaultRoomTitle2', 'c0fab6fb20b0d4c0d3c7cfbdc7bad07e'),
    ('DefaultRoomTitle3', 'bba1b8ae20bfc0bcbcbfe47e'),
    ('DefaultRoomTitle4', 'c3cabab8c0da20c8afbfb521'),
    ('DefaultRoomTitle5', 'b8c5b3cab0d720c7cabcf6'),
)}


def number(expr): return int(gdb.parse_and_eval(expr)) & 0xffffffff
def read(address, length): return bytes(gdb.selected_inferior().read_memory(address, length))
def record(kind, **fields): print('L10N_TRACE ' + json.dumps({'kind': kind, **fields}, sort_keys=True), flush=True)


class Return(gdb.Breakpoint):
    def __init__(self, address, stack, thread, destination, capacity, source_name):
        super().__init__('*%#x' % address, internal=True)
        self.address, self.stack, self.thread = address, stack, thread
        self.destination, self.capacity, self.source_name = destination, capacity, source_name

    def stop(self):
        if gdb.selected_thread().global_num != self.thread or number('$esp') != self.stack:
            return False
        self.enabled = False
        try:
            units = number('$eax')
            if units > self.capacity or units > 128:
                record('return_invalid', units=units, capacity=self.capacity)
            else:
                result = read(self.destination, units * 2)
                record('return', site=hex(self.address), stack=hex(self.stack), thread=self.thread,
                       destination=hex(self.destination), units=units,
                       wide_sha256=hashlib.sha256(result).hexdigest(), source=self.source_name)
        except Exception as exc:
            record('probe_error', stage='return', detail=repr(exc))
        return False


class Entry(gdb.Breakpoint):
    def stop(self):
        try:
            stack = number('$esp')
            words = read(stack, 20)
            ret, dst, capacity, src, length = [int.from_bytes(words[i:i+4], 'little') for i in range(0, 20, 4)]
            if not src or not dst: return False
            for expected, name in SOURCES.items():
                if length not in (len(expected), 0xffffffff): continue
                actual = read(src, len(expected) + (length == 0xffffffff))
                if actual == expected + (b'\0' if length == 0xffffffff else b''):
                    thread = gdb.selected_thread().global_num
                    Return(ret, stack + 4, thread, dst, capacity, name)
                    self.enabled = False
                    record('entry', site='0x149a740', return_site=hex(ret), stack=hex(stack),
                           thread=thread, source_ptr=hex(src), destination=hex(dst),
                           capacity=capacity, length=length, source=name,
                           source_sha256=hashlib.sha256(expected).hexdigest())
                    break
        except Exception as exc:
            record('probe_error', stage='entry', detail=repr(exc))
        return False


Entry('*0x149a740', internal=True)
record('installed', limits='one exact title conversion, no writes or inferior calls')
