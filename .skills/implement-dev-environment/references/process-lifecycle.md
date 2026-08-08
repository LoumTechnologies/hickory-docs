# Long-Running Dev Servers: Starting, Stopping, Not Leaking

`just dev` supervises processes that outlive any single command, and the
"safe to kill and re-run" contract in `$dev-environment` depends far more on
getting their teardown right than on getting their startup right. **Leaked
child processes are the single biggest threat to that contract**, and they
fail in a way that wastes hours: a stale server keeps answering on the
worktree's PortZero name, so the next run appears to work while talking to a
process built from old code, pointed at a database volume you already
deleted. The symptom is an inexplicable 500 or an intermittent connection
failure in something unrelated to what you changed.

Assume every rule below was learned by leaking a server.

## Kill the process group, not the child

`npm run dev` (or any package-manager wrapper) execs the real server as a
*grandchild*. Signalling the direct child kills the wrapper and orphans the
server.

Start each service in its own process group, then signal the group:

```ts
spawn(command, args, { detached: process.platform !== 'win32', /* ... */ });
// ...
process.kill(-child.pid, 'SIGTERM');   // negative pid ⇒ the whole group
```

On Windows there are no process groups; use `taskkill /pid <pid> /T /F`,
which walks the child tree and is synchronous and forceful.

Note the trade-off `detached` brings: the service no longer receives the
terminal's Ctrl-C, so you **must** forward signals yourself (see below).

## Waiting on the child's `exit` is not waiting for shutdown

The wrapper exits the instant it is signalled, while the server it spawned
is still shutting down in the same group. Code that awaits only the direct
child's `exit` event returns to a caller that then exits the process — and
the server survives.

Wait for the *group* to empty, then escalate:

```ts
signalGroup(pid, 'SIGTERM');
await Promise.race([childExited, sleep(graceMs)]);
if (await groupIsGone(pid, graceMs)) return;   // poll process.kill(-pid, 0) for ESRCH
signalGroup(pid, 'SIGKILL');
await groupIsGone(pid, graceMs);
```

`process.kill(-pid, 0)` delivers nothing and just probes existence: `ESRCH`
means the group is empty. Treat `EPERM` as "still there" — something is
alive, it just isn't yours to signal.

## Signal handlers keep the event loop alive

A registered `process.on('SIGINT', …)` creates a libuv handle that on its
own prevents Node from exiting. A one-shot run (the CI smoke check below)
will therefore complete all its work, print success, and then **hang
forever**. Remove the listeners when the supervised phase ends:

```ts
try {
  await run();
} finally {
  process.off('SIGINT', onSignal);
  process.off('SIGTERM', onSignal);
}
```

## Tear down on the failure path too

If bring-up throws after some services are up, the half-started stack keeps
holding this worktree's ports and PortZero names and poisons the next run.
Wrap the supervised section so any error stops everything before it
propagates.

## Discovering a service's address

Two services usually need to find each other before either has a port.

- **Under PortZero, don't discover — decide.** The hostname is derived from
  the worktree slug, so it's known before anything starts and can be handed
  to a frontend as its API base URL immediately.
- **In fallback mode, parse the startup banner.** Have the server print a
  stable, greppable marker (`LISTENING_ON=http://127.0.0.1:41509`) and watch
  its stdout for it. Add the marker to the app deliberately; don't scrape a
  framework's decorative output if you can avoid it.
- **Hand the resolved address to sibling commands via a file.** `dev-seed`
  runs in its own process and has no way to learn an OS-assigned port, so
  `dev` should write the resolved base URL to the untracked local state
  directory (e.g. `.dev-env/api-url`) and `dev-seed` should read it. This is
  the one interaction between "seed through the real HTTP endpoint" and
  "never name a port" that isn't obvious until you're blocked by it.

Line-buffer the output you forward so a marker never sits half-written in a
chunk boundary, and prefix each line with the service name so an interleaved
`[api]` / `[web]` log stays readable.

## Making the dev environment break CI

`$dev-environment` requires that a broken `just dev` fails CI for the same
reason it fails a developer. The mechanism: give `dev` a **smoke mode** that
reuses the identical bring-up path, then

1. polls the API's health URL until 2xx (with a generous timeout — a first
   compile is slow),
2. polls the web URL until 2xx,
3. runs the seed,
4. tears everything down and exits 0.

Because it is the same code path a developer runs, it cannot rot separately.
Wire it in as one of the checks the changed-file mapping can select (see
`references/pre-commit-and-selective-ci.md`).

## Verifying teardown

Don't trust it — check it. After Ctrl-C and after a smoke run:

```sh
pgrep -af "<your server processes>"
```

Beware of counting your own `grep`/`ps`/shell command as a survivor; match
on the concrete binary path, and confirm any hit is real with
`ps -o pid,ppid,pgid,args` before concluding you have a leak.
