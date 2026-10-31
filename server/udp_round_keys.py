"""Emulator allocation policy for the bounded movement key observed in CP047.

Keep one instance for the lifetime in which old peer traffic can still arrive.
The serialized room owner allocates once per accepted Start and sends the same
value to every member in KR153. This module neither validates UDP nor manages
peer lifetimes. Distinct low bytes are useful for the measured stable roster;
they are not authentication or a substitute for validating other packet types.
"""


class UdpRoundKeys:
    """Issue u32 values without reusing a low byte; stop after 256 rounds."""

    def __init__(self, initial_key: int) -> None:
        if type(initial_key) is not int or not 0 <= initial_key <= 0xFFFFFFFF:
            raise ValueError('initial key must be a u32 integer')
        self._initial_key = initial_key
        self._issued_count = 0

    @property
    def issued_count(self) -> int:
        return self._issued_count

    def next_key(self) -> int:
        if self._issued_count == 256:
            raise RuntimeError('no unused low-byte round keys remain')
        key = (self._initial_key + self._issued_count) & 0xFFFFFFFF
        self._issued_count += 1
        return key
