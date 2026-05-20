---
id: TASK-52.1
title: 'AFK 1/5: cross-platform screen-lock detection (Linux D-Bus, macOS IOKit)'
status: To Do
assignee: []
created_date: '2026-05-20 12:35'
labels:
  - afk
  - platform
  - dbus
  - iokit
dependencies: []
references:
  - 'https://specifications.freedesktop.org/idle-inhibit-spec/latest/'
  - >-
    https://developer.apple.com/documentation/corefoundation/cfnotificationcenter
parent_task_id: TASK-52
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a background task to `hick-agent-sdk` that watches for OS-level screen lock/unlock events and emits `UiEvent::ScreenLocked` / `UiEvent::ScreenUnlocked` into the shared event channel. Must be off by default and only spawned when `afk.auto_enter_on_lock = true`.\n\n**Linux**: Subscribe to `org.freedesktop.ScreenSaver` (and `org.gnome.ScreenSaver` as fallback) D-Bus signals via the `zbus` crate. Both GNOME and KDE emit the `ActiveChanged(bool)` signal on these well-known names.\n\n**macOS**: Register for the `com.apple.screenIsLocked` Darwin notification via `notify(3)` / the `CFNotificationCenter` API. Alternatively subscribe to `com.apple.screensaver.didstart` / `com.apple.screensaver.didstop`.\n\n**Other platforms**: Emit a compile-time warning; no-op implementation — AFK auto-entry on lock is unavailable.\n\nThe detection loop runs in its own `tokio::task`. Errors (D-Bus unavailable, permission denied) are logged to the XML session log at warn level and the loop exits gracefully — they must not crash the agent.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 ScreenLocked / ScreenUnlocked UiEvent variants added to hick-agent-sdk
- [ ] #2 Linux: locking the screen (gnome-screensaver-command --lock, loginctl lock-session) triggers ScreenLocked within 1 s
- [ ] #3 macOS: locking the screen triggers ScreenLocked within 1 s
- [ ] #4 Detection task is not spawned unless afk.auto_enter_on_lock = true in agent TOML
- [ ] #5 D-Bus unavailability (no display, headless CI) logs a warning and does not panic
- [ ] #6 Unit test (Linux only, feature-gated) verifies D-Bus subscription setup compiles and the signal type decodes correctly
<!-- AC:END -->
