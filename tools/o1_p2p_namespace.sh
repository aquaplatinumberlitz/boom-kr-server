#!/usr/bin/env bash
# Two peers behind separate IPv4 NATs, with a shared isolated WAN and iroh relay.
set -euo pipefail

if [[ ${1:-} != --inside ]]; then
    if [[ $# != 3 || ! $1 =~ ^(direct|relay)$ || ! -x $2 || ! -d $3 ]]; then
        printf 'Usage: %s direct|relay UDP_LAB_BINARY EMPTY_OUTPUT_DIR\n' "$0" >&2
        exit 2
    fi
    output=$(realpath "$3")
    [[ -z $(ls -A -- "$output") ]] || { printf 'Output directory must be empty\n' >&2; exit 2; }
    exec unshare --user --map-root-user --net --pid --fork --mount-proc --kill-child=TERM \
        bash "$0" --inside "$1" "$(realpath "$2")" "$output"
fi
if [[ $# != 4 || $$ != 1 || $(id -u) != 0 || $(awk 'NR==1 {print $3}' /proc/self/uid_map) != 1 ]]; then
    printf 'Requires owned PID and user namespaces\n' >&2
    exit 2
fi
mode=$2 bin=$3 output=$4
root_net=$(readlink /proc/self/ns/net)
new_net() {
    local -n result=$1
    unshare --net sleep infinity &
    result=$!
    local net
    for _ in {1..100}; do
        net=$(readlink "/proc/$result/ns/net")
        if [[ $net != "$root_net" ]]; then return; fi
        sleep .02
    done
    printf 'Child network namespace did not start\n' >&2
    return 1
}
net() {
    local pid=$1
    shift
    nsenter --net="/proc/$pid/ns/net" -- "$@"
}

# All processes, interfaces, routes and rules live inside the disposable PID tree.
ip link set lo up
ip link add wan type bridge
ip address add 198.18.0.1/24 dev wan
ip link set wan up

for side in 2 3; do
    new_net "router$side"
    new_net "peer$side"
    router_ref=router$side peer_ref=peer$side
    router=${!router_ref} peer=${!peer_ref}
    ip link add outside"$side" type veth peer name wan0 netns "$router"
    ip link set outside"$side" master wan
    ip link set outside"$side" up
    net "$router" ip link add lan0 type veth peer name eth0 netns "$peer"
    net "$router" ip link set lo up
    net "$router" ip address add "198.18.0.$side/24" dev wan0
    net "$router" ip address add "10.92.$side.1/24" dev lan0
    net "$router" ip link set wan0 up
    net "$router" ip link set lan0 up
    net "$router" sysctl -q -w net.ipv4.ip_forward=1
    net "$peer" ip link set lo up
    net "$peer" ip address add "10.92.$side.2/24" dev eth0
    net "$peer" ip link set eth0 up
    net "$peer" ip route add default via "10.92.$side.1"
    net "$router" tc qdisc add dev wan0 root netem delay 20ms 3ms loss 1%

    # SNAT changes the peer-bound source port only in the symmetric profile.
    # Stateful reverse translation requires the original destination tuple.
    peer_port=41600
    [[ $mode != relay ]] || peer_port=42600
    net "$router" nft -f - <<RULES
table ip p2p {
    chain forward {
        type filter hook forward priority filter; policy drop;
        ct state established,related accept
        iifname "lan0" oifname "wan0" accept
    }
    chain post {
        type nat hook postrouting priority srcnat; policy accept;
        ip daddr 198.18.0.1 udp sport 41600 counter snat to 198.18.0.$side:41600
        ip daddr 198.18.0.0/24 udp sport 41600 counter snat to 198.18.0.$side:$peer_port
        oifname "wan0" counter masquerade
    }
    chain origin {
        type filter hook prerouting priority filter; policy accept;
        iifname "wan0" ip saddr != 198.18.0.1 ip protocol udp counter
    }
}
RULES
done

"$bin" relay "$output" >"$output/relay.log" 2>&1 &
relay_pid=$!
for _ in {1..100}; do
    [[ ! -s $output/relay.der ]] || break
    if ! kill -0 "$relay_pid" 2>/dev/null; then cat "$output/relay.log" >&2; exit 1; fi
    sleep .05
done
[[ -s $output/relay.der ]]
net "$peer2" "$bin" host "$output" "$mode" >"$output/host.log" 2>&1 &
host_pid=$!
result=0
net "$peer3" "$bin" guest "$output" "$mode" >"$output/guest.log" 2>&1 || result=$?
if (( result == 0 )); then
    wait "$host_pid" || result=$?
fi
for router in "$router2" "$router3"; do
    { net "$router" nft list table ip p2p; net "$router" tc -s qdisc show dev wan0; }
done >"$output/network.log"
cat "$output/guest.log"
if (( result != 0 )); then
    cat "$output/relay.log" "$output/host.log" >&2
fi
exit "$result"
