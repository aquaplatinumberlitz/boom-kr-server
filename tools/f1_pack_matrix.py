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
from kr_pack_export import (F1_FIXED_ITEMS, F1_HIDDEN_ITEMS, PACK_SHA256, Pack,
                            f1_rules, file_sha, read_map, verified_packs)
from test_kr_pack_install import ROOT, run_smoke, test_bundle


def omitted_items(placements, drops):
    hidden = sorted({name for name, weight in drops if weight > 0 and name not in F1_HIDDEN_ITEMS})
    fixed = {name for name in placements if name not in F1_FIXED_ITEMS}
    return hidden, sorted(fixed.union(hidden))


def load_events(result_dir):
    log = result_dir / 'load.log'
    if log.is_file():
        for line in log.read_text().splitlines():
            if line.startswith('{'):
                try:
                    yield json.loads(line)
                except json.JSONDecodeError:
                    # A process killed by the deadline may leave a partial line.
                    pass


def failure_kind(row, result_dir, exit_code=None):
    """Phân biệt giới hạn nội dung, driver và lỗi service từ một lượt kẹt."""
    if row['script_tags']:
        return 'contract_script'
    if row['omitted_placements'] or row['omitted_positive_drops']:
        return 'contract_placement_or_flag'
    if exit_code == 124:
        # The outer harness killed the run; a partial log does not tell
        # whether the service would have completed its scheduled work.
        return 'harness_timeout'
    if not any(e.get('event') == 'round_active' for e in load_events(result_dir)):
        return 'driver_error'
    return 'server_or_adapter_error'


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
            hidden, unknown = omitted_items(entry['placements'], entry['drops'])
            row = rows[name] = dict(family=family, code=order[name], parse=entry['parse'],
                                    script_tags=entry['script_tags'],
                                    omitted_positive_drops=hidden, omitted_unknown=unknown)
            try:
                profile = read_map(pack, name, row['code'], f1=True)
                rules = f1_rules(profile, table)
            except ValueError as error:
                row.update(status='adapter_rejected', error=str(error),
                           unverified_placements=sorted(set(entry['placements'])))
            else:
                row.update(status='pending', map_sha256=profile['payload_sha256'],
                           terrain_sha256=rules['terrain']['seed_sha256'],
                           omitted_placements=rules['floor']['omitted_placements'],
                           floor={group: rules['floor'][group] for group in ('fixed', 'hidden')})
        if any(file_sha(stream) != PACK_SHA256[name] for name, stream in files.items()):
            raise ValueError('Pack đã đổi trong lúc kiểm kê')
    if Counter(row['family'] for row in rows.values()) != families:
        raise ValueError('Kiểm kê F1-a thiếu map hoặc sai số lượng theo họ')
    return rows


def run_matrix(game, binaries, out, workers, selected=None):
    rows = inventory(game)
    (out / 'inventory.json').write_text(json.dumps(rows, indent=2, ensure_ascii=False) + '\n')
    pending = [name for name, row in rows.items()
               if row['status'] == 'pending' and (selected is None or name in selected)]

    with tempfile.TemporaryDirectory(dir=out, prefix='.package-') as work:
        bundle, manifest = test_bundle(Path(work), binaries, 'f1-a', False)

        def run_catalog(task):
            label, names = task
            result = out / label
            result.mkdir(mode=0o700)
            # Catalog::load needs at least two choices. Keep a measured Clover
            # control, but use a different control when the target itself is
            # Clover01 (the old runner accidentally exercised a one-choice
            # catalog and reported a driver failure).
            control = 'CaJJ_Clover01'
            if control in names:
                control = next((n for n, row in rows.items()
                                if n not in names and row['status'] == 'pending'), None)
            run_names = list(dict.fromkeys([*names, *([control] if control else [])]))
            expected = [(rows[n]['code'], n) for n in run_names]
            (result / 'catalog.json').write_text(json.dumps(expected) + '\n')
            with (result / 'harness.log').open('w') as log:
                try:
                    # Keep the smoke harness's outer budget: setup and both
                    # rounds (including the control) share this deadline.
                    rc = run_smoke(bundle, manifest, game, result, expected, fake='', bwrap=True, image=None,
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
            events = list(load_events(result))
            returned = sum(e.get('event') == 'round_returned' for e in events)
            fixed = [(e['map'], e['items']) for e in events if e.get('event') == 'fixed_items_confirmed']
            expected_fixed = [(rows[n]['code'], rows[n]['floor']['fixed']) for n in run_names]
            db = result / 'state' / 'accounts.sqlite'
            if db.is_file() and returned == len(expected) and fixed == expected_fixed:
                try:
                    complete = {name for state, code, name in sqlite3.connect(db).execute(
                        'SELECT state,map_code,map_name FROM matches WHERE state=\'complete\'')}
                except sqlite3.Error:
                    complete = set()
                if all(name in complete for name in run_names):
                    return {name: dict(status='passed_cycle', run=label, return_events=returned)
                            for name in names}
            # A failed catalog does not make the other maps failures. Replay
            # sibling groups with a fresh service and preserve the original logs.
            if len(names) > 1:
                mid = len(names) // 2
                return run_catalog((label + '-a', names[:mid])) | run_catalog((label + '-b', names[mid:]))
            name = names[0]
            return {name: dict(status='load_failed', failure=failure_kind(rows[name], result, rc),
                                run=label, exit=rc)}

        batches = [(f'catalog-{i + 1:03}', [name]) for i, name in enumerate(pending)]
        with ThreadPoolExecutor(max_workers=workers) as pool:
            for results in pool.map(run_catalog, batches):
                for name, result in results.items():
                    rows[name].update(result)
                print(', '.join(f"{name}={row['status']}" for name, row in results.items()), flush=True)

    report = {}
    failures = {}
    for row in rows.values():
        report.setdefault(row['family'], Counter())[row['status']] += 1
        if row['status'] == 'load_failed':
            failures.setdefault(row['family'], Counter())[row['failure']] += 1
    with (binaries / 'kr-service').open('rb') as service, (binaries / 'kr-load').open('rb') as driver:
        pins = {'kr-service': file_sha(service), 'kr-load': file_sha(driver)}
    (out / 'matrix.json').write_text(json.dumps(
        dict(build='KR 00967', source='emu_choice', status='candidate', packs=PACK_SHA256,
             bundle_manifest_sha256=manifest, binaries=pins, families=report, maps=rows),
        indent=2, ensure_ascii=False) + '\n')
    print('Họ map | Số map | Đạt | Từ chối adapter | Từ chối contract | Lỗi driver | Lỗi server/adapter | Timeout harness', flush=True)
    for name, counts in report.items():
        failed = failures.get(name, Counter())
        print(f"{name} | {sum(counts.values())} | {counts['passed'] + counts['passed_cycle']} | "
              f"{counts['adapter_rejected']} | {failed['contract_script'] + failed['contract_placement_or_flag']} | "
              f"{failed['driver_error']} | {failed['server_or_adapter_error']} | {failed['harness_timeout']}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game-dir', type=Path, required=True)
    parser.add_argument('--binaries', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--workers', type=int, choices=(1, 2, 3, 4), default=2)
    parser.add_argument('--map', dest='maps', action='append', help='Chỉ chạy lại map đã nêu')
    args = parser.parse_args()
    out = args.out.resolve()
    if not out.is_relative_to((ROOT / 'local').resolve()) or out == (ROOT / 'local').resolve():
        parser.error('--out phải là thư mục lượt chạy mới dưới local/')
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    run_matrix(args.game_dir.resolve(strict=True), args.binaries.resolve(strict=True), out, args.workers,
               set(args.maps) if args.maps else None)


if __name__ == '__main__':
    main()
