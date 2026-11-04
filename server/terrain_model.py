"""Pure candidate kernel for a pinned simple KR map, never a native oracle.

Compare its output with retained native callbacks/flag snapshots before using
any part in a service policy. No networking, reports, server imports or assets.
Traversal uses an immutable terrain snapshot for the whole cascade, then clears
hit boxes. First occupied bomb stops each ray. These are bounded model choices;
agreement on measured inputs does not recover a general KR propagation law.
"""

from collections import deque
from dataclasses import dataclass
import re


@dataclass(frozen=True)
class Bomb:
    id: int
    owner: int
    row: int
    column: int
    power: int


@dataclass(frozen=True)
class Blast:
    processed: tuple[int, ...]
    cleared: tuple[tuple[int, int], ...]
    before: tuple[int, ...]
    after: tuple[int, ...]


class TerrainShadow:
    """0=floor, 1=solid, 2=box, only for explicitly supplied target cells."""

    def __init__(self, width, height, kinds, *, map_sha256, maximum_power=2):
        if (type(width) is not int or type(height) is not int
                or not 1 <= width <= 255 or not 1 <= height <= 255
                or len(kinds) != width * height
                or any(type(kind) is not int or kind not in (0, 1, 2) for kind in kinds)
                or type(maximum_power) is not int or not 1 <= maximum_power <= 2
                or not isinstance(map_sha256, str)
                or not re.fullmatch(r'[0-9a-f]{64}', map_sha256)):
            raise ValueError('unsupported shadow profile')
        self.width, self.height = width, height
        self.seed = tuple(kinds)
        self.map_sha256, self.maximum_power = map_sha256, maximum_power
        self.begin()

    def begin(self):
        self.kinds = list(self.seed)
        self.pending = {}
        self.seen_ids = set()

    @property
    def flags(self):
        return tuple(int(kind != 0) for kind in self.kinds)

    def place(self, bomb):
        if (not isinstance(bomb, Bomb)
                or any(type(value) is not int for value in
                       (bomb.id, bomb.owner, bomb.row, bomb.column, bomb.power))
                or not 1 <= bomb.id <= 65535 or bomb.id in self.seen_ids
                or bomb.owner not in (0, 1)
                or not 0 <= bomb.row < self.height or not 0 <= bomb.column < self.width
                or not 1 <= bomb.power <= self.maximum_power
                or self.kinds[bomb.row * self.width + bomb.column] != 0
                or any((other.row, other.column) == (bomb.row, bomb.column)
                       for other in self.pending.values())):
            raise ValueError('unsupported shadow bomb')
        self.pending[bomb.id] = bomb
        self.seen_ids.add(bomb.id)

    def blast(self, root_id):
        before = self.flags
        if type(root_id) is not int or root_id not in self.pending:
            return Blast((), (), before, before)
        terrain = tuple(self.kinds)
        occupied = {(bomb.row, bomb.column): bomb for bomb in self.pending.values()}
        queue, queued, processed, boxes = deque([self.pending[root_id]]), {root_id}, [], set()
        while queue:
            bomb = queue.popleft()
            processed.append(bomb.id)
            for dr, dc in ((0, -1), (-1, 0), (0, 1), (1, 0)):
                for distance in range(1, bomb.power + 1):
                    row, col = bomb.row + dr * distance, bomb.column + dc * distance
                    if not 0 <= row < self.height or not 0 <= col < self.width:
                        break
                    kind = terrain[row * self.width + col]
                    if kind:
                        if kind == 2:
                            boxes.add((row, col))
                        break
                    target = occupied.get((row, col))
                    if target:
                        if target.id not in queued:
                            queued.add(target.id)
                            queue.append(target)
                        break
        for bomb_id in processed:
            del self.pending[bomb_id]
        for row, col in boxes:
            self.kinds[row * self.width + col] = 0
        return Blast(tuple(processed), tuple(sorted(boxes)), before, self.flags)
