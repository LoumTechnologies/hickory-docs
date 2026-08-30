# A Generator Is Written In The Team's Language, Not The Model Server's

Given a language with a code model server, when somebody writes a generator
against it, then the generator may be written in **any** language: the server
speaks GraphQL over a pipe and has no opinion about what is on the other end
of it. `hick code-model --client --target <lang>` emits the types, and
`hick code-model --runtime --target <lang>` emits the transport that carries
them — so a project asking in two languages gets two clients and vendors
neither.

A target with no runtime yet **says so** and names what it does have, rather
than serving another language's file. Its `--client` still works: the types
stand alone, and sending them is that ecosystem's own GraphQL client's job.

## Why

The first version of the warehouse demo made a virtue of the generator being
Python while the code it modelled and emitted was C#, and the sentence in that
document read *"the generator is not written in the language it generates
for"*. That is a true capability and a bad default. A C# shop maintains C#: a
Python script in `tools/` is a file nobody on the team wants to own, and
"look, it can be a different language" is a demonstration nobody asked for.

The capability is worth keeping and worth demonstrating *honestly*, which the
demo now does by leaving exactly one of its two tools in the other language —
a C# generator and a Python layering check, against one C# model server.

The rewrite also produced the strongest available evidence that the rules and
not the implementation are what matter: the C# generator emits both output
files **byte-for-byte identically** to the Python one it replaced.

## What this costs

Measured on that rewrite rather than assumed: the same three rules cost 163
lines in Python and 185 in C#, moving the break-even point of the whole
exercise from eight endpoints to nine. Choosing the language your team already
maintains is worth paying that for. It is not worth paying unknowingly, which
is why `50-review-surface.hick` prints it.

---

Last LLM verification:
- Date: 2026-08-30
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - `crates/hickory-cli/src/typed_client/emit.rs::Target::runtime` returns
    `(filename, source)` per target, `include_str!`'d from
    `code-models/clients/{python,csharp}/`. Go and TypeScript return `None`
    and `cmd_code_model` refuses with the list of what does ship.
  - `code-models/clients/csharp/HickModel.cs` — `Model.Start`/`Query<T>`/
    `Dispose` plus `Emit`, written for a **file-based-app-adjacent** console
    project with no dependency beyond the BCL, because a NuGet package in a
    generator means a restore, a lock file and a network hole in a cell that
    needs none.
  - Run end to end in the `warehouse` project: `tools/ApiGen` (a `.csproj`
    plus top-level-statement `Program.cs`, linking `../model/csharp/*.cs`)
    runs in a sandboxed `mcr.microsoft.com/dotnet/sdk:10.0` cell in ~8s and
    `git diff` on `src/Warehouse.Web/Api/Generated/` is **empty** against the
    output of the Python generator it replaced.
  - The Python client is generated beside it into `tools/model/python/` and
    used by `40-layering.hick`, so both clients are exercised on every run.
- Test coverage:
  `typed_client::emit::runtime_tests::every_runtime_is_the_language_it_claims_to_be`
  (each runtime is the right file and carries its error check — the failure
  mode is a wrong `include_str!` path, which would be silent) and
  `a_target_with_no_runtime_says_so_rather_than_serving_another_languages`.
  End to end, `hick test .` in the warehouse covers the whole path.
- Caveat requiring LLM review: only two runtimes exist. Go and TypeScript get
  types with no transport, which is a real gap for anyone generating from
  those languages — `--sdl` and their own ecosystem's codegen is the stated
  answer and has not been walked through by anybody. Nothing checks that a
  runtime **compiles**; the C# one is proven by the warehouse building, and
  a runtime for a language with no demo behind it would not be.
