"""Read-only KR00967 EAB1C0 range/endpoint probe, loaded by the native harness.

Consumer/chunks: g1-shoe-action-reuse-r11, EAB1C0..EAB326 on the measured
copied executable. This records the actual actor-supplied bound, never sets it.
"""
import json
import time
import gdb

_shoe_calls = {}
_shoe_count = 0


def _shoe_word(address):
    return int.from_bytes(bytes(gdb.selected_inferior().read_memory(address, 4)), 'little')


class ShoeRangeProbe(gdb.Breakpoint):
    def __init__(self, address, kind):
        super().__init__('*'+hex(address), internal=True)
        self.kind = kind

    def stop(self):
        global _shoe_count
        try:
            tid = int(gdb.parse_and_eval('$_thread'))
            if self.kind == 'begin':
                _shoe_count += 1
                if _shoe_count > 100:
                    raise RuntimeError('bounded Shoe range probe exhausted')
                sp = int(gdb.parse_and_eval('$esp'))
                args = [_shoe_word(sp+4*i) for i in range(1,9)]
                _shoe_calls[tid] = dict(source=[args[1],args[2]], direction=args[5],
                                       range_bound=args[7], output=[args[3],args[4]],
                                       caller=hex(_shoe_word(sp)), before_ns=time.monotonic_ns())
            else:
                row = _shoe_calls.pop(tid)
                row['target'] = [_shoe_word(address) for address in row.pop('output')]
                row['changed'] = int(gdb.parse_and_eval('$eax')) & 0xffffffff
                row['ns'] = time.monotonic_ns()
                print('G1_SHOE_RANGE '+json.dumps(row,separators=(',',':')),flush=True)
        except Exception as error:
            print('G1_SHOE_RANGE '+json.dumps(dict(kind='probe_error',error=repr(error))),flush=True)
            self.enabled = False
        return False


ShoeRangeProbe(0xeab1c0,'begin')
ShoeRangeProbe(0xeab320,'end')
