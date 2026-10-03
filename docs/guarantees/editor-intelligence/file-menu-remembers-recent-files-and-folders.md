# File remembers recent files and folders

Given files and folders successfully opened in the desktop app, when the person
opens File → Recent Files or File → Recent Folders, then the corresponding
submenu lists up to ten existing paths, newest first, without duplicates.
Full paths distinguish identically named files or folders. An empty list has a
disabled “No Recent Files” or “No Recent Folders” item.

Given a recent file inside the current workspace, when selected, then it opens
through the same tab navigation as Open File. A file elsewhere or a recent
folder uses the existing session-switching behavior. A path that disappeared
since the menu was drawn reports why it cannot open and offers the ordinary
Open commands; it does not restart the app into a missing path.

Given navigation through the workspace's file tabs, when a file becomes active,
then it enters Recent Files. Non-file tabs and unsaved documents do not enter
the list. The list lives in app configuration, outside repositories, persists
across launches, and shares a locked read/modify/write operation across desktop
processes. Older releases' last-opened path seeds an empty list. Unreadable
history falls back to an empty list or that legacy path rather than failing
startup. Other windows' updates become visible when the window regains focus.

## Verification

- `just test-recent-paths`: persistent separate lists, newest-first ordering,
  deduplication, ten-entry limit, removal of missing paths, migration and corrupt
  history recovery; workspace file navigation and exclusion of non-file tabs.
- `just check-desktop` and `just check-web`: native menu and page integration.
- Native menu placement and platform session switching need manual GUI review.
