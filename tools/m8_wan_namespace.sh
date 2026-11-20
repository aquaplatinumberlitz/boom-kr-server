#!/usr/bin/env bash
# Run a package acceptance controller in an isolated IPv4 WAN. No host routes,
# interfaces, named namespaces, or firewall rules are changed.
set -euo pipefail

if [[ ${1:-} != --inside ]]; then
    if (( $# < 4 )); then
        printf 'Usage: %s PUBLIC_TCP PRELOGIN_UDP RELAY_UDP COMMAND [ARG...]\n' "$0" >&2
        exit 2
    fi
    for port in "${@:1:3}"; do
        if [[ ! $port =~ ^[1-9][0-9]{0,4}$ ]] || (( port > 65535 )); then
            printf 'Invalid service port\n' >&2; exit 2
        fi
    done
    [[ $1 != "$2" && $1 != "$3" && $2 != "$3" ]]
    exec unshare --user --map-root-user --net --pid --fork --mount-proc --kill-child=TERM \
        bash "$0" --inside "$@"
fi
if [[ $$ != 1 || $(id -u) != 0 || $(awk 'NR==1 {print $3}' /proc/self/uid_map) != 1 ]]; then
    printf 'Requires the owned PID and user namespaces\n' >&2
    exit 2
fi
shift
public=$1 prelogin=$2 relay=$3
shift 3

# A PID pins each child namespace; descendants disappear when the owner exits.
unshare --net sleep infinity &
router=$!
unshare --net sleep infinity &
edge=$!
trap 'kill "$edge" "$router" 2>/dev/null || true; wait "$edge" "$router" 2>/dev/null || true' EXIT
parent_net=$(readlink /proc/self/ns/net)
for pid in "$router" "$edge"; do
    for _ in {1..100}; do
        net=$(readlink "/proc/$pid/ns/net")
        [[ $net == "$parent_net" ]] || break
        sleep .02
    done
    [[ $net != "$parent_net" ]]
done
in_router() { nsenter --net="/proc/$router/ns/net" -- "$@"; }
in_edge() { nsenter --net="/proc/$edge/ns/net" -- "$@"; }

ip link set lo up
ip link add wan0 type veth peer name server0 netns "$router"
in_router ip link add nat0 type veth peer name wan0 netns "$edge"
ip address add 10.88.2.2/30 dev wan0
in_router ip address add 10.88.2.1/30 dev server0
in_router ip address add 10.88.1.1/30 dev nat0
in_edge ip address add 10.88.1.2/30 dev wan0
ip link set wan0 up
in_router ip link set lo up
in_router ip link set server0 up
in_router ip link set nat0 up
in_edge ip link set lo up
in_edge ip link set wan0 up
in_edge ip route add 10.88.2.2/32 via 10.88.1.1
in_router sysctl -qw net.ipv4.ip_forward=1
sysctl -qw net.ipv6.conf.all.disable_ipv6=1
in_router sysctl -qw net.ipv6.conf.all.disable_ipv6=1
in_edge sysctl -qw net.ipv6.conf.all.disable_ipv6=1

# For the two exercised UDP destinations, disjoint source port pools make
# mapping destination dependent. Replies require an existing flow; the
# Windows client gets the first NAT from QEMU user networking on the edge.
in_router nft -f - <<EOF
table ip m8_wan {
    chain forward {
        type filter hook forward priority filter; policy drop;
        iifname "server0" oifname "nat0" ct state established,related counter accept
        iifname "nat0" oifname "server0" ip daddr 10.88.2.2 tcp dport $public counter accept
        iifname "nat0" oifname "server0" ip daddr 10.88.2.2 udp dport { $prelogin, $relay } counter accept
    }
    chain postrouting {
        type nat hook postrouting priority srcnat; policy accept;
        oifname "server0" tcp dport $public snat to 10.88.2.1:38000-38999
        oifname "server0" udp dport $prelogin snat to 10.88.2.1:40000-40999 random
        oifname "server0" udp dport $relay snat to 10.88.2.1:41000-41999 random
    }
}
EOF

# Each direction adds 75 ms and independently loses 3% of packets. Only the
# Windows path crosses these links; Wine and the service run in this namespace.
tc qdisc add dev wan0 root netem delay 75ms loss 3%
in_router tc qdisc add dev server0 root netem delay 75ms loss 3%

export M8_WAN_ROUTER_NS="/proc/$router/ns/net" M8_WAN_EDGE_NS="/proc/$edge/ns/net"
export M8_WAN_SERVER=10.88.2.2
"$@"
