// The home page's one demo, as data.
//
// It is deliberately a single document rather than a tour of scenarios. What
// the product is (docs/specs/freeform/local-only.md) is one sentence —
// literate programming where you can edit the generated files — and the page
// shows exactly that sentence: a document holds the fragments, the fragments
// weave into files, the files run, and an edit made at either end lands at the
// other. Every extra scenario the page used to carry (a simulated issue
// tracker, two people typing at once) demonstrated something the product does
// not have.
//
// The weave, the provenance ribbons, and the round trip are computed by the
// real modules in `lib/` — see the header of DemoSplit.tsx. Only the execution
// transcript is a recording, because a browser cannot run Python.

export const PROGRAM_DOC_PATH = "notes/bisect.md";

/**
 * The demo document.
 *
 * Four things have to be visible at once for the picture to make sense: a
 * fragment (`hick:copy`), two files that paste it (`hick:file` + `hick:paste`)
 * so one fragment feeding two outputs is on screen rather than asserted, an
 * executable cell with a pinned expectation, which is the half of the product
 * that makes the document verifiable rather than merely generated, and a cell
 * that pins a STRUCTURAL claim rather than an output one.
 *
 * That last cell is why SCIP is here. Every architecture document ever written
 * asserts a shape — "this layer never calls that one" — and nothing checks it,
 * so it is wrong within a quarter and nobody finds out. A claim about shape
 * can be executed like any other: index the code, query the index, pin the
 * count at zero. The image is named `scip-tools:local` because no public image
 * carries a language indexer, the `scip` CLI, and `jq` together — that one is
 * yours to build, and saying so is better than naming an image that does not
 * exist.
 */
export const PROGRAM_SOURCE = `# Finding an insertion point

We want the leftmost index at which \`target\` could be inserted into a sorted list without breaking the ordering. Two indices bound the answer, and every comparison halves the distance between them.

<hick:copy id="invariant">
# lo <= answer <= hi, and every index below lo is known to be too small.
</hick:copy>

The loop keeps that true. When \`lo\` and \`hi\` meet there is exactly one index left, and the invariant says it is the answer.

<hick:copy id="search">
def bisect_left(xs, target):
    lo, hi = 0, len(xs)
    while lo < hi:
        mid = (lo + hi) // 2
        if xs[mid] < target:
            lo = mid + 1
        else:
            hi = mid
    return lo
</hick:copy>

The prose above is the paper. The two files below are the program, and they are woven from the very same fragments — the explanation cannot drift from the code, because there is only one copy of the code.

<hick:file path="search/bisect.py" language="python">
<hick:paste select="#invariant"/>

<hick:paste select="#search"/>


if __name__ == "__main__":
    print(bisect_left([1, 3, 5, 7, 9], 6))
</hick:file>

<hick:file path="search/test_bisect.py" language="python">
from bisect import bisect_left as reference

<hick:paste select="#search"/>


def test_agrees_with_the_standard_library():
    xs = [1, 3, 5, 7, 9]
    for target in range(0, 11):
        assert bisect_left(xs, target) == reference(xs, target)
</hick:file>

Six sorts between 5 and 7, so the insertion point is index 3. That claim is not a comment — it is pinned below, re-checked on every run, and a build that disagrees fails.

<hick:container name="py" image="python:3.12" />

<hick:exec container="py" mount="search:/project">
python /project/bisect.py
<hick:expect match="exact">3
</hick:expect>
</hick:exec>

<hick:exec container="py" mount="search:/project">
pytest -q /project/test_bisect.py
<hick:expect match="regex-lines">1 passed in \\d+\\.\\d+s
</hick:expect>
</hick:exec>

The last claim is about shape rather than output: \`bisect.py\` is the module the test depends on, so nothing in it may ever point back at the test. That is the kind of rule a diagram asserts and nobody checks. Here SCIP indexes the woven files and the cell counts the violations, so the day someone reverses the dependency, this document stops passing.

<hick:container name="scip" image="scip-tools:local" />

<hick:exec container="scip" mount="search:project">
cd project && scip-python index . --project-name search
scip print --json index.scip | jq '[.documents[] | select(.relative_path == "bisect.py") | .occurrences[] | select(.symbol | contains("test_bisect"))] | length'
<hick:expect match="exact">0
</hick:expect>
</hick:exec>
`;

/**
 * The recorded run of the two exec cells above.
 *
 * The demo labels this as a recording on screen. It has to be one: nothing in
 * a browser tab can start a `python:3.12` container, and pretending otherwise
 * would make the page a mockup of the one thing it exists to be honest about.
 */
export const PROGRAM_TRANSCRIPT: { cmd: string; out: string[]; verdict: string }[] = [
  { cmd: "python /project/bisect.py", out: ["3"], verdict: "matches the pinned output" },
  {
    cmd: "pytest -q /project/test_bisect.py",
    out: [".                                        [100%]", "1 passed in 0.03s"],
    verdict: "matches the pinned pattern",
  },
  {
    cmd: "scip print --json index.scip | jq '[.documents[] | select(.relative_path == \"bisect.py\") | .occurrences[] | select(.symbol | contains(\"test_bisect\"))] | length'",
    out: ["0"],
    verdict: "the dependency still points one way",
  },
];
