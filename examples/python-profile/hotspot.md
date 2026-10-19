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
