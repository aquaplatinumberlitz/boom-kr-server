"""Generate or verify the Rust table; optionally emit an exhaustive Python codec oracle."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
TABLE = ROOT / 'rust/crates/kr-wire/src/kr00967/text949x_table.rs'


def render_table(extension, version, digest):
    if version != 1:
        raise ValueError('Review the wire version before changing the Rust 949x codec')
    lines = [
        '// Generated from tools/l10n_949x.py VERSION=1; extension order is wire ABI.',
        f'// Reference sha256: {digest}',
        '// Regenerate with python3 tools/text949x_reference.py --generate.',
        'pub(super) const EXT: &[char] = &[',
    ]
    line = '    '
    for ch in extension:
        token = f"'\\u{{{ord(ch):x}}}',"
        sep = '' if line.endswith(' ') else ' '
        if len(line) + len(sep) + len(token) > 100:
            lines.append(line)
            line, sep = '    ', ''
        line += sep + token
    if line.strip():
        lines.append(line)
    return '\n'.join([*lines, '];', ''])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--oracle-out', type=Path)
    parser.add_argument('--generate', action='store_true', help='Regenerate the checked-in Rust table')
    args = parser.parse_args()
    reference = ROOT / 'tools/l10n_949x.py'
    spec = importlib.util.spec_from_file_location('owner_949x_reference', reference)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    digest = hashlib.sha256(reference.read_bytes()).hexdigest()
    contents = TABLE.read_text()
    characters = [chr(int(value, 16)) for value in re.findall(r"'\\u\{([0-9a-f]+)\}'", contents)]
    if characters != module.EXT[:len(characters)] or not characters:
        raise SystemExit('Existing Rust 949x entries would change wire bytes')
    expected = render_table(module.EXT, module.VERSION, digest)
    if args.generate:
        TABLE.write_text(expected)
    elif contents != expected:
        raise SystemExit('Rust table differs from reference; run text949x_reference.py --generate')
    report = dict(schema='kre-text949x-reference-v1', extension_entries=len(module.EXT),
                  reference_sha256=digest, table_sha256=hashlib.sha256(TABLE.read_bytes()).hexdigest())
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
