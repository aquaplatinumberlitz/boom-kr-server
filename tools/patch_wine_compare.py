#!/usr/bin/env python3
"""Correct one measured reconstructed import on a separately named offline copy."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INPUT_SHA = 'e9bb476652a5f428c3bbf2f4e20587bc8eeffca5d6bee7d2d58f7823cf089cef'
OUTPUT_SHA = '4b781fbcea21e0f6294c729b63be86ab2953d60ad83b0d0c9219359f826107ed'
SLOT = 0x16EA1CC


def transform(raw):
    """Require exact measured input, import slot/name and resulting artifact."""
    if hashlib.sha256(raw).hexdigest() != INPUT_SHA:
        raise ValueError('input SHA mismatch; requires the measured e9 Wine copy')
    import pefile
    pe = pefile.PE(data=raw)
    matches = [(dll, entry) for dll in pe.DIRECTORY_ENTRY_IMPORT
               for entry in dll.imports if entry.address == SLOT]
    if len(matches) != 1:
        raise ValueError('measured import slot is missing or ambiguous')
    dll, entry = matches[0]
    if dll.dll != b'kernelbase.dll' or entry.name != b'CompareStringA':
        raise ValueError('measured import target differs')
    offset = entry.name_offset
    before = b'CompareStringA\0'
    after = b'lstrcmpiA\0' + bytes(5)
    if raw[offset:offset + 15] != before:
        raise ValueError('import name bytes differ')
    result = raw[:offset] + after + raw[offset + 15:]
    if hashlib.sha256(result).hexdigest() != OUTPUT_SHA:
        raise ValueError('output SHA mismatch; no output written')
    return result, dict(import_slot=hex(SLOT), file_offset=offset,
                        before=before.hex(), after=after.hex())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    try:
        source = args.input.resolve(strict=True)
        output, report = args.out.resolve(), args.report.resolve()
        local = (ROOT / 'local').resolve()
        if any(not p.is_relative_to(local) for p in [source, output, report]):
            raise ValueError('input/output/report must be private copies under this repo local/')
        if len({source, output, report}) != 3:
            raise ValueError('paths must be distinct')
        if any(p.exists() or not p.parent.is_dir() for p in [output, report]):
            raise ValueError('outputs must be new files with existing parent directories')
        raw, delta = transform(source.read_bytes())
        metadata = dict(schema='kre-wine-compare-copy-v1', source='emu_choice',
                        status='confirmed', input=dict(path=str(source), sha256=INPUT_SHA),
                        output=dict(path=str(output), bytes=len(raw), sha256=OUTPUT_SHA),
                        delta=delta,
                        evidence='docs/PROTOCOL.md#own-profile-result-compare',
                        rollback='Use the unchanged input copy; no source binary was overwritten.',
                        limits='One two-string thunk ABI correction on the measured offline Wine copy. '
                               'Original thunk and lstrcmpiA agree on observed CompareStringA flags '
                               'and return adaptation; arbitrary OS error/collation behavior is not certified. '
                               'No original Windows, account or gameplay gate inferred.')
        # Exclusive creation also refuses races with existing outputs.
        with output.open('xb') as stream:
            stream.write(raw)
        with report.open('x') as stream:
            stream.write(json.dumps(metadata, indent=2) + '\n')
        print(json.dumps(metadata))
    except (OSError, ValueError) as error:
        parser.exit(1, str(error) + '\n')


if __name__ == '__main__':
    main()
