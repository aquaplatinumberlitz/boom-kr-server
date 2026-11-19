"""Dựng payload người chơi từ input đã pin, không sửa pack hoặc output L1 nguồn.

Recipe local khai base, pack_stages theo thứ tự nối và ba languages. Không tự
đổi offset .idx: một stage mới phải chứa nguyên byte pack stage trước ở đầu.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil

from content_census import decrypt_idx, parse_idx

ROOT = Path(__file__).resolve().parents[1]
LANGUAGES = ('vi', 'en', 'ko')
LOCAL_EXTENSIONS = {'.ini', '.ssd', '.fft', '.bik'}


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def relative(value):
    path = PurePosixPath(value)
    if (not value or path.is_absolute() or '\\' in value or ':' in value
            or any(part in {'', '.', '..'} for part in value.split('/'))):
        raise ValueError('Đường dẫn payload không hợp lệ')
    return path


def pin(source):
    source = Path(source['path']).resolve(strict=True)
    if not source.is_file():
        raise ValueError('Input phải là file thường')
    return source


def checked_pin(row):
    path = pin(row)
    if path.stat().st_size != row['bytes'] or digest(path) != row['sha256']:
        raise ValueError('Input không khớp pin')
    return path


def prefix_equal(prefix, candidate):
    with prefix.open('rb') as left, candidate.open('rb') as right:
        while block := left.read(1024 * 1024):
            if right.read(len(block)) != block:
                raise ValueError('Pack stage mới không giữ nguyên prefix trước; không được tái dùng idx')


def build(recipe, out):
    if set(recipe['languages']) != set(LANGUAGES):
        raise ValueError('Gói phải khai đủ vi/en/ko đã vá')
    base = Path(recipe['base']).resolve(strict=True)
    files = {p.relative_to(base).as_posix(): p for p in base.rglob('*') if p.is_file()}
    if len({name.casefold() for name in files}) != len(files):
        raise ValueError('Input có tên trùng trên Windows')
    out = Path(out)
    out.mkdir(mode=0o700, parents=False, exist_ok=False)
    payload = {}

    def save(source, destination, offset=0):
        relative(destination)
        source = Path(source)
        target = out / destination
        target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        with source.open('rb') as src, target.open('xb') as dst:
            src.seek(offset)
            shutil.copyfileobj(src, dst, length=1024 * 1024)
        row = dict(path=destination, bytes=target.stat().st_size, sha256=digest(target))
        payload[destination] = row
        return row

    packs = []
    originals = {name: p for name, p in files.items() if p.suffix.lower() == '.idd'}
    if not originals:
        raise ValueError('Không tìm thấy pack gốc')
    stages = recipe.get('pack_stages', [])
    if [stage['language'] for stage in stages] not in [[], ['vi'], ['vi', 'en']]:
        raise ValueError('Thứ tự pack phải là KR → vi → en')
    stage_roots = {stage['language']: Path(stage['root']).resolve(strict=True) for stage in stages}
    for name, original in sorted(originals.items()):
        current = original
        parts = [save(original, 'payload/base/' + name)]
        for stage in stages:
            candidate = stage_roots[stage['language']] / name
            if not candidate.exists():
                continue
            prefix_equal(current, candidate)
            if candidate.stat().st_size > current.stat().st_size:
                parts.append(save(candidate, 'payload/tails/' + stage['language'] + '/' + name + '.tail',
                                  current.stat().st_size))
            current = candidate
        packs.append(dict(path=name, parts=parts, bytes=current.stat().st_size, sha256=digest(current)))

    common, localized = [], {}
    for name, source in sorted(files.items()):
        relative(name)
        if source.suffix.lower() in {'.idd', '.idx'} or name.casefold() == 'ca.exe':
            continue
        if source.suffix.lower() in LOCAL_EXTENSIONS:
            localized[name] = source
        else:
            common.append(dict(path=name, payload=save(source, 'payload/common/' + name)))
    languages = {}
    for language in LANGUAGES:
        entry = recipe['languages'][language]
        if entry.get('encoding') != '949x':
            raise ValueError('Mọi client, kể cả ko, phải là bản vá 949x')
        executable = checked_pin(entry['executable'])
        overrides = dict(localized)
        for root_name in ['data', 'fonts']:
            asset_root = Path(entry[root_name]).resolve(strict=True)
            for source in asset_root.rglob('*'):
                if source.is_file() and source.suffix.lower() in LOCAL_EXTENSIONS:
                    overrides[source.relative_to(asset_root).as_posix()] = source
        overrides['nm.bik'] = checked_pin(entry['video'])
        overrides['CA.exe'] = executable
        index_stage = entry['index_stage']
        if index_stage != 'base' and index_stage not in stage_roots:
            raise ValueError('Index stage chưa có pack')
        for pack in packs:
            name = str(PurePosixPath(pack['path']).with_suffix('.idx'))
            source = base / name
            # A stage may replace only some packs; inherit earlier indices.
            if index_stage != 'base':
                for stage in stages:
                    candidate = stage_roots[stage['language']] / name
                    if candidate.exists():
                        source = candidate
                    if stage['language'] == index_stage:
                        break
            for _, offset, size, _ in parse_idx(decrypt_idx(source.read_bytes())):
                if offset + size > pack['bytes']:
                    raise ValueError('Idx trỏ ra ngoài pack chung')
            overrides[name] = source
        rows = []
        for name, source in sorted(overrides.items()):
            relative(name)
            rows.append(dict(path=name, payload=save(source, 'payload/languages/' + language + '/' + name)))
        languages[language] = dict(encoding='949x', images=entry['images'], files=rows)
    manifest = dict(schema='kre-player-package-v1', packs=packs, common=common, languages=languages,
                    limits='Payload được dựng và kiểm hash; chưa chứng nhận cài sạch, client song song hoặc native Internet.')
    path = out / 'manifest.json'
    path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    return dict(manifest_sha256=digest(path), payload_bytes=sum(row['bytes'] for row in payload.values()),
                languages=list(languages), packs=len(packs))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recipe', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    local = (ROOT / 'local').resolve(strict=True)
    output = args.out.resolve()
    if output == local or not output.is_relative_to(local):
        parser.error('Gói và tài sản client phải ở local/')
    print(json.dumps(build(json.loads(args.recipe.read_text()), output)))


if __name__ == '__main__':
    main()
