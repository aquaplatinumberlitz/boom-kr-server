"""Fixed eight-member report-derived 4v4; no authenticated life authority.

KR175 header/rows reuse CP023/202; eight-row copied native parsing is
emulation only until a native eight-client result/reset run confirms it.
"""
from server.four_battle_policy import FourBattleReports, _rank_rows

class EightBattleReports(FourBattleReports):
    ROLES = 'ABCDEFGH'
    TEAMS = {role:i%2 for i,role in enumerate(ROLES)}
    def outcome(self, now_ns, pending):
        result=super().outcome(now_ns,pending)
        if result is not None:
            result['policy']='current_eight_4v4_report_derived_drained_settle8s'
        return result


def rank_body(winner_team,teams):
    if type(winner_team) is not int or winner_team not in (0,1) or teams!=EightBattleReports.TEAMS:
        raise ValueError('only fixed eight-member 4v4 supported')
    return _rank_rows(winner_team,teams,'ABCDEFGH')
