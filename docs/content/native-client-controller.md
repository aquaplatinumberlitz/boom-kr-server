# Native client controller

Run `tools/native_client_controller.py` with Python 3.11+ on Windows or Linux.
Build `tools/native_client_input.c` with MinGW, for example
`x86_64-w64-mingw32-gcc -O2 tools/native_client_input.c -o local/input.exe -luser32`.
Use the resulting helper on the client's desktop, under the same Wine prefix and
display when applicable. `input.exe windows` lists Win32 PIDs; foreground the
chosen client before running. `input.exe <pid> geometry` reports the client
area's screen origin and size for that foreground PID. Subtract the origin
from desktop coordinates when preparing the plan. The helper checks the PID, uses client-area
coordinates, and releases held keys or buttons. The controller never starts
the game or changes network routing.

```sh
python tools/native_client_controller.py local/plan.json --service-run local/run/service \
  --db local/run/state/account.sqlite --helper local/input.exe --pid 123
```

On Linux add `--wine wine`; on Windows omit it. Use a separate log directory
per server run and a local read-only view of SQLite. For a remote Windows guest,
publish the health and event files and refresh a consistent SQLite backup on the
guest, rather than opening a live WAL database on a network share. The tool does
not copy or share these files for you.

When attaching to an already busy service, pass `--from-tail` to skip completed
events written before the controller attaches. Start the controller before the
two test clients join. Use conditions with the two expected `stream_id` values
and the exact journal key for their room; two arbitrary streams can belong to
unrelated rooms. The same health, event and journal gates work on a local lab
or a read-only export from the M8 test service. Keep the endpoint, account
credentials, room key, and desktop layout in the local run configuration.

A scenario uses named steps and server gates:

```json
{
  "steps": [
    {
      "id": "round-start",
      "gate": {},
      "actions": [{"op": "click", "xy": [723, 616], "hold_ms": 220}]
    },
    {
      "id": "round-active",
      "timeout_seconds": 30,
      "gate": {"events": [
        {"after": "round-start", "match": {"event": "native_tls_room_peer_written", "opcode": 153},
         "count": 2, "distinct": "stream_id"}
      ]},
      "actions": [{"op": "key", "vk": 32, "hold_ms": 100}]
    }
  ]
}
```

In a dedicated two-client run, the second gate waits for peer routes to be
written to both streams. Start/Ready and return use
`native_tls_room_start_written`, `native_tls_room_ready_written` and
`native_tls_room_return_refreshed`. `after: "begin"` skips earlier log entries.
For a journal gate, supply `room_id`, `room_session`, `round`, `state`, and
optionally `reason` as the value of `gate.journal`. Each step also requires fresh
server health from the same boot and epoch. The event counter is scoped to its
log, so use an isolated room or a match filter for known stream IDs.

For a mixed Windows/Wine run, open the game through the player launcher on
both desktops and identify each foreground game PID after Play. For each map,
record a new event mark just before the host clicks Start. Require the Start
writes and peer writes on both room streams before sending gameplay input.
After the exact round is complete in SQLite, check both result screens and
require a return refresh on both streams before selecting the next map. Take
the desktop captures on the same run: the journal does not prove which screen
either client showed. For M8, attach fresh room and round keys and reuse this
sequence once the soak and deployment gates allow the two-client run.

Use `{"op":"unicode","text_file":"chat.txt"}` for a UTF-8 file beside the plan.
Use test text only: the helper receives UTF-16 units as a process argument.
Clicks accept `settle_ms` for hover after pointer movement. Clicks and held keys are ordinary input. Server gates prove that server state
advanced; they cannot prove a client rendered the screen, accepted input, or
showed its match clock. The next server gate can verify the resulting request.

For the isolated KR 00967 Wine probe, `tools/native_wine_login.py` is a separate
Pillow-based helper for the centered 800×600 client on a 1024×768 desktop. It waits
for two frames that match the login panel in an empty-form reference from the
same client build. It clears the existing values with End and Backspace, moves
the game cursor away from the field, and waits for each field to clear and
receive input before clicking Sign In. The helper expects short ASCII test
credentials; a rendered field does not establish that the server accepted
them. The batch driver must still wait for the channel for the expected
account and the lobby marker. Use an owned Wine prefix and display.
