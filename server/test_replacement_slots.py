"""Policy/transport regressions; synthetic test fixtures are not native evidence."""
import copy
import hashlib
from pathlib import Path
import socket
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from server import probe_codec as c, replacement_slots as slots
from server.probe_service import ProbeService, ROLE_SLOTS


class ReplacementSlotTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.out = Path(temp.name)
        self.bodies = {
            'join_C': c.pack_fields([0, 0, 1, 0, 0, 0], [1, 1, 2, 1, 1, 1]),
            'player_C': c.pack_fields([0, 3, 0, 0, 0, 0], [1, 4, 1, 1, 1, 4]) + bytes(range(192)),
            'guest_B': c.pack_fields([0, 1, 0, 0, 0, 0], [1, 4, 1, 1, 1, 4]) + b'a' * 192,
            'guest_C': c.pack_fields([1, 2, 0, 0, 0, 0], [1, 4, 1, 1, 1, 4]) + b'b' * 192,
            'slot_C': c.pack_fields([0, 0, 0], [1, 4, 1]),
            'ready_C': c.pack_fields([0, 4, 0], [1, 4, 1]),
            'slot_B': c.pack_fields([1, 32, 0], [1, 4, 1]),
            'ready_slot': c.pack_fields([1, 36, 0], [1, 4, 1]),
            'slot_A': c.pack_fields([0, 2, 0], [1, 4, 1]),
            'create': c.pack_fields([0, 1, 1, 0, 0, 0], [1, 2, 4, 1, 1, 2]),
            'join': c.pack_fields([0, 1, 1, 0, 0, 0], [1, 1, 2, 1, 1, 1]),
            'return': c.pack_fields([0, 0, 0], [1, 1, 1]),
        }
        self.opcodes = {name: opcode for name, (opcode, _) in slots.PINS.items()}
        self.opcodes.update({'create': 86, 'join': 87, 'return': 180, 'start': 121, 'ready': 241,
                             'roomlist_C': 91, 'roomlist_B': 91, 'player_A': 106, 'slot_A': 110})
        for name in ['rank_Awin', 'rank', 'promote_B', 'auth_C', 'roomlist_C', 'roomlist_B',
                     'start', 'floor', 'ready', 'channel_list', 'player_A', 'table', 'map']:
            self.bodies.setdefault(name, b'fixture')
        self.frames = {name: c.encode(bytes(37), self.opcodes.get(name, 0), body, 55)
                       for name, body in self.bodies.items()}
        self.data = {'items': [], 'initial_key': 12345,
                     'replacement_slot': {'mode': 'vacant_AB_slot'},
                     'room_team_mode': {'mode': 'create81_to_86_87'},
                     'fixtures': {name: {'sequence': 55} for name in self.frames}}
        self.profile = SimpleNamespace(data=self.data, frames=self.frames,
                                       raw=lambda name: self.frames[name],
                                       body=lambda name: self.bodies[name])
        # Isolate policy from native pins: production uses exact known KR body hashes.
        pins = {name: (opcode, hashlib.sha256(self.bodies[name]).hexdigest())
                for name, (opcode, _) in slots.PINS.items()}
        self.pin_patch = patch.dict(slots.PINS, pins, clear=True)
        self.pin_patch.start()
        self.addCleanup(self.pin_patch.stop)
        self.s = ProbeService(self.profile, self.out, {})
        self.addCleanup(self.s.selector.close)
        self.a = {'role': 'A', 'reply_ordinal': 4, 'global_ordinal': 4,
                  'map_response_sent': True, 'create_response_sent': True,
                  'departures_sent': ['B'], 's2c_next_seq': 244}
        self.b = {'role': 'B', 'reply_ordinal': 4, 'global_ordinal': 8,
                  'closed': {'reason': 'eof'}, 'map_response_sent': True}
        self.s.events = [{}, {}, {}, self.a, {}, {}, {}, self.b]
        self.s.channels = {'A': self.a, 'B': self.b}
        self.s.round = {'index': 3, 'refresh': {'monotonic_ns': 1}}
        self.s.room_team_selector = 1
        self.sent = []
        self.s.send_raw = lambda event, raw: self.sent.append((event['role'], raw))

    def accept(self):
        connection, remote = socket.socketpair()
        self.addCleanup(connection.close)
        self.addCleanup(remote.close)
        listener = SimpleNamespace(accept=lambda: (connection, ('10.77.0.3', 1234)),
                                   getsockname=lambda: ('10.77.0.1', 3858))
        # No bootstrap parsing in this policy test; retain real service accept/selector.
        self.frames['bootstrap'] = b'bootstrap'
        self.s.accept(listener)
        return connection

    def bind_c(self):
        self.accept()
        event = {'role': 'C', 'reply_ordinal': 4, 'global_ordinal': 12,
                 's2c_next_seq': 73}
        self.s.channels['C'] = event
        self.s.events.append(event)
        return event

    def dispatch(self, event, opcode):
        frame = c.encode(bytes(37), opcode, b'', 40)
        self.s.receive_channel(event, frame, 40, b'')

    def test_optin_binds_vacant_slot_without_global_or_other_instance_mutation(self):
        self.accept()
        self.assertEqual(self.s.slot('C'), 1)
        self.assertEqual(ROLE_SLOTS['C'], 0)
        other = ProbeService(self.profile, self.out, {})
        self.addCleanup(other.selector.close)
        self.assertEqual(other.slot('C'), 0)
        self.assertEqual(self.s.replacement_binding['connection'], 4)
        self.assertEqual(self.s.replacement_binding['identity'], 3)

    def test_default_still_rejects_a_survivor(self):
        self.s.replacement_slot = False
        connection = self.accept()
        self.assertEqual(connection.fileno(), -1)
        self.assertIsNone(self.s.replacement_binding)
        self.assertEqual(self.s.state['admission_rejections'][-1]['reason'], 'replacement_not_ready')

    def test_cleanup_refresh_host_and_live_survivor_are_required(self):
        cases = [lambda: self.a.pop('departures_sent'),
                 lambda: self.a.update(pending_departures=['B']),
                 lambda: self.a.pop('map_response_sent'),
                 lambda: self.a.update(closed={'reason': 'eof'}),
                 lambda: self.s.round.pop('refresh'),
                 lambda: setattr(self.s, 'host_role', 'B'),
                 lambda: self.b.pop('closed'), lambda: self.s.room.begin()]
        a, b, rnd = copy.deepcopy(self.a), copy.deepcopy(self.b), copy.deepcopy(self.s.round)
        for change in cases:
            self.a.clear(); self.a.update(copy.deepcopy(a))
            self.b.clear(); self.b.update(copy.deepcopy(b))
            self.s.round = copy.deepcopy(rnd); self.s.host_role = 'A'
            change()
            self.assertFalse(self.s.replacement_allowed())
        self.assertEqual(self.sent, [])

    def test_old_b_survivor_still_binds_slot_zero(self):
        self.a['closed'] = {'reason': 'eof'}
        self.b.pop('closed'); self.s.host_role = 'B'
        self.accept()
        self.assertEqual(self.s.slot('C'), 0)
        self.assertEqual(self.s.fixture_body('player_C'), self.bodies['player_C'])

    def test_current_selector_c_slot1_own_profile_tail_and_sequence(self):
        event = self.bind_c()
        self.dispatch(event, 82)
        body = c.frame_body(self.sent[-1][1], 55)
        self.assertEqual(c.read_fields(body, [1, 1, 2, 1, 1, 1]), [0, 1, 1, 0, 1, 0])
        self.dispatch(event, 585)
        own = c.frame_body(self.sent[-2][1], 55)
        peer = c.frame_body(self.sent[-1][1], 244)
        self.assertEqual(own, peer)
        self.assertEqual(c.read_fields(own[:5], [1, 4]), [1, 3])
        self.assertEqual(own[1:], self.bodies['player_C'][1:])
        self.assertEqual(self.sent[-1][0], 'A')
        self.assertEqual(self.a['s2c_next_seq'], 247)
        self.assertEqual(self.bodies['player_C'][0:1], c.pack_fields([0], [1]))
        self.assertEqual(self.s.fixture_body('guest_C'), self.bodies['guest_B'])
        self.assertEqual(c.read_fields(self.s.fixture_body('slot_C'), [1, 4, 1]), [1, 32, 0])

    def test_early_ready_changed_survivor_and_missing_selector_cannot_start(self):
        event = self.bind_c()
        self.dispatch(event, 95)
        self.assertNotIn('guest_ready_received', event)
        self.s.room_team_selector = None
        self.dispatch(event, 82)
        self.assertNotIn('create_response_sent', event)
        self.s.room_team_selector = 1
        self.a['global_ordinal'] = 44
        self.dispatch(event, 82)
        self.assertNotIn('create_response_sent', event)
        self.assertEqual(self.s.state['room_reentry_rejections'][-1]['reason'], 'replacement_survivor_changed')
        self.dispatch(self.a, 104)
        self.assertFalse(self.s.room.active)

    def test_slot1_team_state_reaches_survivor_before_guest_ready(self):
        event = self.bind_c()
        self.dispatch(event, 82)
        self.dispatch(event, 585)
        before = self.a['s2c_next_seq']
        self.s.tick_setup(event['player_response_sent']['monotonic_ns'] + 2 * 10**9)
        self.assertNotIn('guest_ready_received', event)
        update = event['replacement_slot_sent']
        self.assertEqual(update['seq'], before)
        raw = (self.out / update['fixture']).read_bytes()
        self.assertEqual(c.read_fields(c.frame_body(raw, before), [1, 4, 1]), [1, 32, 0])
        self.assertEqual(self.a['s2c_next_seq'], before + 3)

    def test_c_ready_broadcast_slot1_then_a_start_fresh_roster(self):
        event = self.bind_c(); event['map_response_sent'] = True
        self.dispatch(self.a, 104)
        self.assertFalse(self.s.room.active)
        self.dispatch(event, 95)
        for role, raw in self.sent[-2:]:
            seq = 244 if role == 'A' else 73
            self.assertEqual(c.read_fields(c.frame_body(raw, seq), [1, 4, 1]), [1, 36, 0])
        self.dispatch(self.a, 104)
        self.assertEqual(self.s.round['roster'], [{'role': 'A', 'slot': 0, 'identity': 1},
                                                 {'role': 'C', 'slot': 1, 'identity': 3}])
        self.assertEqual(self.s.round['host_role'], 'A')
        self.assertEqual(self.s.slot('B'), 1)  # Historical B slot is not rewritten.

    def test_pin_mismatch_and_missing_explicit_capacity_fail_preflight(self):
        self.bodies['player_C'] += b'bad'
        with self.assertRaisesRegex(ValueError, 'fixture: player_C'):
            ProbeService(self.profile, self.out, {})
        self.bodies['player_C'] = self.bodies['player_C'][:-3]
        self.data['bomb_capacity'] = {'A': {'initial': 1, 'maximum': 7},
                                      'B': {'initial': 2, 'maximum': 7}}
        with self.assertRaisesRegex(ValueError, 'explicit C'):
            ProbeService(self.profile, self.out, {})


if __name__ == '__main__':
    unittest.main()
