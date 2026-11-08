"""Structural controls; eight-native consumer/runtime evidence is separate."""
import unittest
from server import eight_player_profile as eight, probe_codec as c
from server.test_four_player_service import fixtures
from server.room_title import _decode_bytes

class EightProfiles(unittest.TestCase):
    def test_start_eight_rows_preserve_measured_tail(self):
        p=fixtures();body=eight.start_body(p.body('start'))
        size=c.read_fields(body[1:5],[4])[0]
        inner=_decode_bytes(body[5:5+size])[6:]
        self.assertEqual((len(body),len(inner),inner[43]),(285,256,8))
        self.assertEqual([inner[48+26*i] for i in range(8)],list(range(8)))
        self.assertEqual(body[-18:],p.body('start')[-18:])
    def test_eight_peer26_flags_seven_remotes_zero_self(self):
        p=fixtures();ends={r:(0x7f000002+i,24000+i) for i,r in enumerate(eight.ROLES)}
        for role in eight.ROLES:
            body=eight.peer_body(p.body('peer'),12345,role,ends);offset=35;on=[]
            for slot in range(26):
                active=c.read_fields(body[offset:offset+1],[1])[0];offset+=1
                if active:
                    on.append(slot);values=c.read_fields(body[offset:offset+18],[4,2]*3);offset+=18
                    r=eight.ROLES[slot];want=[*ends[r],0,0,0,0] if r!=role else [0]*6
                    self.assertEqual(values,want)
            self.assertEqual((on,offset,len(body)),(list(range(8)),205,205))
    def test_profiles_have_distinct_IDs_slots_and_supported_names(self):
        self.assertEqual([m['identity'] for m in eight.MEMBERS.values()],list(range(1,9)))
        self.assertEqual([m['slot'] for m in eight.MEMBERS.values()],list(range(8)))
        self.assertTrue(all(len(m['name']) in (5,7) for m in eight.MEMBERS.values()))
