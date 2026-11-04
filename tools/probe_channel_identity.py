"""Identify a fixture-service connection for a local measurement proxy.

This reads host-local service metadata, not an authenticated client identity or
wire protocol. A WCP connection can share the channel listener's port.
"""


def find_fixture_connection(state, upstream_address, listen_port):
    matches = [event for event in state.get('tcp_events', [])
               if event.get('peer') == list(upstream_address)
               and event.get('port') == listen_port and not event.get('closed')]
    if not matches:
        return None
    if len(matches) != 1:
        raise ValueError('ambiguous live fixture upstream connection')
    event = matches[0]
    if (event.get('role') not in ('A', 'B', 'C')
            or event.get('reply_ordinal') not in (1, 2, 3, 4)):
        raise ValueError('unsupported fixture connection identity')
    return {name: event[name] for name in
            ('global_ordinal', 'role', 'reply_ordinal', 'peer', 'port', 'accept_monotonic_ns')}
