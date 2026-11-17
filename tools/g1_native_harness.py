"""Harness G1: prepare/run/verify bằng một ma trận JSON, raw chỉ ở local/.

Không sinh script theo selector. Runtime chỉ dùng phím/chuột bình thường và
probe đọc; điều kiện bước lấy từ event Rust hoặc consumer/vị trí client thật.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
LOCAL = (ROOT / 'local').resolve()


def read(path):
    return json.loads(Path(path).read_text())


def write(path, value):
    Path(path).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def pin(path):
    path = Path(path).resolve(strict=True)
    with path.open('rb') as stream:
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    return dict(path=str(path), bytes=path.stat().st_size, sha256=digest)


def check_pins(rows):
    for row in rows:
        assert pin(row['path']) == row, 'Input changed: ' + row['path']


def local_path(path):
    path = Path(path).resolve()
    if path == LOCAL or not path.is_relative_to(LOCAL):
        raise ValueError('Run/copy/output must be below local/')
    return path


def matches(row, expected):
    for path, value in expected.items():
        actual = row
        for key in path.split('.'):
            if not isinstance(actual, dict) or key not in actual:
                return False
            actual = actual[key]
        if actual != value:
            return False
    return True


def tagged(text, tag):
    return [json.loads(line[len(tag)+1:]) for line in text.splitlines()
            if line.startswith(tag + ' {')]


def drive_matrix(matrix, config, out, states, record, topology, click, press, align, snap, seconds=600):
    """Bounded controller; no sleep-only gameplay transitions or inferior writes."""
    assert list(states) == matrix['roster']
    record['matrix'] = {key: matrix[key] for key in ['map', 'item', 'roster', 'cases']}
    total_deadline = time.monotonic() + seconds
    for step in matrix['steps']:
        entered = time.monotonic()
        deadline = min(total_deadline, entered + step.get('timeout_seconds', 60))
        while True:
            topology.observe(states)
            logs = {role: (st['out']/'client.log').read_text(errors='replace')
                    for role, st in states.items()}
            for role, st in states.items():
                if st['_process'].poll() is not None or any(marker in logs[role] for marker in [
                    'received signal SIGSEGV', 'received signal SIGABRT', 'received signal SIGILL',
                    'A fatal error internal to GDB', 'SELECTED_PEER_ERROR', 'ABILITY_MOTION_ERROR',
                    'ABILITY_RETIRE_ERROR', '"kind":"probe_error"',
                ]):
                    raise RuntimeError('Client failed at ' + step['name'] + ': ' + role)
            events = [json.loads(line) for line in (out/'rust-events.jsonl').read_text().splitlines()]
            satisfied = True
            for condition in step.get('wait', []):
                role = condition.get('role')
                if condition['source'] == 'server':
                    rows = events
                    if role:
                        identity = record['account_launches'][role]['response']['player']['wire_id']
                        rows = [row for row in rows if row.get('identity') == identity]
                elif condition['source'] == 'client':
                    rows = tagged(logs[role], condition['tag'])
                else:
                    raise ValueError('Unknown condition source')
                count = sum(matches(row, condition['match']) for row in rows)
                satisfied &= count >= condition.get('count', 1)
            record['controller_stage'] = step['name']
            write(out/'runtime-progress.json', record)
            if satisfied and time.monotonic()-entered >= step.get('minimum_seconds', 0):
                break
            if time.monotonic() >= deadline:
                raise TimeoutError('Matrix state deadline: ' + step['name'])
            time.sleep(.1)
        record.setdefault('matrix_steps', []).append(dict(name=step['name'],ns=time.monotonic_ns()))
        for index, action in enumerate(step.get('actions', [])):
            role = action.get('role', 'A')
            label = step['name'] + '-' + str(index)
            operation = action['op']
            if operation == 'click':
                click(role, *action['xy'], label, action.get('double', False))
            elif operation == 'key':
                press(role, action['key'], action.get('seconds', .1))
            elif operation == 'align':
                align(role, matrix['roster'].index(role), action['field'], action['target'], label)
            elif operation == 'move':
                row, column = action['cell']
                slot = matrix['roster'].index(role)
                align(role, slot, '0x4c', 40*column, label+'-column')
                align(role, slot, '0x58', 40*row+39, label+'-row')
            elif operation == 'feedback':
                if action['enabled']:
                    (out/'feedback-on').write_text('Read-only actor positions; ordinary keys only')
                else:
                    (out/'feedback-on').unlink(missing_ok=True)
            elif operation == 'snapshot':
                snap(label)
            else:
                raise ValueError('Unknown action: ' + operation)
    snap('matrix-complete')
    record['final_nat_rules'] = topology.run(['iptables-save', '-c', '-t', 'nat'])
    record['early_completion'] = dict(ns=time.monotonic_ns(),reason='Matrix steps complete; independent verification required')


def prepare(args):
    base = local_path(args.base)
    out = local_path(args.out)
    matrix = read(args.matrix)
    config = read(base/'config.json')
    assert list(config['roles']) == matrix['roster']
    assert 2 <= len(matrix['roster']) <= 8 and 1 <= matrix['rounds'] <= 16
    assert matrix['item'] in [None, 0, 1, 2, 3, 4, 5, 145]
    assert matrix['steps'] and matrix['cases']
    for name in ['kr-service', 'kr-admin', 'kr-companion']:
        (args.binary_dir/name).resolve(strict=True)
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    (out/'native').mkdir()
    (out/'binary').mkdir()
    for name in ['kr-service', 'kr-admin', 'kr-companion']:
        shutil.copy2(args.binary_dir/name, out/'binary'/name)
    # Existing source and probes are inputs. Copy only support, never generate a
    # new Python program for the selected map, item, direction or roster.
    for source in base.iterdir():
        if source.suffix in ['.py', '.gdb']:
            (out/source.name).write_text(source.read_text().replace(str(base), str(out)))
    shutil.copy2(base/'previous-accounts.sqlite', out/'previous-accounts.sqlite')
    reused = {}
    for role, data in config['roles'].items():
        for field, name in [('game_dir','game-'+role), ('prefix','prefix-'+role)]:
            source = local_path(data[field])
            if args.reuse_resources:
                reused.setdefault(role,{})[field]=str(source)
            else:
                subprocess.run(['cp','-a','--reflink=auto',str(source),str(out/name)], check=True)
                data[field] = str(out/name)
    config = json.loads(json.dumps(config).replace(str(base), str(out)))
    for role, paths in reused.items():config['roles'][role].update(paths)
    server = json.loads((base/'native-config.json').read_text().replace(str(base),str(out)))
    admission = server['diagnostic_two_lobby']['account_admission']
    admission.update(database=str(out/'native/cycles/account-db/accounts.sqlite'),launch_socket='/tmp/kr-g1-matrix/launch.sock')
    config.update(rust_config=str(out/'native-config.json'), launch_socket=admission['launch_socket'],matrix=str(out/'matrix.json'))
    write(out/'config.json',config)
    write(out/'native-config.json',server)
    write(out/'matrix.json',matrix)
    recipe = read(base/'native/recipe.json')
    recipe[1] = str(ROOT/'tools/g1_native_runtime.py')
    for option, value in [('--game-dir', config['roles']['A']['game_dir']),
                          ('--prefix',config['roles']['A']['prefix']),
                          ('--out',str(out/'native/cycles')),('--seconds',str(args.seconds)),
                          ('--commands',config['commands_by_role']['A']),('--pair-config',str(out/'config.json'))]:
        recipe[recipe.index(option)+1] = value
    write(out/'native/recipe.json',recipe)
    inputs = {ROOT/'tools/g1_native_harness.py', ROOT/'tools/g1_native_runtime.py', args.matrix.resolve()}
    inputs.update(p for p in out.iterdir() if p.is_file())
    inputs.update((out/'binary').iterdir())
    inputs.update((ROOT/'rust').rglob('*.rs'))
    inputs.update((ROOT/'rust').rglob('*.sql'))
    inputs.update([ROOT/'rust/Cargo.lock',ROOT/'rust/Cargo.toml',ROOT/'rust/rust-toolchain.toml'])
    inputs.update((ROOT/'rust/data').glob('*.json'))
    def fixtures(value):
        if isinstance(value, dict):
            if {'path','bytes','sha256'} <= value.keys():
                check_pins([value]);inputs.add(Path(value['path']))
            for child in value.values():fixtures(child)
        elif isinstance(value,list):
            for child in value:fixtures(child)
    fixtures(server)
    # Runtime cache files are intentionally outside the immutable input set.
    # The executable/pack copy is bound, Wine prefix and generated caches are not.
    for data in config['roles'].values():
        inputs.update(p for p in Path(data['game_dir']).iterdir()
                      if p.is_file() and p.name not in ['CALog.log','fxdynmapinfo.ssd'])
    previous = base/'native/runtime-preflight.json'
    if previous.is_file():
        inputs.update(Path(row['path']) for row in read(previous)['files']
                      if Path(row['path']).suffix in ['.py', '.gdb'] and not Path(row['path']).is_relative_to(base))
    inputs.add(out/'native/recipe.json')
    write(out/'native/runtime-preflight.json',dict(files=[pin(p) for p in sorted(inputs)],
          limits='Candidate matrix inputs. No runtime or compatibility acceptance.'))
    print('Prepared matrix:',len(matrix['cases']),'cases; not an acceptance verdict')


def run(args):
    root = local_path(args.run)
    rows = read(root/'native/runtime-preflight.json')['files']
    check_pins(rows)
    assert not (root/'native/launch-exit.json').exists()
    with (root/'native/launch.txt').open('x') as stream:
        result = subprocess.run(read(root/'native/recipe.json'),cwd=ROOT,stdout=stream,stderr=subprocess.STDOUT,timeout=3900)
    write(root/'native/launch-exit.json',dict(returncode=result.returncode,limits='Process exit only; run verify for semantics'))
    check_pins(rows)
    if result.returncode:
        raise RuntimeError('Native runtime failed; retained raw and failure')
    print('Runtime ended; semantic verification pending')


def verify(args):
    root = local_path(args.run)
    matrix = read(root/'matrix.json')
    out = root/'native/cycles'
    assert read(root/'native/launch-exit.json')['returncode'] == 0
    check_pins(read(root/'native/runtime-preflight.json')['files'])
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
            else:
                assert [row['round'] for row in rows] == rounds, (role,tag,kind)
        for expected in matrix.get('native_expectations',[]):
            if expected.get('role') not in [None,role]:continue
            rows = tagged(text,expected['tag'])
            assert sum(matches(row,expected['match'])for row in rows) == expected['count'], (role,expected)
    with sqlite3.connect('file:'+str(out/'account-db/accounts.sqlite')+'?mode=ro',uri=True) as db:
        seed = sqlite3.connect('file:'+str(root/'previous-accounts.sqlite')+'?mode=ro',uri=True)
        first = seed.execute('select coalesce(max(id),0)from matches').fetchone()[0]
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
                  limits='Lifecycle/identity/declared controls only. Geometry, wire, effects and G1 closure require group evidence.')
    with (root/'matrix-verification.json').open('x') as stream:
        json.dump(result,stream,ensure_ascii=False,indent=2);stream.write('\n')
    print(result['result'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command',required=True)
    prep = sub.add_parser('prepare')
    for name in ['base','matrix','binary-dir','out']:
        prep.add_argument('--'+name,type=Path,required=True)
    prep.add_argument('--seconds',type=int,default=600)
    prep.add_argument('--reuse-resources',action='store_true',help='Reuse idle copied game/prefix scratch trees; sealed run/raw is never changed')
    for name in ['run','verify']:
        command = sub.add_parser(name);command.add_argument('--run',type=Path,required=True)
    args = parser.parse_args()
    {'prepare':prepare,'run':run,'verify':verify}[args.command](args)


if __name__ == '__main__':
    main()
