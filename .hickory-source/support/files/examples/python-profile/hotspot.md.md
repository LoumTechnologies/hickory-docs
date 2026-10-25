<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# examples/python-profile/hotspot.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `3b8c722898ea31c8691115b35fddabdc55082cd5e26aefcdd396257e8d95201d`.

<book:entry path="examples/python-profile/hotspot.md">
# Profile a real Python hotspot

Run saved Python with cProfile from Runtime observations, or:
`hick observe hotspot.md --program hotspot.py --profile`.
Expand `subtotal` and inspect the same selected evidence in the next context.
Replace its loop with `return n * (n - 1) // 2`, rerun the correctness witness
and capture another profile. Compare separate runs; timing and profiler overhead
vary. The browser fixture replays an explicitly identified local capture.

<hick:file path="hotspot.py">
def subtotal(n):
    total = 0
    for value in range(n):
        total += value
    return total

def recursive(n):
    return n + recursive(n - 1) if n else 0

assert recursive(9) == 45
assert sum(subtotal(8000) for _ in range(500)) == 15998000000
print('correctness: 15998000000; recursion: 45')
</hick:file>
</book:entry>

</book:doc>
