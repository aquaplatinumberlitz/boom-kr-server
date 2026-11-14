"""Generate a bounded, review-only literal relocation candidate; no patching."""
import argparse
import hashlib
import json
from pathlib import Path
import struct

import pefile

DIAGNOSTIC_SHA256 = '4b781fbcea21e0f6294c729b63be86ab2953d60ad83b0d0c9219359f826107ed'


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def build_plan(raw, seeds, xrefs):
    if digest(raw) != DIAGNOSTIC_SHA256:
        raise ValueError('diagnostic image SHA mismatch')
    pe = pefile.PE(data=raw)
    base = pe.OPTIONAL_HEADER.ImageBase
    if base != 0x400000:
        raise ValueError('unexpected image base')
    mapping = {item['va']: item for item in xrefs['rows']}
    seed_vas = {item['va'] for item in seeds['rows']}
    if len(mapping) != len(xrefs['rows']) or len(seeds['rows']) != 134 or len(seed_vas) != 134 or seed_vas != mapping.keys():
        raise ValueError('unexpected or duplicate seed count')
    rows = []
    for seed in seeds['rows']:
        va = int(seed['va'], 16)
        start = pe.get_offset_from_rva(va - base)
        original = raw[start:start + seed['bytes']]
        if digest(original) != seed['cp949_sha256'] or raw[start + len(original):start + len(original) + 1] != b'\0':
            raise ValueError(f'literal bytes differ at {va:#x}')
        translated = original.decode('cp949', errors='strict').encode('utf-8')
        if len(translated) != seed['utf8_bytes']:
            raise ValueError(f'UTF-8 size differs at {va:#x}')
        refs = []
        for ref in mapping[seed['va']]['refs']:
            if ref['type'] != 1:
                raise ValueError(f'unexpected xref type at {ref["site"]}')
            site = int(ref['site'], 16)
            offset = pe.get_offset_from_rva(site - base)
            displacement = 0 if ref['function'] is None else 1
            pointer_offset = offset + displacement
            if raw[pointer_offset:pointer_offset + 4] != struct.pack('<I', va):
                raise ValueError(f'pointer bytes differ at {site:#x}')
            refs.append({'site_va': hex(site), 'pointer_file_offset': hex(pointer_offset),
                         'containing_function': ref['function']})
        rows.append({'literal_va': hex(va), 'literal_file_offset': hex(start),
                     'cp949_bytes': len(original), 'utf8_bytes': len(translated),
                     'utf8_hex': translated.hex(), 'references': refs})
    return {'schema': 'kre-l10n-literal-plan-v1', 'image_sha256': digest(raw),
            'source': 'design', 'status': 'candidate', 'rows': rows,
            'limits': 'Review only: IDA exact-start references and heuristic seed inventory may be incomplete. '
                      'No instruction validation, relocation, lifetime, ASLR, consumer or native proof. '
                      'Not a patch and not a complete list of CP949 literals.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--image', type=Path, required=True)
    parser.add_argument('--seeds', type=Path, required=True)
    parser.add_argument('--xrefs', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    local = (Path(__file__).resolve().parents[1] / 'local/research/l10n-utf8-codex').resolve()
    image, seeds, xrefs, out = (p.resolve() for p in (args.image, args.seeds, args.xrefs, args.out))
    if any(not p.is_relative_to(local) for p in (image, seeds, xrefs, out)):
        parser.error('all paths must be under the private research directory')
    if out.exists() or out in (image, seeds, xrefs) or not out.parent.is_dir():
        parser.error('output must be new and distinct')
    try:
        plan = build_plan(image.read_bytes(), json.loads(seeds.read_text()), json.loads(xrefs.read_text()))
        with out.open('x') as stream:
            json.dump(plan, stream, indent=2)
            stream.write('\n')
        print(f"{len(plan['rows'])} literal candidates; no bytes modified")
    except (OSError, ValueError, KeyError, pefile.PEFormatError) as error:
        parser.exit(1, f'{error}\n')


if __name__ == '__main__':
    main()
