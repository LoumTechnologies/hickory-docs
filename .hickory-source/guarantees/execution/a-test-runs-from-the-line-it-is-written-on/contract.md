<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Test Runs From The Line It Is Written On

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a plain or generated file in a language whose tests have a shape the
editor knows — `#[test]` above a `fn`, `it("…")` or `test("…")`, `def test_…`,
`[Fact]`/`[Test]` above a method, `func TestX(t *testing.T)` — when the file
is open, then a run mark sits in the gutter on each test's line; and when the
mark is clicked, then that one test runs in its ecosystem's own runner
(`cargo test <name>`, `npx vitest run <file> -t <name>` or jest, `python3 -m
pytest <file>::<name>`, `dotnet test --filter`, `go test -run '^<name>$'`),
in the nearest directory that owns the file, as a terminal session named
`test: <name>` that opens beside the file and reports finished or failed the
way every session does. A file with no manifest above it, or a runner the
package does not name, is refused in words that say what is missing.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-test-runs-from-the-line-it-is-written-on.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
