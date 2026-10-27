#!/usr/bin/env python3
"""Rebuild pinned KR Wine experimental recipes from immutable local RE artifacts."""
import argparse
import hashlib
import json
from pathlib import Path
import struct

import pefile

BASE = 0x400000
SOURCE_SHA = 'a08950c833d3acab8281049def7b3dc96b221f107eeda01da3a548e34b91dce5'
HEADER_SHA = 'cf72da112308d9f1ca256df8089594c5f787875310933073bf6bd5ba6f70e63b'
CANDIDATES_SHA = 'e5a618a3d1f6c8b29a5895f5354519a7a904a65a02c44abfb82449c2796aaf74'
EXPECTED_SHA = 'a0c3b9867dae5fe2ff7c315cfe851117661c2cafead45196ce999329f67432f5'
POST_BOOTSTRAP_SHA = '7b3b435b5d8e110185359107376b736615ceabe9452cea8154d02c8628f44c29'
OVERRIDES = {
    0x16ea278: ('kernel32.dll', 'GetProcAddress'),
    0x16ea3a0: ('kernel32.dll', 'InitializeSListHead'),
    0x16ea3b8: ('kernel32.dll', 'WakeAllConditionVariable'),
    0x16ea3bc: ('kernel32.dll', 'AcquireSRWLockExclusive'),
    0x16ea3c0: ('kernel32.dll', 'ReleaseSRWLockExclusive'),
    0x16ea2c8: ('kernel32.dll', 'HeapSize'),
    0x16ea2d0: ('kernel32.dll', 'HeapReAlloc'),
    0x16ea2b0: ('kernel32.dll', 'InitializeCriticalSection'),
    0x16ea368: ('kernel32.dll', 'VerifyVersionInfoW'),
    0x16ea68c: ('user32.dll', 'MapVirtualKeyA'),
    0x16ea64c: ('user32.dll', 'MessageBoxA'),
    0x16ea678: ('user32.dll', 'MessageBoxW'),
    0x16ea5b4: ('user32.dll', 'CreateWindowExA'),
    0x16ea5a8: ('user32.dll', 'DispatchMessageA'),
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recipe', choices=['bootstrap-r1', 'post-bootstrap-r2'], default='bootstrap-r1',
                        help='Keep r1 unchanged; r2 is a separately verified offline experiment')
    parser.add_argument('--memory', type=Path, required=True, help='Original modO.bin memory image')
    parser.add_argument('--headers', type=Path, required=True, help='Original kr-full.exe analysis PE')
    parser.add_argument('--candidates', type=Path, required=True, help='Pinned private wine-imports-r4.json')
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    local = Path(__file__).resolve().parent.parent / 'local'
    inputs = [path.resolve(strict=True) for path in [args.memory,args.headers,args.candidates]]
    outputs = [path.resolve() for path in [args.out,args.report]]
    if len(set(outputs)) != 2 or any(path in inputs or path.exists() or not path.is_relative_to(local) for path in outputs):
        parser.error('output/report must be distinct new paths under this repo local/')
    if any(not path.parent.is_dir() for path in outputs):
        parser.error('output/report parent directories must exist')
    raw, header_data, candidate_data = [path.read_bytes() for path in inputs]
    for data, expected in zip([raw,header_data,candidate_data],[SOURCE_SHA,HEADER_SHA,CANDIDATES_SHA]):
        if sha(data) != expected:
            parser.error('input SHA-256 mismatch; no output written')
    original = pefile.PE(data=header_data, fast_load=True)
    span = original.OPTIONAL_HEADER.SizeOfImage
    image = bytearray(raw[:span])
    headers = bytearray(header_data[:0x1000])
    nt = struct.unpack_from('<I',headers,0x3c)[0]
    opt = nt+24
    section_table = opt+struct.unpack_from('<H',headers,nt+20)[0]
    struct.pack_into('<H',headers,nt+6,2)
    struct.pack_into('<H',headers,nt+22,0x0103)
    struct.pack_into('<I',headers,opt+16,0x1470e40-BASE)
    struct.pack_into('<I',headers,opt+60,0x1000)
    struct.pack_into('<H',headers,opt+70,0x0100)
    for index in range(16):
        if index not in [2,9]:
            struct.pack_into('<II',headers,opt+96+index*8,0,0)
    patches = []

    def patch(address, data, reason, source='kr_static'):
        offset = address-BASE
        before = bytes(image[offset:offset+len(data)])
        image[offset:offset+len(data)] = data
        patches.append({'va':hex(address),'memory_offset':hex(offset),'before':before.hex(),
                        'after':data.hex(),'reason':reason,'source':source})

    for address,length,reason in [
        (0x1a7d71c,4,'CRT startup guard; 1470E77'),
        (0x1a96e64,0x20,'3 lazy module handles and5 API caches; 1532A82'),
        (0x1a9733c,0x18,'Two onexit tables; 1570BC1 / 1570B58'),
        (0x1a7d725,1,'Onexit initialization flag; 146FE1A'),
        (0x1a7d454,4,'Classic locale alias; 1439A68'),
        (0x1a7d470,4,'Classic locale alias; 1439A61'),
        (0x1a7d47c,4,'Classic locale singleton; 1439A1F'),
        (0x19d8750,4,'Facet cache; 45B3E0'),
        (0x1a97dbc,1,'Recreate CRT locale state; 1589F4B'),
        (0x1a979c8,0x200,'128 lowio block pointers; 15865B3'),
        (0x1a97bc8,4,'Lowio descriptor count; 15865B3'),
        (0x1a96e94,4,'Stale FILE pointer table; 157C58C'),
        (0x1a97cb0,0x58,'22 lazy module handles; 1582073 / 1582A82'),
        (0x1a9558c,4,'Run WSAStartup in this process; 14F4FB0'),
    ]:
        patch(address,bytes(length),reason)
    patch(0x19a7858,bytes(20)+struct.pack('<I',15),'Restore empty SSO string before4382D0 assignment')
    patch(0x5725e0,b'\xb0\x01\xc3'+b'\x90'*12,'Omit NexonAnalytics DLL load offline','emu_choice')
    patch(0x573860,b'\x31\xc0\xc3','Analytics unavailable guard; retain572600 parser','emu_choice')
    patch(0x804f90,b'\x31\xc0\xc2\x0c\x00','Analytics property lookup unavailable; thiscall3 args','emu_choice')
    for address in [0x5725f0,0x572950,0x5729d0,0x5729f0,0x572a10,0x572a30]:
        patch(address,b'\x31\xc0\xc3','Omit analytics send/shutdown; cdecl caller cleanup','emu_choice')
    overrides = dict(OVERRIDES)
    if args.recipe == 'post-bootstrap-r2':
        if bytes(image[0x12358e7-BASE:0x12358ec-BASE]) != bytes.fromhex('e8e4f330ff'):
            raise RuntimeError('NGClient callsite differs from the recorded source')
        patch(0x12358e7, b'\x90'*5, 'Omit ignored NGClient init call544CD0; keep545E40 object init', 'emu_choice')
        overrides[0x16ea160] = ('kernel32.dll', 'lstrcpyA')
        overrides[0x16ea664] = ('user32.dll', 'SetWindowsHookExA')
        overrides[0x16ea6c4] = ('user32.dll', 'LoadIconA')
        overrides[0x16ea5ec] = ('user32.dll', 'LoadCursorA')
        overrides[0x16ea5ac] = ('user32.dll', 'RegisterClassExA')
        overrides[0x16ea630] = ('user32.dll', 'AdjustWindowRect')
        overrides[0x16ea5c0] = ('user32.dll', 'DefWindowProcA')
    rows = json.loads(candidate_data)['rows']
    extra_rva = (span+0xfff)&~0xfff
    extra = bytearray((len(rows)+1)*20)
    imports, traps, stubs = [], [], []
    blocked = {'ngclient.aes','cawh.sys','dwk.dll','nxoverlay.dll','nmco3.dll','crashreporter.dll'}

    def alloc(data, alignment=2):
        while len(extra)%alignment:
            extra.append(0)
        address = extra_rva+len(extra)
        extra.extend(data)
        return address

    for row in rows:
        slot = int(row['slot'],16)
        if slot in [0x16ea03c,0x16ea040,0x16ea044,0x16ea048]:
            count = int(row['names'][0][1].rsplit('@',1)[1])
            address = alloc(b'\x31\xc0\xc2'+struct.pack('<H',count),4)
            struct.pack_into('<I',image,slot-BASE,BASE+address)
            stubs.append({'slot':hex(slot),'va':hex(BASE+address),'ret_bytes':count,
                          'source':'emu_choice','reason':'Omit offline CrashReporter'})
            continue
        choices = [overrides[slot]] if slot in overrides else row.get('names',[])
        choices = sorted(choices,key=lambda x:(x[0] in blocked,x[0] not in ['kernel32.dll','user32.dll','gdi32.dll','advapi32.dll'],x[1].startswith('#'),len(x[1])))
        if choices and choices[0][0] not in blocked:
            dll,name = choices[0]
            dll_rva = alloc(dll.encode()+b'\0')
            thunk = 0x80000000|int(name[1:]) if name.startswith('#') else alloc(b'\0\0'+name.encode()+b'\0')
            lookup = alloc(struct.pack('<II',thunk,0),4)
            struct.pack_into('<IIIII',extra,len(imports)*20,lookup,0,0,dll_rva,slot-BASE)
            # Discard bound addresses from the old Windows process; the loader uses ILT.
            struct.pack_into('<I',image,slot-BASE,0)
            imports.append({'slot':hex(slot),'dll':dll,'name':name,
                            'status':'candidate','override':slot in overrides})
        else:
            address = alloc(b'\x0f\x0b'+struct.pack('<I',slot)+b'\xcc\xcc',4)
            struct.pack_into('<I',image,slot-BASE,BASE+address)
            traps.append({'slot':hex(slot),'va':hex(BASE+address),'excluded':choices})
    struct.pack_into('<II',headers,opt+104,extra_rva,(len(imports)+1)*20)
    extra_size = (len(extra)+0x1ff)&~0x1ff
    struct.pack_into('<I',headers,opt+56,(extra_rva+extra_size+0xfff)&~0xfff)
    headers[section_table:section_table+40*7]=bytes(40*7)
    struct.pack_into('<8sIIIIIIHHI',headers,section_table,b'.image',span-0x1000,0x1000,span-0x1000,0x1000,0,0,0,0,0xe0000060)
    struct.pack_into('<8sIIIIIIHHI',headers,section_table+40,b'.repair',len(extra),extra_rva,extra_size,span,0,0,0,0,0xe0000060)
    result = bytes(headers)+bytes(image[0x1000:])+bytes(extra).ljust(extra_size,b'\0')
    expected_sha = EXPECTED_SHA if args.recipe == 'bootstrap-r1' else POST_BOOTSTRAP_SHA
    if sha(result) != expected_sha:
        raise RuntimeError(f'rebuild differs from pinned {args.recipe}; computed SHA {sha(result)}; no output written')
    report = {'schema':'kre-wine-copy-r1' if args.recipe == 'bootstrap-r1' else 'kre-wine-copy-r2','output_sha256':sha(result),'entry_va':'0x1470e40', 'recipe':args.recipe,
              'inputs':[{'path':str(path),'sha256':sha(data)} for path,data in zip(inputs,[raw,header_data,candidate_data])],
              'patches':patches,'imports':imports,'traps':traps,'stubs':stubs,
              'limitations':['Partial experimental reconstruction; imports outside exercised path remain candidates.',
                             'Analytics/CrashReporter omitted; identity from analytics lookup unavailable.',
                             'Login/lobby/gameplay and vanilla equivalence unverified.']}
    if args.recipe == 'post-bootstrap-r2':
        report['limitations'].append('NGClient init omitted offline; callbacks/state unavailable; original-client equivalence unknown.')
    # Exclusive creation preserves prior experiments. No client is executed by this tool.
    with outputs[1].open('x') as handle:
        json.dump(report,handle,indent=2)
        handle.write('\n')
    with outputs[0].open('xb') as handle:
        handle.write(result)
    print(json.dumps({'sha256':sha(result),'imports':len(imports),'traps':len(traps),'stubs':len(stubs)}))


if __name__ == '__main__':
    main()
