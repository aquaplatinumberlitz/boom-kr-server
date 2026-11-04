"""Optional fixture policy for direct chains on declared clear cells.

Not a native map loader, recursive blast simulator, or recovered server rule.
Unknown cells stop a ray. Keeping secondary jobs alive preserves existing wire
timing; only their capacity reservations may be released by this policy.
"""

import hashlib


class ClearCellChains:
    def __init__(self, spec, frames):
        if (not isinstance(spec, dict)
                or set(spec) != {'fixtures', 'clear_cells', 'maximum_power'}):
            raise ValueError('invalid capacity_chain configuration')
        pins = spec['fixtures']
        if (not isinstance(pins, dict) or set(pins) != {'map', 'start'}
                or any(name not in frames or not isinstance(frames[name], bytes)
                       or hashlib.sha256(frames[name]).hexdigest() != expected
                       for name, expected in pins.items())):
            raise ValueError('capacity_chain map/start fixture hash mismatch')
        cells = spec['clear_cells']
        if (not isinstance(cells, list) or not cells
                or any(not isinstance(cell, list) or len(cell) != 2
                       or any(type(value) is not int or not 0 <= value <= 255
                              for value in cell) for cell in cells)):
            raise ValueError('invalid capacity_chain clear_cells')
        self.cells = frozenset(map(tuple, cells))
        if len(self.cells) != len(cells):
            raise ValueError('duplicate capacity_chain clear cell')
        power = spec['maximum_power']
        if type(power) is not int or not 1 <= power <= 255:
            raise ValueError('invalid capacity_chain maximum_power')
        self.maximum_power = power

    def direct_targets(self, root, pending):
        """Select only the first unambiguous occupied clear cell on each ray.

        The caller supplies current, fence-valid, unreleased bombs. We do not
        infer transitive chains, overlap, moving bombs or any unknown terrain.
        """
        row, column, power = root['row'], root['column'], root['power']
        if (row, column) not in self.cells or not 1 <= power <= self.maximum_power:
            return []
        occupied = {}
        for bomb in pending:
            occupied.setdefault((bomb['row'], bomb['column']), []).append(bomb)
        targets = []
        for dr, dc in [(0, -1), (-1, 0), (0, 1), (1, 0)]:
            for distance in range(1, power + 1):
                cell = row + dr * distance, column + dc * distance
                if cell not in self.cells:
                    break
                at_cell = occupied.get(cell, [])
                if at_cell:
                    # Ambiguous overlaps retain all reservations. Stop here
                    # even for an unsupported target power/variant.
                    if len(at_cell) == 1 and 1 <= at_cell[0]['power'] <= self.maximum_power:
                        targets.append(at_cell[0])
                    break
        return targets
