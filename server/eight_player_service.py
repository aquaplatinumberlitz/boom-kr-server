"""Opt-in eight-session room/load/peer preview, without unmeasured combat."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import signal
import sys
from server import eight_player_profile as eight
from server.four_player_service import FourPlayerService, addresses
from server.probe_service import FixtureProfile
from server.eight_bomb_policy import BASELINE, EightBombAdmission
from server.eight_battle_policy import EightBattleReports, rank_body
from server.four_observability import FourObservability

class EightPlayerService(FourPlayerService):
    PLAYER_PROFILE = eight
    RANK_BODY = staticmethod(rank_body)

    def __init__(self,*args,bomb_profile=None,battle_outcome=False,**kwargs):
        if type(battle_outcome) is not bool or (battle_outcome and bomb_profile is None):
            raise ValueError('eight session preview outcome requires explicit measured bomb profile and bool')
        if bomb_profile is not None and json.dumps(bomb_profile,sort_keys=True) != json.dumps(BASELINE,sort_keys=True):
            raise ValueError('eight session preview requires the measured eight bomb baseline')
        super().__init__(*args,**kwargs)
        self.state['schema']='kr-eight-session-preview-v1'
        self.state['eight_session_binding']=self.state.pop('four_session_binding')
        self.state['eight_session_binding']['mode']='eight_operator_IPs_four_transports_each'
        self.state['limits']=('Eight initial sessions/room/load/peer only; '
            'no eight combat/results/items/terrain/kim/rescue/accountauth/resume/Internet or gate.')
        self.state['network']['peer_ips']=dict(self.source_ips)
        if bomb_profile is not None:
            self.bomb_admission=EightBombAdmission(self.room,self.role_slots,
                bomb_profile['capacities'],bomb_profile['powers'])
            self.bomb_fuse_seconds=bomb_profile['fuse_seconds']
            self.state['eight_bomb_policy']=dict(copy.deepcopy(bomb_profile),source='design',
                limits='Fixed native-profile baselines; reservation/fuse are emulator policy, '
                       'not original authority or eight-player outcome/reset.')
            self.state['limits']=('Eight initial sessions with explicit baseline bomb admission/fuse; '
                'no eight reports/results/reset/items/terrain/kim/rescue/accountauth/resume/Internet or gate.')
            module=sys.modules[EightBombAdmission.__module__];path=Path(module.__file__).resolve()
            if not any(row['path']==str(path) for row in self.state['policy_sources']):
                self.state['policy_sources'].append(dict(path=str(path),sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
        if battle_outcome:
            self.battle_reports=EightBattleReports(self.room,self.role_slots,
                {r:m['team'] for r,m in eight.MEMBERS.items()})
            self.state['eight_outcome_policy']=dict(source='design',
                mode='current_eight_4v4_report_derived_drained_settle8s',
                limits='Report-derived life, seven peer forwards and fixed rank8/reset only; '
                       'not authenticated life or original world authority.')
            self.state['limits']=('Eight fixed 4v4 report-derived result/reset with explicit baseline bombs; '
                'no full items/terrain/kim/rescue/accountauth/resume/Internet or gate.')
            module=sys.modules[EightBattleReports.__module__];path=Path(module.__file__).resolve()
            self.state['policy_sources'].append(dict(path=str(path),sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
        self.observability=FourObservability(self)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--profile',type=Path,required=True)
    parser.add_argument('--out',type=Path,required=True)
    parser.add_argument('--source-ip',action='append',required=True,metavar='ROLE=IPv4')
    parser.add_argument('--bind-ip',default='127.0.0.1')
    parser.add_argument('--port',type=int,action='append',required=True)
    parser.add_argument('--channel-ip')
    parser.add_argument('--channel-port',type=int)
    parser.add_argument('--bomb-profile',type=Path,help='explicit measured eight-owner baseline; no outcome policy')
    parser.add_argument('--battle-outcome',action='store_true',help='opt in fixed report-derived eight-member 4v4 result/reset')
    args=parser.parse_args()
    try:
        ips=addresses(args.source_ip,eight.MEMBERS)
        if any(not 0<=port<=65535 for port in args.port):
            raise ValueError('invalid listener port')
        service=EightPlayerService(FixtureProfile(args.profile),args.out,ips,
            bind_ip=args.bind_ip,channel_ip=args.channel_ip,channel_port=args.channel_port,
            bomb_profile=json.loads(args.bomb_profile.read_text()) if args.bomb_profile else None,
            battle_outcome=args.battle_outcome)
    except ValueError as error:
        parser.error(str(error))
    args.out.mkdir(parents=True,exist_ok=True)
    def stop(_signal,_frame):
        service.closed=True
    signal.signal(signal.SIGTERM,stop)
    signal.signal(signal.SIGINT,stop)
    service.run(args.port)

if __name__=='__main__':
    main()
