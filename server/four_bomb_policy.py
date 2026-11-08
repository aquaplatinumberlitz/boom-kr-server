"""Candidate emulator choice for four-owner current-round bomb admission.

No native capacity defaults. Caller must supply measured/operator capacities;
wire encoder, delivery, terrain and game result authority are separate.
"""
import hashlib
from server import probe_codec as codec

WIDTHS = [4,1,1,4,1,1,1,1,1,1,1,4,1]


class FourBombAdmission:
    ROLES = 'ABCD'

    def __init__(self, room, slots, capacities, powers):
        if (not all(isinstance(z, dict) for z in [slots, capacities, powers])
                or set(slots) != set(self.ROLES) or sorted(slots.values()) != list(range(len(self.ROLES)))
                or set(capacities) != set(slots) or set(powers) != set(slots)
                or any(type(z) is not int or not 1 <= z <= 255
                       for z in [*capacities.values(), *powers.values()])):
            raise ValueError('explicit '+str(len(self.ROLES))+' distinct slots/capacity/power required')
        self.room, self.slots = room, dict(slots)
        self.capacities, self.powers = dict(capacities), dict(powers)
        self.generation = None
        self.serials, self.pending = {}, {}

    def begin(self):
        if not self.room.active:
            raise RuntimeError('room must start before bomb policy')
        self.generation = self.room.generation
        self.serials, self.pending = {}, {}

    def admit(self, role, body):
        if (not self.room.active or self.generation != self.room.generation
                or role not in self.slots):
            return {'rejected':'not_current_member_round'}
        try:
            values = codec.read_fields(body, WIDTHS)
        except ValueError:
            return {'rejected':'unsupported_bomb_layout'}
        key, slot, serial, row, column, power = values[0], values[2], values[3], values[4], values[5], values[10]
        if key != self.room.key:
            return {'rejected':'bomb_round_key_mismatch'}
        if slot != self.slots[role]:
            return {'rejected':'bomb_owner_slot_mismatch'}
        if [values[i] for i in [1,6,7,8,9,11,12]] != [0,1,0,0,0,1,0]:
            return {'rejected':'unsupported_bomb_variant'}
        if power != self.powers[role]:
            return {'rejected':'bomb_power_not_configured_baseline'}
        if not 0 <= row < 13 or not 0 <= column < 15:
            return {'rejected':'bomb_cell_outside_pinned_map'}
        digest = hashlib.sha256(body).hexdigest()
        previous = self.serials.get((role,serial))
        if previous:
            return {'rejected':'duplicate_bomb_serial' if previous['body_sha256']==digest else 'conflicting_bomb_serial', 'prior_id':previous['id']}
        reserved = [q['id'] for q in self.pending.values() if q['role']==role
                    and self.room.fence.rejection(q['ticket']) is None]
        if len(reserved) >= self.capacities[role]:
            return {'rejected':'bomb_capacity_reserved','reserved_ids':reserved}
        ticket = self.room.bomb()
        record = dict(id=ticket.wire_id,role=role,slot=slot,serial=serial,row=row,
                      column=column,power=power,fields=values,ticket=ticket,
                      generation=self.room.generation,body_sha256=digest)
        self.serials[(role,serial)] = record
        self.pending[ticket.wire_id] = record
        return record

    def expire(self, ident):
        record = self.pending.pop(ident, None)
        if record is None:
            return {'rejected':'unknown_or_already_expired'}
        ticket = record['ticket'];reason = self.room.fence.rejection(ticket)
        if reason is not None:
            return {'rejected':reason,'id':ident}
        if not self.room.fence.consume(ticket):
            return {'rejected':'ticket_not_current','id':ident}
        return record
