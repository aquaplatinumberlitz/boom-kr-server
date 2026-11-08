"""Opt-in eight-session room/load/peer preview, without unmeasured combat."""
import argparse
from pathlib import Path
import signal
import sys
from server import eight_player_profile as eight
from server.four_player_service import FourPlayerService, addresses
from server.probe_service import FixtureProfile

class EightPlayerService(FourPlayerService):
    PLAYER_PROFILE = eight

    def __init__(self,*args,**kwargs):
        super().__init__(*args,**kwargs)
        self.state['schema']='kr-eight-session-preview-v1'
        self.state['eight_session_binding']=self.state.pop('four_session_binding')
        self.state['eight_session_binding']['mode']='eight_operator_IPs_four_transports_each'
        self.state['limits']=('Eight initial sessions/room/load/peer only; '
            'no eight combat/results/items/terrain/kim/rescue/accountauth/resume/Internet or gate.')
        self.state['network']['peer_ips']=dict(self.source_ips)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--profile',type=Path,required=True)
    parser.add_argument('--out',type=Path,required=True)
    parser.add_argument('--source-ip',action='append',required=True,metavar='ROLE=IPv4')
    parser.add_argument('--bind-ip',default='127.0.0.1')
    parser.add_argument('--port',type=int,action='append',required=True)
    parser.add_argument('--channel-ip')
    parser.add_argument('--channel-port',type=int)
    args=parser.parse_args()
    try:
        ips=addresses(args.source_ip,eight.MEMBERS)
        if any(not 0<=port<=65535 for port in args.port):
            raise ValueError('invalid listener port')
        service=EightPlayerService(FixtureProfile(args.profile),args.out,ips,
            bind_ip=args.bind_ip,channel_ip=args.channel_ip,channel_port=args.channel_port)
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
