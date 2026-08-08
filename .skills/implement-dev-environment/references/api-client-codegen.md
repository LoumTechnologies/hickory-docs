# API Client Codegen: Pipeline And Conflict Policy

## Toolchain

- **Spec generation**: the server framework's own OpenAPI support (e.g.
  NestJS `@nestjs/swagger`'s `SwaggerModule.createDocument`, FastAPI's
  built-in `/openapi.json`, drf-spectacular for Django). Written to a
  checked-in-or-regenerated `openapi.json`/`openapi.yaml` at a known path.
- **Client generation**: **openapi-typescript** (spec → a single `.d.ts` of
  request/response/param types, zero runtime) + **openapi-fetch** (a tiny
  typed wrapper around `fetch` that consumes those types). Chosen over
  heavier generators specifically because:
  - Pure Node/npm — no JVM, no Docker image to pull, nothing beyond what a
    JS/TS project already has installed. This is what makes `just codegen`
    infra-free.
  - Output is small and mechanical (types + one thin client file), which
    minimizes the surface that can conflict on a merge — the entire reason
    the "never hand-resolve, always regenerate" rule is cheap to follow.
  - Framework-agnostic on the consuming side — works the same whether the
    frontend is React, Vue, or a CLI script.
  - `openapi-generator-cli` (Java) and similar heavier generators are
    explicitly **not** the default: they add a JVM/Docker dependency that
    fights the "minimal/no infrastructure" goal, even though they support
    more languages/output styles. Only reach for one if a specific target
    language has no viable Node-based generator.

## Generating the OpenAPI spec without a live database

Frameworks whose app bootstrap normally opens a real database connection
(e.g. NestJS with `TypeOrmModule.forRootAsync` in `AppModule`) still need a
way to produce the spec with zero running services. Write a small dedicated
entrypoint script for this (e.g. `backend/scripts/generate-openapi-spec.ts`)
rather than reusing `main.ts`, so `codegen` never accidentally starts
listening on a port or depends on anything `main.ts` needs that a spec-only
run doesn't.

The mechanism that matters is **stopping the ORM from connecting at all**.
Route and DTO metadata comes from decorators, so the document builds fine
without a database — but you have to prevent the connection, not tolerate
its failure.

**`abortOnError: false` is not sufficient, and assuming it is will cost you
an hour.** In `@nestjs/typeorm`, the DataSource provider factory itself
calls `initialize()`:

```js
// typeorm-core.module.js
return dataSource.initialize && !dataSource.isInitialized && !options.manualInitialization
  ? dataSource.initialize()
  : dataSource;
```

The rejection propagates out of `NestFactory.create` before
`SwaggerModule.createDocument` ever runs, and the script dies with
`ECONNREFUSED`. `abortOnError: false` only stops Nest from calling
`process.exit` — it does not swallow a failed async provider.

The working knob is the `manualInitialization` option that condition reads,
set from the TypeORM options factory and gated on an env var the spec script
exports before Nest boots:

```ts
// app.module.ts — inside TypeOrmModule.forRootAsync's useFactory
manualInitialization: config.get<string>('OPENAPI_ONLY') === '1',
```

```ts
// scripts/generate-openapi-spec.ts
process.env.OPENAPI_ONLY = '1';
const app = await NestFactory.create(AppModule, { abortOnError: false, logger: false });
const document = SwaggerModule.createDocument(app, config);
```

Repositories still resolve against the uninitialized DataSource — nothing
queries through them while the document is being built.

Two details worth copying:

- **Read the env var inside the options factory, not at module scope.** A
  `@Module` decorator's arguments evaluate at *import* time, so a top-level
  `import { AppModule }` in the spec script runs before any
  `process.env.X = ...` line beneath it. Reading it inside `useFactory`
  (via `ConfigService`) sidesteps the ordering trap entirely; otherwise you
  need a lazy `require` after the assignment.
- **`process.exit(0)` when the file is written.** An ORM that was told not
  to connect can still leave a timer behind, and a codegen step that hangs
  is worse than one that fails.

For a framework not covered here, the general shape is the same: find the
knob that makes the data layer lazy, don't rely on error tolerance. If no
such knob exists, override the data-source provider with a stub rather than
booting a real one and hoping the failure is survivable.

## `.gitattributes`

```
frontend/src/api/generated/** linguist-generated=true -diff
backend/openapi.json          linguist-generated=true -diff
```

`linguist-generated=true` tells GitHub's PR review UI to collapse the file
by default (still viewable, just not presented as something to line-by-line
review). `-diff` suppresses it from `git diff` textual output for the same
reason — reviewers should look at the route and DTO changes in the server
source, which is what actually moved.

**Mark the spec as well as the client.** Both are generated output and
neither is worth reviewing line by line; leaving `openapi.json` unmarked
just relocates the noise.

## CI drift check

A required PR check into `main` runs `just check-codegen`, which is exactly
`just codegen` followed by a drift comparison scoped to the generated paths.

**Compare against the index, not `HEAD`.** The question is "did regenerating
change anything that isn't already recorded", and `git status --porcelain`
(or a diff against `HEAD`) answers a different one — it reports a file that
was just `git add`ed but not modified, so the check fails spuriously the
first time the generated output is committed, and again on any branch that
staged it. Use:

```sh
git diff --name-only -- <generated paths>                      # unstaged edits
git ls-files --others --exclude-standard -- <generated paths>  # never recorded
```

Non-empty output from either means drift. (`--name-only` still lists files
marked `-diff` in `.gitattributes`, so the attribute above doesn't blind the
check.)

On failure, the job's output states the fix in one line, e.g.:

```
Generated API client is out of date.
Run `just codegen`, then commit the changes in frontend/src/api/generated/.
```

The same `just check-codegen` command is what the pre-commit hook runs (see
`references/pre-commit-and-selective-ci.md`), so there is exactly one
implementation of "is the client stale" — CI and the hook can never
disagree about it.

## Conflict resolution procedure (the only one, ever)

When a generated client file shows merge-conflict markers:

1. Don't read them. Don't try to merge them by hand.
2. `git checkout --ours -- <generated-path>` (or `--theirs` — it does not
   matter which; the content is about to be discarded and regenerated).
3. `just codegen`
4. `git add <generated-path>`
5. Continue the merge/rebase as normal.

This is the entire procedure, independent of how large or confusing the
conflict looks. State it exactly this way (numbered, copy-pasteable) in
`docs/developers/developer-environment.md` so it reads clearly to someone
who has never set up this repo's dev environment before — they only need
`just codegen` to work, which per the section above requires no running
infrastructure.
