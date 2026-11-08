"""Candidate immutable declaration of the four existing operator identities.

This is a consistency pin across server process restarts, never authentication,
account resume, admission authorization, or persistence of a match.
"""
import hashlib
import json
import os
from pathlib import Path
import uuid


def identity_bytes(members):
    value=dict(schema='kre-four-identities-v1',source='design',members={
        role:{k:member[k] for k in ['identity','slot','name','team']}
        for role,member in members.items()})
    return (json.dumps(value,ensure_ascii=False,sort_keys=True,indent=2)+'\n').encode()


def pin_identities(path,members):
    path=Path(path);expected=identity_bytes(members)
    if path.exists():
        raw=read_bounded(path)
        if raw!=expected:
            raise ValueError('identity store differs from the fixed four-role profile')
        created=False
    else:
        # Publish with an exclusive hard link: readers see complete bytes and a
        # concurrent winner cannot be overwritten. Temporary file shares parent.
        temporary=path.with_name('.'+path.name+'.'+uuid.uuid4().hex+'.tmp')
        try:
            with temporary.open('xb') as stream:
                stream.write(expected);stream.flush();os.fsync(stream.fileno())
            try:
                os.link(temporary,path);created=True
            except FileExistsError:
                created=False
            raw=read_bounded(path)
            if raw!=expected:
                raise ValueError('concurrent identity store conflicts with fixed profile')
        finally:
            temporary.unlink(missing_ok=True)
    return dict(path=str(path.resolve()),sha256=hashlib.sha256(raw).hexdigest(),
                created=created,source='design',scope='Fixedoperatoridentitypin only; noaccountauth/resume/matchstate.')


def read_bounded(path):
    with path.open('rb') as stream:
        raw=stream.read(16385)
    if len(raw)>16384:
        raise ValueError('identity store exceeds 16KiB')
    return raw
