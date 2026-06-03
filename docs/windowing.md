# ChronoOS Windowing

Status: partially implemented, needs runtime verification.

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
those events for title-bar focus, dragging, and close-button clicks. A prior
QEMU pass recorded one mouse click packet and partial `open notes` window
evidence, but cursor movement, drag, close, focus, and `open sysinfo` still need
focused runtime verification.

## Limits

- Maximum open windows: 4.
- Supported window app kinds: notes and sysinfo.
- Closing a window also kills its owning cooperative task.
- `kill <id>` closes any associated window after terminating the task.
- There is no overlapping compositor beyond simple redraw order.
- There are no animations, widgets, menus, layout engine, or full desktop shell.

## Verification Path

Recommended QEMU shell sequence:

```text
windows status
open notes
windows list
open sysinfo
windows focus <id>
windows close <id>
tasks
open paint
```

Record screenshots and serial logs before upgrading any windowing verification
labels.
