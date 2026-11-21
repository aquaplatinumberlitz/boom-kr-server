#!/usr/bin/env bash
# Run only from the private user/PID/network tree of o1_p2p_namespace.sh.
# Inputs: S1_WINDOWS_BUNDLE, S1_GAME, S1_LINUX_BIN (absolute local paths).
set -euo pipefail
umask 077
role=$1 out=$2
[[ $(id -u) == 0 && $(awk 'NR==1 {print $3}' /proc/self/uid_map) == 1 ]]
[[ $out == "$(realpath "$out")" && -d $out ]]
: "${S1_WINDOWS_BUNDLE:?}" "${S1_GAME:?}" "${S1_LINUX_BIN:?}"
tools=$(dirname "$(realpath "$0")")
relay=https://198.18.0.1:8443
wait_for() {
    for ((attempt=0; attempt<$1; attempt++)); do
        if [[ $role == guest && -e $out/host-failed ]]; then return 1; fi
        "${@:2}" && return
        sleep .2
    done
    printf 'Deadline: %s\n' "$2" >&2
    return 1
}
if [[ $role == guest ]]; then
    wait_for 900 test -f "$out/host-ready"
    PYTHONPATH="$tools" python3 - "$out" <<'PY'
import json
from pathlib import Path
import sys

from test_s1_install import load_inputs

out = Path(sys.argv[1])
replay, config = load_inputs(
    (out / 'invite.txt').read_text().strip(),
    dict(private_relay='https://198.18.0.1:8443', relay_ca=str(out / 'relay.der')),
    str(out / 'replay.json'),
    json.loads((out / 'round-accounts.json').read_bytes()),
    active=16,
)
for path, value in [('load', config), ('replay', replay)]:
    (out / (path + '.json')).write_text(json.dumps(value))
PY
    timeout 100 "$S1_LINUX_BIN/kr-load" "$out/load.json" | tee "$out/load.jsonl"
    grep -q '"event":"load_complete".*"players":2' "$out/load.jsonl"
    touch "$out/round-done"
    wait_for 160 test -f "$out/rotate-done"
    PYTHONPATH="$tools" python3 - "$out" <<'PY'
import json
from pathlib import Path
import sys
out = Path(sys.argv[1])
config = json.loads((out / 'load.json').read_bytes())
config['s1']['invite'] = (out / 'new-invite.txt').read_text().strip()
config['active_seconds'] = 2
(out / 'load-after-rotate.json').write_text(json.dumps(config))
PY
    # Allow the new relay registration to settle before exercising TLS again.
    sleep 3
    timeout 70 "$S1_LINUX_BIN/kr-load" "$out/load-after-rotate.json" | tee "$out/load-after-rotate.jsonl"
    grep -q '"event":"load_complete".*"players":2' "$out/load-after-rotate.jsonl"
    touch "$out/stop"
    exit
fi
[[ $role == host ]]
mkdir "$out/scratch"
scratch=$out/scratch
export WINEPREFIX="$scratch/prefix" WINEARCH=win64 WINEDEBUG=-all
export BOOM_RELAY_URL="$relay" BOOM_RELAY_CA="Z:${out//\//\\}\\relay.der"
export XDG_RUNTIME_DIR="$scratch/xdg" WINEDLLOVERRIDES="mscoree,mshtml="
mkdir "$XDG_RUNTIME_DIR"
xvfb_pid=''
service=''
state=''
cleanup() {
    result=$?
    trap - EXIT
    if (( result != 0 )); then
        touch "$out/host-failed"
        if [[ -n ${DISPLAY:-} ]]; then
            import -window root "$out/failure.png" 2>/dev/null || true
        fi
    fi
    if [[ -n $state && -d $state/runs ]]; then
        mkdir -p "$out/state-observations"
        find "$state/runs" -path '*/service/*' -type f \( -name 'health.json' -o -name 'rust-events.jsonl' -o -name 'service-state.json' \) \
            -exec cp -t "$out/state-observations" -- {} +
    fi
    if [[ -d $WINEPREFIX ]]; then
        timeout 15 wineserver -k || true
        timeout 15 wineserver -w || true
    fi
    if [[ -n $xvfb_pid ]]; then kill "$xvfb_pid" 2>/dev/null || true; fi
    du -sm "$scratch" | awk '{print $1}' > "$out/removed-mib.txt"
    chmod -R u+rwX "$scratch"
    rm -rf -- "$scratch"
    exit "$result"
}
trap cleanup EXIT
Xvfb -displayfd 3 -screen 0 1024x768x24 -nolisten tcp 3>"$scratch/display" >"$out/xvfb.log" 2>&1 &
xvfb_pid=$!
wait_for 100 test -s "$scratch/display"
export DISPLAY=":$(cat "$scratch/display")"
timeout 70 wineboot -u >"$out/wineboot.log" 2>&1
drive=$WINEPREFIX/drive_c
mkdir "$drive/Game"
for pack in FxMap FxContents fx; do
    for ext in idx idd; do
        cp --reflink=auto "$S1_GAME/$pack.$ext" "$drive/Game/"
    done
done
chmod -R a-w "$drive/Game"
# Keep the same package layout. This copy is removed with the prefix.
cp -a --reflink=auto "$S1_WINDOWS_BUNDLE" "$drive/Server"
bundle=$drive/Server
wine "$bundle/BoomServer.exe" >"$out/wine.log" 2>&1 &
service=$!
window() { xdotool search --name '^BoomServer$' | head -1; }
window_present() { [[ -n $(window) ]]; }
wait_for 200 window_present
wid=$(window)
click() { xdotool mousemove --window "$wid" "$1" "$2" click 1; }
# The first-run window must infer the adjacent, read-only C:\Game folder.
click 200 25
xdotool key ctrl+a
xdotool type 'Boom Club'
click 57 207
ready() {
    state=$(find "$drive/users" -type d -path '*/AppData/Local/BoomServer' -print -quit)
    [[ -n $state ]] && find "$state/runs" -name health.json -exec grep -l '"healthy":true' {} + 2>/dev/null | grep -q .
}
wait_for 520 ready
sidereal="${state#"$drive/"}"
db="C:\\${sidereal//\//\\}\\accounts.sqlite"
clip="Z:${tools//\//\\}\\s1_wine_clipboard.py"
copy_invite() {
    click 460 164
    timeout 10 wine "$bundle/python/python.exe" "$clip" | tr -d '\r' > "$1"
}
copied() { copy_invite "$1" 2>>"$out/clipboard.log"; }
wait_for 35 copied "$out/invite.txt"
PYTHONPATH="$tools" python3 - "$out" <<'PY'
import json
from pathlib import Path
import sys
from test_s1_install import accounts_for_round, account_request
out = Path(sys.argv[1])
accounts = accounts_for_round()
(out / 'round-accounts.json').write_text(json.dumps(accounts))
for i, row in enumerate(accounts):
    (out / f'account-{i}.json').write_text(json.dumps(account_request(row)))
PY
for i in 0 1; do
    timeout 30 wine "$bundle/bin/kr-admin.exe" "$db" <"$out/account-$i.json" >>"$out/admin.log" 2>&1
done
touch "$out/host-ready"

online() { find "$state/runs" -name health.json -exec grep -l '"online_players":2' {} + | grep -q .; }
wait_for 450 online
sleep 2
# Verify the background lifecycle while the round is active. Opening the
# executable again must restore the original window and leave sessions alive.
timeout 15 wine "$bundle/python/python.exe" "$clip" close
sleep .4
wine "$bundle/BoomServer.exe" >>"$out/wine.log" 2>&1
sleep .6
import -window "$wid" "$out/BoomServer.png"
wait_for 250 test -f "$out/round-done"
click 165 206
backup_present() { compgen -G "$state/manual-backups/*.sqlite" >/dev/null; }
wait_for 90 backup_present
backup=$(find "$state/manual-backups" -name '*.sqlite' -print -quit)
relative="${backup#"$drive/"}"
timeout 20 wine "$bundle/bin/kr-admin.exe" verify-backup "C:\\${relative//\//\\}" > "$out/backup-verify.json"
click 430 206
# Yes/No dialog defaults to No. Move focus left and accept.
sleep 1
xdotool key Left Return
rotated() {
    copied "$out/new-invite.txt" || return
    ! cmp -s "$out/invite.txt" "$out/new-invite.txt"
}
wait_for 150 rotated
touch "$out/rotate-done"
wait_for 450 test -f "$out/stop"
click 56 206
stopped() { find "$state/runs" -name service-state.json -exec grep -l '"stopped":true' {} + | grep -q .; }
wait_for 180 stopped
timeout 15 wine "$bundle/python/python.exe" "$clip" close
for _ in {1..25}; do
    kill -0 "$service" 2>/dev/null || break
    sleep .2
done
if kill -0 "$service" 2>/dev/null; then
    printf 'Wine desktop process did not exit after WM_CLOSE
' >&2
    exit 1
fi
wait "$service"
printf 'S1 Wine: first boot, two-player NAT relay round, background restore, backup, rotation, second round, clean stop\n'
