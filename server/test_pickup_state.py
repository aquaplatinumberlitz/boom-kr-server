"""Regression for report-derived fixture policy, not native authority proof."""
from pathlib import Path
from types import SimpleNamespace
import struct,tempfile,unittest
from server.probe_service import ProbeService
from server import probe_codec as codec

class PickupEligibilityTests(unittest.TestCase):
    def setUp(self):
        tmp=tempfile.TemporaryDirectory();self.addCleanup(tmp.cleanup)
        names=['rank_Awin','promote_B','auth_C','roomlist_C','join_C','player_C','slot_C','guest_C','ready_C']
        profile=SimpleNamespace(frames=dict.fromkeys(names,b''),data={'items':[dict(index=17,row=4,column=3,kind=2)],'initial_key':12345})
        self.s=ProbeService(profile,Path(tmp.name),{});self.addCleanup(self.s.selector.close)
        self.a=dict(role='A',reply_ordinal=4,map_response_sent=True);self.b=dict(role='B',reply_ordinal=4,map_response_sent=True)
        self.s.events=[self.a,self.b];self.s.channels=dict(A=self.a,B=self.b);self.sent=[]
        def send(event,alias,label,**kw):
            q=dict(role=event['role'],alias=alias,label=label,body=kw.get('body'),monotonic_ns=1);self.sent.append(q);return q
        self.s.send=send;self.s.start_round();self.sent.clear()
    def pickup(self,event=None,slot=0,key=None):
        event=self.a if event is None else event
        body=codec.pack_fields([0,slot,4,3,17,2,1234,self.s.room.key if key is None else key,0],[1,1,1,1,4,2,4,4,1]);frame=codec.encode(bytes(37),124,body,100)
        self.s.receive_channel(event,frame,100,body);return self.s.state['item_requests'][-1]
    def report(self,subtype,event=None):
        event=self.a if event is None else event;slot=0 if event is self.a else 1
        body=codec.pack_fields([subtype,slot,120,120,100,12345],[1,1,2,2,4,4])
        if subtype==17:
            blob=bytes.fromhex('180900000000')+struct.pack('<II',0,7)
            body+=codec.pack_fields([14],[4])+codec.pack_fields([int.from_bytes(blob[i:i+min(4,len(blob)-i)],'little') for i in range(0,len(blob),4)],[4,4,4,2])
        elif subtype==19:body+=codec.pack_fields([11, int.from_bytes(bytes.fromhex('18060000'), 'little'), 0, 0, 0], [4, 4, 4, 2, 1])
        frame=codec.encode(bytes(37),127,body,103);self.s.receive_channel(event,frame,103,body)
    def test_trap_and_death_reject_before_consuming_claim(self):
        for subtype in [17,19]:
            with self.subTest(subtype=subtype):
                self.report(subtype);self.sent.clear();q=self.pickup()
                self.assertEqual(q['rejected'],'actor_report_ineligible');self.assertEqual(q['actor_report']['subtype'],subtype);self.assertEqual(self.sent,[])
        # Other bound peer can still win the untouched current catalog entry.
        q=self.pickup(self.b,slot=1);self.assertNotIn('rejected',q);self.assertEqual(len(q['broadcast165']),2)
    def test_next_start_clears_report_block_on_same_socket(self):
        self.report(17);self.assertEqual(self.pickup()['rejected'],'actor_report_ineligible')
        self.s.room.finish();self.s.start_round();self.assertNotIn('pickup_ineligible',self.a)
        self.assertNotIn('rejected',self.pickup());self.assertEqual(self.s.room.key,12346)
    def test_key_guard_remains_before_actor_guard(self):
        self.report(19);self.assertEqual(self.pickup(key=self.s.room.key-1)['rejected'],'round_key_mismatch')
        self.assertEqual(self.pickup()['rejected'],'actor_report_ineligible')
    def test_unrelated_report_does_not_block_and_no_peer_does_not_skip_block(self):
        self.report(39);self.assertNotIn('pickup_ineligible',self.a)
        self.s.events=[self.a];self.report(19)
        self.assertEqual(self.pickup()['rejected'],'actor_report_ineligible')
    def test_mismatched_slot_or_bad_trap_does_not_set_marker(self):
        body=codec.pack_fields([19,1,0,0,0,0],[1,1,2,2,4,4])
        with self.assertRaisesRegex(ValueError,'slot'):self.s.receive_event(self.a,bytes(37),100,body)
        self.assertNotIn('pickup_ineligible',self.a)
        body=codec.pack_fields([17,0,0,0,0,0,0],[1,1,2,2,4,4,4])
        with self.assertRaisesRegex(ValueError,'trap blob'):self.s.receive_event(self.a,bytes(37),100,body)
        self.assertNotIn('pickup_ineligible',self.a)

if __name__=='__main__':unittest.main()
