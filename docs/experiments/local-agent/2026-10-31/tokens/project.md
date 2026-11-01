<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:project/>
<h1>Word Frequency Counter with First Appearance Order</h1>

This document implements a solution to count word frequencies in a string while preserving first appearance order. The approach splits input on whitespace, normalizes case, and maintains the sequence of first encounters. The implementation handles punctuation within tokens and correctly returns empty list for empty or whitespace-only inputs. The algorithm processes each token in order, tracking counts and first occurrence positions. When a new token is encountered, it's added to results with count 1. For existing tokens, the count is incremented. This ensures the final result preserves the order of first appearance rather than alphabetical ordering.

<hick:file path="solution.py">
def solve(data):
    if not data or not data.strip():
        return []
    
    # Split on whitespace and lowercase each token
    tokens = data.split()
    tokens = [token.lower() for token in tokens]
    
    # Track counts and first appearance indices
    count_map = {}
    first_indices = {}
    
    for i, token in enumerate(tokens):
        if token not in count_map:
            count_map[token] = 0
            first_indices[token] = i
        count_map[token] += 1
    
    # Build result preserving first appearance order
    result = []
    seen = set()
    
    for token in tokens:
        if token not in seen:
            result.append([token, count_map[token]])
            seen.add(token)
    
    return result
</hick:file>
</hick:doc>