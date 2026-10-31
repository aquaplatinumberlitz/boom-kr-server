"""Emulator policy for local pickup confirmations, under one room owner.

The caller supplies the slot bound to the socket and the current item catalog.
These checks do not recover a wire epoch: an old request whose fields all match
the current catalog cannot be distinguished here. Serialize claim/consume/send.
"""

from dataclasses import dataclass, field


def _uint(value: int, maximum: int) -> bool:
    return type(value) is int and 0 <= value <= maximum


@dataclass(frozen=True)
class ItemSpec:
    index: int
    row: int
    column: int
    kind: int


@dataclass(frozen=True)
class ItemClaim:
    generation: int
    slot: int
    context: int
    item: ItemSpec
    _scope: object = field(repr=False)


class ItemClaims:
    def __init__(self) -> None:
        self._scope = object()
        self._generation = 0
        self._active = False
        self._context = 0
        self._items: dict[int, ItemSpec] = {}
        self._claimed: set[int] = set()
        self._pending: dict[int, ItemClaim] = {}

    def begin(self, items: list[ItemSpec], context: int = 0) -> None:
        if not _uint(context, 255):
            raise ValueError('context outside u8')
        catalog: dict[int, ItemSpec] = {}
        for item in items:
            if not all(_uint(v, maximum) for v, maximum in (
                (item.index, 0xffffffff), (item.row, 255),
                (item.column, 255), (item.kind, 65535),
            )) or item.index in catalog:
                raise ValueError('invalid or duplicate item')
            catalog[item.index] = item
        self._generation += 1
        self._active = True
        self._context = context
        self._items = catalog
        self._claimed.clear()
        self._pending.clear()

    def finish(self) -> None:
        self._active = False
        self._items.clear()
        self._pending.clear()

    def claim(self, bound_slot: int, reported_slot: int, context: int,
              item: ItemSpec) -> ItemClaim | None:
        if not self._active or not _uint(bound_slot, 25):
            return None
        if not _uint(reported_slot, 25) or reported_slot != bound_slot:
            return None
        if not _uint(context, 255) or context != self._context:
            return None
        if not all(_uint(v, maximum) for v, maximum in (
            (item.index, 0xffffffff), (item.row, 255),
            (item.column, 255), (item.kind, 65535),
        )):
            return None
        if self._items.get(item.index) != item or item.index in self._claimed:
            return None
        claim = ItemClaim(self._generation, bound_slot, context, item, self._scope)
        self._claimed.add(item.index)
        self._pending[item.index] = claim
        return claim

    def rejection(self, claim: ItemClaim) -> str | None:
        if claim._scope is not self._scope:
            return 'different_room_owner'
        if claim.generation != self._generation:
            return 'stale_round'
        if not self._active:
            return 'round_closed'
        if self._pending.get(claim.item.index) is not claim:
            return 'already_consumed_or_unknown'
        return None

    def consume(self, claim: ItemClaim) -> bool:
        if self.rejection(claim) is not None:
            return False
        del self._pending[claim.item.index]
        return True
