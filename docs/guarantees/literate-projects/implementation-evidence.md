# Literate project implementation evidence

Date: 2026-10-12. Reviewer: Codex. This is evidence for the implemented boundary,
not a declaration that every milestone in `literate-changesets.md` is complete.
Human judgment has not been recorded.

The portable evaluator constructs exact entries from declared Markdown inputs,
orders acts by explicit predecessors, applies strict unified diffs, and compares
complete output inventories in memory. Pure services separately check transition
responses over A and incorporation candidates from empty against B. Native
project document evaluation, default New LSP/DAP staging, the patch registry/view,
and complete project refactor baselines use this boundary.

Consequential assumptions and limits:

- Project inventory excludes source closure Markdown, declared woven readings,
  language-defined VCS/Hick metadata, and authored exact/prefix exclusions. It
  never guesses source extensions or consults Git ignore to hide an asset.
- Relative paths use a conservative ASCII spelling; Unicode **content** is
  supported. Non-ASCII path normalization is explicitly unsupported.
- Literal files, explicit UTF-8/hex entries, literal copy/paste/includes, and
  textual patch acts are supported. Ambient/upstream/fact-dependent/recorded-cell
  construction and execution are refused by name in this portable subset.
- Local before/after states depend on declared ancestors, rather than ordering
  unrelated act names. Atomic New is always the completed combined tree.
- Immutable Git/Hg/CRDT readings must be forked into authored comparisons before
  editing. Destination guards retain original captures after proposal edits.
- Binary and metadata transitions are named unsupported operations in final
  textual response reconstruction; maintained source can construct these entries.
- Native symlink/mode materialization has explicit platform limits. The legacy
  browser literal debugger refuses declared project/patch inputs; the full pure
  WASM evaluator is separate and never grants container/native execution.
- New/default native staging and explicit endpoint materialization are implemented.
  Cross-document/old-revision protocol mappings, nested atomic scope selectors and
  the common editable comparison UI are pending. Native origin mapping omits
  foreign spans rather than returning unrelated source lines.
- Shared lifecycle APIs do not imply automatic harness capture. Built-in/Python/ACP
  finalization, concurrent-write attribution, AI incorporation and existing-review
  wiring remain pending. No final-response source file is automatically appended
  to maintained programs by these APIs.
- Exact candidate commit/index checking is available through `hick project verify`
  and the root `verify-literate-project-index` recipe. Automatic entry selection
  in installed commit hooks is pending; the old hook is not claimed as a staged
  project verifier.
- Shared N-source model/provider inputs are implemented; merged/worktree target
  routing and partial-write undo integration remain pending. No cross-target
  filesystem transaction or structural merge is claimed.

Executed checks:

- `just test-literate-projects`: 26 focused Rust tests passed (15 parser/tree/lifecycle,
  3 LSP, 1 DAP, 3 pipeline/baseline, 4 revision/directory tests); the real Hg test
  is deliberately excluded here and executed explicitly below.
- `just test-literate-projects-regressions`: parser (91), LSP (103), debugger (58)
  and refactor (1) Rust regressions passed. Its first browser run found the explicit
  element catalogue missing `patch`; that catalogue was corrected.
- `just test-literate-projects-web-regressions`: TypeScript checking passed; all
  2,054 tests across 212 browser test files passed with the regenerated WASM parser.
- `just test-literate-projects-csharp`: one real EFCore/runtime/netcoredbg acceptance
  passed after the two recorded routing corrections, with SDK configuration and
  the final six-entry program supplied by Markdown.
- `just test-literate-projects-hg`: one real Mercurial 7.0 provider test passed.
- `just verify-literate-project-fixtures`: both root entries and both contributing
  documents passed actual CLI lint and complete source verification.
- `just check-codegen`, `just check-file-length`, `just lint-literate-projects`
  (all targets, warnings denied), formatting and `git diff --check` passed.
- Platform: macOS arm64; Rust 1.96.1; Node v24.21.0; C# fixture pins SDK 10.0.401
  and net10.0, EFCore SQLite 10.0.0, SQLitePCLRaw bundle 3.0.5; netcoredbg archive
  3.2.0-1092 was installed through the existing pinned confined installer.
- The full workspace CI/browser E2E matrix and additional native platforms were
  not run. These results do not claim the remaining plan integrations. The Git
  checkout base during the final checks was `5fa7a84866f92c08b484ed9f799e1763d87c4342`;
  the working file identities below identify the implemented change itself.

Independent witnesses and corrections:

- The two-file fixture intentionally has a caller referring to a function that
  does not exist at the middle patch. Atomic New includes the later library fix.
  Folding the explanation into literal files preserves its complete outputs.
- Copying only its Markdown closure into a fresh Git-free folder constructs the
  same source tree without generated code/cache/config input. Extra migration
  bytes cause inventory failure. Entry routes let contributors select that same
  completed scope.
- A changed staged document cannot borrow its matching unstaged output. Commit
  reads remain valid even after working Markdown changes. Scoped Git reads select
  only the chosen project directory. Real Mercurial and immutable persisted CRDT
  update tests use the same tree model.
- Same-length binary payloads previously compared equal through display strings.
  The shared entry equality now checks bytes. Refactor pinning freezes temporary
  binary output into memory, so mutation/removal cannot move the baseline.
- Real EFCore acceptance first reproduced an inserted-line breakpoint refusal
  because DAP line routing subtracted the diff prefix column. Line routing now
  selects an unambiguous mapped row. A second run loaded symbols but remained
  pending because the compiler used macOS `/private/var` while the request used
  `/var`; staged source identities now canonicalize that alias.
- The explicit native fixture constructs the complete program, fixed migration,
  model snapshot and SDK configuration from Markdown. It applies the migration,
  persists/reads a note, starts the pinned real netcoredbg, stops on inserted code,
  and watches its value. It uses no mocked debugger.
- The fixture pins SQLitePCLRaw's bundle separately after a restore surfaced a
  vulnerable older transitive native library. The package definition is
  [SQLitePCLRaw.bundle_e_sqlite3 3.0.5](https://www.nuget.org/packages/SQLitePCLRaw.bundle_e_sqlite3/3.0.5).
  EFCore migration handling follows the
  [official migration API](https://learn.microsoft.com/en-us/ef/core/managing-schemas/migrations/applying).
  These are test/example dependencies, not Rust dependencies linked into Hickory.

Evidence dependencies:

```text
8d2d2c4b00e669a48abb0028ea169554b0fe39e2513317e760f474d1baf9a681  Cargo.lock
06d357054b86133f4a2494fdc802e36f2f965eff69b7218ff36adc0bd0afbaae  apps/web/src/editor/generated/hick-lang/hick_lang.d.ts
900ca1881c298df8b751dbc942bd6d2db05870805fe31a9626c8c201a28e1964  apps/web/src/editor/generated/hick-lang/hick_lang.js
98d987d5ef9535416d9f32d9555cfcbaf63145c3a7303099805c0e439553635b  apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm
1d64025fcdfdffd4a9bbd234d66bb3be7269f01846c2d99ac330a09635a181ff  apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm.d.ts
51a968c2909ed1b6be523b61f2b06afb373034d2054d5592066136d4a6837ded  apps/web/src/editor/hickLang.ts
18977c5cb6ced9cfe2c15de4385dd868915bd75f9ac9cb6e70f211c791d83711  apps/web/src/elements/index.test.ts
dffbbe553ddb81b187511b71cbc8f16e820c0c5822e0b673830e12d2cd5af991  apps/web/src/elements/index.ts
d9f0a5e4b7c7200dc8dcc9efa566af187c3a34795821414f7047a5674443eea6  apps/web/src/elements/patch/view.test.tsx
b1c4c2639ac7582827ca44427ade3d3f60446ecf888074f1fafaf130de2eb51a  apps/web/src/elements/patch/view.tsx
104dfa98ed318ab48a80176fcc40de7610b58722447f058667623201bedc82e2  apps/web/src/elements/types.ts
82cc4c22a1f5d1c297984196c2fed68ea581820ff63c1fb7a16e1b417a8ff2bb  crates/hick-blocks/src/lib.rs
7422822a7aa30fb80179884555e892f30a743094e253b9b5674d2555fc047214  crates/hick-dap/Cargo.toml
4caf7a783e262abdf387a6ae52d63c0909773ee19b10022601edde8fe6ef5d56  crates/hick-dap/src/program.rs
c9598834ba2da51ebe7d372f4b3fcf795265185338fcad4b25b4d6bafbdce718  crates/hick-dap/src/project.rs
699bf96e662ffb9d94ce663699d8098cf6b0054805f77522c7fd29b29b261ff6  crates/hick-dap/src/session.rs
3b1ceca8557402c9b0c776b99d45e5deaaab414f19aea94c34466ea661fa3778  crates/hick-dap/tests/project.rs
7491fe705b0c69fae7fb479f87d980554fc46e1055d908f654a636d8cf8a6c4f  crates/hick-lang/src/lib.rs
bf3c4c2cec49eb19598b82df88fd4a2e3313b4e77b6b3227f8b1a4f175c98ff7  crates/hick-lang/src/literal_files.rs
c0f3154582752546f85dea30d8a8d6934d3ce87c5f8681e12ed562095735c53a  crates/hick-lang/src/parser.rs
aff5564b515ff4b834f8b288d2eec725c2b8891b46b8dc00ffc5daced72d4ebf  crates/hick-lang/src/project/comparison.rs
ea3bee2af62148559f417c7537a6b2d6311ee76d43fdab7a63b7e0587090d041  crates/hick-lang/src/project/discovery.rs
834b20c06bb22ec848f40e06045479544dec058e210445559e11f93b60f51132  crates/hick-lang/src/project/edit.rs
db99bc30ae6d943e317da977d1e007f2a485207ed4e8ae2a8cb9d0bdc5481e7a  crates/hick-lang/src/project/evaluate.rs
e417ea0d0eab1da1885e134b70bc2509d0ed823ee4a5231a4dd9da2e87322652  crates/hick-lang/src/project/mod.rs
445024f328e6da9a340fde8b513f9bb54e782279a4551fd340d3e05c6bf70101  crates/hick-lang/src/project/patch.rs
ce90bd012169bdf90d7d91901dd67fbed4da9173d4490104cf7174fb852c7fb8  crates/hick-lang/src/project/reconstruct.rs
946dfaf868fe62e0cfa1d931c6d82b7b71a93790d755988082ae16b35e588c50  crates/hick-lang/src/project/tree.rs
b61b28d5c1c7b913ab4ad60c025baf4b8acf66089ec5e2abb6e80bba6e8eec19  crates/hick-lang/tests/fixtures/efcore-project/app/Migrations/Initial.cs
1c6a2d9a117a0c6fd524c75a253b0f7052254546ae9aa04a9af582956880e377  crates/hick-lang/tests/fixtures/efcore-project/app/Migrations/NoteDbSnapshot.cs
995839958093d2de1afb0bd1dcd8ce9ee87422f9ed82fb644433a431985159e8  crates/hick-lang/tests/fixtures/efcore-project/app/NoteDb.cs
edf6656f78179eadaf95d53ffbd270d90f5309076113caeb1a5b9f72b979b9be  crates/hick-lang/tests/fixtures/efcore-project/app/Program.cs
7371e6291bc3a8433937fb005ee43a40316cfac02451f9403dcd56f9b006a4b5  crates/hick-lang/tests/fixtures/efcore-project/app/app.csproj
a0dcc4ee0c1b8044f9498734741bd764d5c2585f4d0d8e31e27e26e9662f751f  crates/hick-lang/tests/fixtures/efcore-project/global.json
839773d70526343a7efaecffc5aecd7d30e0f4d283876e1de642db35e1370d27  crates/hick-lang/tests/fixtures/efcore-project/persistence.md
4a9e321188c6f3a31fec7e7935cbf1ed8ae751c2dba722becf798a6733a9fae6  crates/hick-lang/tests/fixtures/efcore-project/project.md
28ae4bbee11f9365f559429ed1d5a66c5f5f931e066aafb8c809cbaa46e800d7  crates/hick-lang/tests/fixtures/literate-project/app.ts
26046a12320224b64ec58580de87c2a455f67d6a6c19dbd96a39db7cf89f92a4  crates/hick-lang/tests/fixtures/literate-project/library.md
de3cc990aea6a90bb3cda0a15053d3d38d263680be21c8caf72acb753238737a  crates/hick-lang/tests/fixtures/literate-project/library.ts
b35487a8b21bf83bfe6fe12efe96ba1a808443d0cfb8bcc00b6a86489b1031fa  crates/hick-lang/tests/fixtures/literate-project/project.md
2ef79ef56e4ffcef70bdac4a4d0f7855272c680a7d25cf78089a73a92d84f4c8  crates/hick-lang/tests/project.rs
145eb8d5f2b486d2429ea0debebc44c6db8d54ae99be0f5a961896fe69a3881e  crates/hick-lang-wasm/src/lib.rs
1e94c4b7f33ce2e3ab8d1487217b89212c4d9d62afc919abfa36d119aaca5ea2  crates/hick-literate/src/equiv.rs
f25d88d2d6aae0ac9e6ae35c0b9cbf28ab72982ea749bebe12a548968788f2fa  crates/hick-literate/src/lib.rs
4dc312eccb2855de3a78520ba919cf639e6691c672ef5bc053e20db921c78107  crates/hick-literate/src/patch_element.rs
d3beed8d4b57937560cf5d9a3d9e033b3634a4117b37c084faf3ee0f2d730c9e  crates/hick-literate/src/project.rs
153937e48b84053f344432fe281ee344cf1b7ed79bfb1060c0c2fcd169317122  crates/hick-literate/src/render.rs
b6c4dd4541dcce45e740c0c0933bb5d28413688aa9884a227edccd78a5f57ba0  crates/hick-literate/tests/project.rs
00fd8aee488353a9e5472d3c0f25ff996e9f6cce4260fa603f515a7ddb5eb4f8  crates/hick-lsp/src/backend.rs
bbb3d3432f030e53ea0955bd1d7c5a4ec7c68b6cf407f6c753c48f9cadb86634  crates/hick-lsp/src/document.rs
dabc914188bf8efddc1f0b39386eea6aca10f7b02b3bc7af8dbf6bbae452be95  crates/hick-lsp/src/position_map.rs
058553f4c594d3889556c6d80d7892c5f597db629ec28861198a47b4a3da068e  crates/hick-lsp/src/project.rs
6f6936cbcb3d9c0fe71914b508ca61cbf0cc7d22f8acc28ce211e421905d6031  crates/hick-lsp/tests/project.rs
c0f3c9f838b06ad14015d997840480d42c916b64c8d489f434bcb6b1253da8d9  crates/hickory-cli/src/debug_sessions.rs
5fcdfc45e57d96a19f2c0b109188d982b96ae93ff4ed0a9e731594b7501258a8  crates/hickory-cli/src/lib.rs
1ef9fee8a857384aa88b5c0c97fe2d9e94f36fe4b7a0f36ad323a7ecfdc3bb7a  crates/hickory-cli/src/main.rs
6b263d5ca82bc5f0df8ac2f706e61588a5e1e88cb4f1aa3b961e2c4ec369f460  crates/hickory-cli/src/project_command.rs
60d1a55617ace2fa564c0fad7d92827cfc5f1aaa0cf6a13939c4f029614ad09f  crates/hickory-cli/src/project_documents.rs
7689e73ce982460d1b7d2598bed5f70cda0797d9a6d68810be20d97016057b69  crates/hickory-cli/src/project_revision.rs
99ae6a7408b45d79fda40e0a48c772566a0e6379a01563375c25dbc1fc8c4a38  crates/hickory-cli/src/serve/refactor.rs
81720ce6f4c5cfcd281fc2a4616d645fc66c10fe23c21b4c6d3b4e0dc6954479  crates/hickory-cli/tests/literate_project_csharp.rs
fe601bcefefd212b2dca7e4b2629f249fcae37456353d4a9038e466ef0961727  crates/hickory-cli/tests/project_revision.rs
dee572b3503c0cc1198d4e7ce288f7614d67ff8cbac89832e0444d481ec70fc1  justfile
```

The hashes identify this working implementation rather than treating a mutable
working tree as an immutable commit. Changes to these files make their associated
executed evidence stale and require the relevant checks again.
