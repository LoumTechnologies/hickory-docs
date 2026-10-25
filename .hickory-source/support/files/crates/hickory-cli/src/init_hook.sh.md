<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/init_hook.sh

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Candidate revision checks never borrow working bytes](../../../../../guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes/contract.md)

Captured SHA-256: `383c9afbaac700ab063475f370f76d18c585658b7154278a6aa9cd7c8e8cfdd5`.

<book:entry path="crates/hickory-cli/src/init_hook.sh">
# Managed by `hick init` — do not edit inside this block.
# Re-run `hick init` to refresh it.
if [ -n "$(git ls-files -- '*.md')" ]; then
    if command -v hick >/dev/null 2>&1; then
        # Continuity repair: an unwatched edit leaves its correspondence
        # behind before it is committed. Does nothing unless continuity is on
        # for this project, and NEVER blocks a commit — the check is the
        # repair, and a rule with no escape hatch gets this hook disabled
        # entirely, taking the drift gate below down with it.
        hick repair || true
        # Portable project checks select their entry points from the frozen
        # stage-zero index, including changes to generated files alone.
        hick_list=$(mktemp) || exit 1
        if ! hick project verify-index --standalone-list "$hick_list"; then
            rm -f "$hick_list"
            exit 1
        fi
        if [ ! -s "$hick_list" ]; then
            rm -f "$hick_list"
        else
        xargs -0 sh -c '

        if ! hick lint "$@"; then
            echo "pre-commit: \`hick lint\` found invalid hick element attributes. Fix the marked spelling before committing. Commit blocked." >&2
            exit 1
        fi
        hick_worst=0
        for hick_doc do
            hick_code=0
            hick test "$hick_doc" || hick_code=$?
            if [ "$hick_code" -gt "$hick_worst" ]; then
                hick_worst=$hick_code
            fi
        done
        case "$hick_worst" in
            0) ;;
            1) echo "pre-commit: \`hick test\` found DRIFT (exit 1) — a committed output is out of date with what the document produces. Re-run \`hick run <doc>\` (or \`hick refresh <doc>\` for a stale hick:transform) and commit the result. Commit blocked." >&2
               exit 1 ;;
            2) echo "pre-commit: \`hick test\` could NOT VERIFY (exit 2) — a cell has no baseline, so nothing was actually checked. Record one by running \`hick run <doc>\` — a cell declaring freeze=true runs once and records itself — or stop freezing that cell. Commit blocked." >&2
               exit 1 ;;
            3) echo "pre-commit: \`hick test\` found a FAILED EXPECTATION (exit 3) — a hick:expect did not hold, so the document claims something untrue of its own output. Do not regenerate this away: decide whether the claim or the code is wrong. Commit blocked." >&2
               exit 1 ;;
            *) echo "pre-commit: \`hick test\` exited $hick_worst — see the output above. Commit blocked." >&2
               exit 1 ;;
        esac
        ' hick-pre-commit < "$hick_list"
        hick_check=$?
        rm -f "$hick_list"
        [ "$hick_check" -eq 0 ] || exit 1
        fi
    else
        echo "pre-commit: hick not found on PATH; skipping .md verification" >&2
    fi
fi
</book:entry>

</book:doc>
