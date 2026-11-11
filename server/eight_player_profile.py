"""Eight-session preview inputs; eight consumers checked on two real clients.\n\nEight actual sessions, combat/authoritative life and broad layouts remain separate.\n"""
import struct
from server import four_player_profile as four, probe_codec as c
from server.four_player_profile import player_body,slot_body,auth_body,own_body,join_body
from server.room_title import _decode_bytes,_encode_bytes
ROLES='ABCDEFGH'
MEMBERS={r:dict(four.MEMBERS[r]) if r in four.MEMBERS else
    dict(identity=i+1,slot=i,team=i%2,name='KRPeer'+str(i+1))
    for i,r in enumerate(ROLES)}
TEAM_LAYOUTS = {
    '4v4': {role:i%2 for i,role in enumerate(ROLES)},
    '1v7': {role:0 if role=='A' else 1 for role in ROLES},
}

def start_body(template, members=None):
    members = MEMBERS if members is None else members
    # Retain the same measured two-row envelope/26-byte row layout as CP199.
    if len(template)!=129 or c.read_fields(template[:1],[1])!=[0]:
        raise ValueError('expected measured two-row KR121')
    size=c.read_fields(template[1:5],[4])[0];raw=_decode_bytes(template[5:5+size])
    if size!=106 or raw[:6]!=b'\x18'+struct.pack('<I',101)+b'\0':
        raise ValueError('unsupported KR121 envelope')
    inner=raw[6:]
    if len(inner)!=100 or inner[43]!=2 or inner[48]!=0 or inner[74]!=1:
        raise ValueError('unsupported KR121 rows')
    rows=[inner[48:74],inner[74:100]]
    for role in ROLES[2:]:
        m=members[role];row=bytearray(rows[1]);row[0]=row[2]=m['slot']
        struct.pack_into('<H',row,3,m['slot']);row[5]=m['team'];rows.append(row)
    prefix=bytearray(inner[:48]);prefix[43]=8;inner=prefix+b''.join(rows)
    stream=b'\x18'+struct.pack('<I',len(inner)+1)+b'\0'+inner
    return template[:1]+c.pack_fields([len(stream)],[4])+_encode_bytes(stream)+template[5+size:]
def peer_body(template,key,recipient,endpoints):
    if len(template)<35 or recipient not in MEMBERS or set(endpoints)!=set(ROLES):
        raise ValueError('exact eight-role peer roster required')
    body=bytearray(template[:35]);body[15:19]=c.pack_fields([key],[4])
    by_slot={m['slot']:role for role,m in MEMBERS.items()}
    for slot in range(26):
        body+=c.pack_fields([int(slot in by_slot)],[1])
        if slot in by_slot:
            role=by_slot[slot];ip,port=endpoints[role]
            if not 0<ip<0xffffffff or not 1<=port<=65535:raise ValueError('invalid endpoint')
            body+=c.pack_fields([ip,port,0,0,0,0] if role!=recipient else [0]*6,[4,2]*3)
    return bytes(body)


class TeamProfile:
    """Per-service fixed operator teams; no module-global mutation or UI claim."""
    __file__ = __file__
    ROLES = ROLES
    player_body = staticmethod(player_body)
    slot_body = staticmethod(slot_body)
    auth_body = staticmethod(auth_body)
    own_body = staticmethod(own_body)
    join_body = staticmethod(join_body)
    peer_body = staticmethod(peer_body)

    def __init__(self, layout):
        if type(layout) is not str or layout not in TEAM_LAYOUTS:
            raise ValueError('only fixed eight 4v4/1v7 operator layouts supported')
        self.MEMBERS={role:dict(member,team=TEAM_LAYOUTS[layout][role]) for role,member in MEMBERS.items()}

    def start_body(self, template):
        return start_body(template,self.MEMBERS)
