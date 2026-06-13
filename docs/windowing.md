# ChronoOS Windowing

Status: partially implemented, partially verified in QEMU.

ChronoOS has a tiny fixed-capacity framebuffer window layer. It is meant to
make the shell feel more like an educational desktop while staying simple and
honest. It is not a compositor, GUI toolkit, desktop environment, animation
system, or GPU-accelerated renderer.

## Current Window Apps

- `open notes`: opens a notes window and spawns a cooperative notes task.
- `open sysinfo`: opens a sysinfo window and spawns a cooperative sysinfo task.
- `open paint`: not implemented; paint is roadmap/design-only.

Shell-first fallbacks remain available:

- `notes`
- `sysinfo`
- `apps launch notes`
- `apps launch sysinfo`

## Lifecycle Commands

- `windows` / `windows list`: list open windows.
- `windows status`: show capacity, drag state, supported window apps, and
  current boundary.
- `windows focus <id>`: bring a window to the front by its shell-facing window
  ID.
- `windows close <id>`: close a window by ID and terminate its owning
  cooperative task.
- `windows help`: show usage.

Window IDs are shell-facing lifecycle IDs, not process IDs. Task IDs still come
from the cooperative scheduler and can be inspected with `tasks`.

## Mouse Behavior

The PS/2 mouse path publishes movement/click events. The window manager uses
those events for title-bar focus, dragging, and close-button clicks. A
2026-06-13 QEMU pass recorded a click packet at `70,65`, but visible cursor
movement, drag, and mouse close still need focused manual verification.

## 2026-06-13 Window/Input Verification Note

A visible single-core BIOS QEMU pass with a fresh disposable data image recorded
narrow window evidence:

| Test | Status | Evidence | Notes |
| --- | --- | --- | --- |
| `windows status` | verified in QEMU | Serial `cmd: windows status`; `/private/tmp/chronoos-window-input-20260613-193131-windows-status.png`. | Count/capacity, drag state, supported apps, and boundary text observed. |
| `open notes` | verified in QEMU | Serial `cmd: open notes`, task spawn, and `wm: open notes`; `/private/tmp/chronoos-window-input-20260613-193131-open-notes.png`. | Notes window observed. |
| `open sysinfo` | verified in QEMU | Serial `cmd: open sysinfo`, task spawn, and `wm: open sysinfo`; `/private/tmp/chronoos-window-input-20260613-193131-current-after-sysinfo-attempt.png`. | Input retry opened two sysinfo windows; duplication is an input artifact. |
| `windows list` | partially verified in QEMU | Exact `windows list` input garbled as `wwindows list`; exact alias `windows` listed windows in `/private/tmp/chronoos-window-input-20260613-193131-windows-list-exact.png`. | Observed notes ID/task 1 and sysinfo IDs/tasks 2 and 3. |
| `windows focus 1` | verified in QEMU | Serial `cmd: windows focus 1`; `/private/tmp/chronoos-window-input-20260613-193131-mouse-click-notes-attempt.png`. | Notes came to front and output said `Focused window 1`. |
| Mouse click packet | partially verified in QEMU | Serial `mouse: click at 70,65`. | Click delivery observed; visible pointer movement was not proven. |
| `windows close 2` | partially verified in QEMU | Serial `cmd: windows close 2`, `sched: killed task 2`, and `wm: close sysinfo`; `/private/tmp/chronoos-window-input-20260613-193131-windows-close-2.png`. | Serial proves the close path ran for ID 2, but the screenshot was a breakpoint-like black framebuffer and no follow-up list/tasks output was captured. |
| `tasks` / `kill <id>` | implemented in code, not verified | Not run after the close attempt. | Needs a fresh pass with real observed non-shell task IDs. |

Manual typing, Backspace, Shift, drag, and mouse close were not verified in this
pass because the Codex environment could not type manually into the visible QEMU
GUI and HMP mouse commands did not provide enough visual movement evidence.

## Limits

- Maximum open windows: 4.
- Supported window app kinds: notes and sysinfo.
- Closing a window also kills its owning cooperative task.
- `kill <id>` closes any associated window after terminating the task.
- There is no overlapping compositor beyond simple redraw order.
- There are no animations, widgets, menus, layout engine, or full desktop shell.

## Verification Path

Recommended next QEMU shell sequence:

```text
windows status
open notes
open sysinfo
windows
windows focus <id>
windows close <id>
tasks
kill <observed-non-shell-task-id>
```

Use a fresh disposable data image. Record screenshots and serial logs before
upgrading any windowing verification labels, and do not run `kill 0`.
