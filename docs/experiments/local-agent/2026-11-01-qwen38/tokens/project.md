<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">

# Whitespace Token Counter in First-Appearance Order

This document specifies a small pure-Python helper that counts whitespace-separated tokens while preserving their first appearance order. The construction is intentionally dependency-free: it relies only on built-in string and list operations, so it can run in restricted environments without imports.

## Construction

1. **Normalize the input.** If `data` is `None` or not a string, treat it as empty. This keeps the function total and avoids raising on odd inputs.
2. **Split on whitespace.** `str.split()` with no argument splits on any run of whitespace and drops leading/trailing whitespace, which matches the required behavior for empty or whitespace-only input.
3. **Lowercase each token.** `str.lower()` is applied to each token after splitting. Punctuation is preserved because lowercasing only affects case-sensitive letters.
4. **Count in first-appearance order.** A plain dictionary is used as an ordered counter. Python dictionaries preserve insertion order, so the first time a token is seen it is inserted, and later occurrences only increment its count. This avoids the earlier mistake of sorting alphabetically.
5. **Return the required shape.** Convert the ordered dictionary into a list of two-element lists: `[[token, count], ...]`.

## Why this satisfies the samples

- `"B a B"` splits to `["B", "a", "B"]`, lowercases to `["b", "a", "b"]`, and counts to `{"b": 2, "a": 1}`. The first-appearance order is `b` then `a`, giving `[["b", 2], ["a", 1]]`.
- `" Hi! hi HI! "` splits to `["Hi!", "hi", "HI!"]`, lowercases to `["hi!", "hi", "hi!"]`, and counts to `{"hi!": 2, "hi": 1}`. The first-appearance order is `hi!` then `hi`, giving `[["hi!", 2], ["hi", 1]]`.

## Edge cases

- Empty string `""` → `[]`.
- Whitespace-only string `"   "` → `[]`.
- Single token `"Hello"` → `[["hello", 1]]`.
- Mixed punctuation `"a,a a,a"` → `[["a,a", 2]]` because punctuation is preserved inside tokens.

<hick:project/>

<hick:file path="solution.py">
def solve(data):
    if not isinstance(data, str):
        return []
    tokens = data.split()
    counts = {}
    for token in tokens:
        key = token.lower()
        if key in counts:
            counts[key] += 1
        else:
            counts[key] = 1
    return [[token, count] for token, count in counts.items()]
</hick:file>

</hick:doc>