"""Eight-owner policy controls; synthetic requests are not a native oracle."""
import copy
from pathlib import Path
import tempfile
import unittest

from server import probe_codec as c
from server.eight_bomb_policy import BASELINE, EightBombAdmission
from server.eight_player_service import EightPlayerService
from server.four_bomb_policy import WIDTHS
from server.probe_room import ProbeRoom
from server.test_four_player_service import fixtures


class EightBombTests(unittest.TestCase):
    def policy(self):
        room=ProbeRoom([],12345);room.begin()
        policy=EightBombAdmission(room,dict(zip('ABCDEFGH',range(8))),
            BASELINE['capacities'],BASELINE['powers']);policy.begin()
        return room,policy

    def body(self,room,role,serial=1,power=None):
        return c.pack_fields([room.key,0,'ABCDEFGH'.index(role),serial,3,3,1,0,0,0,
            BASELINE['powers'][role] if power is None else power,1,0],WIDTHS)

    def test_all_owners_independent_serials_power2_and_capacity2_for_E(self):
        room,p=self.policy()
        for i,role in enumerate('ABCDEFGH'):
            body=self.body(room,role);item=p.admit(role,body)
            self.assertEqual((item['id'],item['slot'],item['power']),
                (i+1,i,BASELINE['powers'][role]))
            self.assertEqual(p.admit(role,body)['rejected'],'duplicate_bomb_serial')
        for role in 'GH':
            self.assertEqual(p.admit(role,self.body(room,role,2,power=1))['rejected'],
                'bomb_power_not_configured_baseline')
        self.assertIn('id',p.admit('E',self.body(room,'E',2)))
        self.assertEqual(p.admit('E',self.body(room,'E',3))['rejected'],'bomb_capacity_reserved')
        self.assertEqual(p.admit('F',self.body(room,'F',2))['rejected'],'bomb_capacity_reserved')
        self.assertEqual(room._wire_id,9)

    def test_eight_round_tickets_are_fenced_and_serials_rearm(self):
        room,p=self.policy()
        ids=[p.admit(r,self.body(room,r))['id'] for r in 'ABCDEFGH']
        room.finish()
        self.assertEqual([p.expire(i)['rejected'] for i in ids],['round_closed']*8)
        room.begin();p.begin()
        self.assertEqual(p.admit('H',self.body(room,'H'))['id'],9)
        self.assertEqual(p.expire(9)['power'],2)
        self.assertEqual(p.expire(9)['rejected'],'unknown_or_already_expired')

    def test_service_refuses_unmeasured_eight_profiles_including_bool(self):
        with tempfile.TemporaryDirectory() as folder:
            for role,field,value in [('E','capacities',1),('G','powers',1),('A','capacities',True)]:
                spec=copy.deepcopy(BASELINE);spec[field][role]=value
                with self.subTest(spec=spec),self.assertRaisesRegex(ValueError,'measured eight bomb baseline'):
                    EightPlayerService(fixtures(),Path(folder),
                        {r:'127.0.0.'+str(i+2) for i,r in enumerate('ABCDEFGH')},bomb_profile=spec)
