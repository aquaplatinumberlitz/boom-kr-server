"""Explicit loopback profile-consumer fixture, never backend authentication.

Generate a private derivative of the existing capture source. The default
collector remains byte-identical; a pinned reply is sent only after a complete
first opcode1 frame. No production service or client input is changed.
"""
import base64
import hashlib
import json
from pathlib import Path
import struct


def load_reply(declaration, root):
    root = Path(root).resolve()
    local = (root / 'local').resolve(strict=True)
    path = Path(declaration).resolve(strict=True)
    if not path.is_relative_to(local):
        raise ValueError('Profile declaration must stay local')
    row = json.loads(path.read_text())['fixture']
    return _load_frame(row, root, 4, 43)


def _load_frame(row, root, opcode, sequence):
    root = Path(root).resolve()
    local = (root / 'local').resolve(strict=True)
    relative = Path(row['path'])
    if relative.is_absolute() or any(p in {'.', '..'} for p in relative.parts):
        raise ValueError('Profile fixture needs a canonical relative path')
    fixture = (root / relative).resolve(strict=True)
    if not fixture.is_relative_to(local) or not fixture.is_file():
        raise ValueError('Profile fixture must stay local')
    if not 38 <= fixture.stat().st_size <= 65536:
        raise ValueError('Profile fixture size bound')
    raw = fixture.read_bytes()
    if (len(raw) != row['bytes'] or hashlib.sha256(raw).hexdigest() != row['sha256']
            or row['sequence'] != sequence or struct.unpack_from('<I', raw, 5)[0] != opcode
            or struct.unpack_from('<I', raw, 13)[0] + 38 != len(raw)):
        raise ValueError('Profile fixture identity/framing mismatch')
    return raw, dict(path=str(relative), bytes=len(raw), sha256=row['sha256'], sequence=sequence)


def load_continuations(declaration, root):
    rows = json.loads(Path(declaration).read_text()).get('continuations')
    if rows is None:
        return None, None
    required = {'netinfo', 'flags', 'redirect'}
    if set(rows) not in (required, required | {'flags2'}):
        raise ValueError('Diagnostic continuation set must be complete')
    frames, pins = {}, {}
    specifications = [('netinfo', 15, 46), ('flags', 62, 49)]
    if 'flags2' in rows:
        specifications.append(('flags2', 62, 52))
    specifications.append(('redirect', 7, 55 if 'flags2' in rows else 52))
    for name, opcode, sequence in specifications:
        frames[name], pins[name] = _load_frame(rows[name], root, opcode, sequence)
    return frames, pins


def capture_source(source, reply, flow=None):
    encoded = base64.b64encode(reply).decode('ascii')
    substitutions = [
        ('private readonly byte[] hello, wcp;',
         'private readonly byte[] hello, wcp;\n'
         '    private readonly byte[] diagnosticProfile = Convert.FromBase64String("' + encoded + '");'),
        ('int port = ((IPEndPoint)client.Client.LocalEndPoint).Port;',
         'int port = ((IPEndPoint)client.Client.LocalEndPoint).Port;\n'
         '        byte[] firstRequest = new byte[65536]; int firstCount = 0; bool profileSent = false;'),
        ('int copied = Math.Min(9-filled,count);', '''if (!profileSent && firstCount < firstRequest.Length) {
                    int retained = Math.Min(count, firstRequest.Length-firstCount);
                    Array.Copy(buffer,0,firstRequest,firstCount,retained); firstCount += retained;
                    if (firstCount >= 17) {
                        uint firstOpcode = BitConverter.ToUInt32(firstRequest,5);
                        uint bodySize = BitConverter.ToUInt32(firstRequest,13);
                        if (bodySize > 65536-38) throw new IOException("diagnostic frame bound");
                        if (firstOpcode == 1 && firstCount >= (int)bodySize+38) {
                            stream.Write(diagnosticProfile,0,diagnosticProfile.Length);
                            profileSent = true; Array.Clear(firstRequest,0,firstRequest.Length);
                            Log("{\\"event\\":\\"diagnostic_profile_reply_sent\\",\\"connection\\":"+id+",\\"not_backend_authentication\\":true}");
                        }
                    }
                }
                int copied = Math.Min(9-filled,count);'''),
        ('finally { client.Close(); Log(',
         'finally { Array.Clear(firstRequest,0,firstRequest.Length); client.Close(); Log('),
        ('"scope\\\":\\\"loopback_only_no_authentication',
         '"scope\\\":\\\"loopback_only_diagnostic_profile_not_backend_authentication'),
    ]
    for old, new in substitutions:
        if source.count(old) != 1:
            raise ValueError('Profile capture source seam changed')
        source = source.replace(old, new)
    if flow is not None:
        constants = '\n'.join(
            '    private readonly byte[] diagnostic' + name.title() +
            ' = Convert.FromBase64String("' + base64.b64encode(flow[name]).decode('ascii') + '");'
            for name in ['netinfo', 'flags', *(['flags2'] if 'flags2' in flow else []), 'redirect'])
        source = source.replace('private readonly byte[] hello, wcp;',
                                'private readonly byte[] hello, wcp;\n' + constants)
        anchor = 'byte[] firstRequest = new byte[65536]; int firstCount = 0; bool profileSent = false;'
        source = source.replace(anchor, anchor + '\n'
            '        byte[] flowBuffer = new byte[65536]; int flowCount = 0, flowFrames = 0;\n'
            '        bool netinfoSent = false, redirectSent = false;')
        anchor = 'int copied = Math.Min(9-filled,count);'
        source = source.replace(anchor, '''if (flowCount+count > flowBuffer.Length) throw new IOException("diagnostic flow bound");
                Array.Copy(buffer,0,flowBuffer,flowCount,count); flowCount += count;
                while (flowCount >= 38) {
                    uint size = BitConverter.ToUInt32(flowBuffer,13);
                    if (size > 65536-38) throw new IOException("diagnostic flow frame bound");
                    int frameSize = (int)size+38;
                    if (flowCount < frameSize) break;
                    if (++flowFrames > 64) throw new IOException("diagnostic flow packet bound");
                    uint opcode = BitConverter.ToUInt32(flowBuffer,5);
                    if (profileSent && !netinfoSent && opcode == 9 && size == 8) {
                        stream.Write(diagnosticNetinfo,0,diagnosticNetinfo.Length);
                        stream.Write(diagnosticFlags,0,diagnosticFlags.Length); netinfoSent = true;
                        Log("{\\"event\\":\\"diagnostic_netinfo_sent\\",\\"connection\\":"+id+"}");
                    } else if (netinfoSent && !redirectSent && opcode == 5 && size == 9) {
                        stream.Write(diagnosticRedirect,0,diagnosticRedirect.Length); redirectSent = true;
                        Log("{\\"event\\":\\"diagnostic_redirect_sent\\",\\"connection\\":"+id+"}");
                    }
                    Array.Copy(flowBuffer,frameSize,flowBuffer,0,flowCount-frameSize);
                    Array.Clear(flowBuffer,flowCount-frameSize,frameSize); flowCount -= frameSize;
                }
                int copied = Math.Min(9-filled,count);''')
        source = source.replace('finally { Array.Clear(firstRequest,0,firstRequest.Length);',
                                'finally { Array.Clear(flowBuffer,0,flowBuffer.Length); Array.Clear(firstRequest,0,firstRequest.Length);')
        if 'flags2' in flow:
            source = source.replace(
                'stream.Write(diagnosticFlags,0,diagnosticFlags.Length); netinfoSent = true;',
                'stream.Write(diagnosticFlags,0,diagnosticFlags.Length);\n'
                '                        stream.Write(diagnosticFlags2,0,diagnosticFlags2.Length); netinfoSent = true;')
    return source
