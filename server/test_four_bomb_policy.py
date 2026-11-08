"""Current four-owner policy controls, synthetic wire; native evidence remains separate."""
import copy
from pathlib import Path
import tempfile
import time
import unittest
from types import SimpleNamespace

from server import probe_codec as c
from server.four_bomb_policy import FourBombAdmission, WIDTHS
from server.four_player_service import FourPlayerService
from server.probe_room import ProbeRoom
from server.test_four_player_service import fixtures

SLOTS = dict(zip('ABCD', range(4)))
SPEC = dict(mode='four_current_baseline', capacities={'A':1,'B':2,'C':1,'D':1},
            powers=dict.fromkeys('ABCD',1),fuse_seconds=6)

class FourBombTests(unittest.TestCase):
    def policy(self):
        room=ProbeRoom([],12345);room.begin()
        policy=FourBombAdmission(room,SLOTS,SPEC['capacities'],SPEC['powers']);policy.begin()
        return room,policy

    def body(self,room,slot,serial=1,key=None,power=1):
        return c.pack_fields([room.key if key is None else key,0,slot,serial,3,3,1,0,0,0,power,1,0],WIDTHS)

    def test_four_owners_same_serial_are_independent_and_duplicates_never_allocate(self):
        room,policy=self.policy()
        for slot,role in enumerate('ABCD'):
            request=self.body(room,slot);accepted=policy.admit(role,request)
            self.assertEqual((accepted['id'],accepted['slot']),(slot+1,slot))
            self.assertEqual(policy.admit(role,request)['rejected'],'duplicate_bomb_serial')
            self.assertEqual(policy.admit(role,self.body(room,slot,power=2))['rejected'],
                             'bomb_power_not_configured_baseline')
        self.assertEqual(room._wire_id,4)
        self.assertEqual(len(policy.pending),4)

    def test_wrong_key_owner_layout_and_bounds_reject_before_ID_or_serial(self):
        room,policy=self.policy()
        for role in 'ABCD':
            slot=SLOTS[role]
            self.assertIn('rejected',policy.admit(role,b''))
            self.assertEqual(policy.admit(role,self.body(room,slot,key=12344))['rejected'],
                             'bomb_round_key_mismatch')
            self.assertEqual(policy.admit(role,self.body(room,(slot+1)%4))['rejected'],
                             'bomb_owner_slot_mismatch')
        self.assertEqual(room._wire_id,0)
        self.assertEqual(policy.pending,{})
        self.assertEqual(policy.serials,{})

    def test_capacity_config_does_not_copy_B_to_D(self):
        room,policy=self.policy()
        for role in 'BD':
            self.assertIn('id',policy.admit(role,self.body(room,SLOTS[role],serial=1)))
        self.assertIn('id',policy.admit('B',self.body(room,1,serial=2)))
        self.assertEqual(policy.admit('D',self.body(room,3,serial=2))['rejected'],'bomb_capacity_reserved')
        self.assertEqual(policy.admit('B',self.body(room,1,serial=3))['rejected'],'bomb_capacity_reserved')

    def test_closed_round_fences_all_four_jobs_and_new_epoch_rearms_serials(self):
        room,policy=self.policy()
        ids=[policy.admit(r,self.body(room,SLOTS[r]))['id'] for r in 'ABCD']
        room.finish()
        self.assertEqual([policy.expire(i)['rejected'] for i in ids],['round_closed']*4)
        room.begin();policy.begin()
        self.assertEqual(policy.admit('D',self.body(room,3))['id'],5)
        self.assertEqual(policy.expire(5)['slot'],3)
        self.assertEqual(policy.expire(5)['rejected'],'unknown_or_already_expired')

    def test_bad_explicit_config_rejected_without_guessing_defaults(self):
        room=ProbeRoom([],12345)
        for caps in [None,[],{},dict.fromkeys('ABC',1),dict.fromkeys('ABCD',0),dict.fromkeys('ABCD',True)]:
            with self.subTest(caps=caps),self.assertRaises(ValueError):
                FourBombAdmission(room,SLOTS,caps,SPEC['powers'])

    def service(self):
        tmp=tempfile.TemporaryDirectory();self.addCleanup(tmp.cleanup)
        profile=fixtures();base=profile.raw
        frames={name:c.encode(bytes(37),op,b'',43) for name,op in [('bomb',159),('blast',158)]}
        profile.raw=lambda name:frames[name] if name in frames else base(name)
        service=FourPlayerService(profile,Path(tmp.name),{r:'127.0.0.'+str(i+2) for i,r in enumerate('ABCD')},
                                  bomb_profile=copy.deepcopy(SPEC))
        self.addCleanup(service.selector.close)
        sent={r:[] for r in 'ABCD'}
        for i,role in enumerate('ABCD'):
            event=dict(role=role,reply_ordinal=4,global_ordinal=i+1,s2c_next_seq=43,
                map_response_sent={'monotonic_ns':1},guest_ready_received={'monotonic_ns':1})
            service.events.append(event);service.channels[role]=event
            service.connections[i+1]=dict(event=event,socket=SimpleNamespace(
                sendall=sent[role].append,close=lambda:None,fileno=lambda:12345))
        service.start_round()
        for event in service.peers():event['peer153_sent']={'monotonic_ns':1}
        return service,sent

    def test_four_confirms_and_four_expiries_broadcast_each_to_all_four_without_rank(self):
        service,sent=self.service()
        for role in 'ABCD':
            body=self.body(service.room,SLOTS[role]);frame=c.encode(bytes(37),118,body,40)
            service.receive_channel(service.channels[role],frame,40,body)
        self.assertEqual(len(service.state['round_bombs']),4)
        for record in service.state['round_bombs']:
            self.assertEqual({r['role'] for r in record['broadcast159']},set('ABCD'))
        service.bomb_jobs=[(0,ident,record) for _,ident,record in service.bomb_jobs]
        service.tick_bombs()
        for record in service.state['round_bombs']:
            self.assertEqual({r['role'] for r in record['broadcast158']},set('ABCD'))
        self.assertEqual(len(service.state['bomb_deadlines']),4)
        self.assertNotIn('rank',service.round)
        before=[len(rows) for rows in sent.values()];service.tick_bombs()
        self.assertEqual([len(rows) for rows in sent.values()],before)

    def test_EOF_cancels_pending_all_owner_timers_no_synthetic_result(self):
        service,sent=self.service()
        for role in 'ABCD':
            body=self.body(service.room,SLOTS[role]);service.receive_channel(service.channels[role],
                c.encode(bytes(37),118,body,40),40,body)
        service.close_connection(service.channels['C']['global_ordinal'],'owned_test_eof')
        service.bomb_jobs=[(0,ident,record) for _,ident,record in service.bomb_jobs]
        service.tick()
        self.assertFalse(service.room.active)
        self.assertEqual([r['timer_rejected'] for r in service.state['round_bombs']],['round_closed']*4)
        self.assertTrue(all('broadcast158' not in r for r in service.state['round_bombs']))
        self.assertNotIn('rank',service.round)

if __name__=='__main__':unittest.main()
