<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">

# Merging Overlapping Half-Open Intervals

This document presents a solution to merge overlapping half-open intervals. The core challenge is to correctly identify overlaps while respecting the half-open nature of the intervals, where touching endpoints do not constitute an overlap.

## Construction

The algorithm proceeds in three main steps:

1.  **Immutability and Sorting**: First, we create a deep copy of the input list to ensure we do not mutate the original data. We then sort these copied intervals by their start time. Sorting is a prerequisite for the linear merge pass.

2.  **Linear Merge Pass**: We iterate through the sorted intervals. We maintain a `result` list.
    *   If the `result` list is empty, we append the first interval.
    *   Otherwise, we compare the current interval's start time with the end time of the last interval in the `result` list.
    *   **Overlap Condition**: Two half-open intervals `[a, b)` and `[c, d)` overlap if and only if `c < b`. If `c >= b`, they are either disjoint or merely touching (in the case `c == b`).
    *   If they overlap (`current_start < last_end`), we merge them by updating the end time of the last interval in `result` to be the maximum of the two end times.
    *   If they do not overlap (`current_start >= last_end`), we append the current interval to `result`.

3.  **Return**: Finally, we return the `result` list.

This approach ensures that touching intervals (e.g., `[1, 3]` and `[3, 5]`) are kept separate, while overlapping intervals (e.g., `[1, 4]` and `[2, 6]`) are correctly merged into `[1, 6]`.

<hick:project/>

<hick:file path="solution.py">
def solve(data):
    if not data:
        return []
    
    # Create a copy to avoid mutating the input
    intervals = [list(interval) for interval in data]
    
    # Sort by start time
    intervals.sort(key=lambda x: x[0])
    
    result = [intervals[0]]
    
    for i in range(1, len(intervals)):
        current_start, current_end = intervals[i]
        last_start, last_end = result[-1]
        
        # Overlap condition for half-open intervals: current_start < last_end
        if current_start < last_end:
            # Merge: update the end of the last interval in result
            result[-1][1] = max(last_end, current_end)
        else:
            # No overlap (disjoint or touching), append current interval
            result.append([current_start, current_end])
            
    return result
</hick:file>

</hick:doc>