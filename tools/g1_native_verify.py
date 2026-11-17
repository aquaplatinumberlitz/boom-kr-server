"""Kiểm semantic native G1 theo nhóm, dùng chung cho mọi lượt của harness."""
import argparse
import hashlib
import json
from pathlib import Path
import sqlite3
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from server import probe_codec as codec
from g1_native_harness import pin, read, tagged, write


def character(root):
    out = root/'native/cycles'
    run = read(out/'run.json')
    wire = read(root/'decoded-channels-shared.json')
    assert wire['run_sha256'] == hashlib.sha256((out/'run.json').read_bytes()).hexdigest()
    channels = {row['identity']: row for row in wire['channels']}
    actor = run['account_launches']['A']['response']['player']
    peer = run['account_launches']['B']['response']['player']
    a = channels[actor['wire_id']]
    b = channels[peer['wire_id']]
    choices = [row for row in a['incoming'] if row['opcode'] == 92]
    assert [codec.read_fields(bytes.fromhex(row['body_hex']),[1,2]) for row in choices] == [[0,4],[0,0],[0,1]]
    assert not any(row['opcode'] == 92 for row in b['incoming'])
    # The same captured channel carries denial, both valid choices and both Starts.
    assert sum(row['opcode'] == 104 for row in a['incoming']) == 2
    assert sum(row['opcode'] == 95 for row in b['incoming']) == 2
    for row in channels.values():
        assert sum(frame['opcode'] == 121 for frame in row['out']) == 2
        assert sum(frame['opcode'] == 175 for frame in row['out']) == 2
        assert sum(frame['opcode'] == 180 for frame in row['out']) == 2
    # Only the actor receives the restoring slot before the first valid choice.
    actor_log = tagged((out/'A/client.log').read_text(errors='replace'),'M7C_CHOICE')
    peer_log = tagged((out/'B/client.log').read_text(errors='replace'),'M7C_CHOICE')
    steps = {step['name']: step['ns'] for step in run['matrix_steps']}
    start, end = steps['peer-setup'], steps['unsupported-rejected']
    correction = [row for row in actor_log if row['kind']=='slot_end' and start<row['ns']<end and row['slot']==0]
    assert len(correction) == 1
    restored = correction[0]
    assert restored['character'] == 1 and restored['pending_after'] == 0
    assert restored['cursor'] == restored['payload'] == 6 and restored['err54'] == restored['err58'] == 0
    assert not any(row['kind']=='slot_end' and row['slot']==0 and start<row['ns']<end for row in peer_log)
    for logs in [actor_log,peer_log]:
        assert any(row['kind']=='slot_end' and row.get('slot')==0 and row['character']==0 for row in logs)
        assert any(row['kind']=='slot_end' and row.get('slot')==0 and row['character']==1 and row['ns']>end for row in logs)
    # Revision grows only for C0/C1, never for rejected C4. The peer stays untouched.
    with sqlite3.connect('file:'+str(root/'previous-accounts.sqlite')+'?mode=ro',uri=True) as seed:
        initial = seed.execute('select character_key,revision from player_preferences where player_id=?',(actor['player_id'],)).fetchone()
        peer_before = seed.execute('select character_key,revision from player_preferences where player_id=?',(peer['player_id'],)).fetchone()
        first = seed.execute('select coalesce(max(id),0)from matches').fetchone()[0]
    with sqlite3.connect('file:'+str(out/'account-db/accounts.sqlite')+'?mode=ro',uri=True) as db:
        final = db.execute('select character_key,revision from player_preferences where player_id=?',(actor['player_id'],)).fetchone()
        assert initial == (1,1) and final == (1,3)
        assert peer_before == db.execute('select character_key,revision from player_preferences where player_id=?',(peer['player_id'],)).fetchone()
        rows = db.execute('select start_json from matches where id>? order by id',(first,)).fetchall()
        assert len(rows) == 2
        for (body,) in rows:
            players = json.loads(body)['players']
            assert [(p['slot'],p['character_key'])for p in players] == [(0,1),(1,0)]
    events = [json.loads(line)for line in (out/'rust-events.jsonl').read_text().splitlines()]
    for owner in [actor,peer]:
        assert sum(row.get('event')=='account_binding' and row.get('transport')=='channel'
                   and row['identity']==owner['wire_id'] for row in events) == 1
    assert [row.get('character_rejection')for row in events if row.get('opcode')==92] == ['unmeasured_character',None,None]
    result = dict(result='verified_native_character_rejection_same_channel_two_rounds',
                  source='kr_runtime',status='confirmed',run=pin(out/'run.json'),wire=pin(root/'decoded-channels-shared.json'),
                  correction=restored,choices=[4,0,1],revision_before=initial,revision_after=final,
                  rounds=2,roster_characters=[1,0],
                  limits='Wine/private LAN and measured C0/C1 only. Current110 refusal is design, not original server reply recovery; no expanded character roster or G1 closure.')
    target=root/'character-verification.json'
    assert not target.exists()
    write(target,result)
    print(result['result'])


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run',type=Path,required=True)
    parser.add_argument('--group',choices=['character'],required=True)
    args=parser.parse_args()
    root=args.run.resolve(strict=True)
    assert root.is_relative_to((ROOT/'local').resolve())
    character(root)


if __name__ == '__main__':
    main()
