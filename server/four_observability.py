"""Local observability for the fixed four-role service; no wire or auth policy."""
from datetime import datetime, timezone
import copy
import hashlib
import json
from pathlib import Path
import time
import uuid


def encoded(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2)+'\n').encode()


def digest(value):
    return hashlib.sha256(encoded(value)).hexdigest()


class FourObservability:
    def __init__(self, service):
        self.family='four' if len(service.operator_identities)==4 else 'eight'
        self.run_id = uuid.uuid4().hex
        self.started_ns = time.monotonic_ns()
        self.sequence = 0
        self.transitions=[]
        self.transitions_total=0
        sources = sorted((dict(file=Path(q['path']).name, sha256=q['sha256'])
                          for q in service.state['policy_sources']),key=lambda q:(q['file'],q['sha256']))
        config=dict(source_ips=dict(service.source_ips), bind_ip=service.bind_ip,
            channel_endpoint=copy.deepcopy(service.channel_endpoint),
            bomb_policy=copy.deepcopy(service.state.get('four_bomb_policy',service.state.get('eight_bomb_policy'))),
            battle_outcome=service.battle_reports is not None,
            operator_team_layout=getattr(service,'team_layout',None),
            accepted_room_selectors=list(service.ROOM_SELECTORS),
            abort_room_policy=copy.deepcopy(service.state.get('abort_room_policy')),
            io_limits=dict(service.io_budget.limits))
        self.metadata = dict(schema='kre-'+self.family+'-run-v1', source='design', run_id=self.run_id,
            started_utc=datetime.now(timezone.utc).isoformat(), started_monotonic_ns=self.started_ns,
            target='KR00967_declared_not_remote_binary_attestation',
            source_build_sha256=digest(sources), policy_sources=sources,
            profile_sha256=digest(service.profile.data), config=config, config_sha256=digest(config),
            source_ips=dict(service.source_ips), identities=service.operator_identities, identity_store=service.identity_store,
            limits='Localdiagnosticcorrelation/health only; not accountauthentication, signedbuild, nativeclientproof or persistedmatch.')
        self.metadata_written = False

    def phase(self, service):
        if service.state.get('stopped'):
            return 'stopped'
        if service.closed:
            return 'stopping'
        if service.state.get('membership_broken') or service.state.get('errors'):
            return 'degraded'
        if service.room.active:
            return 'active'
        return 'ready' if service.state.get('ready') else 'starting'

    def stamp(self, service):
        self.sequence += 1
        run_id = self.run_id
        room_id = run_id+'/room1'
        round_id = room_id+'/round'+str(service.round['index']) if service.round['index'] else None
        for event in service.events:
            role = event['role']
            event.setdefault('correlation', dict(run_id=run_id,session_id=run_id+'/'+role,
                transport_id=run_id+'/tcp'+str(event['global_ordinal']),room_id=room_id))
        for key in ['round_bombs','bomb_requests','event_reports','policy_events']:
            for row in service.state.get(key,[]):
                index=row.get('round')
                context=dict(run_id=run_id,room_id=room_id,
                    round_id=room_id+'/round'+str(index) if type(index) is int and index>0 else None)
                role=row.get('role',row.get('source_role'))
                if role in service.source_ips:
                    context['session_id']=run_id+'/'+role
                row.setdefault('correlation',context)
        service.round['correlation'] = dict(run_id=run_id,room_id=room_id,round_id=round_id)
        phase=self.phase(service)
        if not self.transitions or self.transitions[-1]['phase']!=phase:
            self.transitions_total+=1
            self.transitions.append(dict(phase=phase,monotonic_ns=time.monotonic_ns()))
            del self.transitions[:-32]
        service.state['observability'] = dict(schema='kre-'+self.family+'-context-v1',source='design',
            run_id=run_id,room_id=room_id,round_id=round_id,snapshot_sequence=self.sequence,
            phase=phase,phase_transitions=self.transitions,phase_transitions_total=self.transitions_total,run_metadata_sha256=digest(self.metadata))
        return service.state['observability']

    def publish(self, service):
        if not self.metadata_written:
            # Refuse an old run directory before replacing any existing metadata.
            with (service.out/'run-metadata.json').open('xb') as stream:
                stream.write(encoded(self.metadata))
            self.metadata_written = True
        context=self.stamp(service)
        service.state['snapshot_monotonic_ns']=time.monotonic_ns()
        raw=encoded(service.state)
        self.atomic(service.out/'service-state.json',raw)
        health=dict(schema='kre-'+self.family+'-health-v1',source='design',**{k:context[k] for k in
            ['run_id','room_id','round_id','snapshot_sequence','phase','run_metadata_sha256']},
            snapshot_monotonic_ns=service.state['snapshot_monotonic_ns'],
            state_sha256=hashlib.sha256(raw).hexdigest(),
            source_build_sha256=self.metadata['source_build_sha256'],
            active_transports=0 if service.state.get('stopped') else sum(not e.get('closed') for e in service.events),
            accepted_transports=len(service.events),room_active=service.room.active,
            membership_broken=bool(service.state.get('membership_broken')),
            errors=len(service.state.get('errors',[])),
            limits='Atomicfiles individually; two-file reader must match snapshot_sequence/state_sha256. Localheartbeat is not an external liveness probe or gate proof.')
        self.atomic(service.out/'health.json',encoded(health))
        return health

    @staticmethod
    def atomic(path, raw):
        tmp=path.with_suffix('.tmp')
        tmp.write_bytes(raw)
        tmp.replace(path)
