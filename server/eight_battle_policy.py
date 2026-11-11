"""Fixed eight-member report-derived 4v4; no authenticated life authority.

KR175 header/rows reuse CP023/202; eight-row copied native parsing is
emulation only until a native eight-client result/reset run confirms it.
"""
from server.four_battle_policy import FourBattleReports, _rank_rows
from server.eight_player_profile import TEAM_LAYOUTS

class EightBattleReports(FourBattleReports):
    ROLES = 'ABCDEFGH'
    TEAMS = TEAM_LAYOUTS['4v4']
    def __init__(self,room,slots,teams):
        if not isinstance(teams,dict) or any(type(z) is not int for z in teams.values()) or teams not in TEAM_LAYOUTS.values():
            raise ValueError('only fixed eight 4v4/1v7 teams supported')
        self.TEAMS=dict(teams)
        self.layout=next(k for k,v in TEAM_LAYOUTS.items() if v==teams)
        super().__init__(room,slots,teams)
    def outcome(self, now_ns, pending):
        result=super().outcome(now_ns,pending)
        if result is not None:
            result['policy']='current_eight_'+self.layout+'_report_derived_drained_settle8s'
        return result


def rank_body(winner_team,teams):
    if type(winner_team) is not int or winner_team not in (0,1) or not isinstance(teams,dict) or any(type(z) is not int for z in teams.values()) or teams not in TEAM_LAYOUTS.values():
        raise ValueError('only fixed eight-member 4v4/1v7 supported')
    return _rank_rows(winner_team,teams,'ABCDEFGH')
