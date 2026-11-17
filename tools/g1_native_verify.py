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
from g1_native_harness import check_pins, matches, pin, read, tagged, write



def check_runtime_inputs(root, rows):
    changes=[(row,pin(row['path']))for row in rows if pin(row['path'])!=row]
    if not changes:
        return
    # KR rewrites this known metadata cache from actual S2C238. Every other
    # pin remains strict; preserve preflight and prove both before/after bytes.
    import shutil
    import zlib
    from tools.rust_map_profile import blob
    config=read(root/'config.json');run=read(root/'native/cycles/run.json')
    wire=read(root/'decoded-channels-shared.json')
    assert wire['run_sha256']==pin(root/'native/cycles/run.json')['sha256']
    expected={str(Path(config['roles'][role]['game_dir'])/'fxdynmapinfo.ssd'):role for role in ['A','B']}
    assert len(changes)==2 and {row['path']for row,_ in changes}==set(expected)
    source_inputs=read(root/'vi-client-inputs.json')['files']
    folder=root/'cache-evidence';folder.mkdir(exist_ok=True);proof=[]
    for before,after in changes:
        role=expected[before['path']]
        source=next(row['source']for row in source_inputs if row['role']==role and row['target']['path']==before['path'])
        assert source['bytes']==before['bytes'] and source['sha256']==before['sha256']
        check_pins([source])
        old=folder/(role+'-before.ssd');new=folder/(role+'-after.ssd')
        for src,dst in [(Path(source['path']),old),(Path(after['path']),new)]:
            if dst.exists():assert dst.read_bytes()==src.read_bytes()
            else:shutil.copy2(src,dst)
        data=new.read_bytes()
        assert data[:4]==b'ssi1' and data[8]==24 and data[13]==1
        assert int.from_bytes(data[4:8],'little')==zlib.adler32(data[8:],0)&0xffffffff
        assert int.from_bytes(data[9:13],'little')==len(data)-13
        dec=zlib.decompressobj();plain=dec.decompress(data[18:],65537)
        assert dec.eof and not dec.unused_data and not dec.unconsumed_tail
        assert len(plain)==int.from_bytes(data[14:18],'little')
        identity=run['account_launches'][role]['response']['player']['wire_id']
        channel=next(row for row in wire['channels']if row['identity']==identity)
        tables=[row for row in channel['out']if row['opcode']==238];assert tables
        for table in tables:
            inner,end,_=blob(bytes.fromhex(table['body_hex']))
            assert end==table['bytes'] and inner==plain
        proof.append(dict(role=role,before=before,after=after,retained_before=pin(old),retained_after=pin(new),actual238_count=len(tables),cache_plain_equals_actual238=True))
    result=dict(result='verified_only_two_client_metadata_cache_outputs',run=pin(root/'native/cycles/run.json'),wire=pin(root/'decoded-channels-shared.json'),mutations=proof,all_other_requested_inputs_unchanged=True,preflight_frozen=False,noncache_inputs_frozen=True,limits='Retained original preflight and postrun mismatch failure. Known fxdynmapinfo cache outputs equal actual238 exactly; no general mutable-input exemption or L1 closure.')
    target=root/'cache-verification.json'
    if target.exists():assert read(target)==result
    else:write(target,result)


def lifecycle(root):
    matrix = read(root/'matrix.json')
    out = root/'native/cycles'
    assert read(root/'native/launch-exit.json')['returncode'] == 0
    check_runtime_inputs(root,read(root/'native/runtime-preflight.json')['files'])
    run = read(out/'run.json')
    assert not run.get('error') and not run.get('cleanup_errors') and run['service_stopped']
    assert run['owned_orphan_cleanup']['result'] == 'all_owned_namespace_descendants_reaped'
    assert not run['owned_orphan_cleanup']['remaining'] and run.get('early_completion')
    assert read(root/'native/storage-export.json')['result'] == 'pass'
    check_pins(read(root/'native/storage-export.json')['files'])
    rounds = list(range(1,matrix['rounds']+1))
    # Native consumer completion is independent of the server's request log.
    for role in matrix['roster']:
        text = (out/role/'client.log').read_text(errors='replace')
        for tag, kind in [('FOUR_SLOT','clock'),('FOUR_LIFE','rank_end'),('FOUR_LIFE','return_before_room')]:
            rows = [row for row in tagged(text,tag) if row['kind'] == kind]
            if kind == 'clock':
                assert len(rows) == len(rounds), (role,tag,kind)
                clocks = rows
            else:
                assert [row['round'] for row in rows] == rounds, (role,tag,kind)
                if kind == 'rank_end':
                    ranks = rows
                else:
                    returns = rows
        for index, (clock, rank, returned) in enumerate(zip(clocks, ranks, returns)):
            assert clock['ns'] < rank['ns'] < returned['ns'], (role,index)
            if index:
                assert returns[index-1]['ns'] < clock['ns'], (role,index)
        expectations=matrix.get('native_expectations',[])
        correction_path=root/'matrix-count-correction.json'
        if correction_path.exists():
            correction=read(correction_path)
            assert correction['matrix_sha256']==pin(root/'matrix.json')['sha256']
            assert expectations==correction['original']==[dict(tag='ABILITY_MOTION',match={'kind':'162_end'},count=8)]
            assert matrix['rounds']==len(matrix['cases'])==correction['corrected_count']==2
            expectations=[dict(expectations[0],count=2)]
        for expected in expectations:
            if expected.get('role') not in [None,role]:continue
            rows = tagged(text,expected['tag'])
            assert sum(matches(row,expected['match'])for row in rows) == expected['count'], (role,expected)
    with sqlite3.connect('file:'+str(out/'account-db/accounts.sqlite')+'?mode=ro',uri=True) as db:
        seed = sqlite3.connect('file:'+str(root/'previous-accounts.sqlite')+'?mode=ro',uri=True)
        if seed.execute("select count(*)from sqlite_master where type='table'and name='matches'").fetchone()[0]:
            first = seed.execute('select coalesce(max(id),0)from matches').fetchone()[0]
        else:
            assert seed.execute('pragma user_version').fetchone()[0]==2
            first = 0  # The measured schema2 seed predates the schema3 match journal.
        seed.close()
        matches_db = db.execute('select id,map_name,state,start_json from matches where id>?order by id',(first,)).fetchall()
        assert len(matches_db) == matrix['rounds']
        owners = run['account_launches']
        for mid,map_name,state,start in matches_db:
            assert map_name == matrix['map'] and state == 'complete'
            players = json.loads(start)['players']
            assert [(p['slot'],p['account_id'],p['player_id'])for p in players] == [
                (i,owners[role]['response']['player']['account_id'],owners[role]['response']['player']['player_id'])
                for i,role in enumerate(matrix['roster'])]
            assert db.execute('select count(*)from match_players where match_id=? and reward=0',(mid,)).fetchone()[0] == len(matrix['roster'])
    # Full item/geometry fidelity requires the group's independently decoded wire
    # and wrapper assertions, which must be supplied explicitly, never inferred.
    result = dict(result='verified_matrix_lifecycle_identity_and_declared_native_controls',
                  matrix=pin(root/'matrix.json'),run=pin(out/'run.json'),
                  count_correction=pin(root/'matrix-count-correction.json')if(root/'matrix-count-correction.json').exists()else None,
                  limits='Lifecycle/identity/declared controls only. Geometry, wire, effects and G1 closure require group evidence.')
    with (root/'matrix-verification.json').open('x') as stream:
        json.dump(result,stream,ensure_ascii=False,indent=2);stream.write('\n')
    print(result['result'])



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


def motion(root):
    out=root/'native/cycles'
    run=read(out/'run.json');matrix=read(root/'matrix.json')
    kind=matrix['item'];assert kind in [3,4]
    wire=read(root/'decoded-channels-shared.json')
    assert wire['run_sha256']==pin(out/'run.json')['sha256']
    actor=run['account_launches']['A']['response']['player']
    channels={row['identity']:row for row in wire['channels']}
    requests=[row for row in channels[actor['wire_id']]['incoming'] if row['opcode']==119]
    fields=[codec.read_fields(bytes.fromhex(row['body_hex']),[1,1,1,2,1,1,1,1,2,2])for row in requests]
    assert len(fields)==len(matrix['cases'])
    expected=[case['selector']for case in matrix['cases']]
    correction_path=root/'selector-name-correction.json'
    corrections=None
    if correction_path.exists():
        assert kind==3
        corrections=read(correction_path)
        assert corrections['matrix_sha256']==pin(root/'matrix.json')['sha256']
        assert corrections['wire_sha256']==pin(root/'decoded-channels-shared.json')['sha256']
        for correction in corrections['corrections']:
            index=correction['round']-1
            assert expected[index]==correction['planned'] and correction['planned']in[4,6]
            assert correction['expected']==10-correction['planned']
            contact=next(step for step in matrix['steps']if step['name'].startswith('round'+str(index+1)+'-')and step['name'].endswith('-contact'))
            key=contact['actions'][0]['key']
            assert correction['expected']=={'Right':6,'Left':4}[key]
            expected[index]=correction['expected']
    assert [row[2]for row in fields]==expected
    assert all(row[:2]==[0,0] for row in fields)
    allowed_selectors={4,5,6,7}if kind==3 else{0,1,2,3}
    assert set(expected)<=allowed_selectors and {row[2]for row in fields}==set(expected)
    # Every action keeps the exact source/target reported by the native client.
    # Endpoint distance is recorded; it is not a flight-length or RNG claim.
    cases=[];getter_checks=[];shared=None
    def scalar(snapshot):
        return snapshot['value'] if 'value'in snapshot else snapshot['u32']
    for role in matrix['roster']:
        owner=run['account_launches'][role]['response']['player']
        channel=channels[owner['wire_id']]
        text=(out/role/'client.log').read_text(errors='replace')
        assert not any(error in text for error in ['ABILITY_MOTION_ERROR','ABILITY_RETIRE_ERROR','SELECTED_PEER_ERROR','received signal SIGSEGV','received signal SIGABRT','"kind":"probe_error"'])
        clocks=[row for row in tagged(text,'FOUR_SLOT')if row['kind']=='clock']
        assert len(clocks)==matrix['rounds']
        for row in clocks:
            assert set(row['actors'])=={'0','1'}
            for slot,levels in [('0',(2,7,1,9,4,8)),('1',(1,10,1,7,5,7))]:
                values=row['actors'][slot]['fields']
                assert tuple(scalar(values[f])for f in ['0x4a4','0x50c','0x4bc','0x524','0x5e8','0x5dc'])==levels
                assert scalar(values['0x5f4'])==scalar(values['0x60c'])==0
                getter_checks.extend(values.values())
        items=[row for row in tagged(text,'RUST_M3_ITEM')if row['kind']=='end']
        assert len(items)==matrix['rounds']
        for item in items:
            assert item['cursor']==item['payload']==12 and item['err54']==item['err58']==0
            for slot in ['0','1']:
                before=item['before'][slot]['fields'];after=item['after'][slot]['fields']
                for field in before:
                    expected=1 if slot=='0'and field==('0x5f4'if kind==3 else'0x60c') else scalar(before[field])
                    assert scalar(after[field])==expected
                    getter_checks.extend([before[field],after[field]])
                assert scalar(after['0x60c'if kind==3 else'0x5f4'])==0
        native=tagged(text,'ABILITY_MOTION');retire=tagged(text,'ABILITY_RETIRE')
        starts=[row for row in native if row['kind']==('kick_end'if kind==3 else'throw_end')]
        readers=[row for row in native if row['kind']=='162_end']
        assert len(starts)==len(readers)==len(fields)
        output=[(row['opcode'],row['body_hex'])for row in channel['out']if row['opcode']in[158,159,162,165]]
        if shared is None:shared=output
        else:assert output==shared
        emitted=[row for row in channel['out']if row['opcode']==162]
        assert len(emitted)==len(requests)
        for index,(case,request,values,entry,reader,reply)in enumerate(zip(matrix['cases'],requests,fields,starts,readers,emitted)):
            assert entry['round']==reader['round']==case['round']
            assert reader['cursor']==reader['size']==9 and reader['errors']==[0,0]
            assert reply['body_hex']==request['body_hex'][:18]
            obj=entry['object'];stop=starts[index+1]['ns']if index+1<len(starts)else 2**64
            rb=next(row for row in retire if row['kind']=='rebind_end'and row['object']==obj and entry['ns']<row['ns']<stop and row['requested_id']==0xffffffff)
            landed=next(row for row in native if row['kind']=='motionstate_set'and row.get('object')==obj and entry['ns']<row['ns']<rb['ns']and row['requested_state']==4)
            cleanup=next(row for row in native if row['kind']=='cleanup'and row['object']==obj and rb['ns']<row['ns']<stop)
            assert entry['fields']['0xcc']['u32']==0 and rb['fields']['0xcc']['u32']==1
            assert cleanup['fields']['0x1c']['u32']==0
            assert [cleanup['fields'][f]['u32']for f in ['0x1ac','0x1b8']]==values[6:8]
            for snapshot in [entry,landed,rb,cleanup]:assert all(f.get('checksum_valid')for f in snapshot['fields'].values())
            at=next(i for i,row in enumerate(channel['out'])if row is reply)
            short=codec.pack_fields([1,values[3],1],[1,2,1]).hex()
            blast=codec.pack_fields([1,values[3],0,2,0xffffffff,255],[1,2,1,4,4,1]).hex()
            after=channel['out'][at+1:];short_index=next(i for i,row in enumerate(after)if row['opcode']==158 and row['body_hex']==short)
            assert after[short_index+1]['opcode']==158 and after[short_index+1]['body_hex']==blast
            if role=='A':cases.append(dict(round=case['round'],selector=values[2],bomb_id=values[3],source=values[4:6],target=values[6:8],endpoint_delta=[values[6]-values[4],values[7]-values[5]],opaque_tail=values[8:]))
        if role=='B':assert not any(row['opcode']==119 for row in channel['incoming'])
    # Independently captured native wrappers replay through the retained getter.
    # This is post-run emulation of read-only observations, not a reference server.
    import struct
    support=(ROOT/'local/research/codex').resolve();oracle={}
    exec((support/'bootstrap-recv-oracle.py').read_text().rsplit('\nrows=[]',1)[0],oracle)
    mapped=set();u=oracle['u'];invoke=oracle['invoke'];obj=oracle['obj']
    for snapshot in getter_checks:
        words=bytes.fromhex(snapshot['words_hex']);data=bytes.fromhex(snapshot['data_hex']);address=snapshot['data_pointer']
        for page in range(address&~4095,((address+len(data)-1)&~4095)+1,4096):
            if page not in mapped:u.mem_map(page,4096);mapped.add(page)
        u.mem_write(0x19d88d4,struct.pack('<I',0));u.mem_write(0x1a7e438,struct.pack('<I',0));u.mem_write(address,data);u.mem_write(obj,words)
        assert invoke(0x52a350,[],obj)==scalar(snapshot)
    # Missing-item input control and one pickup per current round, on the same channel.
    current=[];round_commands=[]
    for row in channels[actor['wire_id']]['incoming']:
        if row['opcode']==104:
            if current:round_commands.append(current)
            current=[]
        if row['opcode']in[118,119,124]:current.append(row)
    if current:round_commands.append(current)
    assert len(round_commands)==matrix['rounds']
    for commands in round_commands:
        first_pickup=next(i for i,row in enumerate(commands)if row['opcode']==124)
        assert any(row['opcode']==118 for row in commands[:first_pickup])
        assert not any(row['opcode']==119 for row in commands[:first_pickup])
        assert sum(row['opcode']==124 for row in commands)==1
    result=dict(result='verified_native_motion_matrix_peer_retirement_reset_and_wrappers',source='kr_runtime',status='confirmed',
                run=pin(out/'run.json'),wire=pin(root/'decoded-channels-shared.json'),matrix=pin(root/'matrix.json'),
                item=kind,rounds=matrix['rounds'],cases=cases,native_getter_checks=len(getter_checks),
                covered_selectors=sorted({row[2]for row in fields}),selector_name_correction=pin(correction_path)if corrections else None,
                limits='Native Wine/private LAN, persisted C1 actor/C0 peer. Report-derived motion/retirement policy is design. Endpoint delta is not flight length; no original server authority, Internet/load or whole G1 closure.')
    target=root/'motion-verification.json';assert not target.exists();write(target,result);print(result['result'])



def vi_smoke(root):
    inputs=read(root/'vi-client-inputs.json');out=root/'native/cycles'
    expected='37655a36d11b94dbe18bcfdcaaf68ec92c9271743d8e3fdf919492e8f0edaa84'
    assert inputs['exe']['sha256']==expected
    check_runtime_inputs(root,[row['target']for row in inputs['files']])
    wire=read(root/'decoded-channels-shared.json')
    assert wire['run_sha256']==pin(out/'run.json')['sha256']
    run=read(out/'run.json');matrix=read(root/'matrix.json')
    assert matrix['map']=='Factory07' and matrix['rounds']==2
    events=[json.loads(line)for line in (out/'rust-events.jsonl').read_text().splitlines()]
    pickups=[row for row in events if (row.get('pickup')or{}).get('accepted')]
    assert [row['pickup']['index']for row in pickups]==[701,701]
    views={}
    for role in matrix['roster']:
        owner=run['account_launches'][role]['response']['player']
        channel=next(row for row in wire['channels']if row['identity']==owner['wire_id'])
        for opcode in [121,165,175,180]:assert sum(row['opcode']==opcode for row in channel['out'])==2
        text=(out/role/'client.log').read_text(errors='replace')
        assert not any(marker in text for marker in ['SELECTED_PEER_ERROR','CAMP_FLAGS_PROBE_ERROR','received signal SIGSEGV','received signal SIGABRT','"kind":"probe_error"'])
        maps=[row for row in tagged(text,'G1_MAP')if row['kind']=='name_compare']
        assert len(maps)==2 and all(row['incoming_name']['text']=='Factory07'and row['result']==0 for row in maps)
        clocks=[row for row in tagged(text,'FOUR_SLOT')if row['kind']=='clock']
        assert len(clocks)==2
        for clock in clocks:
            assert set(clock['actors'])=={'0','1'}
            assert clock['actors']['0']['fields']['0x4a4']['value']==2
            assert clock['actors']['1']['fields']['0x4a4']['value']==1
        records=tagged(text,'RUST_M3_ITEM');items=[row for row in records if row['kind']=='end']
        assert len(items)==2
        fields=[row for row in records if row['kind']=='fields']
        assert len(fields)==2 and all((row['row'],row['column'],row['index'],row['item_kind'],row['slot'])==(2,0,701,0,0)for row in fields)
        for item in items:
            assert item['cursor']==item['payload']==12 and item['err54']==item['err58']==0
            for slot in ['0','1']:
                before=item['before'][slot]['fields'];after=item['after'][slot]['fields']
                for field,value in before.items():
                    delta=1 if slot=='0'and field=='0x4a4'else 0
                    assert after[field]['value']==value['value']+delta
        views[role]=dict(maps=2,clocks=2,pickups=2,bomb_stat=[2,3],peer_stat=1)
    result=dict(result='verified_vi_Factory07_item_two_rounds_reset',source='kr_runtime',status='confirmed',
                run=pin(out/'run.json'),wire=pin(root/'decoded-channels-shared.json'),inputs=pin(root/'vi-client-inputs.json'),views=views,
                limits='Current produced L1 vi client on isolated Wine LAN. Integration smoke only; KR unmodified-language client remains gameplay oracle. Manual image review recorded separately; no L1 closure, Internet or M8 claim.')
    target=root/'vi-smoke-verification.json';assert not target.exists();write(target,result);print(result['result'])


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run',type=Path,required=True)
    parser.add_argument('--group',choices=['character','motion','lifecycle','vi-smoke'],required=True)
    args=parser.parse_args()
    root=args.run.resolve(strict=True)
    assert root.is_relative_to((ROOT/'local').resolve())
    {'character':character,'motion':motion,'lifecycle':lifecycle,'vi-smoke':vi_smoke}[args.group](root)


if __name__ == '__main__':
    main()
