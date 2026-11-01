"""Serialized owner for the fixture probe; emulator choices, not original rules."""

from server.item_claims import ItemClaims, ItemSpec
from server.round_fence import RoundFence
from server.udp_round_keys import UdpRoundKeys


class ProbeRoom:
    def __init__(self, items: list[ItemSpec], initial_key: int = 12345):
        self.items = items
        self.keys = UdpRoundKeys(initial_key)
        self.fence = RoundFence()
        self.claims = ItemClaims()
        self.active = False
        self.generation = 0
        self.key = None
        self._wire_id = 0

    def begin(self) -> int:
        if self.active:
            raise RuntimeError('round already active')
        # Exhaustion fails before modifying the active policies.
        key = self.keys.next_key()
        self.claims.begin(self.items)
        self.generation = self.fence.begin()
        self.key = key
        self.active = True
        return key

    def finish(self) -> None:
        self.active = False
        self.fence.finish()
        self.claims.finish()

    def bomb(self):
        if not self.active:
            raise RuntimeError('no active round')
        if self._wire_id == 65535:
            raise RuntimeError('diagnostic bomb ID space exhausted')
        self._wire_id += 1
        return self.fence.register(self._wire_id)

    def claim(self, bound_slot: int, slot: int, context: int, item: ItemSpec):
        return self.claims.claim(bound_slot, slot, context, item)
