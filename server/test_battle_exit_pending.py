"""Narrow normal-exit pending-ticket policy, distinct from original KR authority."""
import time
import unittest
from server import test_battle_exit as fixture


class PendingBattleExitTests(unittest.TestCase):
    setUp = fixture.BattleExitTests.setUp
    tick = fixture.BattleExitTests.tick
    send_request = fixture.BattleExitTests.send_request
    body = fixture.BattleExitTests.body

    def begin(self):
        self.service.battle_exit = True
        self.service.battle_exit_mode = 'guest_B_pending_202_86'
        self.service.start_round()
        self.sent.clear()

    def bomb(self, owner=1):
        ticket = self.service.room.bomb()
        bomb = {'round': self.service.room.generation, 'id': ticket.wire_id, 'owner': owner}
        self.service.state['round_bombs'].append(bomb)
        self.service.jobs.append((time.monotonic_ns()+100*10**9, ticket, bomb))
        return ticket, bomb

    def test_pair_closes_fence_pending_job_drops_without158_or_terrain_commit(self):
        self.begin()
        ticket, bomb = self.bomb()
        self.send_request(202, self.body)
        self.send_request(86, seq=103)
        self.assertEqual(self.b['last_room_leave']['variant'], 'active_guest_B')
        self.assertEqual(self.service.room.fence.rejection(ticket), 'round_closed')
        self.service.jobs[0] = (0, ticket, bomb)
        self.tick(11)
        self.assertEqual(bomb['timer_rejected'], 'round_closed')
        self.assertNotIn('broadcast158', bomb)
        self.assertNotIn('terrain_blast', bomb)
        self.assertEqual(self.service.jobs, [])
        self.assertEqual([(q['role'], q['alias']) for q in self.sent], [('B','ready'), ('B','roomlist_B')])
        self.assertNotIn('death_event', self.b)

    def test_bomb_race_after_stage_prevents_leave_if_already_broadcast(self):
        self.begin()
        _, bomb = self.bomb()
        self.send_request(202, self.body)
        bomb['broadcast158'] = [{'monotonic_ns': 1}]
        self.send_request(86, seq=103)
        self.assertEqual(self.service.state['room_leave_requests'][-1]['rejected'], 'round_not_returned')
        self.assertTrue(self.service.room.active)
        self.assertIn('map_response_sent', self.b)
        self.assertEqual(self.sent, [])

    def test_other_owner_multiple_bombs_dead_or_missing_ticket_do_not_stage(self):
        for variant in ['ownerA','two','expired','missing','trap']:
            with self.subTest(variant=variant):
                self.service.state['round_bombs'].clear()
                self.service.state['event_reports'].clear()
                self.service.jobs.clear()
                self.b.pop('battle_exit202', None)
                if not self.service.room.active:
                    self.begin()
                ticket,bomb=self.bomb(owner=0 if variant=='ownerA' else 1)
                if variant=='two': self.bomb()
                if variant=='expired': bomb['timer_rejected']='round_closed'
                if variant=='missing': self.service.jobs.clear()
                if variant=='trap': self.service.state['event_reports'].append({'round':self.service.room.generation,'subtype':17})
                self.send_request(202, self.body)
                self.assertIn(self.service.state['battle_exit_reports'][-1]['rejected'],
                              ['unsupported_battle_exit_pending_bomb','unsupported_battle_exit_after_trap'])
                self.assertNotIn('battle_exit202', self.b)
                self.assertEqual(self.sent, [])

    def test_new_mode_accepts_unbombed_case_and_profile_configuration(self):
        from server.probe_service import ProbeService
        from pathlib import Path
        self.service.profile.data['battle_exit']={'mode':'guest_B_pending_202_86'}
        s=ProbeService(self.service.profile,Path('/tmp'),{})
        self.addCleanup(s.selector.close)
        self.assertEqual(s.battle_exit_mode,'guest_B_pending_202_86')
        self.begin()
        self.send_request(202,self.body)
        self.send_request(86,seq=103)
        self.assertFalse(self.service.room.active)

if __name__=='__main__':unittest.main()
