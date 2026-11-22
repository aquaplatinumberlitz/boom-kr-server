#!/bin/sh
set -eu
if [ "$#" -lt 1 ] || [ "$#" -gt 2 ] || [ "$(id -u)" != 0 ]; then
    printf 'Usage: sudo %s /absolute/path/to/game [server name]\n' "$0" >&2
    exit 2
fi
bundle=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
case "$1" in
    /*) game=$(realpath -e -- "$1");;
    *) printf 'Use an absolute game path.\n' >&2; exit 2;;
esac
name=${2:-Boom Server}
newline='
'
case $game$name in
    *"$newline"*) printf 'Game path and name must be on one line.\n' >&2; exit 2;;
esac
[ -f "$game/FxMap.idx" ] && [ -f "$game/FxMap.idd" ] || {
    printf 'Choose the KR 00967 directory containing FxMap.idx and FxMap.idd.\n' >&2
    exit 2
}
command -v python3 >/dev/null
command -v systemctl >/dev/null
if [ -e /opt/boom-server ] || [ -e /etc/boom-server.env ] ||
   [ -e /etc/systemd/system/boom-server.service ] || [ -e /usr/local/bin/boom-server ]; then
    printf 'An installation already exists. Back up state before upgrading.\n' >&2
    exit 1
fi
if ! id boom-server >/dev/null 2>&1; then
    useradd --system --user-group --home-dir /var/lib/boom-server --shell /usr/sbin/nologin boom-server
fi
if ! runuser -u boom-server -- test -r "$game/FxMap.idx"; then
    printf 'The boom-server system account must be able to read the game directory.\n' >&2
    exit 1
fi
python3 "$bundle/kr-package.py" verify --manifest-sha256 "$(cat "$bundle/manifest.sha256")"
install -d -m 0700 -o boom-server -g boom-server /opt/boom-server
cp -R --preserve=mode -- "$bundle/." /opt/boom-server/
chown -R boom-server:boom-server /opt/boom-server
install -m 0644 "$bundle/deploy/boom-server.service" /etc/systemd/system/boom-server.service
export BOOM_INSTALL_GAME="$game" BOOM_INSTALL_NAME="$name"
python3 - <<'PYTHON'
import os
from pathlib import Path

def quote(value):
    return '"' + value.replace('\\', '\\\\').replace('"', '\\"') + '"'

with Path('/etc/boom-server.env').open('x', encoding='utf-8') as handle:
    for key in ('GAME', 'NAME'):
        handle.write(f'BOOM_{key}={quote(os.environ["BOOM_INSTALL_" + key])}\n')
PYTHON
# Public operator entry point resolves to the bundle after checking OS permissions.
cat >/usr/local/bin/boom-server <<'SCRIPT'
#!/bin/sh
exec /opt/boom-server/bin/boom-server "$@"
SCRIPT
chmod 0755 /usr/local/bin/boom-server
systemctl daemon-reload
systemctl enable --now boom-server
printf 'Once the server is ready, run: sudo -u boom-server boom-server invite\n'
