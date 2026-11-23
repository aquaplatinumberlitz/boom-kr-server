# LANBoom player guide

<!-- overview -->
## Two ways to play

**Server** is for communities that run their own server. You use an account for that server, which saves progress according to its settings.

**Private room** is for private sessions. One friend hosts the room, and nobody needs an account.

<!-- compare -->
## Compare the two modes

| Topic | Server | Private room |
| --- | --- | --- |
| Who hosts? | The community runs a PC or VPS. | The friend who creates the room. |
| Account | A separate account for each server. | No account; just choose a name. |
| Progress | Saved by the server, according to its settings. | Does not carry over to community servers. |
| Content | Depends on the server's settings and supported features. | All supported items unlocked. |
| Connection | Bundled server today; LAN/Internet invites in S1 with no manual port forwarding for P2P. | Room codes over LAN or Internet in O1; no manual port forwarding. |
| How long? | Can stay up long term. | The room closes when the host leaves. |
| Players | Depends on the server and room limits. | Small groups; O1 limits will be published at launch. |

<!-- choose -->
## Which should I choose?

- For a quick game with a few people, choose **Private room** for an account-free game.
- For a regular group that wants to keep items or progress, choose **Server**.
- For a long-running community, consider a VPS or a PC that stays on 24/7.

<!-- servers -->
## Play on a server

1. Open the launcher, choose **Server**, and select a server. To add one, paste the S1 invite and click **Add server** or press Enter.
2. Need an account? Choose **Create account** and keep the recovery code somewhere safe. The account belongs to the selected server.
3. Press **PLAY**, then sign in on the game's login screen.
4. To get a newer version, use **Download latest** if your bundle has the link. It opens Google Drive, where you can download the full bundle.

The launcher remembers servers you add. Use **Remove** to clear a server from your list.
An S1 invite connection can relay gameplay traffic when direct UDP is unavailable.

<!-- friends -->
## Play in a private room

1. Open the launcher, choose **Private room**, and enter a **Character name**. The launcher remembers it for next time.
2. The host clicks **Host a private room**. When the code appears, click **Copy code** and share it privately with friends. The host's game opens automatically.
3. Friends paste the code and click **Join room**, or select the room from **Rooms on this LAN**. Keep the launcher open while playing.
4. At the game's login screen, choose **Play trial 1P** for one player or **Play trial 2P** for two players on one PC. Leave the account and password fields blank. On separate PCs, choose 1P to chat under separate names. In the lobby, one player creates a game room and the others join it.
5. If your package differs from the host's, use **Download latest** when the bundle provides a link. If the host closes the game or launcher, friends are told that the room closed.

![Host room in the launcher](o1-create-room.png)

![Join by room code](o1-join-code.png)

![Join from the LAN list](o1-join-lan.png)

Playing across networks needs a bundle with a working room directory. If the room cannot be found, check the code and ask the host whether their game is still open.

<!-- connection -->
## Can't connect

- Select the right bundled server, check your connection, and press **Retry**. If it still fails, ask the server operator whether the service is running.
- Start the game through the launcher so the connection is set up correctly.

<!-- version -->
## Different version or file errors

- Ask the server operator which version to use. If available, choose **Download latest** and download the full matching bundle. The launcher does not update automatically.
- If the launcher reports a file error, use **Settings → Check & repair files** and try again.

<!-- firewall -->
## Windows Firewall

Allow LAN access when Windows asks the first time you host a private room. To retry, go to **Settings → Allow LAN in Windows Firewall**. Keep Windows Firewall on.

<!-- Standalone guide appendix; the launcher embeds the sections above. -->

Host your own server (S1)
--------------------------

An S1 server stores accounts and match results on your computer. Keep it running while people play. The server package already includes the game-rule data (maps, items, character stats), so you do not choose a game folder.

**Windows**

1. Extract the entire Windows server package into one folder. Open **BoomServer.exe**.
2. Enter a **Server name** and click **Run**. The message box lists each startup step; wait for the green light and **Running**.
3. Click **Copy** next to **S1 invite** (it changes to **Copied ✓**) and send the code to players. They can use **Server → Add server** in LANBoom.
4. Closing the window keeps the server running in the system tray. Click the BoomServer icon to reopen it, or right-click it to **Copy invite** or **Stop & Exit**. Click **Stop** before shutting down. **Back up** saves a database snapshot and shows its file name and time. **Settings** lets you choose the language (Tiếng Việt / English / 한국어), a folder for backup copies, **Open data folder**, and **New invite code** (the old code stops working and current players are disconnected). Errors appear in full in the message box, where you can read or copy them; **Open log** shows the log.

If Windows Firewall asks to allow direct connections, you can allow them on **Private networks**. BoomServer still runs through the relay if you decline or your router cannot open a port. If players cannot join immediately, wait for **Relay connected** and try again.

![BoomServer window on Windows 10 (Vietnamese UI) with four blocks: status (running and relay connected, 2 online, 1 room), server name, S1 invite code, and Stop/Back up/Settings/Open log buttons](images/boomserver-s1.png)

**Linux**

1. Extract the Linux server package. For Docker Compose, open a terminal in the extracted directory and run:

   ```sh
   export BOOM_NAME="Boom Club"
   docker compose -f deploy/compose.yaml up -d
   docker compose -f deploy/compose.yaml exec boom /opt/boom/bin/boom-server invite
   ```

2. On Linux with systemd, you can instead run `sudo ./deploy/install.sh "Boom Club"`. Once it starts, get the code with `sudo -u boom-server boom-server invite`.
3. Send the S1 invite to your players. Keep the Docker state volume or the systemd data folder at `/var/lib/boom-server` to retain your server identity, accounts and results.

Find the source and operations guide in the [GitHub repository](https://github.com/aquaplatinumberlitz/boom-kr-server).
