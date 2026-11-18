"""Check Rust's 949x extension against the owner's Python reference.

Optionally write an independent exhaustive Unicode→949x oracle under local/.
No native runtime, client patch, or L1 source modification is performed.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--oracle-out', type=Path)
    args = parser.parse_args()
    reference = ROOT / 'tools/l10n_949x.py'
    spec = importlib.util.spec_from_file_location('owner_949x_reference', reference)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    table = ROOT / 'rust/crates/kr-wire/src/kr00967/text949x_table.rs'
    characters = [chr(int(value, 16)) for value in re.findall(r"'\\u\{([0-9a-f]+)\}'", table.read_text())]
    if module.VERSION != 1 or characters != module.EXT:
        raise SystemExit('Rust table differs from owner 949x VERSION=1 reference')
    report = dict(schema='kre-text949x-reference-v1', extension_entries=len(characters),
                  reference_sha256=hashlib.sha256(reference.read_bytes()).hexdigest(),
                  table_sha256=hashlib.sha256(table.read_bytes()).hexdigest())
    if args.oracle_out:
        output = args.oracle_out.resolve()
        local = (ROOT / 'local').resolve(strict=True)
        if not output.is_relative_to(local) or output == local or output.exists():
            parser.error('Oracle output must be new and under local/')
        data = bytearray()
        for codepoint in range(0x110000):
            if 0xd800 <= codepoint <= 0xdfff:
                continue
            try:
                encoded = module.encode(chr(codepoint))
            except UnicodeEncodeError:
                continue
            assert len(encoded) in (1, 2)
            data.extend(codepoint.to_bytes(4, 'little') + encoded.ljust(2, b'\0'))
        with output.open('xb') as stream:
            stream.write(data)
        report.update(oracle_rows=len(data) // 6, oracle_sha256=hashlib.sha256(data).hexdigest())
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
