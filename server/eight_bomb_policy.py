"""Fixed eight-slot extension of the measured four-owner admission mechanism.

Baseline values come from the eight-native clock/request experiment, not from
slot count or VN defaults. Admission/fuse remain explicit emulator choices.
"""
from server.four_bomb_policy import FourBombAdmission

BASELINE = dict(mode='eight_current_baseline',
    capacities={'A':1,'B':2,'C':1,'D':1,'E':2,'F':1,'G':1,'H':1},
    powers={'A':1,'B':1,'C':1,'D':1,'E':1,'F':1,'G':2,'H':2},
    fuse_seconds=6)


class EightBombAdmission(FourBombAdmission):
    ROLES = 'ABCDEFGH'
