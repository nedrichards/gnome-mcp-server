# Upstream review — 15 November 2026

The fork is rebased onto upstream main `9087049`. Upstream has no open pull
requests. Its merged PRs are retained rather than reimplemented:

- [#3](https://github.com/bilelmoussaoui/gnome-mcp-server/pull/3): client/build documentation.
- [#4](https://github.com/bilelmoussaoui/gnome-mcp-server/pull/4): GNOME Shell screenshot fallback and maximized-state compatibility.
- [#5](https://github.com/bilelmoussaoui/gnome-mcp-server/pull/5): earlier vulnerable dependency updates.
- [#7](https://github.com/bilelmoussaoui/gnome-mcp-server/pull/7): Newelle client documentation.

The dependency refresh moves to zbus 6.3, ashpd 0.14 and oo7 0.7. The retired
zbus_names/zvariant/zvariant_derive crates disappear from the graph. Rust 1.95
is required by oo7. The lockfile records the resolved graph; Cargo Deny retains
an empty advisory ignore list and no obsolete duplicate-version exemptions.

## Tools proposal (#6)

[Original proposal](https://github.com/bilelmoussaoui/gnome-mcp-server/issues/6).

| Request | Evaluation and current disposition |
| --- | --- |
| List workspaces | Implemented as `window_management` action `list_workspaces`, reporting indexes, active state and window counts. It uses the existing Shell Eval route and its existing unsafe-mode requirement; the server never enables that mode. |
| Get monitors | Implemented as read-only `get_monitors` using Mutter DisplayConfig.GetCurrentState. Reports connectors/current modes and logical positions/scales/transforms, including mirrors and disabled outputs. Omits hardware serial numbers. No Shell Eval is needed. |
| Non-interactive screenshots | Already supported, including upstream #4's Shell fallback. Portal cancellation or permission refusal is now final; a fallback must not capture after cancellation. Desktop policy can still require interaction. |
| Window/area screenshots | GNOME Shell exposes backend-specific capture methods, but the portable Screenshot portal does not accept arbitrary window IDs/rectangles. A backend-specific API needs explicit target/coordinate contracts and cancellation tests before addition. |
| Delayed notifications | Needs bounded, cancellable scheduling and a defined outcome when the stdio client disconnects. Current notifications remain immediate; no detached or persistent queue is added. |
| Power profiles/battery/brightness/lock | Separate value/read/action operations are appropriate; they do not fit the existing required boolean `enabled`. Candidate services are power-profiles-daemon, UPower, GNOME SettingsDaemon Power and ScreenSaver. These require their own service/error tests before exposure. |
| Mouse/keyboard/search input | Requires a consented RemoteDesktop session (or a deliberately installed Shell extension), with session lifecycle and cancellation. Shell Eval and external command execution are not substitutes for that consent contract. |
| Pick color | Supported by the interactive Screenshot portal. A future dedicated tool can return RGB without producing a screenshot file. |
| Clean screenshots | Needs an ownership-scoped capture directory and retention rules first. Do not delete arbitrary user screenshots or paths supplied by another process. |
| Ping/enabled state/extension controls | The server already has MCP initialization and configuration-based tool listing. No Shell extension is shipped, so extension enable/disable controls would be inert. |
| Open web URLs | Already supported by `open_file` (GIO/xdg-open). This update does not launch an external application during tests. |

The monitor wire signature is checked against the
[Mutter GNOME 51 interface](https://github.com/GNOME/mutter/blob/gnome-51/data/dbus-interfaces/org.gnome.Mutter.DisplayConfig.xml).
New D-Bus tests use isolated peer connections and invented data, without reading
accounts, modifying a desktop, capturing a screen or accessing a real keyring.

## calcard maintenance (#8)

[Issue #8](https://github.com/bilelmoussaoui/gnome-mcp-server/issues/8) and the
maintainer's reply confirm that calcard is moving into Stalwart and needs a
replacement evaluation. It remains pinned by Cargo.lock at 0.1.3 in this update.
It is not ignored by the advisory gate.

A replacement must preserve folded/escaped fields, repeated and structured vCard
values, date-only and timezone handling, nested calendar components and recurrence
rules. Existing parser regression tests cover much of that contract. The `ical`
crate parses both formats, but its latest published 0.11.0 release dates to 2024;
that alone does not establish a better maintenance path. A parser migration needs
fixture comparison and a dependency/license review, rather than a blind crate
substitution or a new handwritten parser. No parser replacement is included here.
