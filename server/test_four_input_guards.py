"""Four-session dead-owner admission and reset controls; no native-life oracle."""
import unittest
from unittest.mock import patch
from server import probe_codec as c
from server import test_four_bomb_policy as fixtures
from server import test_four_battle_policy as reports
from server.four_player_service import FourPlayerService


class FourInputGuards(unittest.TestCase):
    def service(self):
        helper=fixtures.FourBombTests()
        self.addCleanup(helper.doCleanups)
        def enabled(*args,**kwargs):
            return FourPlayerService(*args,**kwargs,battle_outcome=True)
        with patch.object(fixtures,'FourPlayerService',enabled):
            service,sent=helper.service()
        return helper,service,sent

    def report(self,service,role,sub,tick):
        body=reports.Reports().body(role,sub,tick)
        service.receive_four_report(service.channels[role],c.encode(bytes(37),127,body,40),40,body)

    def test_dead_owner_no_ID_job_serial_send_alive_and_new_round_positives(self):
        helper,service,sent=self.service()
        self.report(service,'A',17,10);self.report(service,'A',19,11)
        before=(service.room._wire_id,len(service.bomb_jobs),len(service.bomb_admission.serials),
                [len(x) for x in sent.values()])
        body=helper.body(service.room,0,serial=99)
        service.receive_bomb(service.channels['A'],c.encode(bytes(37),118,body,40),40,body)
        self.assertEqual(service.state['bomb_requests'][-1]['rejected'],'bomb_owner_reported_dead')
        self.assertEqual(before,(service.room._wire_id,len(service.bomb_jobs),
            len(service.bomb_admission.serials),[len(x) for x in sent.values()]))
        body=helper.body(service.room,1,serial=99)
        service.receive_bomb(service.channels['B'],c.encode(bytes(37),118,body,40),40,body)
        self.assertEqual(service.room._wire_id,1)
        service.room.finish();service.start_round()
        service.channels['A']['peer153_sent']={'monotonic_ns':1}
        body=helper.body(service.room,0,serial=99)
        service.receive_bomb(service.channels['A'],c.encode(bytes(37),118,body,40),40,body)
        self.assertEqual(service.room._wire_id,2)

    def test_inactive_or_unloaded_reports_never_commit_life_or_cursor(self):
        _,service,_=self.service()
        service.channels['A'].pop('peer153_sent')
        self.report(service,'A',17,10)
        self.assertEqual(service.battle_reports.cursors,{})
        service.room.finish();self.report(service,'B',17,10)
        self.assertEqual(service.battle_reports.cursors,{})


if __name__=='__main__':
    unittest.main()
