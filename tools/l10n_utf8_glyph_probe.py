"""GDB-only, exact-string glyph probe on an explicitly authorized client copy.

This deliberately changes one observed conversion result. It does not patch the
binary or test UTF-8 input, keyboard/IME, shared text or packet semantics. Install
only after the diagnostic image has loaded; use an unmodified peer as control.
"""
import gdb
import json

SOURCES = {bytes.fromhex(value): name for name, value in (
    ('DefaultRoomTitle0', 'c1f1b0c5bfee20b0d4c0d3c7d8bfe4'),
    ('DefaultRoomTitle1', 'bec6b9abb3aa20bfc0bcbcbfe4'),
    ('DefaultRoomTitle2', 'c0fab6fb20b0d4c0d3c7cfbdc7bad07e'),
    ('DefaultRoomTitle3', 'bba1b8ae20bfc0bcbcbfe47e'),
    ('DefaultRoomTitle4', 'c3cabab8c0da20c8afbfb521'),
    ('DefaultRoomTitle5', 'b8c5b3cab0d720c7cabcf6'),
)}
WIDE = bytes.fromhex('5cd5200056006900c71e7400200045004e00')
MODE = globals().get('L10N_GLYPH_MODE', 'pass')
ENTRY_ADDRESS = globals().get('L10N_GLYPH_ENTRY', 0x149a740)
RECORD_PREFIX = globals().get('L10N_GLYPH_PREFIX', 'L10N_GLYPH ')
if MODE not in ('pass', 'replace'):
    raise ValueError('L10N_GLYPH_MODE must be pass or replace')


def value(expression):
    return int(gdb.parse_and_eval(expression)) & 0xffffffff


def record(kind, **fields):
    print(RECORD_PREFIX + json.dumps({'kind': kind, **fields}, sort_keys=True), flush=True)


class Return(gdb.Breakpoint):
    def __init__(self, address, stack_after_return, thread, destination, capacity, null_terminated, name, expected):
        super().__init__('*%#x' % address, internal=True)
        self.stack_after_return = stack_after_return
        self.thread = thread
        self.destination = destination
        self.capacity = capacity
        self.null_terminated = null_terminated
        self.name = name
        self.expected = expected

    def retire(self):
        if self.is_valid():
            self.enabled = False

    def stop(self):
        try:
            if gdb.selected_thread().global_num != self.thread or value('$esp') != self.stack_after_return:
                return False
            self.retire()
            units = len(WIDE) // 2
            if self.capacity < units + 1:
                record('capacity_skip', capacity=self.capacity, required=units + 1)
                return False
            previous = value('$eax')
            actual = bytes(gdb.selected_inferior().read_memory(self.destination, len(self.expected) + 2))
            if previous != len(self.expected) // 2 + int(self.null_terminated) or actual != self.expected + b'\x00\x00':
                record('conversion_mismatch_skip', previous_units=previous)
                return False
            if MODE == 'pass':
                record('pass_through', previous_units=previous, capacity=self.capacity, input=self.name)
                return False
            gdb.selected_inferior().write_memory(self.destination, WIDE + b'\x00\x00')
            returned = units + int(self.null_terminated)
            gdb.execute('set $eax = %d' % returned, to_string=True)
            record('replacement', previous_units=previous, output_units=units,
                   returned_units=returned, capacity=self.capacity, input=self.name)
        except Exception as exc:
            try:
                self.retire()
            except Exception:
                pass
            record('probe_error', stage='return', detail=repr(exc))
        return False


class Entry(gdb.Breakpoint):
    def stop(self):
        try:
            stack = value('$esp')
            return_address = int.from_bytes(bytes(gdb.selected_inferior().read_memory(stack, 4)), 'little')
            raw = bytes(gdb.selected_inferior().read_memory(stack + 4, 16))
            destination, capacity, source, length = [int.from_bytes(raw[i:i+4], 'little')
                                                     for i in range(0, 16, 4)]
            if not source or not destination:
                return False
            for expected, name in SOURCES.items():
                if length not in (len(expected), 0xffffffff):
                    continue
                data = bytes(gdb.selected_inferior().read_memory(source, len(expected) + (length == 0xffffffff)))
                if data == expected + (b'\x00' if length == 0xffffffff else b''):
                    expected_wide = expected.decode('cp949').encode('utf-16le')
                    Return(return_address, stack + 4, gdb.selected_thread().global_num, destination,
                           capacity, length == 0xffffffff, name, expected_wide)
                    self.enabled = False
                    record('match', input=name, input_bytes=len(expected), capacity=capacity)
                    break
        except Exception as exc:
            record('probe_error', stage='entry', detail=repr(exc))
        return False


Entry('*%#x' % ENTRY_ADDRESS, internal=True)
def retire_on_exit(event):
    for bp in gdb.breakpoints() or ():
        if isinstance(bp, Return) and bp.is_valid():
            bp.retire()
gdb.events.exited.connect(retire_on_exit)
record('installed', mode=MODE, limits='one exact default CP949 title conversion; diagnostic copy only; not UTF-8 input or protocol')
