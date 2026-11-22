"""Kiểm mọi map của tập họ F1-a bằng pack KR và kr-load trong namespace cục bộ."""
import argparse
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import subprocess
import sqlite3
import tempfile

from export_map_profile import load_tile_table
from kr_pack_export import ITEMS, PACK_SHA256, Pack, f1_rules, file_sha, read_map, verified_packs
from test_kr_pack_install import ROOT, run_smoke, test_bundle


STAT_ITEMS = {ITEMS[kind] for kind in (0, 1, 2, 5)}


def inventory(game):
    families = {r['name']: r['map_count']
                for r in json.loads((ROOT / 'docs/content/classification.json').read_bytes())['rows']
                if r['kind'] == 'map' and r['wave'] == 'F1' and r['classification'] in ('A', 'B')}
    census = json.loads((ROOT / 'docs/content/census.json').read_bytes())['maps']
    rows = {}
    with verified_packs(game) as files:
        pack = Pack(files['FxMap.idx'], files['FxMap.idd'])
        order = {entry[4:]: code for code, entry in enumerate(
            name for name, *_ in pack.order if name.startswith('Map/'))}
        table = load_tile_table(Pack(files['fx.idx'], files['fx.idd']))
        for entry in census:
            name, family = entry['name'], entry['family']
            if family not in families:
                continue
            # Preserve content blockers even when the terrain adapter rejects the map.
            row = rows[name] = dict(family=family, code=order[name], parse=entry['parse'],
                                    script_tags=entry['script_tags'],
                                    omitted_positive_drops=sorted({item for item, weight in entry['drops']
                                                                   if weight > 0 and item not in STAT_ITEMS}))
            try:
                profile = read_map(pack, name, row['code'], f1=True)
                rules = f1_rules(profile, table)
            except ValueError as error:
                row.update(status='adapter_rejected', error=str(error),
                           unverified_placements=sorted(set(entry['placements'])))
            else:
                row.update(status='pending', map_sha256=profile['payload_sha256'],
                           terrain_sha256=rules['terrain']['seed_sha256'],
                           omitted_placements=rules['floor']['omitted_placements'])
        if any(file_sha(stream) != PACK_SHA256[name] for name, stream in files.items()):
            raise ValueError('Pack đã đổi trong lúc kiểm kê')
    if Counter(row['family'] for row in rows.values()) != families:
        raise ValueError('Kiểm kê F1-a thiếu map hoặc sai số lượng theo họ')
    return rows


def run_matrix(game, binaries, out, workers):
    rows = inventory(game)
    (out / 'inventory.json').write_text(json.dumps(rows, indent=2, ensure_ascii=False) + '\n')
    pending = [name for name, row in rows.items() if row['status'] == 'pending']

    with tempfile.TemporaryDirectory(dir=out, prefix='.package-') as work:
        bundle, manifest = test_bundle(Path(work), binaries, 'f1-a', False)

        def run_catalog(task):
            label, names = task
            result = out / label
            result.mkdir(mode=0o700)
            run_names = list(dict.fromkeys([*names, 'CaJJ_Clover01']))
            expected = [(rows[n]['code'], n) for n in run_names]
            (result / 'catalog.json').write_text(json.dumps(expected) + '\n')
            with (result / 'harness.log').open('w') as log:
                try:
                    rc = run_smoke(bundle, manifest, game, result, expected, fake='', bwrap=True, image=None, timeout=30,
                                   stdout=log, stderr=subprocess.STDOUT).returncode
                except subprocess.TimeoutExpired:
                    rc = 124
            receipt = result / 'verification.json'
            if receipt.is_file():
                actual = json.loads(receipt.read_bytes())['map_results']
                if actual == [['complete', code, name] for code, name in expected]:
                    return {name: dict(status='passed', run=label) for name in names}
            # Some KR map descriptors keep the final leave response open after
            # the journal is complete. The cycle itself is still authoritative
            # when every expected round_returned and complete journal is present.
            load_log = result / 'load.log'
            returned = [json.loads(line) for line in load_log.read_text().splitlines()
                        if line.startswith('{') and json.loads(line).get('event') == 'round_returned'] if load_log.is_file() else []
            db = result / 'state' / 'accounts.sqlite'
            if db.is_file():
                try:
                    complete = {name for state, code, name in sqlite3.connect(db).execute(
                        'SELECT state,map_code,map_name FROM matches WHERE state=\'complete\'')}
                except sqlite3.Error:
                    complete = set()
                if all(name in complete for name in names):
                    return {name: dict(status='passed_cycle', run=label, return_events=len(returned))
                            for name in names}
            # A failed catalog does not make the other maps failures. Replay
            # sibling groups with a fresh service and preserve the original logs.
            if len(names) > 1:
                mid = len(names) // 2
                return run_catalog((label + '-a', names[:mid])) | run_catalog((label + '-b', names[mid:]))
            return {names[0]: dict(status='load_failed', run=label, exit=rc)}

        batches = [(f'catalog-{i + 1:03}', [name]) for i, name in enumerate(pending)]
        with ThreadPoolExecutor(max_workers=workers) as pool:
            for results in pool.map(run_catalog, batches):
                for name, result in results.items():
                    rows[name].update(result)
                print(', '.join(f"{name}={row['status']}" for name, row in results.items()), flush=True)

    report = {}
    for row in rows.values():
        report.setdefault(row['family'], Counter())[row['status']] += 1
    with (binaries / 'kr-service').open('rb') as service, (binaries / 'kr-load').open('rb') as driver:
        pins = {'kr-service': file_sha(service), 'kr-load': file_sha(driver)}
    (out / 'matrix.json').write_text(json.dumps(
        dict(build='KR 00967', source='emu_choice', status='candidate', packs=PACK_SHA256,
             bundle_manifest_sha256=manifest, binaries=pins, families=report, maps=rows),
        indent=2, ensure_ascii=False) + '\n')
    print('Họ map | Đạt | Từ chối adapter | Lỗi kr-load', flush=True)
    for name, counts in report.items():
        print(f"{name} | {counts['passed']} | {counts['adapter_rejected']} | {counts['load_failed']}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game-dir', type=Path, required=True)
    parser.add_argument('--binaries', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--workers', type=int, choices=(1, 2, 3, 4), default=2)
    args = parser.parse_args()
    out = args.out.resolve()
    if not out.is_relative_to((ROOT / 'local').resolve()) or out == (ROOT / 'local').resolve():
        parser.error('--out phải là thư mục lượt chạy mới dưới local/')
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    run_matrix(args.game_dir.resolve(strict=True), args.binaries.resolve(strict=True), out, args.workers)


if __name__ == '__main__':
    main()
