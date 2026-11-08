"""Bounded four-member team result choice; reports are not authenticated life.

KR wire derives from CP023/028/102/103. Team survivor choice follows the VN
reference/owner rules; zero economy and settling are explicit emulator choices.
"""
import hashlib
import struct
from server import probe_codec as codec
from server.probe_service import supported_event_blob
from server.room_title import _encode_bytes


def rank_body(winner_team, teams):
    if (type(winner_team) is not int or winner_team not in (0, 1)
            or teams != {'A': 0, 'B': 1, 'C': 0, 'D': 1}):
        raise ValueError('only declared four-member 2v2 supported')
    header = bytearray(16)
    header[9] = 255  # Measured feature-off signed branch; not all rank schemas.
    rows = []
    for role in 'ABCD':
        row = bytearray(131)
        row[0] = 1
        row[5] = 1 if teams[role] == winner_team else 2
        rows.append(row)
    inner = header + b''.join(rows) + bytes(12)
    stream = b'\x18' + struct.pack('<I', len(inner)+1) + b'\0' + inner
    return codec.pack_fields([len(stream)], [4]) + _encode_bytes(stream)


class FourBattleReports:
    def __init__(self, room, slots, teams):
        if slots != dict(zip('ABCD', range(4))) or teams != {'A':0,'B':1,'C':0,'D':1}:
            raise ValueError('explicit current four-member 2v2 required')
        self.room, self.slots, self.teams = room, dict(slots), dict(teams)
        # Per-connection clock is retained across generations, not compared peers.
        self.cursors = {}
        self.generation = None
        self.life, self.reports = {}, []
        self.last_blast_ns = 0

    def begin(self):
        if not self.room.active:
            raise RuntimeError('room must be active')
        self.generation = self.room.generation
        self.life = {r: {'trap':None,'death':None,'survivor':None} for r in self.slots}
        self.reports = []
        self.last_blast_ns = 0

    def prepare(self, role, body, now_ns):
        if (not self.room.active or self.generation != self.room.generation
                or role not in self.slots):
            return {'rejected':'not_current_member_round'}
        if len(body) < 14:
            return {'rejected':'unsupported_report_prefix'}
        sub, slot, x, y, tick, aux = codec.read_fields(body[:14],[1,1,2,2,4,4])
        if slot != self.slots[role]:
            return {'rejected':'report_slot_mismatch'}
        if sub not in (17,19,21):
            return {'rejected':'unsupported_four_report_subtype'}
        try:
            if sub in (17,19):
                raw = supported_event_blob(body,sub)
                payload = (body[:6]+codec.pack_fields([int.from_bytes(raw[10:14],'little')],[4])
                           if sub==17 else body[:6]+body[14:])
            elif len(body) != 14:
                return {'rejected':'unsupported_report21_shape'}
            else:
                payload = body[:6]
        except ValueError:
            return {'rejected':'unsupported_report_blob'}
        digest = hashlib.sha256(body).hexdigest()
        cursor = self.cursors.get(role)
        if cursor:
            delta = (tick-cursor['tick']) & 0xffffffff
            if delta >= 0x80000000:
                return {'rejected':'report_tick_not_forward'}
            if delta == 0 and digest in cursor['bodies']:
                return {'rejected':'duplicate_report_body'}
        life = self.life[role]
        if life['death'] is not None:
            return {'rejected':'report_after_accepted_death'}
        if sub==19 and life['trap'] is None:
            return {'rejected':'death_no_current_trap'}
        return dict(role=role,slot=slot,subtype=sub,client_tick=tick,actor_field=aux,
                    x=x,y=y,body_sha256=digest,monotonic_ns=now_ns,
                    generation=self.generation,forward_payload=payload,
                    forward=(sub in (17,19) or life['trap'] is not None))

    def commit(self, report):
        """Caller commits only after all intended transport sends succeeded."""
        role=report['role'];tick=report['client_tick'];cursor=self.cursors.get(role)
        if not cursor or cursor['tick']!=tick:
            cursor={'tick':tick,'bodies':set()};self.cursors[role]=cursor
        cursor['bodies'].add(report['body_sha256'])
        life=self.life[role];sub=report['subtype']
        if sub==17:
            life['trap']=report;life['survivor']=None
        elif sub==19:
            life['death']=report;life['survivor']=None
        else:
            life['survivor']=report
        self.reports.append(report)

    def outcome(self, now_ns, pending):
        if (not self.room.active or self.generation!=self.room.generation or pending):
            return None
        alive_teams={self.teams[r] for r,z in self.life.items() if z['death'] is None}
        if len(alive_teams)!=1:
            return None  # All eliminated/tie is deliberately unsupported here.
        winner=next(iter(alive_teams))
        deaths=[z['death'] for z in self.life.values() if z['death'] is not None]
        latest=max([d['monotonic_ns'] for d in deaths]+[self.last_blast_ns])
        if now_ns-latest < 8*10**9:
            return None
        witnesses=[r for r,z in self.life.items() if z['death'] is None
                   and z['survivor'] is not None
                   and z['survivor']['monotonic_ns']>latest
                   and (z['trap'] is None or z['survivor']['monotonic_ns']>z['trap']['monotonic_ns'])]
        if not witnesses:
            return None
        return dict(winner_team=winner,witnesses=witnesses,
                    death_roles=[r for r,z in self.life.items() if z['death'] is not None],
                    source='design',policy='current_four_2v2_report_derived_drained_settle8s',
                    last_blast_ns=self.last_blast_ns,latest_death_ns=max(d['monotonic_ns'] for d in deaths))
