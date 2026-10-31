"""Local emulator policy for serialized room owners; not a KR wire field.

Keep tickets with scheduled work instead of looking up a new bomb by wire ID
when an old callback fires. A fresh fence must be created for a new room owner.
The caller owns deadlines, transport and synchronization. This is not an input
validator, persistence layer, or protection against stale packets already sent.
"""

from dataclasses import dataclass, field


@dataclass(frozen=True)
class BombTicket:
    generation: int
    allocation: int
    wire_id: int
    _scope: object = field(repr=False)


class RoundFence:
    """Invalidate local jobs on round transitions and permit one consumption."""

    def __init__(self) -> None:
        self._scope = object()
        self._generation = 0
        self._allocation = 0
        self._active = False
        self._bombs: dict[int, BombTicket] = {}

    @property
    def generation(self) -> int:
        return self._generation

    def begin(self) -> int:
        self._generation += 1
        self._bombs.clear()
        self._active = True
        return self._generation

    def finish(self) -> None:
        self._active = False
        self._bombs.clear()

    def register(self, wire_id: int) -> BombTicket:
        if not self._active:
            raise RuntimeError('no active round')
        if type(wire_id) is not int or not 1 <= wire_id <= 65535:
            raise ValueError('bomb wire ID must be an integer in 1..65535')
        if wire_id in self._bombs:
            raise ValueError('bomb wire ID already active')
        self._allocation += 1
        ticket = BombTicket(self._generation, self._allocation, wire_id, self._scope)
        self._bombs[wire_id] = ticket
        return ticket

    def rejection(self, ticket: BombTicket) -> str | None:
        if ticket._scope is not self._scope:
            return 'different_room_owner'
        if ticket.generation != self._generation:
            return 'stale_round'
        if not self._active:
            return 'round_closed'
        if self._bombs.get(ticket.wire_id) is not ticket:
            return 'inactive_or_replaced_bomb'
        return None

    def consume(self, ticket: BombTicket) -> bool:
        if self.rejection(ticket) is not None:
            return False
        del self._bombs[ticket.wire_id]
        return True
