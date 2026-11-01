"""Validate declared feature metadata and generate navigation; never certify semantics."""
import argparse
from collections import Counter
import json
from pathlib import Path
import re
import subprocess
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[1]
CATALOG = 'docs/features.json'
OUTPUT = 'docs/COVERAGE.md'
STATES = {
    'presence': {'present', 'candidate', 'absent', 'unknown'},
    'vn_reference': {'static', 'runtime', 'both', 'unknown'},
    'kr_static': {'partial', 'unknown'},
    'emulation': {'checked', 'unknown'},
    'runtime': {'observed', 'unknown'},
    'implementation': {'partial', 'design', 'unknown'},
    'integration': {'partial', 'measured', 'pending', 'unknown'},
    'gate': {'open', 'passed', 'unknown'},
}
ENVIRONMENTS = {'wine_diagnostic_copy', 'windows_original', 'lan', 'internet'}
DELIVERY = {'baseline_merged', 'branch_pending', 'unknown'}


def load(root=ROOT):
    return json.loads((root / CATALOG).read_text())


def link_error(target, root):
    if not isinstance(target, str) or not target:
        return 'empty/non-string link'
    if re.match(r'^https://', target):
        return None  # External provenance is declared, not fetched/verified.
    name, _, fragment = target.partition('#')
    path = (root / unquote(name)).resolve()
    if not path.is_relative_to(root.resolve()) or not path.is_file():
        return f'missing/outside file: {target}'
    if fragment:
        text = path.read_text()
        anchors = set(re.findall(r'<a id="([^"]+)"', text))
        for heading in re.findall(r'^#{1,6} (.+)$', text, re.M):
            slug = re.sub(r'[^\w\- ]', '', heading.lower()).replace(' ', '-')
            anchors.add(slug)
        if unquote(fragment) not in anchors:
            return f'missing anchor: {target}'
    return None


def research_rows(root=ROOT):
    rows = {}
    for line in (root / 'docs/RESEARCH.md').read_text().splitlines():
        match = re.match(r'\| <a id="gap-([a-z0-9-]+)"></a>(KR-[A-Z][0-9]{3}) \| (.*?) \| (.*?) \|$', line)
        if match:
            rows[match[2]] = {'anchor': 'gap-' + match[1], 'gap': match[3], 'test': match[4]}
    return rows


def validate(data, root=ROOT, previous=None):
    errors = []
    def check(ok, message):
        if not ok:
            errors.append(message)
    check(data.get('schema') == 'kre-feature-index-v1', 'schema invalid')
    check(data.get('build') == 'KR00967', 'build invalid')
    for key, target in {'plan': 'docs/PLAN.md', 'contract': 'docs/PROTOCOL.md',
                        'questions': 'docs/RESEARCH.md',
                        'history': 'docs/evidence/checkpoints/manifest.json'}.items():
        check(data.get('sources', {}).get(key) == target, f'canonical source {key} invalid')
    nodes = data.get('nodes', [])
    ids = [n.get('id') for n in nodes]
    check(len(set(ids)) == len(ids), 'duplicate ID')
    mechanisms = [n.get('mechanism') for n in nodes]
    check(len(set(mechanisms)) == len(mechanisms), 'duplicate mechanism key')
    by_id = {n.get('id'): n for n in nodes}
    rows = research_rows(root)
    for node in nodes:
        ident = node.get('id', '')
        kind = node.get('kind')
        check(kind in {'group', 'feature'}, f'{ident}: kind invalid')
        pattern = r'KR-G[0-9]{3}' if kind == 'group' else r'KR-[BLSRFX][0-9]{3}'
        check(bool(re.fullmatch(pattern, ident)), f'{ident}: ID invalid')
        check(bool(node.get('title')) and bool(node.get('mechanism')), f'{ident}: title/mechanism required')
        check(node.get('lifecycle') in {'active', 'retired'}, f'{ident}: lifecycle invalid')
        parent = node.get('parent')
        check(parent is None or (parent in by_id and by_id[parent].get('kind') == 'group'), f'{ident}: parent invalid')
        dependencies = node.get('depends_on', [])
        check(len(dependencies) == len(set(dependencies)), f'{ident}: duplicate dependency')
        for dep in dependencies:
            check(dep in by_id and by_id[dep].get('kind') == 'feature', f'{ident}: missing/non-feature dependency {dep}')
        if kind == 'group':
            continue
        refs = node.get('refs', {})
        check(set(refs) == {'contract', 'question', 'plan', 'evidence'}, f'{ident}: canonical refs invalid')
        check(refs.get('contract', '').startswith('docs/PROTOCOL.md#'), f'{ident}: contract must have locator')
        check(refs.get('plan') == 'docs/PLAN.md#current-order', f'{ident}: work order must link canonical PLAN')
        check(ident in rows, f'{ident}: missing canonical gap/test row')
        if ident in rows:
            check(refs.get('question') == 'docs/RESEARCH.md#' + rows[ident]['anchor'], f'{ident}: question locator mismatch')
        check(bool(refs.get('evidence')), f'{ident}: evidence/scope reference required even for unknown')
        for key, value in refs.items():
            for target in value if isinstance(value, list) else [value]:
                error = link_error(target, root)
                check(error is None, f'{ident}: {error}')
        dimensions = node.get('coverage', {})
        check(set(dimensions) == set(STATES), f'{ident}: missing/extra coverage dimension')
        for dimension, allowed in STATES.items():
            declarations = dimensions.get(dimension, {})
            if dimension == 'runtime':
                check(set(declarations) == ENVIRONMENTS, f'{ident}: runtime environments missing/invalid')
                entries = declarations.items()
            else:
                entries = [(dimension, declarations)]
            for name, entry in entries:
                state = entry.get('state')
                check(state in allowed, f'{ident}/{name}: state invalid: {state}')
                check(state == 'unknown' or bool(entry.get('refs')), f'{ident}/{name}: positive declaration without refs')
                for target in entry.get('refs', []):
                    error = link_error(target, root)
                    check(error is None, f'{ident}/{name}: {error}')
                if dimension == 'implementation':
                    check(entry.get('delivery') in DELIVERY, f'{ident}: implementation delivery invalid')
        gate = dimensions.get('gate', {})
        check(gate.get('id') in {f'K{x}' for x in range(8)}, f'{ident}: gate ID invalid')
        # A passed declaration must be explicit and sourced; never inferred from other columns.
        if gate.get('state') == 'passed':
            check(bool(gate.get('acceptance')), f'{ident}: passed gate needs explicit acceptance reference')
            if gate.get('acceptance'):
                error = link_error(gate['acceptance'], root)
                check(error is None, f'{ident}/acceptance: {error}')
    feature_ids = {n['id'] for n in nodes if n.get('kind') == 'feature'}
    check(set(rows) == feature_ids, 'RESEARCH gap IDs differ from catalog feature IDs')
    for relation in ['parent', 'depends_on']:
        visiting, visited = set(), set()
        def walk(ident):
            if ident in visiting:
                errors.append(f'{relation}: cycle at {ident}')
                return
            if ident in visited or ident not in by_id:
                return
            visiting.add(ident)
            value = by_id[ident].get(relation)
            for target in ([value] if relation == 'parent' and value else value or []):
                walk(target)
            visiting.remove(ident)
            visited.add(ident)
        for ident in ids:
            walk(ident)
    if previous:
        for old in previous['nodes']:
            new = by_id.get(old['id'])
            check(new is not None, f"stable ID removed: {old['id']}; retain as retired")
            if new:
                check(new.get('mechanism') == old['mechanism'] and new.get('kind') == old['kind'], f"stable ID repurposed: {old['id']}")
    return errors


def mdlink(target, label):
    if target.startswith('https://'):
        return f'[{label}]({target})'
    return f'[{label}]({target.removeprefix("docs/") if target.startswith("docs/") else "../" + target})'


def render(data, root=ROOT):
    nodes = data['nodes']
    features = [n for n in nodes if n['kind'] == 'feature']
    rows = research_rows(root)
    out = ['# Tra cứu chức năng KR00967', '',
           'Sinh từ [catalog](features.json) bằng `python3 tools/feature_catalog.py --generate`; không sửa tay.', '',
           'Catalog là chỉ mục khai báo có giới hạn theo nguồn. `present` chỉ xác nhận cơ chế hẹp hiện diện,',
           'không xác nhận mọi biến thể. `partial` là một phần đã đọc/viết; `checked` là emulation có oracle;',
           '`observed` là phạm vi runtime ở evidence. `unknown` không có nghĩa vắng mặt.',
           'Integration `partial` chỉ một phần đường chạy đã đo; luôn đọc giới hạn tại nguồn, kể cả một hàng gồm nhiều tác dụng.',
           'Không tính phần trăm hoặc suy gate từ các cột. Đồ thị thể hiện dependency của kịch bản, không phải thứ tự ưu tiên.', '',
           'Tra một hàng → thẻ cùng ID → contract/giới hạn, bằng chứng/hash, khoảng trống/phép kiểm.',
           'Thứ tự thực hiện chỉ ở [PLAN](PLAN.md#current-order); câu hỏi ở [RESEARCH](RESEARCH.md#feature-gaps);',
           'lịch sử/review/merge ở [manifest](evidence/checkpoints/manifest.json).',
           'Runtime tách Wine copy chẩn đoán / Windows nguyên bản / LAN / Internet; không suy tương đương giữa môi trường.',
           'Implementation `branch_pending` chờ tích hợp qua PR; `baseline_merged` nói về code đã có tại baseline.',
           'Review độc lập: [waived theo chủ dự án](evidence/checkpoints/review-waiver.json); Codex tự kiểm.', '',
           '## Coverage khai báo', '',
           '| ID / chức năng | Presence | VN ref | KR static | Emulation | Wine copy | Windows gốc | LAN | Internet | Implementation / delivery | Integration | Gate |',
           '|---|---|---|---|---|---|---|---|---|---|---|---|']
    for n in features:
        c = n['coverage']
        cells = [f"[{n['id']} · {n['title']}](#{n['id'].lower()})"]
        cells += [c[d]['state'] for d in ['presence', 'vn_reference', 'kr_static', 'emulation']]
        cells += [c['runtime'][e]['state'] for e in ['wine_diagnostic_copy', 'windows_original', 'lan', 'internet']]
        cells += [c['implementation']['state'] + ' / ' + c['implementation']['delivery'], c['integration']['state'], c['gate']['id'] + ': ' + c['gate']['state']]
        out.append('| ' + ' | '.join(cells) + ' |')
    out += ['', 'Số lượng theo trạng thái khai báo (không phải tỷ lệ hoàn thành):', '']
    for d in ['presence', 'kr_static', 'emulation', 'implementation', 'integration', 'gate']:
        counts = Counter(n['coverage'][d]['state'] for n in features)
        out.append(f"- {d}: " + ', '.join(f'{key}={value}' for key, value in sorted(counts.items())) + '.')
    for title, relation in [('Cây cơ chế', 'parent'), ('Dependency kịch bản', 'depends_on')]:
        out += ['', '## ' + title, '', '```mermaid', 'flowchart TD']
        for n in nodes:
            if relation == 'depends_on' and n['kind'] == 'group':
                continue
            out.append(f'  {n["id"].replace("-", "_")}["{n["id"]}: {n["title"]}"]')
        for n in nodes:
            targets = [n['parent']] if relation == 'parent' and n['parent'] else n.get('depends_on', []) if relation == 'depends_on' else []
            for dep in targets:
                out.append(f'  {dep.replace("-", "_")} --> {n["id"].replace("-", "_")}')
        out.append('```')
    out += ['', '## Tra theo cơ chế', '']
    for n in features:
        refs, c, q = n['refs'], n['coverage'], rows[n['id']]
        out += [f'<a id="{n["id"].lower()}"></a>', f'### {n["id"]} · {n["title"]}', '',
                'Dependency: ' + (', '.join(f'[{d}](#{d.lower()})' for d in n['depends_on']) or '—') + '.', '',
                mdlink(refs['contract'], 'Contract và giới hạn') + ' · ' + mdlink(refs['question'], 'Câu hỏi gốc') + ' · ' + mdlink(refs['plan'], 'Thứ tự ở PLAN') + '.', '',
                'Khoảng trống (trích bảng RESEARCH): ' + q['gap'], '',
                'Phép kiểm tối thiểu (trích cùng nguồn): ' + q['test'], '',
                'Bằng chứng/build/hash/recipe: ' + ', '.join(mdlink(r, r.split('/')[-2]) for r in refs['evidence']) + '.', '',
                '| Chiều chứng minh | Khai báo | Nguồn/phạm vi áp dụng |', '|---|---|---|']
        for dimension in STATES:
            entries = c[dimension].items() if dimension == 'runtime' else [(dimension, c[dimension])]
            for name, entry in entries:
                state = entry['state']
                if dimension == 'implementation':
                    state += ' / ' + entry['delivery']
                if dimension == 'gate':
                    state = entry['id'] + ': ' + state
                out.append('| ' + name + ' | ' + state + ' | ' + (', '.join(mdlink(r, 'nguồn' + str(i+1)) for i, r in enumerate(entry.get('refs', []))) or 'Chưa có bằng chứng trong phạm vi rà') + ' |')
        out.append('')
    return '\n'.join(out)


def previous_catalog(root=ROOT):
    result = subprocess.run(['git', '-c', 'core.fsmonitor=false', 'show', 'HEAD:' + CATALOG], cwd=root, capture_output=True, text=True)
    return json.loads(result.stdout) if result.returncode == 0 else None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--generate', action='store_true')
    args = parser.parse_args()
    data = load()
    previous = previous_catalog()
    errors = validate(data, previous=previous)
    if errors:
        raise SystemExit('\n'.join(errors))
    output = render(data)
    if args.generate:
        (ROOT / OUTPUT).write_text(output)
    elif not (ROOT / OUTPUT).is_file() or (ROOT / OUTPUT).read_text() != output:
        raise SystemExit('COVERAGE stale: run python3 tools/feature_catalog.py --generate')
    print(f'Feature index OK: {sum(n["kind"] == "feature" for n in data["nodes"])} features; links/anchors/DAG/declarations/generated view checked.')
    print('ID baseline: HEAD compared' if previous else 'ID baseline: first catalog, no previous IDs to compare')
    print('Declared pending implementation: ' + ', '.join(n['id'] for n in data['nodes'] if n.get('coverage', {}).get('implementation', {}).get('delivery') == 'branch_pending'))
    print('No semantic confirmation, external-link fetch, runtime certification or gate closure performed.')


if __name__ == '__main__':
    main()
