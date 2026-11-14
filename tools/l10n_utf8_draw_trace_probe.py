"""Bounded read-only observer for exact default-title width and draw candidates."""
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
MAX_STOPS = 20000
stops = 0
breakpoints = []


def number(expr): return int(gdb.parse_and_eval(expr)) & 0xffffffff
def read(address, length): return bytes(gdb.selected_inferior().read_memory(address, length))
def record(kind, **fields): print('L10N_DRAW_TRACE ' + json.dumps({'kind': kind, **fields}, sort_keys=True), flush=True)
def word(address): return int.from_bytes(read(address, 4), 'little')


def match(src, length):
    if not src: return None
    for expected, name in SOURCES.items():
        if length not in (len(expected), 0xffffffff): continue
        suffix = b'\0' if length == 0xffffffff else b''
        if read(src, len(expected) + len(suffix)) == expected + suffix:
            return name, hashlib.sha256(expected).hexdigest()
    return None


class Probe(gdb.Breakpoint):
    def __init__(self, address, source_offset, length_offset, width=False):
        super().__init__('*%#x' % address, internal=True)
        self.site, self.source_offset, self.length_offset, self.width = address, source_offset, length_offset, width
        breakpoints.append(self)

    def stop(self):
        global stops
        stops += 1
        try:
            stack = number('$esp')
            ret = word(stack)
            if not self.width or ret == 0x14bae45:
                src, length = word(stack + self.source_offset), word(stack + self.length_offset)
                found = match(src, length)
                if found:
                    fields = dict(site=hex(self.site), return_site=hex(ret), stack=hex(stack),
                                  thread=gdb.selected_thread().global_num, source_ptr=hex(src),
                                  length=length, source=found[0], source_sha256=found[1])
                    if self.width:
                        # 0x14BADF0: 0x804 locals, three saved registers, four conversion args and return address.
                        fields['width_caller'] = hex(word(stack + 0x824))
                    record('match', **fields)
                    self.enabled = False
        except Exception as exc:
            self.enabled = False
            record('probe_error', site=hex(self.site), detail=repr(exc))
        if stops >= MAX_STOPS:
            for bp in breakpoints: bp.enabled = False
            record('budget_exhausted', stops=stops)
        return False


Probe(0x149a740, 12, 16, width=True)
Probe(0x14bb5e0, 24, 28)
Probe(0x14bb7b0, 20, 24)
record('installed', max_stops=MAX_STOPS, limits='one exact title per site; no writes or inferior calls')
