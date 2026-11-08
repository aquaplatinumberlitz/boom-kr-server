"""Finite diagnostic I/O budgets and coalesced-frame/chunk flood regressions."""
import copy
import struct
import unittest
from server.four_input_limits import FourInputBudget
from server.four_player_service import RecipientSendFailure
from server import test_four_bomb_policy as fixtures
from server import probe_codec as c


class InputBudgetTests(unittest.TestCase):
    def test_exact_input_boundary_and_chunk_limit_do_not_commit_rejection(self):
        q=FourInputBudget(input_bytes=8,input_chunks=2)
        self.assertIsNone(q.chunk(1,4,0));self.assertIsNone(q.chunk(1,4,0))
        before=copy.deepcopy(q.records)
        self.assertEqual(q.chunk(1,1,0),'input_lifetime_bytes');self.assertEqual(q.records,before)
        q=FourInputBudget(input_chunks=1);q.chunk(1,1,0)
        self.assertEqual(q.chunk(1,1,1),'input_lifetime_chunks')

    def test_chunk_and_frame_bursts_are_independent_per_connection_with_refill(self):
        q=FourInputBudget(rate=1,burst=2)
        self.assertIsNone(q.chunk(1,1,0));self.assertIsNone(q.chunk(1,1,0))
        self.assertEqual(q.chunk(1,1,0),'input_chunk_rate')
        self.assertIsNone(q.chunk(2,1,0))
        self.assertIsNone(q.frame(1,0));self.assertIsNone(q.frame(1,0))
        self.assertEqual(q.frame(1,0),'input_frame_rate')
        self.assertEqual(q.frame(1,-1),'input_frame_rate')
        self.assertEqual(q.frame(1,.5),'input_frame_rate')
        self.assertIsNone(q.frame(1,1));self.assertEqual(q.frame(1,1),'input_frame_rate')

    def test_output_attempt_reservations_are_not_refunded_or_overcommitted(self):
        q=FourInputBudget(output_bytes=8,output_calls=2)
        self.assertIsNone(q.sending(1,8));before=copy.deepcopy(q.records)
        self.assertEqual(q.sending(1,1),'output_lifetime_bytes');self.assertEqual(q.records,before)
        q=FourInputBudget(output_calls=1);q.sending(1,1)
        self.assertEqual(q.sending(1,1),'output_lifetime_calls')
        with self.assertRaises(ValueError):FourInputBudget(burst=True)

    def service(self):
        helper=fixtures.FourBombTests();self.addCleanup(helper.doCleanups)
        service,sent=helper.service();return service,sent

    def connection(self,service,role):
        event=service.channels[role];event['chunks']=[]
        connection=service.connections[event['global_ordinal']]
        connection.update(buffer=bytearray(),sequence=40)
        return event,connection

    def test_single_coalesced_chunk_cannot_bypass_frame_rate_limit(self):
        service,_=self.service();event,connection=self.connection(service,'B')
        raw=b''.join(c.encode(bytes(37),15,b'',40+3*i) for i in range(129))
        connection['socket'].recv=lambda _size:raw
        pending=service.room.bomb();service.receive(event['global_ordinal'])
        self.assertEqual(service.state['input_rejections'][-1]['reason'],'input_frame_rate')
        self.assertEqual(event['c2s_frame_count'],128)
        self.assertEqual(service.room.fence.rejection(pending),'round_closed')

    def test_tiny_chunks_hit_lifetime_log_count_before_unbounded_append(self):
        service,_=self.service();event,connection=self.connection(service,'B')
        service.io_budget=FourInputBudget(rate=10000,burst=10000)
        raw=bytearray(65536);struct.pack_into('<I',raw,13,65000);offset=0
        def receive(_size):
            nonlocal offset
            data=bytes(raw[offset:offset+1]);offset+=1;return data
        connection['socket'].recv=receive
        for _ in range(4097):service.receive(event['global_ordinal'])
        self.assertEqual(service.state['input_rejections'][-1]['reason'],'input_lifetime_chunks')
        self.assertEqual(len(event['chunks']),4096);self.assertEqual(len(connection['buffer']),4096)
        self.assertTrue(event.get('closed'))

    def test_unmapped_admission_diagnostics_keep_a_bounded_tail_and_total(self):
        from types import SimpleNamespace
        service,_=self.service();count=0
        def accept():
            nonlocal count
            count+=1
            return SimpleNamespace(close=lambda:None),('127.0.0.99',count)
        listener=SimpleNamespace(accept=accept)
        for _ in range(300):service.accept(listener)
        self.assertEqual(service.state['admission_rejections_total'],300)
        self.assertEqual(len(service.state['admission_rejections']),256)
        self.assertEqual(service.state['admission_rejections'][0]['peer'][1],45)

    def test_output_quota_failure_identifies_its_recipient_before_socket_send(self):
        service,sent=self.service();service.io_budget=FourInputBudget(output_bytes=8)
        event=service.channels['C'];before=len(sent['C'])
        service.send_raw(event,b'12345678')
        with self.assertRaises(RecipientSendFailure) as error:service.send_raw(event,b'x')
        self.assertEqual(error.exception.recipient_ordinal,event['global_ordinal'])
        self.assertEqual(len(sent['C']),before+1)


if __name__=='__main__':unittest.main()
